use std::sync::{Arc, Mutex};
use tauri::{Webview, Wry};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_8;
use windows::core::Interface;

pub fn definir_mudo(webview: &Webview<Wry>, mudo: bool) -> Result<(), String> {
    let resultado: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
    let resultado_callback = resultado.clone();
    webview
        .with_webview(move |plataforma| {
            let tentativa = (|| {
                let core = unsafe { plataforma.controller().CoreWebView2() }
                    .map_err(|erro| erro.to_string())?;
                let core: ICoreWebView2_8 = core.cast().map_err(|erro| erro.to_string())?;
                unsafe { core.SetIsMuted(mudo) }.map_err(|erro| erro.to_string())
            })();
            if let Ok(mut guardiao) = resultado_callback.lock() {
                *guardiao = Some(tentativa);
            }
        })
        .map_err(|erro| erro.to_string())?;
    let resultado_final = resultado
        .lock()
        .map_err(|_| "Não foi possível acessar o estado de áudio da página.".to_string())?
        .take()
        .unwrap_or_else(|| Err("Não foi possível alterar o áudio desta página.".into()));
    resultado_final
}
