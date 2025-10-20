use async_trait::async_trait;
use tauri::{async_runtime::Mutex, Builder, Manager, State, Window};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use ydapi::{self, Ydapi};
#[async_trait]
trait Translator: Send + Sync {
    async fn translate(
        &self,
        origin_text: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<String, String>;
    fn support_lang(&self) -> Vec<(String, String)>;
    fn name(&self) -> &'static str;
}

#[async_trait]
impl Translator for Ydapi {
    async fn translate(
        &self,
        origin_text: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<String, String> {
        let res = self
            .translate(origin_text, 0, from, to)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(res);
    }

    fn support_lang(&self) -> Vec<(String, String)> {
        let res = self.support_lang();
        return res.clone();
    }

    fn name(&self) -> &'static str {
        return "youdao";
    }
}

struct ManagerState {
    api: Option<Box<dyn Translator>>,
}

#[tauri::command]
fn support_apis() -> Vec<String> {
    return vec!["youdao".into()];
}

#[tauri::command]
async fn support_lang(
    state: State<'_, Mutex<ManagerState>>,
) -> Result<Vec<(String, String)>, String> {
    let state = state.lock().await;
    let api = state.api.as_ref().ok_or("please choose api")?;
    return Ok(api.support_lang());
}

#[tauri::command]
async fn choose_api(
    state: State<'_, Mutex<ManagerState>>,
    name: &str,
    args: Option<&str>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    match name {
        "youdao" => {
            let ydapi = Ydapi::new().await.map_err(|e| e.to_string())?;
            state.api = Some(Box::new(ydapi))
        }
        _ => {
            return Err("can not find this api".into());
        }
    }
    Ok(())
}

#[tauri::command]
async fn translate(
    state: State<'_, Mutex<ManagerState>>,
    origin_text: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<String, String> {
    // let api = state.api.as_ref().ok_or("please choose api")?;
    let state = state.lock().await;
    let res = state.api.as_ref().ok_or("please choose api")?;

    Ok(res.translate(origin_text, from, to).await?)
}

fn main() {
    Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(ManagerState { api: None }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            translate,
            choose_api,
            support_lang,
            support_apis
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let app = app.get_webview_window("main")
                .expect("no main window");
            app.show().unwrap();
            app.set_focus().unwrap();
        }))
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(ManagerState { api: None }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            translate,
            choose_api,
            support_lang,
            support_apis
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
