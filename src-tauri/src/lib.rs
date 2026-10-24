use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindow, WindowEvent,
};
use tauri_plugin_global_shortcut::ShortcutState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DockSide {
    Right,
    Left,
}

static CURRENT_DOCK_SIDE: Mutex<DockSide> = Mutex::new(DockSide::Right);
static IS_WINDOW_OPEN: AtomicBool = AtomicBool::new(true);

#[cfg(target_os = "macos")]
unsafe fn get_or_register_orbit_panel_class() -> *mut std::ffi::c_void {
    use objc2::class;
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use std::ffi::c_void;

    unsafe extern "C" {
        fn objc_getClass(name: *const u8) -> *mut c_void;
        fn objc_allocateClassPair(superclass: *mut c_void, name: *const u8, extra_bytes: usize) -> *mut c_void;
        fn objc_registerClassPair(cls: *mut c_void);
        fn class_addMethod(cls: *mut c_void, name: *mut c_void, imp: unsafe extern "C" fn(), types: *const u8) -> bool;
        fn sel_registerName(name: *const u8) -> *mut c_void;
    }

    unsafe {
        let class_name = b"OrbitPanel\0".as_ptr();
        let existing = objc_getClass(class_name);
        if !existing.is_null() {
            return existing;
        }

        let superclass = objc_getClass(b"NSPanel\0".as_ptr());
        let new_class = objc_allocateClassPair(superclass, class_name, 0);
        if !new_class.is_null() {
            unsafe extern "C" fn return_yes(_this: *mut c_void, _cmd: *mut c_void) -> bool {
                true
            }
            unsafe extern "C" fn return_yes_first_mouse(_this: *mut c_void, _cmd: *mut c_void, _event: *mut c_void) -> bool {
                true
            }
            unsafe extern "C" fn orbit_send_event(this: *mut AnyObject, _cmd: *mut c_void, event: *mut AnyObject) {
                let event_type: usize = msg_send![event, type];
                // 1 = LeftMouseDown, 3 = RightMouseDown, 25 = OtherMouseDown
                if event_type == 1 || event_type == 3 || event_type == 25 {
                    let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                    let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
                    let _: () = msg_send![this, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()];
                }
                let superclass = class!(NSPanel);
                let _: () = msg_send![super(this, superclass), sendEvent: event];
            }
            unsafe extern "C" fn orbit_become_key_window(this: *mut AnyObject, _cmd: *mut c_void) {
                let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
                let superclass = class!(NSPanel);
                let _: () = msg_send![super(this, superclass), becomeKeyWindow];
            }

            let types_bool = b"c@:\0".as_ptr();
            let types_first_mouse = b"c@:@\0".as_ptr();
            let types_send_event = b"v@:@\0".as_ptr();
            let types_become_key = b"v@:\0".as_ptr();

            let imp_bool: unsafe extern "C" fn() = std::mem::transmute(return_yes as *const () as usize);
            let imp_first_mouse: unsafe extern "C" fn() = std::mem::transmute(return_yes_first_mouse as *const () as usize);
            let imp_send_event: unsafe extern "C" fn() = std::mem::transmute(orbit_send_event as *const () as usize);
            let imp_become_key: unsafe extern "C" fn() = std::mem::transmute(orbit_become_key_window as *const () as usize);

            class_addMethod(new_class, sel_registerName(b"canBecomeKeyWindow\0".as_ptr()), imp_bool, types_bool);
            class_addMethod(new_class, sel_registerName(b"canBecomeMainWindow\0".as_ptr()), imp_bool, types_bool);
            class_addMethod(new_class, sel_registerName(b"needsPanelToBecomeKey\0".as_ptr()), imp_bool, types_bool);
            class_addMethod(new_class, sel_registerName(b"acceptsFirstMouse:\0".as_ptr()), imp_first_mouse, types_first_mouse);
            class_addMethod(new_class, sel_registerName(b"sendEvent:\0".as_ptr()), imp_send_event, types_send_event);
            class_addMethod(new_class, sel_registerName(b"becomeKeyWindow\0".as_ptr()), imp_become_key, types_become_key);

            objc_registerClassPair(new_class);
            new_class
        } else {
            superclass
        }
    }
}

