#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(db::Db::new())
        .invoke_handler(tauri::generate_handler![
            db::list_tasks,
            db::add_task,
            db::update_task,
            db::set_done,
            db::soft_delete,
            db::restore,
            db::hard_delete,
            db::reorder,
            db::list_categories,
            db::list_projects,
            db::add_project,
            db::update_project,
            db::delete_project,
            db::add_category,
            db::rename_category,
            db::set_category_color,
            db::delete_category,
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
