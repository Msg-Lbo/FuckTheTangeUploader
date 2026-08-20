mod commands;
mod config;
mod genata;
mod sftp;
mod tange;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_updater::Builder::new()
                .header("User-Agent", "Firmware-Pusher-Updater")
                .expect("设置更新请求头失败")
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::upload_sftp,
            commands::sftp_list_files,
            commands::genata_login,
            commands::genata_list_files,
            commands::genata_list_versions,
            commands::genata_insert_version,
            commands::tange_list_versions,
            commands::tange_add_rule,
            commands::tange_remove_rule,
            commands::tange_delete_version,
            commands::tange_publish_version,
        ])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}
