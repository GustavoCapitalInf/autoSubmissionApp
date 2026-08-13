//! CapDesk Submission Desk — Tauri shell.
//!
//! Thin client over the hosted `conduit-server`: commands proxy the HTTP API
//! (desk API key attached), an SSE relay re-emits live server events as Tauri
//! events (no polling) and pops the desktop notification when a new deal
//! arrives. When configured against a local server (dev), the server binary
//! is auto-spawned if it isn't running.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use conduit_core::config::DeskConfig;
use futures_util::StreamExt;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};

struct Desk {
    client: reqwest::Client,
    cfg: DeskConfig,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn api_client(cfg: &DeskConfig) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Ok(v) = reqwest::header::HeaderValue::from_str(&cfg.desk_api_key) {
        headers.insert("x-api-key", v);
    }
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("http client")
}

/// Initial load: full deal list plus stats. Retries briefly so a cold start
/// (local dev server still being spawned) doesn't greet reviewers with an
/// error. `fileKey` lets the WebView fetch PDFs/images directly.
#[tauri::command]
async fn desk_state(state: State<'_, Desk>) -> Result<Value, String> {
    let base = &state.cfg.server_url;
    let mut last_err = String::new();
    for _ in 0..20 {
        match state.client.get(format!("{base}/api/deals")).send().await {
            Ok(resp) if resp.status().is_success() => {
                let deals: Value = resp.json().await.map_err(err)?;
                let stats: Value = state
                    .client
                    .get(format!("{base}/api/stats"))
                    .send()
                    .await
                    .map_err(err)?
                    .json()
                    .await
                    .map_err(err)?;
                return Ok(json!({
                    "baseUrl": base,
                    "fileKey": state.cfg.desk_api_key,
                    "deals": deals["deals"],
                    "stats": stats,
                }));
            }
            Ok(resp) if resp.status() == reqwest::StatusCode::UNAUTHORIZED => {
                return Err(format!(
                    "The submission service at {base} rejected the desk API key. \
                     Set CONDUIT_DESK_KEY (or desk.toml) to the key configured on the server."
                ));
            }
            Ok(resp) => last_err = format!("service answered {}", resp.status()),
            Err(e) => last_err = e.to_string(),
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    Err(format!(
        "The CapDesk submission service at {base} is not reachable ({last_err})."
    ))
}

async fn post_decision(state: &Desk, id: i64, action: &str, body: Value) -> Result<Value, String> {
    let resp = state
        .client
        .post(format!("{}/api/deals/{id}/{action}", state.cfg.server_url))
        .json(&body)
        .send()
        .await
        .map_err(err)?;
    let status = resp.status();
    let payload: Value = resp.json().await.map_err(err)?;
    if status.is_success() {
        Ok(payload)
    } else {
        Err(payload["error"]
            .as_str()
            .unwrap_or("the submission service rejected the request")
            .to_string())
    }
}

/// UI settings live in a small JSON file in the app's data directory so a
/// preference (currently just the theme) survives a WebView data reset.
fn ui_settings_path() -> Option<std::path::PathBuf> {
    conduit_core::data_dir().ok().map(|dir| dir.join("ui-settings.json"))
}

#[tauri::command]
fn get_theme() -> Option<String> {
    let text = std::fs::read_to_string(ui_settings_path()?).ok()?;
    let settings: Value = serde_json::from_str(&text).ok()?;
    match settings["theme"].as_str() {
        Some(theme @ ("light" | "dark")) => Some(theme.to_string()),
        _ => None,
    }
}

#[tauri::command]
fn set_theme(theme: String) -> Result<(), String> {
    if theme != "light" && theme != "dark" {
        return Err(format!("unknown theme {theme:?}"));
    }
    let path = ui_settings_path().ok_or("could not resolve the data directory")?;
    let mut settings = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));
    settings["theme"] = json!(theme);
    std::fs::write(&path, serde_json::to_string_pretty(&settings).map_err(err)?).map_err(err)
}

