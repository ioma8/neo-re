mod android_saf;
mod commands;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_device,
            commands::switch_hid_to_direct,
            commands::get_inventory,
            commands::backup_file,
            commands::backup_all_files,
            commands::backup_everything,
            commands::default_backup_root,
            commands::runtime_platform,
            commands::debug_bypass_enabled,
            commands::pick_backup_folder,
            commands::list_bundled_applets,
            commands::install_alpha_usb,
            commands::flash_applets,
            commands::restore_original_stock_applets,
            commands::flash_system_image,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run AlphaGUI");
}
