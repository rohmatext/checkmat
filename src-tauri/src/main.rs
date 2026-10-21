#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod db;

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = url;
        Ok(())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_maximizable(false);
                let _ = w.set_max_size(Some(tauri::Size::Logical(tauri::LogicalSize {
                    width: 480.0,
                    height: 600.0,
                })));
            }
            Ok(())
        })
        .manage(db::Db::new())
        .invoke_handler(tauri::generate_handler![
            open_url,
            db::list_tasks,
            db::add_task,
            db::update_task,
            db::set_done,
            db::set_done_batch,
            db::soft_delete,
            db::soft_delete_batch,
            db::restore,
            db::restore_batch,
            db::hard_delete,
            db::hard_delete_batch,
            db::reorder,
            db::list_categories,
            db::list_projects,
            db::add_project,
            db::update_project,
            db::set_project_status,
            db::reorder_projects,
            db::delete_project,
            db::add_category,
            db::rename_category,
            db::set_category_color,
            db::delete_category,
            db::reorder_categories,
            db::export_csv,
            db::default_export_dir,
            db::write_csv_template,
            db::import_csv,
            db::get_lock_status,
            db::set_pin,
            db::verify_pin,
            db::clear_pin,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan CheckMat");
}
