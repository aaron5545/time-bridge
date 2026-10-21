// Time Bridge: a menu bar clock for Budapest ↔ Bali (and friends).
// Click the menu bar icon to open the clock right under it; click anywhere else and it tucks away.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindow, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_positioner::{Position, WindowExt};

/// When the popup last hid itself. Clicking the menu bar icon while the popup is open
/// first takes focus away from it (which hides it), so without this the same click
/// would immediately reopen it.
struct LastHidden(Mutex<Option<Instant>>);

/// Snap the clock back to the current time (the page follows the real clock from there).
fn jump_to_now(w: &WebviewWindow) {
    let _ = w.eval("window.__tbNow && window.__tbNow()");
}

fn hide_popup(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
            jump_to_now(&w); // ready at the current time for the next open
            *app.state::<LastHidden>().0.lock().unwrap() = Some(Instant::now());
        }
    }
}

fn toggle_popup(app: &AppHandle) {
    let Some(w) = app.get_webview_window("main") else { return };
    if w.is_visible().unwrap_or(false) {
        hide_popup(app);
        return;
    }
    let just_hidden = app
        .state::<LastHidden>()
        .0
        .lock()
        .unwrap()
        .map(|t| t.elapsed() < Duration::from_millis(250))
        .unwrap_or(false);
    if just_hidden {
        return;
    }
    jump_to_now(&w);
    let _ = w.move_window(Position::TrayBottomCenter);
    let _ = w.show();
    let _ = w.set_focus();
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .manage(LastHidden(Mutex::new(None)))
        .setup(|app| {
            // Menu bar only: no Dock icon, no app switcher entry.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let starts_at_login = app.autolaunch().is_enabled().unwrap_or(false);
            let login = CheckMenuItem::with_id(app, "login", "Open at Login", true, starts_at_login, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Time Bridge", true, Some("CmdOrCtrl+Q"))?;
            let sep = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&login, &sep, &quit])?;

            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;
            let login_item = login.clone();

            TrayIconBuilder::with_id("main")
                .icon(icon)
                .icon_as_template(true) // follows the menu bar's light/dark colour
                .tooltip("Time Bridge")
                .menu(&menu)
                .show_menu_on_left_click(false) // left click = clock, right click = menu
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "login" => {
                        let al = app.autolaunch();
                        let _ = if al.is_enabled().unwrap_or(false) { al.disable() } else { al.enable() };
                        let _ = login_item.set_checked(al.is_enabled().unwrap_or(false));
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_popup(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Click anywhere outside the clock and it tucks away.
            if let WindowEvent::Focused(false) = event {
                hide_popup(window.app_handle());
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Time Bridge");
}