#[tauri::command]
async fn approve_deal(state: State<'_, Desk>, id: i64, reviewer: String) -> Result<Value, String> {
    post_decision(&state, id, "approve", json!({ "reviewer": reviewer })).await
}

#[tauri::command]
async fn reject_deal(
    state: State<'_, Desk>,
    id: i64,
    reviewer: String,
    reason: Option<String>,
) -> Result<Value, String> {
    post_decision(&state, id, "reject", json!({ "reviewer": reviewer, "reason": reason })).await
}

/// Dev convenience: when pointed at a local server, health-check it and spawn
/// the sibling `conduit-server` binary if it's down. Hosted deployments skip
/// this entirely.
async fn ensure_local_server(client: &reqwest::Client, base: &str) {
    let healthy = |c: reqwest::Client, b: String| async move {
        c.get(format!("{b}/health"))
            .timeout(Duration::from_millis(800))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    };
    if healthy(client.clone(), base.to_string()).await {
        return;
    }
    let server_bin = std::env::current_exe().ok().and_then(|exe| {
        let name = if cfg!(windows) { "conduit-server.exe" } else { "conduit-server" };
        exe.parent().map(|dir| dir.join(name))
    });
    match server_bin {
        Some(bin) if bin.exists() => {
            match std::process::Command::new(&bin)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(_) => println!("spawned {}", bin.display()),
                Err(e) => eprintln!("failed to spawn {}: {e}", bin.display()),
            }
        }
        _ => eprintln!("conduit-server binary not found next to the app; start it manually"),
    }
    for _ in 0..20 {
        if healthy(client.clone(), base.to_string()).await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    eprintln!("conduit-server did not become healthy in time");
}

/// Reveal the desk window — also from the hidden-to-background state.
fn show_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

/// Bring the desk to the front and ask the WebView to open the deal's
/// review panel. Runs on notification click, i.e. any thread.
fn open_deal(app: &AppHandle, deal_id: i64) {
    show_main(app);
    let _ = app.emit("open-deal", deal_id);
}

/// Closing the window hides the desk instead of quitting; tell the reviewer
/// once per run so the disappearing window doesn't read as an exit.
fn notify_background_once() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static TOLD: AtomicBool = AtomicBool::new(false);
    if TOLD.swap(true, Ordering::Relaxed) {
        return;
    }
    const TITLE: &str = "CapDesk is still running";
    const BODY: &str =
        "New-deal notifications stay on. Reopen or quit from the tray icon.";
    std::thread::spawn(|| {
        #[cfg(not(windows))]
        let result = notify_rust::Notification::new()
            .appname("CapDesk")
            .summary(TITLE)
            .body(BODY)
            .show()
            .map(|_| ());
        #[cfg(windows)]
        let result = tauri_winrt_notification::Toast::new(
            tauri_winrt_notification::Toast::POWERSHELL_APP_ID,
        )
        .title(TITLE)
        .text1(BODY)
        .show();
        if let Err(e) = result {
            eprintln!("notification failed: {e}");
        }
    });
}

