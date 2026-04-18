use std::path::PathBuf;

use anyhow::{Context, bail};
use jni::{
    Env, JValue, JavaVM, jni_sig, jni_str,
    objects::{JObject, JString},
};

const MANAGE_FILES_REQUEST_MESSAGE: &str = concat!(
    "Android storage access requested; enable \"Allow access to manage all files\" ",
    "for Alpha GUI, then return to the app and retry the backup"
);
const WRITE_PERMISSION_REQUEST_MESSAGE: &str =
    "Android storage permission requested; approve the dialog and retry the backup";

pub fn public_documents_app_dir() -> anyhow::Result<PathBuf> {
    with_env(|env| {
        ensure_public_storage_access(env)?;
        let documents = public_documents_dir(env)?;
        Ok(documents.join("alpha-cli"))
    })
}

fn with_env<T>(f: impl FnOnce(&mut Env<'_>) -> anyhow::Result<T>) -> anyhow::Result<T> {
    let context = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(context.vm().cast()) };
    vm.attach_current_thread(f)
}

fn ensure_public_storage_access(env: &mut Env<'_>) -> anyhow::Result<()> {
    let sdk = sdk_int(env)?;
    if sdk >= 30 {
        if is_external_storage_manager(env)? {
            return Ok(());
        }
        request_manage_external_storage(env)?;
        bail!(MANAGE_FILES_REQUEST_MESSAGE);
    }
    if sdk >= 23 && !has_write_external_storage(env)? {
        request_write_external_storage(env)?;
        bail!(WRITE_PERMISSION_REQUEST_MESSAGE);
    }
    Ok(())
}

fn sdk_int(env: &mut Env<'_>) -> anyhow::Result<i32> {
    Ok(env
        .get_static_field(
            jni_str!("android/os/Build$VERSION"),
            jni_str!("SDK_INT"),
            jni_sig!("I"),
        )?
        .i()?)
}

fn is_external_storage_manager(env: &mut Env<'_>) -> anyhow::Result<bool> {
    Ok(env
        .call_static_method(
            jni_str!("android/os/Environment"),
            jni_str!("isExternalStorageManager"),
            jni_sig!("()Z"),
            &[],
        )?
        .z()?)
}

fn has_write_external_storage(env: &mut Env<'_>) -> anyhow::Result<bool> {
    let permission = env.new_string("android.permission.WRITE_EXTERNAL_STORAGE")?;
    let context = android_context_object(env)?;
    let result = env
        .call_method(
            &context,
            jni_str!("checkSelfPermission"),
            jni_sig!("(Ljava/lang/String;)I"),
            &[JValue::Object(&permission)],
        )?
        .i()?;
    Ok(result == 0)
}

fn request_write_external_storage(env: &mut Env<'_>) -> anyhow::Result<()> {
    let permission = env.new_string("android.permission.WRITE_EXTERNAL_STORAGE")?;
    let array = env.new_object_array(1, jni_str!("java/lang/String"), JObject::null())?;
    array.set_element(env, 0, permission)?;
    let context = android_context_object(env)?;
    env.call_method(
        &context,
        jni_str!("requestPermissions"),
        jni_sig!("([Ljava/lang/String;I)V"),
        &[JValue::Object(array.as_ref()), JValue::Int(41)],
    )?;
    Ok(())
}

fn request_manage_external_storage(env: &mut Env<'_>) -> anyhow::Result<()> {
    let context = android_context_object(env)?;
    let package = env
        .call_method(
            &context,
            jni_str!("getPackageName"),
            jni_sig!("()Ljava/lang/String;"),
            &[],
        )?
        .l()?;
    let package = env.cast_local::<JString>(package)?;
    let package = package.try_to_string(env)?;
    let uri_text = env.new_string(format!("package:{package}"))?;
    let uri = env
        .call_static_method(
            jni_str!("android/net/Uri"),
            jni_str!("parse"),
            jni_sig!("(Ljava/lang/String;)Landroid/net/Uri;"),
            &[JValue::Object(&uri_text)],
        )?
        .l()?;
    let action = env.new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION")?;
    let intent = env.new_object(
        jni_str!("android/content/Intent"),
        jni_sig!("(Ljava/lang/String;)V"),
        &[JValue::Object(&action)],
    )?;
    env.call_method(
        &intent,
        jni_str!("setData"),
        jni_sig!("(Landroid/net/Uri;)Landroid/content/Intent;"),
        &[JValue::Object(&uri)],
    )?;
    env.call_method(
        &intent,
        jni_str!("addFlags"),
        jni_sig!("(I)Landroid/content/Intent;"),
        &[JValue::Int(0x1000_0000)],
    )?;
    env.call_method(
        &context,
        jni_str!("startActivity"),
        jni_sig!("(Landroid/content/Intent;)V"),
        &[JValue::Object(&intent)],
    )
    .context("open Android all-files storage settings")?;
    Ok(())
}

fn public_documents_dir(env: &mut Env<'_>) -> anyhow::Result<PathBuf> {
    let documents = env
        .get_static_field(
            jni_str!("android/os/Environment"),
            jni_str!("DIRECTORY_DOCUMENTS"),
            jni_sig!("Ljava/lang/String;"),
        )?
        .l()?;
    let file = env
        .call_static_method(
            jni_str!("android/os/Environment"),
            jni_str!("getExternalStoragePublicDirectory"),
            jni_sig!("(Ljava/lang/String;)Ljava/io/File;"),
            &[JValue::Object(&documents)],
        )?
        .l()?;
    if file.is_null() {
        bail!("Android public Documents directory is not available");
    }
    let path = env
        .call_method(
            &file,
            jni_str!("getAbsolutePath"),
            jni_sig!("()Ljava/lang/String;"),
            &[],
        )?
        .l()?;
    let path = env.cast_local::<JString>(path)?;
    let path = path.try_to_string(env)?;
    Ok(PathBuf::from(path))
}

fn android_context_object<'local>(env: &mut Env<'local>) -> anyhow::Result<JObject<'local>> {
    let context = ndk_context::android_context();
    let borrowed = unsafe { JObject::from_raw(env, context.context().cast()) };
    env.new_local_ref(&borrowed).map_err(Into::into)
}
