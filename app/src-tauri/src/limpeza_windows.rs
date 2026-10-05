use tauri::{AppHandle, Emitter, EventTarget, Webview, Wry};
use webview2_com::{
    ClearBrowsingDataCompletedHandler, GetCookiesCompletedHandler,
    Microsoft::Web::WebView2::Win32::{
        ICoreWebView2CookieManager, ICoreWebView2Profile2, ICoreWebView2_13, ICoreWebView2_2,
        COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES, COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE,
    },
};
use windows::core::{Interface, HSTRING};

#[derive(Clone, serde::Serialize)]
struct ResultadoLimpeza {
    sucesso: bool,
    mensagem: String,
}

fn concluir(app: &AppHandle, sucesso: bool, mensagem: String) {
    let _ = app.emit_to(
        EventTarget::webview("main"),
        "limpeza-dados-concluida",
        ResultadoLimpeza { sucesso, mensagem },
    );
}

pub fn limpar_perfil(webview: &Webview<Wry>, app: AppHandle) -> Result<(), String> {
    let app_erro = app.clone();
    webview
        .with_webview(move |plataforma| {
            let resultado = (|| {
                let core = unsafe { plataforma.controller().CoreWebView2() }
                    .map_err(|erro| erro.to_string())?;
                let core: ICoreWebView2_13 = core.cast().map_err(|erro| erro.to_string())?;
                let profile = unsafe { core.Profile() }.map_err(|erro| erro.to_string())?;
                let profile: ICoreWebView2Profile2 = profile.cast().map_err(|erro| erro.to_string())?;
                let app_final = app.clone();
                unsafe {
                    profile.ClearBrowsingData(
                        COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES
                            | COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE,
                        &ClearBrowsingDataCompletedHandler::create(Box::new(move |erro| {
                            match erro {
                                Ok(()) => concluir(&app_final, true, "Cookies e cache de todos os sites foram limpos. Você poderá sair de contas abertas.".into()),
                                Err(erro) => concluir(&app_final, false, format!("Falha ao limpar cookies e cache: {erro}")),
                            }
                            Ok(())
                        })),
                    )
                }.map_err(|erro| erro.to_string())
            })();
            if let Err(erro) = resultado {
                concluir(&app_erro, false, erro);
            }
        })
        .map_err(|erro| erro.to_string())
}

pub fn limpar_cookies_site(
    webview: &Webview<Wry>,
    app: AppHandle,
    endereco: String,
) -> Result<(), String> {
    let uri = HSTRING::from(endereco);
    let app_erro = app.clone();
    webview
        .with_webview(move |plataforma| {
            let resultado = (|| {
                let core = unsafe { plataforma.controller().CoreWebView2() }
                    .map_err(|erro| erro.to_string())?;
                let core: ICoreWebView2_2 = core.cast().map_err(|erro| erro.to_string())?;
                let manager: ICoreWebView2CookieManager = unsafe { core.CookieManager() }
                    .map_err(|erro| erro.to_string())?;
                let manager_callback = manager.clone();
                let app_final = app.clone();
                unsafe {
                    manager.GetCookies(
                        &uri,
                        &GetCookiesCompletedHandler::create(Box::new(move |erro, cookies| {
                            let resultado = (|| {
                                erro.map_err(|erro| erro.to_string())?;
                                let Some(cookies) = cookies else { return Ok(0_u32) };
                                let mut quantidade = 0_u32;
                                cookies.Count(&mut quantidade).map_err(|erro| erro.to_string())?;
                                let mut removidos = 0_u32;
                                for indice in 0..quantidade {
                                    let cookie = cookies.GetValueAtIndex(indice).map_err(|erro| erro.to_string())?;
                                    manager_callback.DeleteCookie(&cookie).map_err(|erro| erro.to_string())?;
                                    removidos += 1;
                                }
                                Ok::<u32, String>(removidos)
                            })();
                            match resultado {
                                Ok(quantidade) => concluir(&app_final, true, format!("{quantidade} cookies da página atual foram removidos. Cache e armazenamento do site foram mantidos.")),
                                Err(erro) => concluir(&app_final, false, format!("Falha ao limpar os cookies do site: {erro}")),
                            }
                            Ok(())
                        })),
                    )
                }.map_err(|erro| erro.to_string())
            })();
            if let Err(erro) = resultado {
                concluir(&app_erro, false, erro);
            }
        })
        .map_err(|erro| erro.to_string())
}
