//! Mantém o endereço da aba alinhado com mudanças na mesma página (hash e History API).

use tauri::{AppHandle, Emitter, EventTarget, Manager, Url, Webview, Wry};
use webview2_com::{
    take_pwstr, Microsoft::Web::WebView2::Win32::ICoreWebView2, SourceChangedEventHandler,
};
use windows::core::PWSTR;

use crate::Estado;

fn endereco_publico(valor: &str) -> Option<String> {
    let url = Url::parse(valor).ok()?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str() == Some("canoa.invalid") {
        return None;
    }
    Some(url.to_string())
}

fn registrar(core: &ICoreWebView2, app: AppHandle, id: u64) -> windows::core::Result<()> {
    let handler = SourceChangedEventHandler::create(Box::new(move |sender, _| {
        let Some(sender) = sender else {
            return Ok(());
        };
        let mut ponteiro = PWSTR::null();
        unsafe { sender.Source(&mut ponteiro)? };
        let Some(endereco) = endereco_publico(&take_pwstr(ponteiro)) else {
            return Ok(());
        };
        let mudou = {
            let estado = app.state::<Estado>();
            let Ok(mut abas) = estado.0.lock() else {
                return Ok(());
            };
            if abas.ativa != id {
                false
            } else if let Some(aba) = abas.itens.iter_mut().find(|aba| aba.id == id) {
                if aba.endereco.as_deref() == Some(endereco.as_str()) {
                    false
                } else {
                    aba.endereco = Some(endereco.clone());
                    true
                }
            } else {
                false
            }
        };
        if mudou {
            let _ = app.emit_to(EventTarget::webview("main"), "pagina-endereco", endereco);
        }
        Ok(())
    }));
    // A WebView mantém o handler até sua destruição, assim como no filtro experimental.
    let mut token = 0;
    unsafe { core.add_SourceChanged(&handler, &mut token)? };
    Ok(())
}

pub fn acompanhar(webview: &Webview<Wry>, id: u64) -> Result<(), String> {
    let app = webview.app_handle().clone();
    webview
        .with_webview(
            move |plataforma| match unsafe { plataforma.controller().CoreWebView2() } {
                Ok(core) => {
                    if let Err(erro) = registrar(&core, app, id) {
                        eprintln!("CANOA_FONTE_REGISTRO_ERRO {erro}");
                    }
                }
                Err(erro) => eprintln!("CANOA_FONTE_WEBVIEW_ERRO {erro}"),
            },
        )
        .map_err(|erro| erro.to_string())
}

#[cfg(test)]
mod testes {
    use super::endereco_publico;

    #[test]
    fn so_aceita_paginas_web_publicas() {
        assert_eq!(
            endereco_publico("https://www.openstreetmap.org/#map=5/-15.13/-53.19"),
            Some("https://www.openstreetmap.org/#map=5/-15.13/-53.19".into())
        );
        assert_eq!(endereco_publico("about:blank"), None);
        assert_eq!(endereco_publico("https://canoa.invalid/__atalho/x"), None);
    }
}