fn notify_new_deal(app: &AppHandle, deal: &Value) {
    let company = deal["company"].as_str().unwrap_or("New deal").to_string();
    let deal_id = deal["id"].as_i64().unwrap_or(0);
    let request = deal["request"].as_i64().unwrap_or(0);
    let lenders = deal["lenders"].as_array().map(|l| l.len()).unwrap_or(0);
    let body = if request > 0 {
        format!(
            "{company} — ${} requested · {lenders} lenders matched",
            format_thousands(request)
        )
    } else {
        format!("{company} — {lenders} lenders matched")
    };

    // Reviewers step away from the desk; a toast that auto-expires (4s on
    // some setups) gets missed. Keep it up until dismissed, and open the
    // deal when the notification is clicked.
    #[cfg(not(windows))]
    {
        let app = app.clone();
        std::thread::spawn(move || {
            let mut notification = notify_rust::Notification::new();
            notification
                .appname("CapDesk")
                .summary("New deal submitted for review")
                .body(&body)
                .action("default", "Open deal")
                .timeout(notify_rust::Timeout::Never);
            #[cfg(all(unix, not(target_os = "macos")))]
            notification.urgency(notify_rust::Urgency::Critical);
            match notification.show() {
                // Blocks this thread until the toast is acted on or closed.
                Ok(handle) => handle.wait_for_action(|action| {
                    if action == "default" {
                        open_deal(&app, deal_id);
                    }
                }),
                Err(e) => eprintln!("notification failed: {e}"),
            }
        });
    }

    #[cfg(windows)]
    {
        use tauri_winrt_notification::{Scenario, Toast};
        let app = app.clone();
        std::thread::spawn(move || {
            let result = Toast::new(Toast::POWERSHELL_APP_ID)
                .title("New deal submitted for review")
                .text1(&body)
                // Reminder scenario keeps the toast on screen until dismissed.
                .scenario(Scenario::Reminder)
                .on_activated(move |_action| {
                    open_deal(&app, deal_id);
                    Ok(())
                })
                .show();
            if let Err(e) = result {
                eprintln!("notification failed: {e}");
            }
        });
    }
}

fn format_thousands(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// Long-lived SSE subscription; each `data:` line is re-emitted to the
/// WebView as a `server-event`. New deals additionally trigger the desktop
/// notification. Reconnects with a short backoff.
async fn sse_relay(app: AppHandle, cfg: DeskConfig) {
    let client = api_client(&cfg);
    loop {
        if let Ok(resp) = client
            .get(format!("{}/api/events", cfg.server_url))
            .send()
            .await
        {
            let mut stream = resp.bytes_stream();
            let mut buf: Vec<u8> = Vec::new();
            while let Some(Ok(chunk)) = stream.next().await {
                buf.extend_from_slice(&chunk);
                while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    let line: Vec<u8> = buf.drain(..=pos).collect();
                    let line = String::from_utf8_lossy(&line);
                    if let Some(data) = line.trim_end().strip_prefix("data: ") {
                        if let Ok(value) = serde_json::from_str::<Value>(data) {
                            if value["type"] == "deal.created" {
                                notify_new_deal(&app, &value["deal"]);
                            }
                            let _ = app.emit("server-event", value);
                        }
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

fn main() {
    let data_dir = conduit_core::data_dir().ok();
    let cfg = DeskConfig::load(data_dir.as_deref());
    let desk = Desk {
        client: api_client(&cfg),
        cfg: cfg.clone(),
    };

    tauri::Builder::default()
        // Relaunching the exe while a hidden instance is in the background
        // must surface that instance, not start a second SSE relay.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .manage(desk)
        .invoke_handler(tauri::generate_handler![
            desk_state,
            approve_deal,
            reject_deal,
            get_theme,
            set_theme
        ])
        // Closing the window backgrounds the desk: the SSE relay keeps
        // running so new-deal notifications still fire. Quit is in the tray.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                notify_background_once();
            }
        })
        .setup(move |app| {
            let open = tauri::menu::MenuItem::with_id(app, "open", "Open CapDesk", true, None::<&str>)?;
            let quit = tauri::menu::MenuItem::with_id(app, "quit", "Quit CapDesk", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&open, &quit])?;
            let mut tray = tauri::tray::TrayIconBuilder::with_id("main")
                .tooltip("CapDesk — Submission Desk")
                .menu(&menu)
                // Left click opens the desk (Windows); Linux appindicators
                // only do menus, where Open sits one click away.
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            // A missing tray (bare Linux setups) shouldn't take the desk down;
            // without one, closing still backgrounds and notifications reopen.
            if let Err(e) = tray.build(app) {
                eprintln!("tray unavailable: {e}");
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if cfg.is_local() {
                    ensure_local_server(&api_client(&cfg), &cfg.server_url).await;
                }
                sse_relay(handle, cfg).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the CapDesk desk");
}
