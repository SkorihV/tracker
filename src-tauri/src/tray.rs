//! Значок в трее: число идущих задач в подсказке, сворачивание в трей
//! при закрытии окна и напоминания о «забытых» таймерах.

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Window, WindowEvent};
use tauri_plugin_notification::NotificationExt;
use tracker_core::dt::{fmt_td, now_naive};

use crate::commands::AppStore;

const TRAY_ID: &str = "main";
const TICK: Duration = Duration::from_secs(30);

fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Открыть", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Time Tracker")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;

    let handle = app.clone();
    thread::spawn(move || watch(handle));
    Ok(())
}

/// Закрытие окна сворачивает в трей, если это включено в настройках.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        let app = window.app_handle();
        let minimize = app
            .state::<AppStore>()
            .lock()
            .map(|st| st.settings.minimize_to_tray)
            .unwrap_or(false);
        if minimize {
            api.prevent_close();
            let _ = window.hide();
        }
    }
}

/// Фоновый цикл: обновляет подсказку трея и шлёт напоминания.
fn watch(app: AppHandle) {
    // id задачи -> сколько порогов напоминания уже отправлено для её текущего диапазона
    let mut reminded: HashMap<String, i64> = HashMap::new();
    loop {
        thread::sleep(TICK);
        let Some(store) = app.try_state::<AppStore>() else {
            continue;
        };
        let (running, due) = {
            let Ok(st) = store.lock() else { continue };
            let elapsed = st.running_elapsed(now_naive());
            let threshold = i64::from(st.settings.reminder_hours) * 3600;
            reminded.retain(|id, _| elapsed.iter().any(|(t, _)| &t.task_id == id));
            let mut due: Vec<(String, i64)> = Vec::new();
            if threshold > 0 {
                for (t, secs) in &elapsed {
                    let level = secs / threshold;
                    let seen = reminded.entry(t.task_id.clone()).or_insert(0);
                    if level > *seen {
                        *seen = level;
                        due.push((describe(t), *secs));
                    }
                }
            }
            (elapsed.len(), due)
        };

        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let tip = if running == 0 {
                "Time Tracker".to_string()
            } else {
                format!("Time Tracker — идёт задач: {running}")
            };
            let _ = tray.set_tooltip(Some(tip));
        }
        for (what, secs) in due {
            let _ = app
                .notification()
                .builder()
                .title("Таймер всё ещё идёт")
                .body(format!("{what} — уже {}", fmt_td(secs as f64)))
                .show();
        }
    }
}

fn describe(t: &tracker_core::models::Task) -> String {
    let order = t.orders.first().map(String::as_str).unwrap_or("");
    if order.is_empty() {
        t.task_id.clone()
    } else {
        format!("Заявка {order}")
    }
}
