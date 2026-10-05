use std::sync::{Arc, Mutex};

use tauri::{Webview, Wry};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;

fn operar_historico(
    webview: &Webview<Wry>,
    operar: impl FnOnce(&ICoreWebView2) -> windows::core::Result<()> + Send + 'static,
) -> Result<(), String> {
    let resultado = Arc::new(Mutex::new(None));
    let resultado_ui = resultado.clone();
    webview
        .with_webview(move |plataforma| {
            let operacao = unsafe { plataforma.controller().CoreWebView2() }
                .map_err(|erro| erro.to_string())
                .and_then(|core| operar(&core).map_err(|erro| erro.to_string()));
            if let Ok(mut destino) = resultado_ui.lock() {
                *destino = Some(operacao);
            }
        })
        .map_err(|erro| erro.to_string())?;
    let valor = resultado
        .lock()
        .map_err(|_| "O histórico da página está temporariamente indisponível.".to_string())?
        .take()
        .unwrap_or_else(|| Err("Não foi possível acessar o histórico da página.".into()));
    valor
}

pub fn voltar(webview: &Webview<Wry>) -> Result<(), String> {
    operar_historico(webview, |core| unsafe {
        let mut disponivel = windows::core::BOOL::default();
        core.CanGoBack(&mut disponivel).map_err(|erro| erro)?;
        if disponivel.as_bool() {
            core.GoBack()?;
        }
        Ok(())
    })
}

pub fn avancar(webview: &Webview<Wry>) -> Result<(), String> {
    operar_historico(webview, |core| unsafe {
        let mut disponivel = windows::core::BOOL::default();
        core.CanGoForward(&mut disponivel).map_err(|erro| erro)?;
        if disponivel.as_bool() {
            core.GoForward()?;
        }
        Ok(())
    })
}