#[cfg(target_os = "macos")]
fn setup_macos_auxiliary_window(window: &WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    if let Ok(ptr) = window.ns_window() {
        unsafe {
            let ns_window = ptr as *mut AnyObject;

            // 1. Convert to OrbitPanel (a specialized NSPanel subclass that enables key window & typing)
            let orbit_panel_class = get_or_register_orbit_panel_class();
            unsafe extern "C" {
                fn object_setClass(obj: *mut std::ffi::c_void, cls: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
            }
            let _ = object_setClass(ptr.cast(), orbit_panel_class);

            // 2. Completely remove and hide standard window traffic light buttons (close=0, miniaturize=1, zoom=2)
            for button_type in 0isize..=2isize {
                let btn: *mut AnyObject = msg_send![ns_window, standardWindowButton: button_type];
                if !btn.is_null() {
                    let _: () = msg_send![btn, setHidden: true];
                }
            }

            // 3. Set NSPanel floating & key properties
            let _: () = msg_send![ns_window, setFloatingPanel: true];
            let _: () = msg_send![ns_window, setHidesOnDeactivate: false];
            let _: () = msg_send![ns_window, setBecomesKeyOnlyIfNeeded: false];

            // 4. Set collection behaviors:
            // CanJoinAllSpaces (1 << 0 = 1) | FullScreenAuxiliary (1 << 8 = 256) = 257
            let behavior: usize = 257;
            let _: () = msg_send![ns_window, setCollectionBehavior: behavior];

            // 5. Set window level to NSMainMenuWindowLevel + 1 (25) so it floats above full-screen windows
            let level: isize = 25;
            let _: () = msg_send![ns_window, setLevel: level];

            // 6. Force order to front on active space
            let _: () = msg_send![ns_window, orderFrontRegardless];
        }
    }
}

fn slide_in(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "macos")]
        {
            use objc2::class;
            use objc2::msg_send;
            use objc2::runtime::AnyObject;
            use objc2_foundation::{NSPoint, NSRect, NSSize};

            if let Ok(ptr) = window.ns_window() {
                unsafe {
                    let ns_window = ptr as *mut AnyObject;
                    setup_macos_auxiliary_window(&window);

                    let main_s: *mut AnyObject = msg_send![class!(NSScreen), mainScreen];
                    let screen_to_use = if !main_s.is_null() {
                        main_s
                    } else {
                        let screen: *mut AnyObject = msg_send![ns_window, screen];
                        screen
                    };

                    if !screen_to_use.is_null() {
                        let frame: NSRect = msg_send![screen_to_use, frame];
                        let visible_frame: NSRect = msg_send![screen_to_use, visibleFrame];

                        let side = *CURRENT_DOCK_SIDE.lock().unwrap();
                        let sidebar_w: f64 = 420.0;
                        let sidebar_h: f64 = visible_frame.size.height;
                        let y: f64 = visible_frame.origin.y;

                        let (x_closed, x_open) = match side {
                            DockSide::Right => (
                                frame.origin.x + frame.size.width,
                                frame.origin.x + frame.size.width - sidebar_w,
                            ),
                            DockSide::Left => (
                                frame.origin.x - sidebar_w,
                                frame.origin.x,
                            ),
                        };

                        let closed_rect = NSRect::new(NSPoint::new(x_closed, y), NSSize::new(sidebar_w, sidebar_h));
                        let open_rect = NSRect::new(NSPoint::new(x_open, y), NSSize::new(sidebar_w, sidebar_h));

                        // 1. Instantly place window in closed position off-screen
                        let _: () = msg_send![ns_window, setFrame: closed_rect, display: false];

                        // 2. Make visible and order front
                        let _: () = msg_send![ns_window, orderFrontRegardless];

                        // 3. Animate smoothly to open position (slide in!)
                        let animator: *mut AnyObject = msg_send![ns_window, animator];
                        let _: () = msg_send![animator, setFrame: open_rect, display: true];

                        // 4. Activate app and make window key so keyboard events/typing work immediately
                        let _: () = msg_send![ns_window, makeKeyWindow];
                        let ns_app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                        let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
                        let _ = window.set_focus();
                    }
                }
            }
        }

        IS_WINDOW_OPEN.store(true, Ordering::SeqCst);
    }
}

