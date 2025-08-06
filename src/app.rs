use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

// Import CSS styles
const LOGS_CSS: &str = include_str!("logs.css");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> JsValue;
}

// Helper function to check if we're in Tauri environment
fn is_tauri() -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &"__TAURI__".into()).ok())
        .map(|v| !v.is_undefined())
        .unwrap_or(false)
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["console"])]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "dialog"])]
    async fn open(options: JsValue) -> JsValue;

}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RustFsConfig {
    data_path: String,
    port: Option<u16>,
    host: Option<String>,
    access_key: Option<String>,
    secret_key: Option<String>,
    console_enable: bool,
}

impl Default for RustFsConfig {
    fn default() -> Self {
        Self {
            data_path: String::new(),
            port: Some(9000),
            host: Some("127.0.0.1".to_string()),
            access_key: Some("rustfsadmin".to_string()),
            secret_key: Some("rustfsadmin".to_string()),
            console_enable: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum LogType {
    App,
    RustFS,
}

#[component]
pub fn App() -> impl IntoView {
    let (config, set_config) = signal(RustFsConfig::default());
    let (status, set_status) = signal(String::new());
    let (is_running, set_is_running) = signal(false);
    let (show_secret, set_show_secret) = signal(false);
    let (app_logs, set_app_logs) = signal(Vec::<String>::new());
    let (rustfs_logs, set_rustfs_logs) = signal(Vec::<String>::new());
    let (current_log_type, set_current_log_type) = signal(LogType::App);
    let logs_ref = NodeRef::<leptos::html::Div>::new();

    let select_folder = move |_| {
        spawn_local(async move {
            let options = serde_wasm_bindgen::to_value(&serde_json::json!({
                "directory": true,
                "title": "Select RustFS Data Directory"
            }))
            .unwrap();

            if let Some(result) = open(options).await.as_string() {
                if !result.is_empty() {
                    set_config.update(|c| c.data_path = result);
                }
            }
        });
    };

    let add_app_log = move |msg: String| {
        set_app_logs.update(|logs| {
            logs.push(msg);
            if logs.len() > 100 {
                logs.remove(0);
            }
        });
    };

    let add_rustfs_log = move |msg: String| {
        set_rustfs_logs.update(|logs| {
            logs.push(msg);
            if logs.len() > 1000 {
                logs.remove(0);
            }
        });
    };

    add_app_log("[DEBUG] RustFS Launcher started111".to_string());
    add_rustfs_log("[DEBUG] RustFS Launcher started222".to_string());

    // Set up real-time event listeners when component mounts (only in Tauri environment)
    spawn_local(async move {
        // Check if we're in Tauri environment
        if !is_tauri() {
            add_app_log("[WARN] Not running in Tauri environment - logs disabled".to_string());
            return;
        }

        add_app_log("[DEBUG] Setting up real-time log listeners...".to_string());

        const APP_LOG_EVENT: &str = "app-log";
        const RUSTFS_LOG_EVENT: &str = "rustfs-log";

        fn create_log_listener(
            logs_signal: WriteSignal<Vec<String>>,
            max_logs: usize,
            logs_ref: NodeRef<leptos::html::Div>,
        ) -> Closure<dyn FnMut(JsValue)> {
            Closure::wrap(Box::new(move |event: JsValue| {
                if let Ok(payload) = js_sys::Reflect::get(&event, &"payload".into()) {
                    if let Some(log) = payload.as_string() {
                        logs_signal.update(|logs| {
                            logs.push(log);
                            if logs.len() > max_logs {
                                logs.remove(0);
                            }
                        });
                        if let Some(element) = logs_ref.get() {
                            element.scroll_to_with_x_and_y(0.0, f64::MAX);
                        }
                    }
                }
            }) as Box<dyn FnMut(JsValue)>)
        }

        if let Some(window) = web_sys::window() {
            let app_listener = create_log_listener(set_app_logs, 100, logs_ref);
            let rustfs_listener = create_log_listener(set_rustfs_logs, 1000, logs_ref);

            if let Ok(tauri) = js_sys::Reflect::get(&window, &"__TAURI__".into()) {
                if let Ok(event) = js_sys::Reflect::get(&tauri, &"event".into()) {
                    if let Ok(listen) = js_sys::Reflect::get(&event, &"listen".into()) {
                        let listen_fn = js_sys::Function::from(listen);

                        let _ = listen_fn.call2(
                            &event,
                            &APP_LOG_EVENT.into(),
                            app_listener.as_ref().unchecked_ref(),
                        );
                        let _ = listen_fn.call2(
                            &event,
                            &RUSTFS_LOG_EVENT.into(),
                            rustfs_listener.as_ref().unchecked_ref(),
                        );
                    }
                }
            }

            app_listener.forget();
            rustfs_listener.forget();
        }

        // Add initial logs
        let app_logs_result = tauri_invoke("get_app_logs", js_sys::Object::new().into()).await;
        if let Some(logs) = app_logs_result.as_string() {
            if let Ok(logs_vec) = serde_json::from_str::<Vec<String>>(&logs) {
                set_app_logs.set(logs_vec);
            }
        }

        let rustfs_logs_result =
            tauri_invoke("get_rustfs_logs", js_sys::Object::new().into()).await;
        if let Some(logs) = rustfs_logs_result.as_string() {
            if let Ok(logs_vec) = serde_json::from_str::<Vec<String>>(&logs) {
                set_rustfs_logs.set(logs_vec);
            }
        }
    });

    let launch_rustfs = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_is_running.set(true);
        set_status.set("Launching RustFS...".to_string());

        let now = js_sys::Date::new_0().to_locale_time_string("en-US".into());
        add_app_log(format!("[{}] Launch button clicked", now));
        add_app_log(format!("[{}] Config: {:?}", now, config.get()));

        spawn_local(async move {
            // Check if we're in Tauri environment
            if !is_tauri() {
                set_status.set("Error: Not running in Tauri environment".to_string());
                add_app_log("[ERROR] Not running in Tauri environment".to_string());
                set_is_running.set(false);
                return;
            }

            let current_config = config.get_untracked();

            // 添加详细日志
            leptos::logging::log!(
                "Starting RustFS with config: data_path={}, port={:?}, host={:?}",
                current_config.data_path,
                current_config.port,
                current_config.host
            );

            let now = js_sys::Date::new_0().to_locale_time_string("en-US".into());
            add_app_log(format!(
                "[{}] Calling tauri_invoke with command: launch_rustfs",
                now
            ));

            // Create args object with config parameter
            let args = js_sys::Object::new();
            let config_js = serde_wasm_bindgen::to_value(&current_config).unwrap();
            js_sys::Reflect::set(&args, &"config".into(), &config_js).unwrap();

            let result = tauri_invoke("launch_rustfs", args.into()).await;
            let now = js_sys::Date::new_0().to_locale_time_string("en-US".into());
            add_app_log(format!("[{}] Invoke result: {:?}", now, result));

            if result.is_string() {
                if let Some(msg) = result.as_string() {
                    let now = js_sys::Date::new_0().to_locale_time_string("en-US".into());
                    add_app_log(format!("[{}] Result message: {}", now, msg));
                    if msg.contains("success") {
                        set_status.set("RustFS launched successfully!".to_string());
                        let now = js_sys::Date::new_0().to_locale_time_string("en-US".into());
                        add_app_log(format!("[{}] Launch successful!", now));
                    } else {
                        set_status.set(format!("Launch result: {}", msg));
                        let now = js_sys::Date::new_0().to_locale_time_string("en-US".into());
                        add_app_log(format!("[{}] Launch result: {}", now, msg));
                    }
                }
            } else {
                set_status.set("RustFS launch command sent".to_string());
                let now = js_sys::Date::new_0().to_locale_time_string("en-US".into());
                add_app_log(format!(
                    "[{}] Launch completed but no message returned",
                    now
                ));
            }
            set_is_running.set(false);
        });
    };

    view! {
        <style>{LOGS_CSS}</style>
        <main class="container">
            <div class="header">
                <h1>"RustFS Launcher"</h1>
                <p class="subtitle">"Simple launcher for RustFS project"</p>
            </div>

            <form class="config-form" on:submit=launch_rustfs>
                <div class="form-group">
                    <label for="data-path">"Data Path" <span class="required">"*"</span></label>
                    <div class="path-input-group">
                        <input
                            id="data-path"
                            type="text"
                            placeholder="Select data directory..."
                            prop:value=move || config.get().data_path
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_config.update(|c| c.data_path = value);
                            }
                        />
                        <button type="button" class="browse-btn" on:click=select_folder>
                            "Browse"
                        </button>
                    </div>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label for="port">"Port"</label>
                        <input
                            id="port"
                            type="number"
                            placeholder="8080"
                            prop:value=move || config.get().port.map(|p| p.to_string()).unwrap_or_default()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                let port = if value.is_empty() { None } else { value.parse().ok() };
                                set_config.update(|c| c.port = port);
                            }
                        />
                    </div>
                    <div class="form-group">
                        <label for="host">"Host"</label>
                        <input
                            id="host"
                            type="text"
                            placeholder="127.0.0.1"
                            prop:value=move || config.get().host.unwrap_or_default()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                let host = if value.is_empty() { None } else { Some(value) };
                                set_config.update(|c| c.host = host);
                            }
                        />
                    </div>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <label for="access-key">"Access Key"</label>
                        <input
                            id="access-key"
                            type="text"
                            placeholder="rustfsadmin"
                            prop:value=move || config.get().access_key.unwrap_or_default()
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                let access_key = if value.is_empty() { None } else { Some(value) };
                                set_config.update(|c| c.access_key = access_key);
                            }
                        />
                    </div>
                    <div class="form-group">
                        <label for="secret-key">"Secret Key"</label>
                        <div class="input-with-toggle">
                            <input
                                id="secret-key"
                                type=move || if show_secret.get() { "text" } else { "password" }
                                placeholder="rustfsadmin"
                                prop:value=move || config.get().secret_key.unwrap_or_default()
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    let secret_key = if value.is_empty() { None } else { Some(value) };
                                    set_config.update(|c| c.secret_key = secret_key);
                                }
                            />
                            <button
                                type="button"
                                class="toggle-visibility"
                                on:click=move |_| set_show_secret.update(|show| *show = !*show)
                            >
                                {move || if show_secret.get() { "🙈" } else { "👁️" }}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="form-row">
                    <div class="form-group">
                        <div class="checkbox-group">
                            <input
                                id="console-enable"
                                type="checkbox"
                                prop:checked=move || config.get().console_enable
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_config.update(|c| c.console_enable = checked);
                                }
                            />
                            <label for="console-enable">"Enable Console"</label>
                        </div>
                    </div>
                </div>

                <div class="form-actions">
                    <button
                        type="submit"
                        class="launch-btn"
                        disabled=move || is_running.get() || config.get().data_path.is_empty()
                    >
                        { move || if is_running.get() { "Launching..." } else { "Launch RustFS" } }
                    </button>
                </div>
            </form>

            <div class="status" class:hidden=move || status.get().is_empty()>
                <p>{ move || status.get() }</p>
            </div>

            <div class="logs-section">
                <div class="log-panel">
                    <div class="log-tabs">
                        <button
                            class="log-tab"
                            class:active=move || current_log_type.get() == LogType::App
                            on:click=move |_| set_current_log_type.set(LogType::App)
                        >
                            "App Logs"
                        </button>
                        <button
                            class="log-tab"
                            class:active=move || current_log_type.get() == LogType::RustFS
                            on:click=move |_| set_current_log_type.set(LogType::RustFS)
                        >
                            "RustFS Output"
                        </button>
                    </div>
                    <div class="log-output" node_ref=logs_ref>
                        <For
                            each=move || {
                                match current_log_type.get() {
                                    LogType::App => app_logs.get(),
                                    LogType::RustFS => rustfs_logs.get(),
                                }
                            }
                            key=|log| log.clone()
                            let:log
                        >
                            <div class="log-line">{log}</div>
                        </For>
                        <Show when=move || {
                            match current_log_type.get() {
                                LogType::App => app_logs.get().is_empty(),
                                LogType::RustFS => rustfs_logs.get().is_empty(),
                            }
                        }>
                            <div class="log-line">"No logs available"</div>
                        </Show>
                    </div>
                </div>
            </div>
        </main>
    }
}
