#[cfg(target_os = "android")]
mod imp {
    use anyhow::{Context, bail, ensure};
    use jni::{
        Env, JValue, JavaVM, jni_sig, jni_str,
        objects::{JByteArray, JClass, JObject, JString},
        sys::{jbyte, jstring},
    };

    use super::{is_android_content_uri, parse_picker_result};

    pub fn pick_backup_folder() -> anyhow::Result<Option<String>> {
        with_env(|env| {
            let activity_class = main_activity_class(env)?;
            let selected = env.call_static_method(
                &activity_class,
                jni_str!("pickBackupDirectoryBlocking"),
                jni_sig!("()Ljava/lang/String;"),
                &[],
            );
            check_exception(env).context("Android call pickBackupDirectoryBlocking failed")?;
            let selected = selected?.l()?;
            if selected.is_null() {
                return Ok(None);
            }
            let selected = unsafe { JString::from_raw(env, selected.into_raw() as jstring) };
            let selected = selected.try_to_string(env)?;
            parse_picker_result(&selected)
        })
    }

    pub fn read_content_uri(uri: &str) -> anyhow::Result<Vec<u8>> {
        with_env(|env| {
            ensure!(
                is_android_content_uri(uri),
                "Android URI reader only accepts content:// URIs"
            );
            let uri = env.new_string(uri)?;
            let activity_class = main_activity_class(env)?;
            let bytes = env.call_static_method(
                &activity_class,
                jni_str!("readUriBytes"),
                jni_sig!("(Ljava/lang/String;)[B"),
                &[JValue::Object(uri.as_ref())],
            );
            check_exception(env).context("Android call readUriBytes failed")?;
            let bytes = bytes?.l()?;
            if bytes.is_null() {
                bail!("Android content URI returned no bytes");
            }
            let bytes = unsafe { JByteArray::from_raw(env, bytes.into_raw().cast()) };
            let len = bytes.len(env)?;
            let mut data = vec![0_i8; len];
            bytes.get_region(env, 0, &mut data)?;
            Ok(data.into_iter().map(|byte| byte as u8).collect())
        })
    }

    pub fn write_backup_file(
        root_uri: &str,
        relative_path: &str,
        bytes: &[u8],
    ) -> anyhow::Result<()> {
        with_env(|env| {
            let root_uri = env.new_string(root_uri)?;
            let relative_path = env.new_string(relative_path)?;
            let array = env.new_byte_array(bytes.len())?;
            let signed = bytes.iter().map(|byte| *byte as jbyte).collect::<Vec<_>>();
            array.set_region(env, 0, &signed)?;
            let activity_class = main_activity_class(env)?;
            let result = env.call_static_method(
                &activity_class,
                jni_str!("writeBackupFile"),
                jni_sig!("(Ljava/lang/String;Ljava/lang/String;[B)V"),
                &[
                    JValue::Object(root_uri.as_ref()),
                    JValue::Object(relative_path.as_ref()),
                    JValue::Object(array.as_ref()),
                ],
            );
            check_exception(env).context("Android SAF backup writer failed")?;
            result?;
            Ok(())
        })
    }

    fn with_env<T>(f: impl FnOnce(&mut Env<'_>) -> anyhow::Result<T>) -> anyhow::Result<T> {
        let context = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(context.vm().cast()) };
        vm.attach_current_thread(f)
    }

    fn main_activity_class<'local>(env: &mut Env<'local>) -> anyhow::Result<JClass<'local>> {
        let context = ndk_context::android_context();
        let borrowed = unsafe { JObject::from_raw(env, context.context().cast()) };
        let activity = env.new_local_ref(&borrowed)?;
        let class = env
            .call_method(&activity, jni_str!("getClass"), jni_sig!("()Ljava/lang/Class;"), &[])?
            .l()?;
        if class.is_null() {
            bail!("Android activity class is not available");
        }
        Ok(unsafe { JClass::from_raw(env, class.into_raw().cast()) })
    }

    fn check_exception(env: &mut Env<'_>) -> anyhow::Result<()> {
        if !env.exception_check() {
            return Ok(());
        }
        let throwable = env
            .exception_occurred()
            .ok_or_else(|| anyhow::anyhow!("Java exception occurred but could not be retrieved"))?;
        env.exception_clear();
        let message = env
            .call_method(&throwable, jni_str!("toString"), jni_sig!("()Ljava/lang/String;"), &[])
            .ok()
            .and_then(|value| value.l().ok())
            .and_then(|object| {
                if object.is_null() {
                    None
                } else {
                    let message =
                        unsafe { JString::from_raw(env, object.into_raw() as jstring) };
                    message.try_to_string(env).ok()
                }
            })
            .unwrap_or_else(|| "Java exception".to_owned());
        bail!("{message}")
    }
}

#[cfg(not(target_os = "android"))]
mod imp {
    pub fn pick_backup_folder() -> anyhow::Result<Option<String>> {
        anyhow::bail!("Android folder picker is only available on Android")
    }

    pub fn read_content_uri(_uri: &str) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("Android content URI reader is only available on Android")
    }

    pub fn write_backup_file(
        _root_uri: &str,
        _relative_path: &str,
        _bytes: &[u8],
    ) -> anyhow::Result<()> {
        anyhow::bail!("Android SAF backup writer is only available on Android")
    }
}

pub use imp::*;

#[cfg(any(target_os = "android", test))]
const PICKER_ERROR_PREFIX: &str = "__ALPHAGUI_ERROR__:";

#[cfg(any(target_os = "android", test))]
pub fn parse_picker_result(value: &str) -> anyhow::Result<Option<String>> {
    if value.is_empty() {
        return Ok(None);
    }
    if let Some(message) = value.strip_prefix(PICKER_ERROR_PREFIX) {
        anyhow::bail!("{message}");
    }
    Ok(Some(value.to_owned()))
}

pub fn is_android_content_uri(value: &str) -> bool {
    value.starts_with("content://")
}

#[cfg(test)]
mod tests {
    use super::{PICKER_ERROR_PREFIX, is_android_content_uri, parse_picker_result};

    #[test]
    fn empty_picker_result_is_cancel() {
        assert_eq!(parse_picker_result("").unwrap(), None);
    }

    #[test]
    fn picker_error_marker_becomes_error() {
        let error = parse_picker_result(&format!("{PICKER_ERROR_PREFIX}folder picker failed"))
            .unwrap_err();
        assert!(error.to_string().contains("folder picker failed"));
    }

    #[test]
    fn content_uri_detection_is_strict() {
        assert!(is_android_content_uri("content://com.android.providers/foo"));
        assert!(!is_android_content_uri("/sdcard/Download/foo.os3kapp"));
    }
}