fn slide_out(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "macos")]
        {
            use objc2::class;
            use objc2::msg_send;
            use objc2::runtime::AnyObject;
            use objc2_foundation::{NSPoint, NSRect, NSSize};

            if let Ok(ptr) = window.ns_window() {
                unsafe {
                    let ns_window = ptr as *mut AnyObject;
                    let main_s: *mut AnyObject = msg_send![class!(NSScreen), mainScreen];
                    let screen_to_use = if !main_s.is_null() {
                        main_s
                    } else {
                        let screen: *mut AnyObject = msg_send![ns_window, screen];
                        screen
                    };

                    if !screen_to_use.is_null() {
                        let frame: NSRect = msg_send![screen_to_use, frame];
                        let visible_frame: NSRect = msg_send![screen_to_use, visibleFrame];

                        let side = *CURRENT_DOCK_SIDE.lock().unwrap();
                        let sidebar_w: f64 = 420.0;
                        let sidebar_h: f64 = visible_frame.size.height;
                        let y: f64 = visible_frame.origin.y;

                        let x_closed = match side {
                            DockSide::Right => frame.origin.x + frame.size.width,
                            DockSide::Left => frame.origin.x - sidebar_w,
                        };

                        let closed_rect = NSRect::new(NSPoint::new(x_closed, y), NSSize::new(sidebar_w, sidebar_h));

                        // Animate smoothly to closed position off-screen (slide out!)
                        let animator: *mut AnyObject = msg_send![ns_window, animator];
                        let _: () = msg_send![animator, setFrame: closed_rect, display: true];
                    }
                }
            }
        }

        IS_WINDOW_OPEN.store(false, Ordering::SeqCst);
    }
}

fn toggle_window(app: &AppHandle) {
    let is_open = IS_WINDOW_OPEN.load(Ordering::SeqCst);
    if is_open {
        slide_out(app);
    } else {
        slide_in(app);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Configure Main Window positioning on startup with slide-in
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "macos")]
                setup_macos_auxiliary_window(&window);

                slide_in(app.handle());

                // Intercept close requests and slide out instead
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        slide_out(&app_handle);
                    }
                });
            }

            // Setup Global Shortcuts (Option+Space and Cmd+Shift+A)
            #[cfg(desktop)]
            {
                let _ = app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts(["alt+space", "super+shift+a"])?
                        .with_handler(|app, _shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                toggle_window(app);
                            }
                        })
                        .build(),
                );
            }

            // Setup System Tray Menu (macOS Menu Bar)
            let toggle_item = MenuItemBuilder::with_id("toggle", "Toggle Orbit (⌥ Space / ⌘⇧A)").build(app)?;
            let always_on_top_item = CheckMenuItemBuilder::with_id("always_on_top", "Always on Top")
                .checked(true)
                .build(app)?;
            let dock_right_item = MenuItemBuilder::with_id("dock_right", "Dock to Right Edge").build(app)?;
            let dock_left_item = MenuItemBuilder::with_id("dock_left", "Dock to Left Edge").build(app)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let reset_home_item = MenuItemBuilder::with_id("reset_home", "Restart to Antigravity Home").build(app)?;
            let reload_item = MenuItemBuilder::with_id("reload", "Reload Page (⌘ R)").build(app)?;
            let relaunch_item = MenuItemBuilder::with_id("relaunch", "Relaunch Orbit").build(app)?;
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
                    &reset_home_item,
                    &reload_item,
                    &relaunch_item,
                    &open_browser_item,
                    &sep2,
                    &quit_item,
                ])
                .build()?;

            let default_icon = app.default_window_icon().cloned().unwrap();
            let _tray = TrayIconBuilder::new()
                .icon(default_icon)
                .icon_as_template(true)
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
                            *CURRENT_DOCK_SIDE.lock().unwrap() = DockSide::Right;
                            slide_in(app);
                        }
                        "dock_left" => {
                            *CURRENT_DOCK_SIDE.lock().unwrap() = DockSide::Left;
                            slide_in(app);
                        }
                        "reset_home" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.eval("window.location.href = 'https://antigravity.google.com';");
                                slide_in(app);
                            }
                        }
                        "reload" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.eval("window.location.reload();");
                            }
                        }
                        "relaunch" => {
                            app.restart();
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
