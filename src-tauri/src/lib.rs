use tauri::{
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent,
};
use tauri_plugin_global_shortcut::{ShortcutState};

#[derive(Clone, Copy)]
enum DockSide {
    Right,
    Left,
}

fn dock_window(window: &WebviewWindow, side: DockSide) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let monitor_pos = monitor.position();
        let monitor_size = monitor.size();
        let scale_factor = monitor.scale_factor();

        let sidebar_width = (420.0 * scale_factor).round() as u32;
        let sidebar_height = monitor_size.height;

        let x = match side {
            DockSide::Right => monitor_pos.x + monitor_size.width as i32 - sidebar_width as i32,
            DockSide::Left => monitor_pos.x,
        };
        let y = monitor_pos.y;

        let _ = window.set_size(PhysicalSize::new(sidebar_width, sidebar_height));
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        let is_focused = window.is_focused().unwrap_or(false);

        if is_visible && is_focused {
            let _ = window.hide();
        } else {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Configure Main Window positioning on startup
            if let Some(window) = app.get_webview_window("main") {
                dock_window(&window, DockSide::Right);
                let _ = window.set_always_on_top(true);

                // Hide instead of destroy when close button (red traffic light) is clicked
                let win_clone = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            // Setup Global Shortcut (Option + Space / Alt + Space)
            #[cfg(desktop)]
            {
                let _ = app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts(["alt+space"])?
                        .with_handler(|app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                toggle_window(app);
                            }
                        })
                        .build(),
                );
            }

            // Setup System Tray Menu (macOS Menu Bar)
            let toggle_item = MenuItemBuilder::with_id("toggle", "Toggle Orbit (⌥ Space)").build(app)?;
            let always_on_top_item = CheckMenuItemBuilder::with_id("always_on_top", "Always on Top")
                .checked(true)
                .build(app)?;
            let dock_right_item = MenuItemBuilder::with_id("dock_right", "Dock to Right Edge").build(app)?;
            let dock_left_item = MenuItemBuilder::with_id("dock_left", "Dock to Left Edge").build(app)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let reload_item = MenuItemBuilder::with_id("reload", "Reload Antigravity (⌘ R)").build(app)?;
            let open_browser_item = MenuItemBuilder::with_id("open_browser", "Open in Web Browser").build(app)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit Orbit").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[
                    &toggle_item,
                    &always_on_top_item,
                    &dock_right_item,
                    &dock_left_item,
                    &sep1,
                    &reload_item,
                    &open_browser_item,
                    &sep2,
                    &quit_item,
                ])
                .build()?;

            let default_icon = app.default_window_icon().cloned().unwrap();
            let _tray = TrayIconBuilder::new()
                .icon(default_icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "toggle" => {
                            toggle_window(app);
                        }
                        "always_on_top" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if let Ok(is_checked) = always_on_top_item.is_checked() {
                                    let new_state = !is_checked;
                                    let _ = always_on_top_item.set_checked(new_state);
                                    let _ = window.set_always_on_top(new_state);
                                }
                            }
                        }
                        "dock_right" => {
                            if let Some(window) = app.get_webview_window("main") {
                                dock_window(&window, DockSide::Right);
                            }
                        }
                        "dock_left" => {
                            if let Some(window) = app.get_webview_window("main") {
                                dock_window(&window, DockSide::Left);
                            }
                        }
                        "reload" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.eval("window.location.reload();");
                            }
                        }
                        "open_browser" => {
                            let _ = tauri_plugin_opener::open_url("https://antigravity.google.com", None::<&str>);
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        toggle_window(app);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
