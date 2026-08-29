mod db;
mod events;
mod scheduler;
mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(db::DatabaseState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            db::get_data_status,
            db::configure_data_directory,
            db::open_data_directory,
            events::fetch_events,
            events::fetch_month_events,
            events::create_event,
            events::update_event,
            events::delete_event,
            events::toggle_complete,
        ])
        .setup(|app| {
            tauri::async_runtime::block_on(db::initialize(app.handle()));
            tray::setup_tray(app.handle())?;

            scheduler::start_reminder_scheduler(app.handle().clone());

            let window = app
                .get_webview_window("main")
                .ok_or("Main window 'main' not found")?;

            #[cfg(target_os = "windows")]
            {
                let _ = window.set_always_on_top(false);
                let _ = window.set_always_on_bottom(true);
            }
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
