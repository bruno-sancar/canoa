//! Bloqueio nativo de requisições no WebView2, instalado antes da primeira navegação.

use crate::bloqueio::Protecao;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, EventTarget, Manager, Url, Webview, Wry};
use webview2_com::{
    take_pwstr,
    Microsoft::Web::WebView2::Win32::{
        ICoreWebView2, ICoreWebView2Environment, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST,
    },
    WebResourceRequestedEventHandler,
};
use windows::{
    core::{HSTRING, PWSTR},
    Win32::System::Com::IStream,
};

fn tipo_recurso(contexto: i32) -> &'static str {
    match contexto {
        x if x == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT.0 => "document",
        x if x == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT.0 => "script",
        x if x == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE.0 => "image",
        x if x == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET.0 => "stylesheet",
        x if x == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT.0 => "font",
        x if x == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA.0 => "media",
        x if x == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH.0 => "xmlhttprequest",
        x if x == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST.0 => "xmlhttprequest",
        _ => "other",
    }
}

fn registrar(
    app: AppHandle,
    core: &ICoreWebView2,
    ambiente: &ICoreWebView2Environment,
    protecao: Arc<Protecao>,
    origem: Arc<Mutex<String>>,
) -> windows::core::Result<()> {
    let ambiente = ambiente.clone();
    let handler = WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        let pedido = unsafe { args.Request()? };
        let mut ponteiro = PWSTR::null();
        unsafe { pedido.Uri(&mut ponteiro)? };
        let endereco = take_pwstr(ponteiro);
        if !matches!(
            Url::parse(&endereco)
                .map(|u| u.scheme().to_string())
                .as_deref(),
            Ok("http" | "https")
        ) {
            return Ok(());
        }
        let mut contexto = Default::default();
        unsafe { args.ResourceContext(&mut contexto)? };
        crate::bloqueio::preparar_motor();
        let origem_atual = origem.lock().map(|valor| valor.clone()).unwrap_or_default();
        if !protecao.bloquear(&endereco, &origem_atual, tipo_recurso(contexto.0)) {
            return Ok(());
        }
        let resumo = protecao.registrar_bloqueio(tipo_recurso(contexto.0));
        if resumo.bloqueados_pagina <= 3 || resumo.bloqueados_pagina % 10 == 0 {
            let _ = app.emit_to(
                EventTarget::webview("main"),
                "contador-bloqueador-atualizado",
                resumo,
            );
        }
        let resposta = unsafe {
            ambiente.CreateWebResourceResponse(
                None::<&IStream>,
                204,
                &HSTRING::from("No Content"),
                &HSTRING::new(),
            )?
        };
        unsafe { args.SetResponse(&resposta)? };
        Ok(())
    }));

    let mut token = 0;
    unsafe { core.add_WebResourceRequested(&handler, &mut token)? };
    if let Err(erro) = unsafe {
        core.AddWebResourceRequestedFilter(
            &HSTRING::from("*"),
            COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
        )
    } {
        let _ = unsafe { core.remove_WebResourceRequested(token) };
        return Err(erro);
    }
    Ok(())
}

pub fn instalar_e_navegar(
    webview: &Webview<Wry>,
    url: Url,
    protecao: Arc<Protecao>,
    origem: Arc<Mutex<String>>,
) -> Result<(), String> {
    let destino = url.to_string();
    let app_eventos = webview.app_handle().clone();
    webview
        .with_webview(move |plataforma| {
            crate::bloqueio::preparar_motor();
            let core = unsafe { plataforma.controller().CoreWebView2() };
            match core {
                Ok(core) => {
                    if let Err(erro) = registrar(
                        app_eventos,
                        &core,
                        &plataforma.environment(),
                        protecao,
                        origem,
                    ) {
                        eprintln!("CANOA_BLOQUEADOR_FILTRO_ERRO {erro}");
                    }
                    if let Err(erro) = unsafe { core.Navigate(&HSTRING::from(destino)) } {
                        eprintln!("CANOA_BLOQUEADOR_NAVEGACAO_ERRO {erro}");
                    }
                }
                Err(erro) => eprintln!("CANOA_BLOQUEADOR_WEBVIEW_ERRO {erro}"),
            }
        })
        .map_err(|erro| erro.to_string())
}
