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

#[cfg(target_os = "windows")]
const WM_FOCUS_CHECKMAT: u32 = 0x8000 + 101;

fn get_dir_hash() -> String {
    use sha2::{Digest, Sha256};
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_string_lossy().to_string()))
        .unwrap_or_else(|| "checkmat_dir".to_string());
    let mut hasher = Sha256::new();
    hasher.update(exe_dir.to_lowercase().as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex[..16].to_string()
}

#[cfg(target_os = "windows")]
fn ensure_single_instance_per_folder(dir_hash: &str) {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AllowSetForegroundWindow, FindWindowW, GetWindowThreadProcessId, PostMessageW,
    };

    let mutex_name: Vec<u16> = format!("CheckMat_Mtx_{}\0", dir_hash).encode_utf16().collect();
    let handle = unsafe { CreateMutexW(std::ptr::null(), 1, mutex_name.as_ptr()) };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        let class_name: Vec<u16> = format!("CheckMat_Cls_{}\0", dir_hash).encode_utf16().collect();
        let win_name: Vec<u16> = format!("CheckMat_Win_{}\0", dir_hash).encode_utf16().collect();

        // Cari window host instance pertama di folder ini, izinkan bawa ke depan, dan kirim pesan fokus
        for _ in 0..15 {
            let hwnd = unsafe { FindWindowW(class_name.as_ptr(), win_name.as_ptr()) };
            if !hwnd.is_null() {
                let mut pid = 0;
                unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
                if pid != 0 {
                    unsafe { AllowSetForegroundWindow(pid) };
                }
                unsafe { PostMessageW(hwnd, WM_FOCUS_CHECKMAT, 0, 0) };
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        std::process::exit(0);
    }
    let _ = handle;
}

#[cfg(target_os = "windows")]
fn setup_instance_listener(app_handle: tauri::AppHandle, dir_hash: &str) {
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, RegisterClassExW, WNDCLASSEXW, WS_OVERLAPPED,
    };

    static APP_HANDLE: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();
    let _ = APP_HANDLE.set(app_handle);

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_FOCUS_CHECKMAT {
            if let Some(app) = APP_HANDLE.get() {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.unminimize();
                    let _ = w.set_always_on_top(true);
                    let _ = w.set_focus();
                    let _ = w.set_always_on_top(false);
                }
            }
            return 0;
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    let class_name: Vec<u16> = format!("CheckMat_Cls_{}\0", dir_hash).encode_utf16().collect();
    let win_name: Vec<u16> = format!("CheckMat_Win_{}\0", dir_hash).encode_utf16().collect();

    unsafe {
        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(std::ptr::null()),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wnd_class);
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            win_name.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            GetModuleHandleW(std::ptr::null()),
            std::ptr::null(),
        );
    }
}

fn main() {
    let dir_hash = get_dir_hash();

    #[cfg(target_os = "windows")]
    ensure_single_instance_per_folder(&dir_hash);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            #[cfg(target_os = "windows")]
            setup_instance_listener(app.handle().clone(), &dir_hash);

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
            db::set_task_status,
            db::set_task_status_batch,
            db::soft_delete,
            db::soft_delete_batch,
            db::restore,
            db::restore_batch,
            db::hard_delete,
            db::hard_delete_batch,
            db::duplicate_tasks_batch,
            db::duplicate_task,
            db::duplicate_task_to_categories,
            db::duplicate_tasks_to_categories_batch,
            db::set_category_batch,
            db::list_task_events,
            db::list_subtasks,
            db::add_subtask,
            db::set_subtask_done,
            db::update_subtask_title,
            db::delete_subtask,
            db::reorder_subtasks,
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
