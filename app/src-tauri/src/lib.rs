use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    webview::{DownloadEvent, PageLoadEvent, WebviewBuilder},
    AppHandle, Emitter, EventTarget, LogicalPosition, LogicalSize, Manager, Position, Rect, Size,
    Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window, WindowEvent,
};
use tauri_plugin_dialog::DialogExt;

mod atualizador_listas;
#[cfg(windows)]
mod audio_windows;
mod biblioteca;
mod bloqueio;
mod configuracoes;
mod favoritos;
#[cfg(windows)]
mod filtro_windows;
#[cfg(windows)]
mod fonte_windows;
mod fontes;
#[cfg(windows)]
mod limpeza_windows;
#[cfg(windows)]
mod memoria;
#[cfg(windows)]
mod navegacao_windows;
mod privacidade;
mod senhas;
mod sessao;

#[derive(serde::Serialize)]
struct Memoria {
    privado_bytes: u64,
    trabalho_bytes: u64,
    processos: usize,
    instancias: usize,
}

#[derive(serde::Serialize)]
struct ProcessoMedido {
    id: u32,
    nome: String,
    categoria: String,
    privado_bytes: u64,
    trabalho_bytes: u64,
}

#[derive(serde::Serialize)]
struct Diagnostico {
    memoria: Memoria,
    processos: Vec<ProcessoMedido>,
}

#[derive(serde::Serialize)]
struct InformacoesVersao {
    versao: String,
    compilado_em_epoch: Option<u64>,
}

#[derive(serde::Serialize)]
struct Armazenamento {
    executavel_bytes: u64,
    dados_locais_bytes: u64,
    cache_identificado_bytes: u64,
    configuracoes_bytes: u64,
    incompleto: bool,
}

fn somar_arquivos(raiz: &Path, identificar_cache: bool) -> (u64, u64, bool) {
    let mut total = 0_u64;
    let mut cache = 0_u64;
    let mut incompleto = false;
    let mut pendentes = vec![(raiz.to_path_buf(), false)];
    while let Some((caminho, dentro_cache)) = pendentes.pop() {
        let metadados = match std::fs::symlink_metadata(&caminho) {
            Ok(valor) => valor,
            Err(erro) if erro.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => {
                incompleto = true;
                continue;
            }
        };
        if metadados.file_type().is_symlink() {
            continue;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadados.file_attributes() & 0x400 != 0 {
                continue;
            }
        }
        if metadados.is_file() {
            total = total.saturating_add(metadados.len());
            if dentro_cache {
                cache = cache.saturating_add(metadados.len());
            }
        } else if metadados.is_dir() {
            let nome_cache = identificar_cache
                && caminho
                    .file_name()
                    .map(|nome| {
                        nome.to_string_lossy()
                            .to_ascii_lowercase()
                            .contains("cache")
                    })
                    .unwrap_or(false);
            match std::fs::read_dir(&caminho) {
                Ok(entradas) => {
                    for entrada in entradas {
                        match entrada {
                            Ok(entrada) => {
                                pendentes.push((entrada.path(), dentro_cache || nome_cache))
                            }
                            Err(_) => incompleto = true,
                        }
                    }
                }
                Err(_) => incompleto = true,
            }
        }
    }
    (total, cache, incompleto)
}

#[derive(Clone, serde::Serialize)]
struct DownloadInfo {
    id: String,
    nome: String,
    caminho: Option<String>,
    sucesso: bool,
    tipo: String,
}

#[derive(Clone, serde::Serialize)]
struct DownloadAtivo {
    id: String,
    nome: String,
    bytes_recebidos: u64,
    total_bytes: Option<u64>,
    cancelavel: bool,
    tipo: String,
}

#[derive(Clone)]
struct DownloadEmAndamento {
    info: DownloadAtivo,
    cancelamento: Option<Arc<AtomicBool>>,
}

#[derive(Default)]
struct GerenciadorDownloads {
    itens: Mutex<HashMap<String, DownloadEmAndamento>>,
    urls_nativas: Mutex<HashMap<String, Vec<String>>>,
    sequencia: AtomicU64,
}

#[derive(Clone, serde::Serialize)]
struct Aba {
    id: u64,
    endereco: Option<String>,
    titulo: String,
    grupo: Option<String>,
    mutada: bool,
}

#[derive(Clone, serde::Serialize)]
struct Retrato {
    abas: Vec<Aba>,
    ativa: u64,
}

struct Estado(Mutex<Abas>);
struct Faixa(Mutex<bool>);
struct TextoContexto(Mutex<String>);
const URL_CONFIGURACOES: &str = "canoa://configuracoes";

fn eh_configuracoes(endereco: &str) -> bool {
    endereco == URL_CONFIGURACOES || endereco.starts_with("canoa://configuracoes/")
}
struct Abas {
    itens: Vec<Aba>,
    ativa: u64,
    proximo: u64,
    fechadas: Vec<(Aba, usize)>,
}

impl Abas {
    fn retrato(&self) -> Retrato {
        Retrato {
            abas: self.itens.clone(),
            ativa: self.ativa,
        }
    }
}

fn retrato(app: &AppHandle) -> Result<Retrato, String> {
    Ok(app
        .state::<Estado>()
        .0
        .lock()
        .map_err(|_| "Falha ao ler as abas.")?
        .retrato())
}

fn avisar(app: &AppHandle) {
    if let Ok(valor) = retrato(app) {
        let _ = app.emit_to(EventTarget::webview("main"), "abas-atualizadas", valor);
    }
}

fn area(window: &Window) -> Result<Rect, String> {
    let tamanho = window.inner_size().map_err(|erro| erro.to_string())?;
    let escala = window.scale_factor().map_err(|erro| erro.to_string())?;
    let mais_de_uma = window
        .state::<Estado>()
        .0
        .lock()
        .map(|e| !e.itens.is_empty())
        .unwrap_or(false);
    let aviso = window
        .state::<Faixa>()
        .0
        .lock()
        .map(|valor| *valor)
        .unwrap_or(false);
    let topo = (if mais_de_uma { 84.0 } else { 52.0 }) + if aviso { 40.0 } else { 0.0 };
    Ok(Rect {
        position: Position::Logical(LogicalPosition::new(0.0, topo)),
        size: Size::Logical(LogicalSize::new(
            f64::from(tamanho.width) / escala,
            (f64::from(tamanho.height) / escala - topo).max(1.0),
        )),
    })
}

fn ajustar(window: &Window) {
    if let Some(webview) = window.app_handle().get_webview("conteudo") {
        if let Ok(limites) = area(window) {
            if let Err(erro) = webview.set_bounds(limites) {
                eprintln!("CANOA_AJUSTE_ERRO {erro}");
            }
        }
    }
}

fn validar(entrada: &str) -> Result<Url, String> {
    let entrada = entrada.trim();
    if entrada.is_empty() {
        return Err("Digite um endereço.".into());
    }
    let completo = if entrada.contains("://") || entrada.starts_with("javascript:") {
        entrada.to_string()
    } else {
        format!("https://{entrada}")
    };
    let url = Url::parse(&completo).map_err(|_| "Endereço inválido.".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("Use um endereço http ou https.".into());
    }
    Ok(url)
}

fn titulo_pagina(valor: &str, alternativa: &str) -> String {
    let titulo = serde_json::from_str::<String>(valor).unwrap_or_default();
    let titulo = titulo.trim();
    if titulo.is_empty() {
        alternativa.chars().take(300).collect()
    } else {
        titulo.chars().take(300).collect()
    }
}

fn interpretar_entrada(entrada: &str, buscador: configuracoes::Buscador) -> Result<Url, String> {
    let entrada = entrada.trim();
    if entrada.is_empty() {
        return Err("Digite um endereço ou uma pesquisa.".into());
    }
    let minusculas = entrada.to_ascii_lowercase();
    let esquema_no_inicio = entrada.split_once("://").is_some_and(|(esquema, _)| {
        esquema.starts_with(|caractere: char| caractere.is_ascii_alphabetic())
            && esquema.chars().all(|caractere| {
                caractere.is_ascii_alphanumeric() || matches!(caractere, '+' | '-' | '.')
            })
    });
    if esquema_no_inicio || minusculas.starts_with("http:") || minusculas.starts_with("https:") {
        return validar(entrada);
    }
    if ["javascript:", "file:", "data:", "about:", "mailto:", "ftp:"]
        .iter()
        .any(|esquema| minusculas.starts_with(esquema))
    {
        return Err("Use um endereço http ou https.".into());
    }
    if !minusculas.starts_with("site:")
        && !minusculas.starts_with("localhost:")
        && Url::parse(entrada).is_ok()
    {
        return Err("Use um endereço http ou https.".into());
    }

    let host = entrada
        .split(['/', '?', '#', ':'])
        .next()
        .unwrap_or_default();
    let parece_endereco = !entrada.chars().any(char::is_whitespace)
        && (host.contains('.') || host.eq_ignore_ascii_case("localhost"));
    if parece_endereco {
        if host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" {
            return validar(&format!("http://{entrada}"));
        }
        return validar(entrada);
    }

    let consulta = entrada.strip_prefix('?').unwrap_or(entrada).trim();
    if consulta.is_empty() {
        return Err("Digite o que deseja pesquisar.".into());
    }
    fontes::url_busca(buscador, consulta)
}

fn destino_disponivel(pasta: &Path, nome: &Path) -> PathBuf {
    let original = pasta.join(nome);
    if !original.exists() {
        return original;
    }
    let base = nome.file_stem().unwrap_or_default().to_string_lossy();
    let extensao = nome.extension().map(|valor| valor.to_string_lossy());
    for numero in 1..=999 {
        let nome = match extensao.as_deref() {
            Some(extensao) => format!("{base} ({numero}).{extensao}"),
            None => format!("{base} ({numero})"),
        };
        let candidato = pasta.join(nome);
        if !candidato.exists() {
            return candidato;
        }
    }
    let instante = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|valor| valor.as_nanos())
        .unwrap_or_default();
    pasta.join(format!("{base}-{}-{instante}", std::process::id()))
}

fn abrir(window: &Window, id: u64, endereco: &str) -> Result<(), String> {
    let mutada = window
        .state::<Estado>()
        .0
        .lock()
        .map_err(|_| "Falha ao ler o estado de áudio da aba.")?
        .itens
        .iter()
        .find(|aba| aba.id == id)
        .is_some_and(|aba| aba.mutada);
    let url = validar(endereco)?;
    let url_inicial = Url::parse("about:blank").map_err(|erro| erro.to_string())?;
    let chave = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|erro| erro.to_string())?
            .as_nanos()
    );
    let origem_privacidade = Arc::new(Mutex::new(url.to_string()));
    let origem_navegacao = origem_privacidade.clone();
    let script = format!(
        r#"
      if (window === window.top) {{
        window.addEventListener('keydown', (evento) => {{
          if (!evento.isTrusted || evento.repeat) return;
          const tecla = evento.key.toLowerCase();
          let acao = '';
          if (evento.ctrlKey && !evento.altKey && !evento.shiftKey) {{
            acao = ({{ t: 'nova_aba', w: 'fechar_aba', l: 'endereco', r: 'recarregar', j: 'downloads', d: 'favorito', ',': 'configuracoes' }})[tecla] || '';
            if (tecla === 'tab' || tecla === 'pagedown') acao = 'proxima_aba';
            if (tecla === 'pageup') acao = 'aba_anterior';
            if (/^[1-9]$/.test(tecla)) acao = 'aba_' + tecla;
            if (tecla === 'f4') acao = 'fechar_aba';
          }} else if (evento.ctrlKey && evento.shiftKey && !evento.altKey) {{
            acao = tecla === 'tab' ? 'aba_anterior' : tecla === 'o' ? 'ver_favoritos' : tecla === 'm' ? 'alternar_memoria' : tecla === 't' ? 'reabrir_aba' : tecla === 's' ? 'baixar_recursos' : tecla === 'g' ? 'grupar_aba' : '';
          }} else if (evento.shiftKey && !evento.ctrlKey && !evento.altKey && (tecla === 'f10' || tecla === 'contextmenu')) {{
            acao = 'menu_contexto_aba';
          }} else if (evento.shiftKey && !evento.ctrlKey && !evento.altKey && tecla === 'escape') {{
            acao = 'tarefas';
          }} else if (evento.altKey && !evento.ctrlKey && !evento.shiftKey) {{
            acao = tecla === 'arrowleft' ? 'voltar' : tecla === 'arrowright' ? 'avancar' : tecla === 'd' ? 'endereco' : '';
          }} else if (evento.ctrlKey && evento.altKey && !evento.shiftKey) {{
            acao = tecla === 'p' ? 'alternar_protecao_site' : tecla === 'a' ? 'acesso_artigo' : tecla === 'b' ? 'alternar_bloqueio_site' : tecla === 'm' ? 'mutar_pagina' : '';
          }} else if (evento.ctrlKey && evento.shiftKey && !evento.altKey && tecla === 'f') {{
            acao = 'alternar_foco';
          }} else if (evento.ctrlKey && !evento.altKey && !evento.shiftKey && tecla === 'h') {{
            acao = 'historico';
          }} else if (tecla === 'f5' && !evento.ctrlKey && !evento.altKey && !evento.shiftKey) {{
            acao = 'recarregar';
          }}
          if (!acao) return;
          evento.preventDefault();
          evento.stopImmediatePropagation();
          location.href = 'https://canoa.invalid/__atalho/{chave}/' + acao;
        }}, true);
        window.addEventListener('contextmenu', (evento) => {{
          if (!evento.isTrusted) return;
          const alvo = evento.target instanceof Element ? evento.target : null;
          if (alvo && alvo.closest('input, textarea, select, [contenteditable="true"]')) return;
          evento.preventDefault();
          evento.stopImmediatePropagation();
          const link = alvo?.closest('a[href]');
          const imagem = alvo?.closest('img');
          const dados = {{ x: evento.clientX, y: evento.clientY, endereco: location.href, titulo: document.title,
            link: link instanceof HTMLAnchorElement ? link.href : null,
            imagem: imagem instanceof HTMLImageElement ? (imagem.currentSrc || imagem.src) : null,
            texto: (window.getSelection()?.toString() || '').slice(0, 4000) }};
          location.href = 'https://canoa.invalid/__atalho/{chave}/menu_contexto_pagina?dados=' + encodeURIComponent(JSON.stringify(dados));
        }}, true);
        window.addEventListener('canoa:solicitar-recursos', () => {{
          const recursos = new Map();
          const extensoes = {{
            imagem: /\.(png|jpe?g|gif|webp|svg|avif|bmp|ico)(?:$|[?#])/i,
            video: /\.(mp4|webm|ogv?|mov|m4v)(?:$|[?#])/i,
            audio: /\.(mp3|m4a|aac|oga|ogg|wav|flac|opus)(?:$|[?#])/i,
            documento: /\.(pdf|epub|docx?|xlsx?|pptx?|odt|rtf)(?:$|[?#])/i,
          }};
          const adicionar = (bruto, tipo, nome) => {{
            if (!bruto || recursos.size >= 80) return;
            try {{
              const url = new URL(bruto, location.href);
              if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password) return;
              if (/\.(m3u8|mpd)(?:$|[?#])/i.test(url.href)) return;
              const endereco = url.href;
              if (!recursos.has(endereco)) recursos.set(endereco, {{ endereco, tipo, nome: (nome || url.pathname.split('/').pop() || url.hostname).slice(0, 180) }});
            }} catch {{}}
          }};
          document.querySelectorAll('video, audio, source').forEach((el) => {{
            const bruto = el.currentSrc || el.src;
            if (!bruto) return;
            let url;
            try {{ url = new URL(bruto, location.href); }} catch {{ return; }}
            const tipo = el.tagName.toLowerCase() === 'audio' ? 'audio' : 'video';
            adicionar(url.href, tipo, el.getAttribute('title') || el.getAttribute('aria-label'));
            el.querySelectorAll('source').forEach((filho) => adicionar(filho.src, tipo, filho.type));
          }});
          performance.getEntriesByType('resource').forEach((entrada) => {{
            if (/\.(mp4|webm|ogv?|mov|m4v)(?:$|[?#])/i.test(entrada.name)) adicionar(entrada.name, 'video', 'Mídia detectada na página');
            else if (/\.(mp3|m4a|aac|oga|ogg|wav|flac|opus)(?:$|[?#])/i.test(entrada.name)) adicionar(entrada.name, 'audio', 'Áudio detectado na página');
          }});
          document.querySelectorAll('img').forEach((el) => adicionar(el.currentSrc || el.src, 'imagem', el.alt));
          document.querySelectorAll('a[href]').forEach((el) => {{
            const url = el.href;
            const tipo = Object.entries(extensoes).find(([, regex]) => regex.test(url))?.[0];
            if (tipo) adicionar(url, tipo, el.textContent.trim());
          }});
          const payload = {{ endereco: location.href, titulo: document.title, itens: Array.from(recursos.values()), instante: Date.now() }};
          location.href = 'https://canoa.invalid/__atalho/{chave}/recursos_pagina?dados=' + encodeURIComponent(JSON.stringify(payload));
        }});
        const canoaDetectarRestricao = () => {{
          try {{
            const termos = [
              'assine para continuar', 'assine para ler', 'assine para ter acesso',
              'conteúdo exclusivo para assinantes', 'conteudo exclusivo para assinantes',
              'somente para assinantes', 'apenas para assinantes', 'seja assinante',
              'faça login para continuar', 'faca login para continuar', 'já é assinante',
              'subscribe to continue', 'subscribe to read', 'subscriber-only',
              'members-only', 'continue reading with a subscription', 'already a subscriber'
            ];
            const seletores = '[class*="paywall" i], [id*="paywall" i], [class*="subscriber-only" i], [class*="subscription-required" i], [class*="premium-content" i], [data-testid*="paywall" i]';
            const elemento = document.querySelector(seletores);
            const texto = (document.body?.innerText || '').slice(0, 12000).toLocaleLowerCase();
            const termo = termos.find((item) => texto.includes(item));
            const sinais = [];
            if (elemento) sinais.push('marcador de conteúdo restrito encontrado na página');
            if (termo) sinais.push('mensagem de assinatura ou acesso restrito encontrada');
            const origem = location.origin;
            const links = Array.from(document.querySelectorAll('a[href]')).filter((a) => {{
              const r = a.getBoundingClientRect();
              const rotulo = (a.innerText || a.getAttribute('aria-label') || a.title || '').trim().toLocaleLowerCase();
              return r.width > 0 && r.height > 0 && /assine|assinatura|assinar|entrar|login|acesso|planos|subscribe|subscription|sign in|log in|membership/.test(rotulo);
            }}).map((a) => {{
              try {{ const u = new URL(a.href, location.href); return u.origin === origem ? u.href : null; }} catch {{ return null; }}
            }}).filter(Boolean).slice(0, 4);
            const payload = {{ endereco: location.href, titulo: document.title, restrito: sinais.length > 0, sinais, links }};
            location.href = 'https://canoa.invalid/__atalho/{chave}/acesso_artigo?dados=' + encodeURIComponent(JSON.stringify(payload));
          }} catch {{}}
        }};
        if (document.readyState === 'complete' || document.readyState === 'interactive') setTimeout(canoaDetectarRestricao, 900);
        else document.addEventListener('DOMContentLoaded', () => setTimeout(canoaDetectarRestricao, 900), {{ once: true }});
      }}
    "#
    );
    let app_atalho = window.app_handle().clone();
    let app_nova_janela = app_atalho.clone();
    let protecao_navegacao = window.state::<Arc<bloqueio::Protecao>>().inner().clone();
    let builder = WebviewBuilder::new("conteudo", WebviewUrl::External(url_inicial))
        .initialization_script(script)
        .on_navigation(move |url| {
            if url.host_str() == Some("canoa.invalid") {
                let prefixo = format!("/__atalho/{chave}/");
                if let Some(acao) = url.path().strip_prefix(&prefixo) {
                    if matches!(acao, "nova_aba" | "fechar_aba" | "reabrir_aba" | "endereco" | "recarregar" | "downloads" | "favorito" | "ver_favoritos" | "configuracoes" | "alternar_memoria" | "tarefas" | "menu_contexto_aba" | "baixar_recursos" | "alternar_protecao_site" | "alternar_bloqueio_site" | "grupar_aba" | "mutar_pagina" | "historico" | "alternar_foco" | "voltar" | "avancar" | "proxima_aba" | "aba_anterior" | "aba_1" | "aba_2" | "aba_3" | "aba_4" | "aba_5" | "aba_6" | "aba_7" | "aba_8" | "aba_9") {
                        let _ = app_atalho.emit_to(EventTarget::webview("main"), "atalho", acao);
                    }
                    if acao == "menu_contexto_pagina" {
                        if let Some((_, dados)) = url.query_pairs().find(|(chave, _)| chave == "dados") {
                            let _ = app_atalho.emit_to(EventTarget::webview("main"), "menu-contexto-pagina", dados.into_owned());
                        }
                    }
                    if acao == "recursos_pagina" {
                        if let Some((_, dados)) = url.query_pairs().find(|(chave, _)| chave == "dados") {
                            let _ = app_atalho.emit_to(EventTarget::webview("main"), "recursos-pagina", dados.into_owned());
                        }
                    }
                    if acao == "acesso_artigo" {
                        if let Some((_, dados)) = url.query_pairs().find(|(chave, _)| chave == "dados") {
                            let _ = app_atalho.emit_to(EventTarget::webview("main"), "acesso-artigo-detectado", dados.into_owned());
                        } else {
                            let _ = app_atalho.emit_to(EventTarget::webview("main"), "atalho", acao);
                        }
                    }
                }
                return false;
            }
            if matches!(url.scheme(), "http" | "https") {
                if let Ok(config) = configuracoes::carregar(&app_atalho) {
                    if let Some(host) = url.host_str() {
                        if sessao::foco_bloqueia(&config, host, sessao::agora_epoch()) {
                            let _ = app_atalho.emit_to(EventTarget::webview("main"), "site-bloqueado-foco", host.to_string());
                            return false;
                        }
                        if let Some((uso, limite)) = sessao::limite(&config, host) {
                            if uso >= limite {
                                let _ = app_atalho.emit_to(EventTarget::webview("main"), "site-limite-atingido", host.to_string());
                                return false;
                            }
                        }
                        if privacidade::dominio_bloqueado(host, &config.sites_bloqueados) {
                            let _ = app_atalho.emit_to(EventTarget::webview("main"), "site-bloqueado", host.to_string());
                            return false;
                        }
                    }
                    if config.limpar_rastreamento {
                        let mut limpa = url.clone();
                        if privacidade::limpar_parametros_rastreamento(&mut limpa) {
                            let _ = app_atalho.emit_to(EventTarget::webview("main"), "limpar-url-rastreamento", limpa.to_string());
                            return false;
                        }
                    }
                }
                if let Ok(mut origem) = origem_navegacao.lock() {
                    *origem = url.to_string();
                }
                protecao_navegacao.iniciar_pagina(url.as_str());
            }
            url.as_str() == "about:blank"
                || matches!(url.scheme(), "http" | "https")
        })
        .on_new_window(move |url, _| {
            if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() {
                let _ = app_nova_janela.emit_to(
                    EventTarget::webview("main"),
                    "abrir-link-nova-aba",
                    url.to_string(),
                );
            }
            tauri::webview::NewWindowResponse::Deny
        })
        .on_download(|webview, evento| {
            match evento {
                DownloadEvent::Requested { url, destination } => {
                    if let Ok(config) = configuracoes::carregar(webview.app_handle()) {
                        if let Some(pasta) = config.pasta_downloads {
                            if configuracoes::validar_pasta_downloads(&pasta).is_ok() {
                                if let Some(nome) = destination.file_name() {
                                    *destination = destino_disponivel(&pasta, Path::new(nome));
                                }
                            } else {
                                let _ = webview.app_handle().emit_to(EventTarget::webview("main"), "download-pasta-falha", "A pasta configurada está indisponível; o arquivo será salvo em Downloads.");
                            }
                        }
                    }
                    let nome = destination.file_name().map(|nome| nome.to_string_lossy().to_string()).unwrap_or_else(|| url
                        .path_segments()
                        .and_then(|mut partes| partes.next_back())
                        .filter(|parte| !parte.is_empty())
                        .unwrap_or("arquivo")
                        .to_string());
                    let tipo = if nome.starts_with("Canoa-leitura-offline-") { "leitura-offline" } else { "arquivo" };
                    let id = iniciar_registro_download(webview.app_handle(), &nome, tipo, None);
                    vincular_url_download_nativo(webview.app_handle(), url.as_str(), &id);
                }
                DownloadEvent::Finished { url, path, success } => {
                    let id = consumir_id_download_nativo(webview.app_handle(), url.as_str())
                        .unwrap_or_else(|| format!("{}-native", instante_ms()));
                    let tipo = remover_registro_download(webview.app_handle(), &id)
                        .map(|registro| registro.tipo)
                        .unwrap_or_else(|| "arquivo".into());
                    let endereco_registro = if matches!(url.scheme(), "http" | "https") {
                        url.to_string()
                    } else {
                        let estado = webview.app_handle().state::<Estado>();
                        estado.0.lock().ok()
                            .and_then(|estado| estado.itens.iter().find(|aba| aba.id == estado.ativa).and_then(|aba| aba.endereco.clone()))
                            .unwrap_or_default()
                    };
                    let nome = path
                        .as_ref()
                        .and_then(|caminho| caminho.file_name())
                        .map(|nome| nome.to_string_lossy().to_string())
                        .or_else(|| {
                            url.path_segments()
                                .and_then(|mut partes| partes.next_back())
                                .map(str::to_string)
                        })
                        .unwrap_or_else(|| "arquivo".into());
                    let informacao = DownloadInfo {
                        id,
                        nome: nome.clone(),
                        caminho: path.as_ref().map(|caminho| caminho.to_string_lossy().to_string()),
                        sucesso: success,
                        tipo: tipo.clone(),
                    };
                    registrar_historico_download(
                        webview.app_handle(),
                        configuracoes::ItemDownload {
                            nome: informacao.nome.clone(),
                        caminho: informacao.caminho.as_ref().map(PathBuf::from),
                        sucesso: informacao.sucesso,
                        registrado_em: instante_ms(),
                        tipo: tipo.clone(),
                        },
                    );
                    if success {
                        if let Some(caminho) = path {
                            if let Ok(mut configuracao) = configuracoes::carregar(webview.app_handle()) {
                                match biblioteca::registrar_download(
                                    &mut configuracao,
                                    &endereco_registro,
                                    &nome,
                                    caminho.clone(),
                                ) {
                                    Ok(true) => {
                                        if tipo == "leitura-offline" {
                                            if let Some(item) = configuracao.salvos.iter_mut().find(|item| item.caminho.as_ref() == Some(&caminho)) {
                                                item.tipo = "leitura-offline".into();
                                            }
                                        }
                                        if let Err(erro) = configuracoes::salvar(webview.app_handle(), &configuracao) {
                                            eprintln!("CANOA_BIBLIOTECA_SALVAR_ERRO {erro}");
                                        } else {
                                            let _ = webview.app_handle().emit_to(EventTarget::webview("main"), "biblioteca-atualizada", ());
                                        }
                                    }
                                    Ok(false) => {}
                                    Err(erro) => eprintln!("CANOA_BIBLIOTECA_DOWNLOAD_ERRO {erro}"),
                                }
                            }
                        }
                    }
                    let _ = webview.app_handle().emit_to(
                        EventTarget::webview("main"),
                        "download-finalizado",
                        informacao,
                    );
                }
                _ => {}
            }
            true
        })
        .on_page_load(move |webview, carga| {
            if carga.url().as_str() == "about:blank" {
                return;
            }
            let endereco = carga.url().to_string();
            let app = webview.app_handle();
            let atual = {
                let estado_compartilhado = app.state::<Estado>();
                let Ok(mut estado) = estado_compartilhado.0.lock() else {
                    return;
                };
                if estado.ativa != id {
                    false
                } else {
                    if let Some(aba) = estado.itens.iter_mut().find(|aba| aba.id == id) {
                        aba.endereco = Some(endereco.clone());
                        aba.titulo = carga.url().host_str().unwrap_or("Página").to_string();
                    }
                    true
                }
            };
            if !atual {
                return;
            }
            avisar(app);
            match carga.event() {
                PageLoadEvent::Started => {
                    println!("CANOA_NAVEGACAO_INICIADA {endereco}");
                    let _ = app.emit_to(EventTarget::webview("main"), "pagina-iniciada", endereco);
                }
                PageLoadEvent::Finished => {
                    println!("CANOA_NAVEGACAO_ENCERRADA {endereco}");
                    let app_titulo = app.clone();
                    let titulo_alternativo = carga.url().host_str().unwrap_or("Página").to_string();
                    let titulo_aba = id;
                    let url_historico = endereco.clone();
                    if let Err(erro) = webview.eval_with_callback(
                        "document.title",
                        move |valor| {
                            let titulo = titulo_pagina(&valor, &titulo_alternativo);
                            let estado = app_titulo.state::<Estado>();
                            let Ok(mut abas) = estado.0.lock() else { return; };
                            let existe = if let Some(aba) = abas.itens.iter_mut().find(|aba| aba.id == titulo_aba) {
                                aba.titulo = titulo.clone();
                                true
                            } else { false };
                            let retrato = (existe && abas.ativa == titulo_aba).then(|| abas.retrato());
                            drop(abas);
                            registrar_visita_historico(&app_titulo, &url_historico, &titulo);
                            if let Some(retrato) = retrato {
                                let _ = app_titulo.emit_to(EventTarget::webview("main"), "abas-atualizadas", retrato);
                            }
                        },
                    ) {
                        eprintln!("CANOA_TITULO_PAGINA_ERRO {erro}");
                    }
                    let app = app.clone();
                    let url = endereco.clone();
                    let sondagem = webview.eval_with_callback(
                        "location.protocol === 'chrome-error:'",
                        move |valor| {
                            let evento = if valor == "true" {
                                "pagina-erro"
                            } else {
                                "pagina-encerrada"
                            };
                            let _ = app.emit_to(EventTarget::webview("main"), evento, url.clone());
                        },
                    );
                    if sondagem.is_err() {
                        let _ = webview.app_handle().emit_to(
                            EventTarget::webview("main"),
                            "pagina-encerrada",
                            endereco,
                        );
                    }
                }
            }
        });
    let limites = area(window)?;
    let webview = window
        .add_child(builder, limites.position, limites.size)
        .map_err(|erro| format!("Não foi possível abrir a página: {erro}"))?;
    #[cfg(windows)]
    if mutada {
        if let Err(erro) = audio_windows::definir_mudo(&webview, true) {
            eprintln!("CANOA_AUDIO_MUDO_ERRO {erro}");
        }
    }
    #[cfg(windows)]
    if let Err(erro) = fonte_windows::acompanhar(&webview, id) {
        eprintln!("CANOA_FONTE_ERRO {erro}");
    }
    #[cfg(windows)]
    {
        let protecao = window.state::<Arc<bloqueio::Protecao>>().inner().clone();
        if let Err(erro) =
            filtro_windows::instalar_e_navegar(&webview, url.clone(), protecao, origem_privacidade)
        {
            eprintln!("CANOA_BLOQUEADOR_REGISTRO_ERRO {erro}");
            webview
                .navigate(url)
                .map_err(|erro| format!("Não foi possível abrir a página: {erro}"))?;
        }
    }
    #[cfg(not(windows))]
    webview.navigate(url).map_err(|erro| erro.to_string())?;
    println!("CANOA_WEBVIEW_ABERTA aba={id}");
    Ok(())
}

fn parar(app: &AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview("conteudo") {
        webview
            .close()
            .map_err(|erro| format!("Não foi possível pausar a aba: {erro}"))?;
        println!("CANOA_WEBVIEW_FECHADA");
    }
    Ok(())
}

#[tauri::command]
fn listar_abas(app: AppHandle) -> Result<Retrato, String> {
    retrato(&app)
}

fn registrar_visita_historico(app: &AppHandle, endereco: &str, titulo: &str) {
    let Ok(mut config) = configuracoes::carregar(app) else {
        return;
    };
    if !config.historico_ativo {
        return;
    }
    sessao::registrar_visita(&mut config, endereco, titulo, sessao::agora_epoch());
    if let Err(erro) = configuracoes::salvar(app, &config) {
        eprintln!("CANOA_HISTORICO_SALVAR_ERRO {erro}");
    } else {
        let _ = app.emit_to(EventTarget::webview("main"), "historico-atualizado", ());
    }
}

#[tauri::command]
fn definir_bloqueador(app: AppHandle, ativo: bool) -> Result<configuracoes::Configuracoes, String> {
    let mut config = configuracoes::carregar(&app)?;
    config.bloqueador_ativo = ativo;
    configuracoes::salvar(&app, &config)?;
    app.state::<Arc<bloqueio::Protecao>>()
        .atualizar(ativo, config.excecoes_bloqueador.clone());
    Ok(config)
}

#[tauri::command]
fn definir_limpeza_rastreamento(
    app: AppHandle,
    ativo: bool,
) -> Result<configuracoes::Configuracoes, String> {
    let mut config = configuracoes::carregar(&app)?;
    config.limpar_rastreamento = ativo;
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn alternar_site_bloqueado(
    app: AppHandle,
    dominio: String,
) -> Result<configuracoes::Configuracoes, String> {
    let dominio = privacidade::normalizar_dominio(&dominio)?;
    let mut config = configuracoes::carregar(&app)?;
    if config.sites_bloqueados.iter().any(|item| item == &dominio) {
        config.sites_bloqueados.retain(|item| item != &dominio);
    } else {
        if config.sites_bloqueados.len() >= 500 {
            return Err("A lista de sites bloqueados atingiu 500 domínios.".into());
        }
        config.sites_bloqueados.push(dominio);
        config.sites_bloqueados.sort();
    }
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn alternar_bloqueio_site_atual(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    let endereco = {
        let estado = app.state::<Estado>();
        let estado = estado
            .0
            .lock()
            .map_err(|_| "Estado das abas indisponível.")?;
        estado
            .itens
            .iter()
            .find(|aba| aba.id == estado.ativa)
            .and_then(|aba| aba.endereco.clone())
            .ok_or("Abra um site HTTP ou HTTPS antes de bloqueá-lo.")?
    };
    let host = validar(&endereco)?
        .host_str()
        .ok_or("Esta página não tem domínio válido.")?
        .to_string();
    alternar_site_bloqueado(app, host)
}

#[tauri::command]
fn alternar_excecao_bloqueador(
    app: AppHandle,
    dominio: Option<String>,
) -> Result<configuracoes::Configuracoes, String> {
    let dominio = if let Some(dominio) = dominio {
        let dominio = dominio.trim().trim_end_matches('.').to_ascii_lowercase();
        if dominio.len() > 253
            || dominio.is_empty()
            || !dominio.contains('.')
            || !dominio
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return Err("Domínio inválido para uma exceção de proteção.".into());
        }
        dominio
    } else {
        let endereco = {
            let estado = app.state::<Estado>();
            let estado = estado
                .0
                .lock()
                .map_err(|_| "Estado das abas indisponível.")?;
            estado
                .itens
                .iter()
                .find(|aba| aba.id == estado.ativa)
                .and_then(|aba| aba.endereco.clone())
                .ok_or("Abra uma página HTTP ou HTTPS antes de alterar a proteção.")?
        };
        let url = validar(&endereco)?;
        url.host_str()
            .ok_or("Esta página não tem um domínio válido.")?
            .to_ascii_lowercase()
    };
    let mut config = configuracoes::carregar(&app)?;
    if config
        .excecoes_bloqueador
        .iter()
        .any(|item| item == &dominio)
    {
        config.excecoes_bloqueador.retain(|item| item != &dominio);
    } else {
        if config.excecoes_bloqueador.len() >= 500 {
            return Err("A lista de exceções atingiu 500 sites.".into());
        }
        config.excecoes_bloqueador.push(dominio);
        config.excecoes_bloqueador.sort();
        config.excecoes_bloqueador.dedup();
    }
    configuracoes::salvar(&app, &config)?;
    app.state::<Arc<bloqueio::Protecao>>()
        .atualizar(config.bloqueador_ativo, config.excecoes_bloqueador.clone());
    Ok(config)
}

#[tauri::command]
fn faixa_aviso(window: Window, visivel: bool) -> Result<(), String> {
    *window
        .state::<Faixa>()
        .0
        .lock()
        .map_err(|_| "Falha ao mostrar aviso.")? = visivel;
    ajustar(&window);
    Ok(())
}

#[tauri::command]
fn abrir_menu_nativo(
    window: Window,
    tipo: String,
    x: f64,
    y: f64,
    pode_fechar_outras: Option<bool>,
    pode_fechar_direita: Option<bool>,
    tem_endereco: Option<bool>,
    tem_link: Option<bool>,
    tem_imagem: Option<bool>,
    favorito: Option<bool>,
    mutada: Option<bool>,
    texto_selecionado: Option<String>,
    grupo_atual: Option<String>,
    site_bloqueado: Option<bool>,
    protecao_pausada: Option<bool>,
) -> Result<(), String> {
    if let Some(texto) = texto_selecionado {
        if let Ok(mut salvo) = window.state::<TextoContexto>().0.lock() {
            *salvo = texto.chars().take(4_000).collect();
        }
    }
    let menu = if tipo == "abas" {
        let audio = MenuItemBuilder::with_id(
            "canoa_aba_mudo",
            if mutada.unwrap_or(false) {
                "Ativar áudio da aba"
            } else {
                "Mutar aba"
            },
        )
        .accelerator("Ctrl+Alt+M")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let agrupar = MenuItemBuilder::with_id(
            "canoa_aba_grupo",
            if grupo_atual.as_deref().is_some_and(|nome| !nome.is_empty()) {
                "Renomear ou mover para grupo…"
            } else {
                "Adicionar a um grupo…"
            },
        )
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let desagrupar = MenuItemBuilder::with_id("canoa_aba_desagrupar", "Remover do grupo")
            .enabled(grupo_atual.is_some())
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let central_baixados =
            MenuItemBuilder::with_id("canoa_principal_baixados", "Central de Baixados")
                .accelerator("Ctrl+Alt+S")
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let bloqueio = MenuItemBuilder::with_id(
            "canoa_privacidade_bloquear",
            if site_bloqueado.unwrap_or(false) {
                "Desbloquear este site"
            } else {
                "Bloquear este site"
            },
        )
        .accelerator("Ctrl+Alt+B")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let protecao = MenuItemBuilder::with_id(
            "canoa_privacidade_protecao",
            if protecao_pausada.unwrap_or(false) {
                "Retomar proteção neste site"
            } else {
                "Pausar proteção neste site"
            },
        )
        .accelerator("Ctrl+Alt+P")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let recarregar = MenuItemBuilder::with_id("canoa_aba_recarregar", "Recarregar")
            .enabled(tem_endereco.unwrap_or(false))
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let duplicar = MenuItemBuilder::with_id("canoa_aba_duplicar", "Duplicar aba")
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let fechar = MenuItemBuilder::with_id("canoa_aba_fechar", "Fechar aba")
            .accelerator("Ctrl+W")
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let outras = MenuItemBuilder::with_id("canoa_aba_fechar_outras", "Fechar outras abas")
            .enabled(pode_fechar_outras.unwrap_or(false))
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let direita = MenuItemBuilder::with_id("canoa_aba_fechar_direita", "Fechar abas à direita")
            .enabled(pode_fechar_direita.unwrap_or(false))
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        MenuBuilder::new(&window)
            .item(&agrupar)
            .item(&desagrupar)
            .item(&central_baixados)
            .separator()
            .item(&bloqueio)
            .item(&protecao)
            .separator()
            .item(&recarregar)
            .item(&audio)
            .item(&duplicar)
            .separator()
            .item(&fechar)
            .item(&outras)
            .item(&direita)
            .build()
    } else if tipo == "pagina" {
        let audio = MenuItemBuilder::with_id(
            "canoa_pagina_mudo",
            if mutada.unwrap_or(false) {
                "Ativar áudio da página"
            } else {
                "Mutar página"
            },
        )
        .accelerator("Ctrl+Alt+M")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let favorito = MenuItemBuilder::with_id(
            "canoa_pagina_favorito",
            if favorito.unwrap_or(false) {
                "Remover dos Favoritos"
            } else {
                "Adicionar aos Favoritos"
            },
        )
        .accelerator("Ctrl+D")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let leitura =
            MenuItemBuilder::with_id("canoa_principal_ler_depois", "Salvar na Lista de leitura")
                .accelerator("Ctrl+Alt+R")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let salvo =
            MenuItemBuilder::with_id("canoa_principal_salvar_pagina", "Guardar link em Baixados")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let fontes = MenuItemBuilder::with_id("canoa_pagina_fontes", "Gerenciar Fontes")
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let abrir_link =
            MenuItemBuilder::with_id("canoa_pagina_abrir_link", "Abrir link em nova aba")
                .enabled(tem_link.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let abrir_imagem =
            MenuItemBuilder::with_id("canoa_pagina_abrir_imagem", "Abrir imagem em nova aba")
                .enabled(tem_imagem.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let copiar_texto = MenuItemBuilder::with_id("canoa_pagina_copiar_texto", "Copiar texto")
            .enabled(
                window
                    .state::<TextoContexto>()
                    .0
                    .lock()
                    .map(|texto| !texto.trim().is_empty())
                    .unwrap_or(false),
            )
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let voltar = MenuItemBuilder::with_id("canoa_pagina_voltar", "Voltar")
            .accelerator("Alt+Left")
            .enabled(tem_endereco.unwrap_or(false))
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let avancar = MenuItemBuilder::with_id("canoa_pagina_avancar", "Avançar")
            .accelerator("Alt+Right")
            .enabled(tem_endereco.unwrap_or(false))
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let copiar_link =
            MenuItemBuilder::with_id("canoa_pagina_copiar_link", "Copiar link da página")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let salvar_como =
            MenuItemBuilder::with_id("canoa_pagina_salvar_como", "Salvar página como…")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let recarregar = MenuItemBuilder::with_id("canoa_pagina_recarregar", "Recarregar")
            .accelerator("Ctrl+R")
            .enabled(tem_endereco.unwrap_or(false))
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let imprimir = MenuItemBuilder::with_id("canoa_pagina_imprimir", "Imprimir")
            .accelerator("Ctrl+P")
            .enabled(tem_endereco.unwrap_or(false))
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let baixar = MenuItemBuilder::with_id(
            "canoa_principal_baixar_recursos",
            "Baixar itens desta página",
        )
        .accelerator("Ctrl+Shift+S")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let limpar_cookies =
            MenuItemBuilder::with_id("canoa_limpar_cookies_site", "Limpar cookies deste site…")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let acesso_artigo =
            MenuItemBuilder::with_id("canoa_acesso_artigo", "Opções de acesso ao artigo")
                .accelerator("Ctrl+Alt+A")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let central_baixados =
            MenuItemBuilder::with_id("canoa_principal_baixados", "Central de Baixados")
                .accelerator("Ctrl+Alt+S")
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let bloqueio = MenuItemBuilder::with_id(
            "canoa_privacidade_bloquear",
            if site_bloqueado.unwrap_or(false) {
                "Desbloquear este site"
            } else {
                "Bloquear este site"
            },
        )
        .accelerator("Ctrl+Alt+B")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let protecao = MenuItemBuilder::with_id(
            "canoa_privacidade_protecao",
            if protecao_pausada.unwrap_or(false) {
                "Retomar proteção neste site"
            } else {
                "Pausar proteção neste site"
            },
        )
        .accelerator("Ctrl+Alt+P")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        MenuBuilder::new(&window)
            .item(&favorito)
            .item(&leitura)
            .item(&salvo)
            .item(&baixar)
            .item(&limpar_cookies)
            .item(&central_baixados)
            .item(&acesso_artigo)
            .item(&bloqueio)
            .item(&protecao)
            .item(&fontes)
            .separator()
            .item(&audio)
            .item(&copiar_texto)
            .item(&voltar)
            .item(&avancar)
            .item(&recarregar)
            .item(&salvar_como)
            .item(&imprimir)
            .item(&copiar_link)
            .separator()
            .item(&abrir_link)
            .item(&abrir_imagem)
            .build()
    } else {
        let configuracoes =
            MenuItemBuilder::with_id("canoa_principal_configuracoes", "Configurações")
                .accelerator("Ctrl+,")
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let tarefas = MenuItemBuilder::with_id("canoa_principal_tarefas", "Tarefas")
            .accelerator("Shift+Esc")
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let memoria =
            MenuItemBuilder::with_id("canoa_principal_memoria", "Mostrar ou ocultar memória")
                .accelerator("Ctrl+Shift+M")
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let leitura =
            MenuItemBuilder::with_id("canoa_principal_ler_depois", "Salvar na Lista de leitura")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let salvo =
            MenuItemBuilder::with_id("canoa_principal_salvar_pagina", "Guardar link em Baixados")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let copiar =
            MenuItemBuilder::with_id("canoa_principal_copiar_link", "Copiar link da página")
                .enabled(tem_endereco.unwrap_or(false))
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let baixar = MenuItemBuilder::with_id(
            "canoa_principal_baixar_recursos",
            "Baixar itens desta página",
        )
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let acesso_artigo = MenuItemBuilder::with_id("canoa_acesso_artigo", "Acesso ao artigo")
            .accelerator("Ctrl+Alt+A")
            .enabled(tem_endereco.unwrap_or(false))
            .build(&window)
            .map_err(|erro| erro.to_string())?;
        let central_baixados =
            MenuItemBuilder::with_id("canoa_principal_baixados", "Central de Baixados")
                .accelerator("Ctrl+Alt+S")
                .build(&window)
                .map_err(|erro| erro.to_string())?;
        let bloqueio = MenuItemBuilder::with_id(
            "canoa_privacidade_bloquear",
            if site_bloqueado.unwrap_or(false) {
                "Desbloquear este site"
            } else {
                "Bloquear este site"
            },
        )
        .accelerator("Ctrl+Alt+B")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        let protecao = MenuItemBuilder::with_id(
            "canoa_privacidade_protecao",
            if protecao_pausada.unwrap_or(false) {
                "Retomar proteção neste site"
            } else {
                "Pausar proteção neste site"
            },
        )
        .accelerator("Ctrl+Alt+P")
        .enabled(tem_endereco.unwrap_or(false))
        .build(&window)
        .map_err(|erro| erro.to_string())?;
        MenuBuilder::new(&window)
            .item(&configuracoes)
            .item(&tarefas)
            .item(&memoria)
            .separator()
            .item(&leitura)
            .item(&salvo)
            .item(&baixar)
            .item(&central_baixados)
            .item(&acesso_artigo)
            .item(&bloqueio)
            .item(&protecao)
            .separator()
            .item(&copiar)
            .build()
    }
    .map_err(|erro| format!("Não foi possível montar o menu: {erro}"))?;
    let escala = window.scale_factor().map_err(|erro| erro.to_string())?;
    let tamanho = window.inner_size().map_err(|erro| erro.to_string())?;
    let largura = f64::from(tamanho.width) / escala;
    let altura = f64::from(tamanho.height) / escala;
    let (largura_menu, altura_menu) = if tipo == "abas" {
        (280.0, 260.0)
    } else if tipo == "pagina" {
        (350.0, 370.0)
    } else {
        (360.0, 260.0)
    };
    let x = x.clamp(8.0, (largura - largura_menu).max(8.0));
    let y = y.clamp(8.0, (altura - altura_menu).max(8.0));
    window
        .popup_menu_at(&menu, LogicalPosition::new(x, y))
        .map_err(|erro| format!("Não foi possível abrir o menu: {erro}"))
}

#[tauri::command]
fn copiar_endereco_atual(window: Window) -> Result<(), String> {
    let endereco = {
        let estado = window.state::<Estado>();
        let abas = estado.0.lock().map_err(|_| "Falha ao copiar o link.")?;
        abas.itens
            .iter()
            .find(|aba| aba.id == abas.ativa)
            .and_then(|aba| aba.endereco.clone())
            .filter(|url| validar(url).is_ok())
            .ok_or("A aba atual não contém um link http ou https para copiar.")?
    };
    #[cfg(windows)]
    {
        use std::ptr;
        use windows_sys::Win32::{
            Foundation::GlobalFree,
            System::{
                DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
                Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
            },
        };
        let mut texto: Vec<u16> = endereco.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = texto.len() * std::mem::size_of::<u16>();
        unsafe {
            if OpenClipboard(ptr::null_mut()) == 0 {
                return Err("Não foi possível acessar a área de transferência.".into());
            }
            let memoria = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if memoria.is_null() {
                CloseClipboard();
                return Err("Não foi possível reservar espaço para o link.".into());
            }
            let destino = GlobalLock(memoria) as *mut u16;
            if destino.is_null() {
                GlobalFree(memoria);
                CloseClipboard();
                return Err("Não foi possível preparar o link para cópia.".into());
            }
            ptr::copy_nonoverlapping(texto.as_mut_ptr(), destino, texto.len());
            GlobalUnlock(memoria);
            if EmptyClipboard() == 0 {
                GlobalFree(memoria);
                CloseClipboard();
                return Err("Não foi possível atualizar a área de transferência.".into());
            }
            if SetClipboardData(13, memoria).is_null() {
                GlobalFree(memoria);
                CloseClipboard();
                return Err("Não foi possível copiar o link.".into());
            }
            CloseClipboard();
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = endereco;
        Err("Copiar links ainda está disponível apenas no Windows.".into())
    }
}

#[tauri::command]
fn ler_configuracoes(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    configuracoes::carregar(&app)
}

#[tauri::command]
fn resumo_cofre(window: WebviewWindow, app: AppHandle) -> Result<senhas::ResumoCofre, String> {
    validar_interface_cofre(&window)?;
    Ok(app.state::<senhas::Cofre>().resumo())
}

#[tauri::command]
fn criar_cofre(
    window: WebviewWindow,
    app: AppHandle,
    senha_mestra: String,
) -> Result<senhas::ResumoCofre, String> {
    validar_interface_cofre(&window)?;
    app.state::<senhas::Cofre>().criar(senha_mestra)
}

#[tauri::command]
fn desbloquear_cofre(
    window: WebviewWindow,
    app: AppHandle,
    senha_mestra: String,
) -> Result<senhas::ResumoCofre, String> {
    validar_interface_cofre(&window)?;
    app.state::<senhas::Cofre>().desbloquear(senha_mestra)
}

#[tauri::command]
fn bloquear_cofre(window: WebviewWindow, app: AppHandle) -> Result<senhas::ResumoCofre, String> {
    validar_interface_cofre(&window)?;
    app.state::<senhas::Cofre>().travar()
}

#[tauri::command]
fn listar_senhas(
    window: WebviewWindow,
    app: AppHandle,
) -> Result<Vec<senhas::ResumoCredencial>, String> {
    validar_interface_cofre(&window)?;
    app.state::<senhas::Cofre>().listar()
}

#[tauri::command]
fn salvar_senha(
    window: WebviewWindow,
    app: AppHandle,
    id: Option<String>,
    endereco: String,
    titulo: String,
    usuario: String,
    senha: String,
) -> Result<senhas::ResumoCredencial, String> {
    validar_interface_cofre(&window)?;
    let cofre = app.state::<senhas::Cofre>();
    let origem = senhas::normalizar_origem(&endereco)?;
    let mut config = configuracoes::carregar(&app)?;
    let config_original = config.clone();
    if id.is_none() {
        favoritos::marcar_senha_associada(&mut config, &origem, true)?;
        configuracoes::salvar(&app, &config)?;
    }
    match cofre.salvar(id.as_deref(), &origem, &titulo, &usuario, senha) {
        Ok(credencial) => Ok(credencial),
        Err(erro) => {
            if id.is_none() {
                let _ = configuracoes::salvar(&app, &config_original);
            }
            Err(erro)
        }
    }
}

#[tauri::command]
fn remover_senha(
    window: WebviewWindow,
    app: AppHandle,
    id: String,
) -> Result<senhas::ResumoCofre, String> {
    validar_interface_cofre(&window)?;
    let cofre = app.state::<senhas::Cofre>();
    let (credencial, remover_marcador) = cofre.remover(&id)?;
    if remover_marcador {
        let mut config = configuracoes::carregar(&app)?;
        favoritos::marcar_senha_associada(&mut config, &credencial.origem, false)?;
        configuracoes::salvar(&app, &config)?;
    }
    Ok(cofre.resumo())
}

#[tauri::command]
fn copiar_senha(window: WebviewWindow, app: AppHandle, id: String) -> Result<(), String> {
    validar_interface_cofre(&window)?;
    let senha = app.state::<senhas::Cofre>().senha_para_copiar(&id)?;
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|_| "Não foi possível acessar a área de transferência.".to_string())?;
    clipboard
        .set_text(senha.as_str())
        .map_err(|_| "Não foi possível copiar a senha.".to_string())?;
    let para_limpar = senha.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(30));
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.get_text().ok().as_deref() == Some(para_limpar.as_str()) {
                let _ = clipboard.set_text(String::new());
            }
        }
    });
    Ok(())
}

fn validar_interface_cofre(window: &WebviewWindow) -> Result<(), String> {
    if window.label() != "main" {
        return Err("O cofre só pode ser acessado pela interface principal do Canoa.".into());
    }
    let url = window
        .url()
        .map_err(|_| "Não foi possível validar a origem da interface do cofre.".to_string())?;
    if !origem_cofre_confiavel(&url) {
        return Err("O cofre só pode ser acessado pela origem local do Canoa.".into());
    }
    Ok(())
}

fn origem_cofre_confiavel(url: &Url) -> bool {
    let host = url.host_str().unwrap_or_default();
    (url.scheme() == "tauri" && host == "localhost")
        || (matches!(url.scheme(), "http" | "https") && host == "tauri.localhost")
        || (cfg!(debug_assertions)
            && url.scheme() == "http"
            && matches!(host, "localhost" | "127.0.0.1")
            && url.port() == Some(1420))
}

#[tauri::command]
fn resumo_bloqueador(app: AppHandle) -> bloqueio::ResumoProtecao {
    app.state::<Arc<bloqueio::Protecao>>().resumo()
}

#[tauri::command]
async fn atualizar_listas_bloqueio(app: AppHandle) -> Result<bloqueio::ResumoProtecao, String> {
    let pasta = app
        .path()
        .app_data_dir()
        .map_err(|erro| format!("Não foi possível localizar os dados do aplicativo: {erro}"))?
        .join("listas-bloqueador");
    let listas =
        tauri::async_runtime::spawn_blocking(move || atualizador_listas::baixar_e_salvar(&pasta))
            .await
            .map_err(|erro| format!("A tarefa de atualização falhou: {erro}"))??;
    let protecao = app.state::<Arc<bloqueio::Protecao>>();
    protecao.carregar_listas(&listas);
    let resumo = protecao.resumo();
    let _ = app.emit_to(
        EventTarget::webview("main"),
        "contador-bloqueador-atualizado",
        resumo.clone(),
    );
    Ok(resumo)
}

#[tauri::command]
async fn reverter_listas_bloqueio(app: AppHandle) -> Result<bloqueio::ResumoProtecao, String> {
    let pasta = app
        .path()
        .app_data_dir()
        .map_err(|erro| format!("Não foi possível localizar os dados do aplicativo: {erro}"))?
        .join("listas-bloqueador");
    let listas = tauri::async_runtime::spawn_blocking(move || {
        atualizador_listas::reverter_para_anterior(&pasta)
    })
    .await
    .map_err(|erro| format!("A tarefa de reversão falhou: {erro}"))??;
    let protecao = app.state::<Arc<bloqueio::Protecao>>();
    protecao.carregar_listas(&listas);
    let resumo = protecao.resumo();
    let _ = app.emit_to(
        EventTarget::webview("main"),
        "contador-bloqueador-atualizado",
        resumo.clone(),
    );
    Ok(resumo)
}

#[tauri::command]
fn definir_historico_navegacao(
    app: AppHandle,
    ativo: bool,
    dias: u16,
) -> Result<configuracoes::Configuracoes, String> {
    let mut config = configuracoes::carregar(&app)?;
    config.historico_ativo = ativo;
    config.dias_historico = dias.clamp(1, 365);
    if ativo {
        let agora = sessao::agora_epoch();
        let desde = agora.saturating_sub(u64::from(config.dias_historico) * 86_400);
        config
            .historico_navegacao
            .retain(|item| item.visitado_em >= desde);
    }
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn remover_entrada_historico(
    app: AppHandle,
    endereco: String,
) -> Result<configuracoes::Configuracoes, String> {
    let mut config = configuracoes::carregar(&app)?;
    config
        .historico_navegacao
        .retain(|item| item.endereco != endereco);
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn limpar_historico_navegacao(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    let mut config = configuracoes::carregar(&app)?;
    config.historico_navegacao.clear();
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn definir_limite_site(
    app: AppHandle,
    dominio: String,
    minutos: u16,
) -> Result<configuracoes::Configuracoes, String> {
    let dominio = privacidade::normalizar_dominio(&dominio)?;
    if !(1..=1_440).contains(&minutos) {
        return Err("Defina um limite entre 1 e 1.440 minutos por dia.".into());
    }
    let mut config = configuracoes::carregar(&app)?;
    if let Some(item) = config
        .limites_sites
        .iter_mut()
        .find(|item| item.dominio == dominio)
    {
        item.minutos_diarios = minutos;
    } else {
        if config.limites_sites.len() >= 200 {
            return Err("A lista atingiu 200 domínios.".into());
        }
        config.limites_sites.push(configuracoes::LimiteSite {
            dominio,
            minutos_diarios: minutos,
        });
        config
            .limites_sites
            .sort_by(|a, b| a.dominio.cmp(&b.dominio));
    }
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn remover_limite_site(
    app: AppHandle,
    dominio: String,
) -> Result<configuracoes::Configuracoes, String> {
    let dominio = privacidade::normalizar_dominio(&dominio)?;
    let mut config = configuracoes::carregar(&app)?;
    config.limites_sites.retain(|item| item.dominio != dominio);
    config.uso_diario_sites.retain(|uso| uso.dominio != dominio);
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[derive(serde::Serialize)]
struct EstadoUsoSite {
    dominio: String,
    segundos: u64,
    limite_segundos: u64,
    atingido: bool,
}

#[tauri::command]
fn registrar_tempo_site(
    app: AppHandle,
    dominio: String,
    segundos: u64,
) -> Result<Option<EstadoUsoSite>, String> {
    let dominio = privacidade::normalizar_dominio(&dominio)?;
    let endereco_atual = pagina_ativa(&app)?.0;
    let host_atual = validar(&endereco_atual)?
        .host_str()
        .map(str::to_string)
        .ok_or("A página ativa não tem domínio.")?;
    if !sessao::dominio_corresponde(&host_atual, &dominio) {
        return Ok(None);
    }
    let mut config = configuracoes::carregar(&app)?;
    if !config
        .limites_sites
        .iter()
        .any(|item| sessao::dominio_corresponde(&host_atual, &item.dominio))
    {
        return Ok(None);
    }
    let Some((usado, maximo)) = sessao::registrar_uso(&mut config, &host_atual, segundos) else {
        return Ok(None);
    };
    configuracoes::salvar(&app, &config)?;
    let atingido = usado >= maximo;
    if atingido {
        let rotulo = serde_json::to_string(&host_atual).unwrap_or_else(|_| "\"site\"".into());
        if let Some(webview) = app.get_webview("conteudo") {
            let html = format!(
                "(() => {{ const raiz=document.documentElement; if(!raiz)return; raiz.innerHTML=''; const corpo=document.createElement('body'); corpo.style.cssText='margin:0;min-height:100vh;background:#f5f3ea;color:#103b3b;font:16px Segoe UI, sans-serif;display:grid;place-items:center'; const caixa=document.createElement('main'); caixa.style.cssText='max-width:560px;padding:32px;text-align:center'; const titulo=document.createElement('h1'); titulo.textContent='Limite diário atingido'; const texto=document.createElement('p'); texto.textContent='O tempo configurado para '+{rotulo}+' terminou por hoje. Você pode continuar usando outras páginas.'; caixa.append(titulo,texto); corpo.append(caixa); raiz.append(corpo); }})()"
            );
            let _ = webview.eval(&html);
        }
        let _ = app.emit_to(
            EventTarget::webview("main"),
            "site-limite-atingido",
            host_atual.clone(),
        );
    }
    Ok(Some(EstadoUsoSite {
        dominio: host_atual,
        segundos: usado,
        limite_segundos: maximo,
        atingido,
    }))
}

#[tauri::command]
fn definir_sites_foco(
    app: AppHandle,
    dominio: String,
    adicionar: bool,
) -> Result<configuracoes::Configuracoes, String> {
    let dominio = privacidade::normalizar_dominio(&dominio)?;
    let mut config = configuracoes::carregar(&app)?;
    if adicionar {
        if !config.sites_foco.contains(&dominio) {
            if config.sites_foco.len() >= 200 {
                return Err("A lista do modo foco atingiu 200 domínios.".into());
            }
            config.sites_foco.push(dominio);
            config.sites_foco.sort();
        }
    } else {
        config.sites_foco.retain(|item| item != &dominio);
    }
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn definir_duracao_foco(
    app: AppHandle,
    minutos: u16,
) -> Result<configuracoes::Configuracoes, String> {
    if !(5..=180).contains(&minutos) {
        return Err("O modo foco deve durar entre 5 e 180 minutos.".into());
    }
    let mut config = configuracoes::carregar(&app)?;
    config.duracao_foco_minutos = minutos;
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn alternar_modo_foco(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    let mut config = configuracoes::carregar(&app)?;
    let agora = sessao::agora_epoch();
    let encerrando = config.foco_ate_epoch.is_some_and(|ate| ate > agora);
    if !encerrando && config.sites_foco.is_empty() {
        return Err(
            "Adicione ao menos um domínio para bloquear antes de iniciar o modo foco.".into(),
        );
    }
    config.foco_ate_epoch = if encerrando {
        None
    } else {
        Some(agora.saturating_add(u64::from(config.duracao_foco_minutos.clamp(5, 180)) * 60))
    };
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn limpar_cookies_pagina(app: AppHandle) -> Result<(), String> {
    #[cfg(windows)]
    {
        let endereco = pagina_ativa(&app)?.0;
        validar(&endereco)?;
        let principal = app
            .get_webview("main")
            .ok_or("A interface principal do Canoa não está disponível.")?;
        limpeza_windows::limpar_cookies_site(&principal, app.clone(), endereco)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("A limpeza de cookies está disponível no WebView2 do Windows.".into())
    }
}

#[tauri::command]
fn limpar_cookies_e_cache(app: AppHandle) -> Result<(), String> {
    #[cfg(windows)]
    {
        let principal = app
            .get_webview("main")
            .ok_or("A interface principal do Canoa não está disponível.")?;
        limpeza_windows::limpar_perfil(&principal, app.clone())
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("A limpeza de cookies e cache está disponível no WebView2 do Windows.".into())
    }
}

#[tauri::command]
fn definir_memoria(app: AppHandle, mostrar: bool) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    valor.mostrar_memoria = mostrar;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn definir_buscador(
    app: AppHandle,
    buscador: configuracoes::Buscador,
) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    valor.buscador = buscador;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn definir_acoes_barra(
    app: AppHandle,
    acoes_barra: configuracoes::AcoesBarra,
) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    valor.acoes_barra = acoes_barra;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn adicionar_fonte(
    app: AppHandle,
    dominio: String,
    tema: String,
) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    fontes::adicionar(&mut valor, &dominio, &tema)?;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn remover_fonte(
    app: AppHandle,
    dominio: String,
    tema: String,
) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    fontes::remover(&mut valor, &dominio, &tema)?;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn alternar_favorito(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    let atual = retrato(&app)?;
    let aba = atual
        .abas
        .iter()
        .find(|aba| aba.id == atual.ativa)
        .ok_or("Aba não encontrada.")?;
    let endereco = aba
        .endereco
        .as_deref()
        .ok_or("Abra uma página para adicioná-la aos favoritos.")?;
    let mut valor = configuracoes::carregar(&app)?;
    favoritos::alternar(&mut valor, endereco, &aba.titulo)?;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

fn pagina_ativa(app: &AppHandle) -> Result<(String, String), String> {
    let atual = retrato(app)?;
    let aba = atual
        .abas
        .iter()
        .find(|aba| aba.id == atual.ativa)
        .ok_or("Aba não encontrada.")?;
    let endereco = aba
        .endereco
        .as_deref()
        .ok_or("Abra uma página para salvá-la.")?;
    if eh_configuracoes(endereco) {
        return Err("Abra uma página da Web para salvá-la.".into());
    }
    Ok((endereco.to_string(), aba.titulo.clone()))
}

#[tauri::command]
fn salvar_para_leitura(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    let (endereco, titulo) = pagina_ativa(&app)?;
    let mut valor = configuracoes::carregar(&app)?;
    biblioteca::salvar_para_leitura(&mut valor, &endereco, &titulo)?;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn alternar_lido(app: AppHandle, endereco: String) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    biblioteca::alternar_lido(&mut valor, &endereco)?;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn remover_item_leitura(
    app: AppHandle,
    endereco: String,
) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    biblioteca::remover_leitura(&mut valor, &endereco)?;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn salvar_pagina_em_salvos(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    let (endereco, titulo) = pagina_ativa(&app)?;
    let mut valor = configuracoes::carregar(&app)?;
    biblioteca::salvar_pagina(&mut valor, &endereco, &titulo)?;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn solicitar_recursos_pagina(app: AppHandle) -> Result<(), String> {
    app.get_webview("conteudo")
        .ok_or("Abra uma página antes de listar os itens disponíveis.")?
        .eval("window.dispatchEvent(new Event('canoa:solicitar-recursos'))")
        .map_err(|erro| erro.to_string())
}

#[tauri::command]
fn mostrar_conteudo_pagina(app: AppHandle, mostrar: bool) -> Result<(), String> {
    let webview = app
        .get_webview("conteudo")
        .ok_or("A página ativa não está disponível.")?;
    (if mostrar {
        webview.show()
    } else {
        webview.hide()
    })
    .map_err(|erro| erro.to_string())
}

#[tauri::command]
fn copiar_texto_contexto(app: AppHandle) -> Result<(), String> {
    let texto = app
        .state::<TextoContexto>()
        .0
        .lock()
        .map_err(|_| "Não foi possível acessar o texto selecionado.")?
        .clone();
    if texto.is_empty() {
        return Err("Nenhum texto foi selecionado.".into());
    }
    let mut area = arboard::Clipboard::new()
        .map_err(|erro| format!("Não foi possível abrir a área de transferência: {erro}"))?;
    area.set_text(texto)
        .map_err(|erro| format!("Não foi possível copiar o texto: {erro}"))
}

fn nome_arquivo_seguro(nome: &str, padrao: &str) -> String {
    let mut limpo: String = nome
        .chars()
        .map(|caractere| {
            if caractere.is_ascii_alphanumeric() || matches!(caractere, ' ' | '-' | '_' | '.') {
                caractere
            } else {
                '_'
            }
        })
        .take(120)
        .collect();
    limpo = limpo.trim_matches([' ', '.']).to_string();
    if limpo.is_empty() {
        padrao.to_string()
    } else {
        limpo
    }
}

fn registrar_historico_download(app: &AppHandle, item: configuracoes::ItemDownload) {
    let Ok(mut config) = configuracoes::carregar(app) else {
        return;
    };
    config.historico_downloads.insert(0, item);
    config.historico_downloads.truncate(200);
    if let Err(erro) = configuracoes::salvar(app, &config) {
        eprintln!("CANOA_HISTORICO_DOWNLOAD_ERRO {erro}");
    }
}

fn instante_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duracao| duracao.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

fn iniciar_registro_download(
    app: &AppHandle,
    nome: &str,
    tipo: &str,
    cancelamento: Option<Arc<AtomicBool>>,
) -> String {
    let gerente = app.state::<GerenciadorDownloads>();
    let id = format!(
        "{}-{}",
        instante_ms(),
        gerente.sequencia.fetch_add(1, Ordering::Relaxed)
    );
    let info = DownloadAtivo {
        id: id.clone(),
        nome: nome.to_string(),
        bytes_recebidos: 0,
        total_bytes: None,
        cancelavel: cancelamento.is_some(),
        tipo: tipo.to_string(),
    };
    if let Ok(mut itens) = gerente.itens.lock() {
        itens.insert(
            id.clone(),
            DownloadEmAndamento {
                info: info.clone(),
                cancelamento,
            },
        );
    }
    let _ = app.emit_to(EventTarget::webview("main"), "download-ativo", info);
    id
}

fn atualizar_registro_download(app: &AppHandle, id: &str, recebido: u64, total: Option<u64>) {
    let gerente = app.state::<GerenciadorDownloads>();
    let atualizado = gerente.itens.lock().ok().and_then(|mut itens| {
        let registro = itens.get_mut(id)?;
        registro.info.bytes_recebidos = recebido;
        registro.info.total_bytes = total;
        Some(registro.info.clone())
    });
    if let Some(info) = atualizado {
        let _ = app.emit_to(EventTarget::webview("main"), "download-progresso", info);
    }
}

fn vincular_url_download_nativo(app: &AppHandle, url: &str, id: &str) {
    let gerente = app.state::<GerenciadorDownloads>();
    if let Ok(mut urls) = gerente.urls_nativas.lock() {
        urls.entry(url.to_string())
            .or_default()
            .push(id.to_string());
    };
}

fn remover_registro_download(app: &AppHandle, id: &str) -> Option<DownloadAtivo> {
    let gerente = app.state::<GerenciadorDownloads>();
    let resultado = gerente
        .itens
        .lock()
        .ok()?
        .remove(id)
        .map(|registro| registro.info);
    resultado
}

fn consumir_id_download_nativo(app: &AppHandle, url: &str) -> Option<String> {
    let gerente = app.state::<GerenciadorDownloads>();
    let mut urls = gerente.urls_nativas.lock().ok()?;
    let ids = urls.get_mut(url)?;
    let id = (!ids.is_empty()).then(|| ids.remove(0));
    if ids.is_empty() {
        urls.remove(url);
    }
    id
}

#[tauri::command]
fn listar_downloads_ativos(app: AppHandle) -> Result<Vec<DownloadAtivo>, String> {
    let gerente = app.state::<GerenciadorDownloads>();
    let itens = gerente
        .itens
        .lock()
        .map_err(|_| "Não foi possível ler os downloads ativos.")?;
    Ok(itens.values().map(|item| item.info.clone()).collect())
}

#[tauri::command]
fn cancelar_download(app: AppHandle, id: String) -> Result<(), String> {
    let gerente = app.state::<GerenciadorDownloads>();
    let itens = gerente
        .itens
        .lock()
        .map_err(|_| "Não foi possível acessar o download.")?;
    let registro = itens.get(&id).ok_or("Este download não está mais ativo.")?;
    let cancelamento = registro.cancelamento.as_ref().ok_or("Este download está sendo gerenciado pelo WebView2 e não pode ser cancelado por este controle.")?;
    cancelamento.store(true, Ordering::Relaxed);
    Ok(())
}

fn iniciar_download_http(app: AppHandle, url: Url, nome: String, referer: String, tipo: String) {
    let cancelamento = Arc::new(AtomicBool::new(false));
    let id = iniciar_registro_download(&app, &nome, &tipo, Some(cancelamento.clone()));
    std::thread::spawn(move || {
        let resultado = (|| -> Result<PathBuf, String> {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(90))
                .connect_timeout(std::time::Duration::from_secs(10))
                .user_agent(concat!("Canoa/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|erro| erro.to_string())?;
            let mut pedido = client.get(url.clone());
            if let Ok(origem) = Url::parse(&referer) {
                if matches!(origem.scheme(), "http" | "https") {
                    pedido = pedido.header(reqwest::header::REFERER, origem.as_str());
                }
            }
            let mut resposta = pedido
                .send()
                .map_err(|erro| format!("Falha ao baixar o recurso: {erro}"))?;
            if !resposta.status().is_success() {
                return Err(format!(
                    "O servidor respondeu {} ao baixar o recurso.",
                    resposta.status()
                ));
            }
            const LIMITE_BYTES: u64 = 512 * 1024 * 1024;
            if resposta
                .content_length()
                .is_some_and(|tamanho| tamanho > LIMITE_BYTES)
            {
                return Err("O arquivo excede o limite de 512 MiB por download.".into());
            }
            let configuracao = configuracoes::carregar(&app)?;
            let pasta = match configuracao.pasta_downloads {
                Some(pasta) if configuracoes::validar_pasta_downloads(&pasta).is_ok() => pasta,
                _ => app.path().download_dir().map_err(|erro| erro.to_string())?,
            };
            std::fs::create_dir_all(&pasta)
                .map_err(|erro| format!("Não foi possível acessar a pasta de downloads: {erro}"))?;
            let destino = destino_disponivel(&pasta, Path::new(&nome));
            let temporario = destino.with_extension(format!(
                "{}.part",
                destino
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("download")
            ));
            let total = resposta.content_length();
            let mut arquivo =
                std::fs::File::create(&temporario).map_err(|erro| erro.to_string())?;
            let mut recebido = 0_u64;
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                if cancelamento.load(Ordering::Relaxed) {
                    drop(arquivo);
                    let _ = std::fs::remove_file(&temporario);
                    return Err("Download cancelado pelo usuário.".into());
                }
                let quantidade = match resposta.read(&mut buffer) {
                    Ok(quantidade) => quantidade,
                    Err(erro) if erro.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(erro) => {
                        drop(arquivo);
                        let _ = std::fs::remove_file(&temporario);
                        return Err(format!("Falha durante o download: {erro}"));
                    }
                };
                if quantidade == 0 {
                    break;
                }
                recebido = recebido.saturating_add(quantidade as u64);
                if recebido > LIMITE_BYTES {
                    drop(arquivo);
                    let _ = std::fs::remove_file(&temporario);
                    return Err("O arquivo excede o limite de 512 MiB por download.".into());
                }
                if let Err(erro) = arquivo.write_all(&buffer[..quantidade]) {
                    drop(arquivo);
                    let _ = std::fs::remove_file(&temporario);
                    return Err(erro.to_string());
                }
                atualizar_registro_download(&app, &id, recebido, total);
            }
            if let Err(erro) = arquivo.sync_all() {
                drop(arquivo);
                let _ = std::fs::remove_file(&temporario);
                return Err(erro.to_string());
            }
            std::fs::rename(&temporario, &destino).map_err(|erro| {
                let _ = std::fs::remove_file(&temporario);
                erro.to_string()
            })?;
            let mut configuracao = configuracoes::carregar(&app)?;
            if biblioteca::registrar_download(
                &mut configuracao,
                referer.as_str(),
                &nome,
                destino.clone(),
            )
            .unwrap_or(false)
            {
                let _ = configuracoes::salvar(&app, &configuracao);
                let _ = app.emit_to(EventTarget::webview("main"), "biblioteca-atualizada", ());
            }
            Ok(destino)
        })();
        let (caminho, sucesso) = match resultado {
            Ok(caminho) => (Some(caminho.to_string_lossy().to_string()), true),
            Err(erro) => {
                eprintln!("CANOA_DOWNLOAD_RECURSO_ERRO {erro}");
                (None, false)
            }
        };
        remover_registro_download(&app, &id);
        registrar_historico_download(
            &app,
            configuracoes::ItemDownload {
                nome: nome.clone(),
                caminho: caminho.as_ref().map(PathBuf::from),
                sucesso,
                registrado_em: instante_ms(),
                tipo: tipo.clone(),
            },
        );
        let _ = app.emit_to(
            EventTarget::webview("main"),
            "download-finalizado",
            DownloadInfo {
                id,
                nome,
                caminho,
                sucesso,
                tipo,
            },
        );
    });
}

#[tauri::command]
fn baixar_item_pagina(
    app: AppHandle,
    endereco: String,
    nome: String,
    tipo: String,
) -> Result<(), String> {
    let webview = app.get_webview("conteudo");
    if tipo == "pagina-offline" || tipo == "pagina-texto" {
        let titulo = nome_arquivo_seguro(&nome, "pagina");
        let (nome_saida, script, mime) = if tipo == "pagina-offline" {
            let nome_saida = format!("Canoa-leitura-offline-{titulo}.html");
            let nome_json = serde_json::to_string(&nome_saida).map_err(|erro| erro.to_string())?;
            let script = r#"(()=>{const e=s=>String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/\"/g,'&quot;').replace(/'/g,'&#39;');const t=document.title||'Leitura offline';const u=location.href;const c=document.body?.innerText||'';const h='<!doctype html><html lang=\"pt-BR\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>'+e(t)+'</title><style>body{max-width:760px;margin:40px auto;padding:0 20px;font:18px/1.65 system-ui,sans-serif;color:#173a3b}header{border-bottom:1px solid #cbdad3;margin-bottom:2rem;padding-bottom:1rem}small{color:#57706e}pre{white-space:pre-wrap;font:inherit}</style></head><body><header><h1>'+e(t)+'</h1><small>Captura local de '+e(u)+' · conteúdo visível no momento do salvamento</small></header><main><pre>'+e(c)+'</pre></main></body></html>';const b=new Blob([h],{type:'text/html;charset=utf-8'});const a=document.createElement('a');a.href=URL.createObjectURL(b);a.download=__NOME__;a.click();setTimeout(()=>URL.revokeObjectURL(a.href),60000)})()"#.replace("__NOME__", &nome_json);
            (nome_saida, script, "text/html;charset=utf-8")
        } else {
            let nome_saida = format!("{titulo}.txt");
            let nome_json = serde_json::to_string(&nome_saida).map_err(|erro| erro.to_string())?;
            let script = format!("(()=>{{const b=new Blob([document.body?.innerText||''],{{type:'text/plain;charset=utf-8'}});const a=document.createElement('a');a.href=URL.createObjectURL(b);a.download={nome_json};a.click();setTimeout(()=>URL.revokeObjectURL(a.href),60000)}})()");
            (nome_saida, script, "text/plain;charset=utf-8")
        };
        let _ = mime;
        let webview = webview.ok_or("Abra uma página antes de salvar o conteúdo.")?;
        let _ = nome_saida;
        return webview.eval(&script).map_err(|erro| erro.to_string());
    };
    let url = Url::parse(&endereco).map_err(|_| "Endereço do recurso inválido.")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || endereco.len() > 4_096
    {
        return Err("O recurso precisa ter um endereço HTTP ou HTTPS válido.".into());
    }
    let mut nome_saida = nome_arquivo_seguro(&nome, "recurso");
    if Path::new(&nome_saida).extension().is_none() {
        if let Some(extensao) = url
            .path_segments()
            .and_then(|mut partes| partes.next_back())
            .and_then(|parte| Path::new(parte).extension())
            .and_then(|ext| ext.to_str())
            .filter(|ext| ext.len() <= 8 && ext.chars().all(|c| c.is_ascii_alphanumeric()))
        {
            nome_saida.push('.');
            nome_saida.push_str(extensao);
        }
    }
    let referer = pagina_ativa(&app)
        .map(|(endereco, _)| endereco)
        .unwrap_or_default();
    iniciar_download_http(app, url, nome_saida, referer, tipo);
    Ok(())
}

#[tauri::command]
fn ler_leitura_offline(app: AppHandle, caminho: String) -> Result<String, String> {
    let solicitado = PathBuf::from(&caminho);
    let config = configuracoes::carregar(&app)?;
    let registrado = config.salvos.iter().any(|item| {
        item.tipo == "leitura-offline"
            && item
                .caminho
                .as_ref()
                .is_some_and(|salvo| salvo == &solicitado)
    });
    if !registrado {
        return Err("Este arquivo não está registrado como uma captura offline do Canoa.".into());
    }
    let arquivo = solicitado
        .canonicalize()
        .map_err(|_| "A captura offline não está mais disponível.")?;
    if arquivo
        .extension()
        .and_then(|ext| ext.to_str())
        .is_none_or(|ext| !ext.eq_ignore_ascii_case("html"))
    {
        return Err("O caminho da captura offline é inválido.".into());
    }
    let metadados = std::fs::metadata(&arquivo).map_err(|erro| erro.to_string())?;
    if metadados.len() > 8 * 1024 * 1024 {
        return Err("A captura offline excede o limite de 8 MiB para leitura na janela.".into());
    }
    std::fs::read_to_string(arquivo)
        .map_err(|erro| format!("Não foi possível ler a captura offline: {erro}"))
}

#[tauri::command]
fn remover_item_salvo(
    app: AppHandle,
    endereco: String,
    caminho: Option<String>,
) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    if let Some(caminho) = caminho {
        biblioteca::remover_item_salvo(&mut valor, &caminho);
    } else {
        biblioteca::remover_salvo(&mut valor, &endereco)?;
    }
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
async fn abrir_link_nova_aba(window: Window, endereco: String) -> Result<Retrato, String> {
    let url = validar(&endereco)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("O link precisa usar HTTP ou HTTPS.".into());
    }
    nova_aba(window.clone()).await?;
    navegar_url(&window, url)?;
    retrato(&window.app_handle())
}

#[tauri::command]
fn remover_favorito(
    app: AppHandle,
    endereco: String,
) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    if valor.favoritos.iter().any(|item| {
        item.endereco == endereco
            && item
                .marcadores
                .iter()
                .any(|marcador| marcador == "Senha associada")
    }) {
        return Err("Este favorito tem uma senha associada. Remova a senha no cofre antes de excluir o favorito.".into());
    }
    valor.favoritos.retain(|item| item.endereco != endereco);
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
async fn exportar_favoritos(app: AppHandle, formato: String) -> Result<Option<usize>, String> {
    let (filtro, extensao) = match formato.as_str() {
        "json" => ("JSON", "json"),
        "html" => ("HTML", "html"),
        _ => return Err("Formato de exportação desconhecido.".into()),
    };
    let seletor = app.clone();
    let nome = format!("canoa-favoritos.{extensao}");
    let escolhida = tauri::async_runtime::spawn_blocking(move || {
        seletor
            .dialog()
            .file()
            .add_filter(filtro, &[extensao])
            .set_file_name(&nome)
            .blocking_save_file()
    })
    .await
    .map_err(|erro| format!("Não foi possível escolher o arquivo: {erro}"))?;
    let Some(escolhida) = escolhida else {
        return Ok(None);
    };
    let caminho = escolhida
        .into_path()
        .map_err(|erro| format!("Arquivo inválido: {erro}"))?;
    let configuracoes = configuracoes::carregar(&app)?;
    let bytes = if formato == "html" {
        favoritos::exportar_html(&configuracoes)
    } else {
        favoritos::exportar(&configuracoes)?
    };
    std::fs::write(&caminho, bytes)
        .map_err(|erro| format!("Não foi possível exportar os favoritos: {erro}"))?;
    Ok(Some(configuracoes.favoritos.len()))
}

#[derive(serde::Serialize)]
struct ResultadoImportacao {
    configuracoes: configuracoes::Configuracoes,
    adicionados: usize,
    ignorados: usize,
}

#[tauri::command]
async fn importar_favoritos(app: AppHandle) -> Result<Option<ResultadoImportacao>, String> {
    let seletor = app.clone();
    let escolhida = tauri::async_runtime::spawn_blocking(move || {
        seletor
            .dialog()
            .file()
            .add_filter("Favoritos", &["html", "htm", "json"])
            .blocking_pick_file()
    })
    .await
    .map_err(|erro| format!("Não foi possível escolher o arquivo: {erro}"))?;
    let Some(escolhida) = escolhida else {
        return Ok(None);
    };
    let caminho = escolhida
        .into_path()
        .map_err(|erro| format!("Arquivo inválido: {erro}"))?;
    let tamanho = std::fs::metadata(&caminho)
        .map_err(|erro| format!("Não foi possível ler o arquivo: {erro}"))?
        .len();
    if tamanho > favoritos::LIMITE_ARQUIVO as u64 {
        return Err("O arquivo de favoritos excede 10 MiB.".into());
    }
    let bytes = std::fs::read(&caminho)
        .map_err(|erro| format!("Não foi possível ler o arquivo: {erro}"))?;
    let mut valor = configuracoes::carregar(&app)?;
    let (adicionados, ignorados) =
        if bytes.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'<') {
            favoritos::importar_html(&mut valor, &bytes)?
        } else {
            (favoritos::importar(&mut valor, &bytes)?, 0)
        };
    if adicionados > 0 {
        configuracoes::salvar(&app, &valor)?;
    }
    Ok(Some(ResultadoImportacao {
        configuracoes: valor,
        adicionados,
        ignorados,
    }))
}

#[tauri::command]
fn pasta_padrao(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    let mut valor = configuracoes::carregar(&app)?;
    valor.pasta_downloads = None;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
async fn escolher_pasta_downloads(
    app: AppHandle,
) -> Result<Option<configuracoes::Configuracoes>, String> {
    let seletor = app.clone();
    let escolhida = tauri::async_runtime::spawn_blocking(move || {
        seletor.dialog().file().blocking_pick_folder()
    })
    .await
    .map_err(|erro| format!("Não foi possível escolher a pasta: {erro}"))?;
    let Some(escolhida) = escolhida else {
        return Ok(None);
    };
    let pasta = escolhida
        .into_path()
        .map_err(|erro| format!("Pasta inválida: {erro}"))?;
    configuracoes::validar_pasta_downloads(&pasta)?;
    let mut valor = configuracoes::carregar(&app)?;
    valor.pasta_downloads = Some(pasta);
    configuracoes::salvar(&app, &valor)?;
    Ok(Some(valor))
}

#[tauri::command]
async fn abrir_configuracoes(window: Window) -> Result<Retrato, String> {
    let app = window.app_handle();
    let existente = {
        let estado = window.state::<Estado>();
        let abas = estado
            .0
            .lock()
            .map_err(|_| "Falha ao abrir configurações.")?;
        abas.itens
            .iter()
            .find(|aba| aba.endereco.as_deref().is_some_and(eh_configuracoes))
            .map(|aba| aba.id)
    };
    if let Some(id) = existente {
        return trocar_aba(window, id).await;
    }
    parar(app)?;
    {
        let estado = window.state::<Estado>();
        let mut abas = estado
            .0
            .lock()
            .map_err(|_| "Falha ao abrir configurações.")?;
        let id = abas.proximo;
        abas.proximo += 1;
        abas.ativa = id;
        abas.itens.push(Aba {
            id,
            endereco: Some(URL_CONFIGURACOES.into()),
            titulo: "Configurações".into(),
            grupo: None,
            mutada: false,
        });
    }
    avisar(app);
    retrato(app)
}

#[tauri::command]
fn secao_configuracoes(app: AppHandle, secao: String) -> Result<Retrato, String> {
    let titulo = match secao.as_str() {
        "" => "Configurações",
        "inicio" => "Configurações: Visão geral",
        "pesquisa" => "Configurações: Pesquisa e fontes",
        "favoritos" => "Configurações: Favoritos",
        "leitura" => "Configurações: Lista de leitura",
        "barra" => "Configurações: Barra de endereço",
        "biblioteca" => "Configurações: Biblioteca pessoal",
        "privacidade" => "Configurações: Proteção e bloqueios",
        "senhas" => "Configurações: Senhas",
        "memoria" => "Configurações: Memória e armazenamento",
        "downloads" => "Configurações: Baixados",
        "historico" => "Configurações: Histórico",
        "foco" => "Configurações: Tempo e foco",
        "limpeza" => "Configurações: Cookies e cache",
        "armazenamento" => "Configurações: Armazenamento",
        "sobre" => "Configurações: Sobre",
        "atalhos" => "Configurações: Atalhos",
        _ => return Err("Seção de configurações desconhecida.".into()),
    };
    {
        let estado = app.state::<Estado>();
        let mut abas = estado
            .0
            .lock()
            .map_err(|_| "Falha ao abrir configurações.")?;
        let ativa = abas.ativa;
        let aba = abas
            .itens
            .iter_mut()
            .find(|aba| aba.id == ativa)
            .ok_or("Aba não encontrada.")?;
        if !aba.endereco.as_deref().is_some_and(eh_configuracoes) {
            return Err("Abra a aba de configurações primeiro.".into());
        }
        aba.endereco = Some(if secao.is_empty() {
            URL_CONFIGURACOES.into()
        } else {
            format!("{URL_CONFIGURACOES}/{secao}")
        });
        aba.titulo = titulo.into();
    }
    avisar(&app);
    retrato(&app)
}

#[tauri::command]
fn mover_favorito(
    app: AppHandle,
    endereco: String,
    titulo: String,
    pastas: Vec<String>,
) -> Result<configuracoes::Configuracoes, String> {
    let titulo = titulo.trim();
    if titulo.is_empty() || titulo.chars().count() > 100 {
        return Err("O nome precisa ter entre 1 e 100 caracteres.".into());
    }
    if pastas.len() > 12
        || pastas.iter().any(|pasta| {
            let pasta = pasta.trim();
            pasta.is_empty()
                || pasta == "."
                || pasta == ".."
                || pasta.chars().count() > 80
                || pasta.chars().any(char::is_control)
        })
    {
        return Err("Nome ou caminho de pasta inválido.".into());
    }
    let mut valor = configuracoes::carregar(&app)?;
    let favorito = valor
        .favoritos
        .iter_mut()
        .find(|item| item.endereco == endereco)
        .ok_or("Favorito não encontrado.")?;
    favorito.titulo = titulo.to_string();
    favorito.pastas = pastas;
    configuracoes::salvar(&app, &valor)?;
    Ok(valor)
}

#[tauri::command]
fn focar_interface(app: AppHandle) -> Result<(), String> {
    app.get_webview("main")
        .ok_or("Interface indisponível.")?
        .set_focus()
        .map_err(|erro| erro.to_string())
}

#[tauri::command]
fn abrir_downloads(app: AppHandle) -> Result<(), String> {
    #[cfg(windows)]
    {
        let pasta = match configuracoes::carregar(&app)?.pasta_downloads {
            Some(pasta) => pasta,
            None => app.path().download_dir().map_err(|erro| erro.to_string())?,
        };
        if !pasta.is_dir() {
            return Err("A pasta de downloads configurada está indisponível.".into());
        }
        std::process::Command::new("explorer.exe")
            .arg(pasta)
            .spawn()
            .map_err(|erro| format!("Não foi possível abrir Downloads: {erro}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("Abrir Downloads ainda não está disponível neste sistema.".into())
    }
}

#[tauri::command]
fn limpar_historico_downloads(app: AppHandle) -> Result<configuracoes::Configuracoes, String> {
    let mut config = configuracoes::carregar(&app)?;
    config.historico_downloads.clear();
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn revelar_download(caminho: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        let arquivo = std::fs::canonicalize(&caminho)
            .map_err(|_| "O arquivo não está mais disponível no local salvo.".to_string())?;
        if !arquivo.is_file() {
            return Err("O caminho salvo não é um arquivo disponível.".into());
        }
        std::process::Command::new("explorer.exe")
            .arg(format!("/select,{}", arquivo.display()))
            .spawn()
            .map_err(|erro| format!("Não foi possível localizar o arquivo: {erro}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = caminho;
        Err("Localizar um download ainda não está disponível neste sistema.".into())
    }
}

#[tauri::command]
async fn memoria_atual() -> Result<Memoria, String> {
    #[cfg(windows)]
    {
        let amostra = tauri::async_runtime::spawn_blocking(memoria::medir)
            .await
            .map_err(|erro| format!("Falha ao medir memória: {erro}"))??;
        return Ok(amostra);
    }
    #[cfg(not(windows))]
    {
        Err("Medição de memória ainda não disponível neste sistema.".into())
    }
}

#[tauri::command]
async fn diagnostico_atual() -> Result<Diagnostico, String> {
    #[cfg(windows)]
    {
        return tauri::async_runtime::spawn_blocking(memoria::diagnosticar)
            .await
            .map_err(|erro| format!("Falha ao medir processos: {erro}"))?;
    }
    #[cfg(not(windows))]
    {
        Err("Diagnóstico de processos ainda não disponível neste sistema.".into())
    }
}

#[tauri::command]
async fn abrir_gerenciador(app: AppHandle) -> Result<(), String> {
    if let Some(janela) = app.get_webview_window("tarefas") {
        janela.show().map_err(|erro| erro.to_string())?;
        janela.set_focus().map_err(|erro| erro.to_string())?;
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, "tarefas", WebviewUrl::App("tarefas.html".into()))
        .title("Canoa — Tarefas")
        .inner_size(850.0, 620.0)
        .min_inner_size(590.0, 420.0)
        .build()
        .map_err(|erro| format!("Não foi possível abrir o gerenciador: {erro}"))?;
    Ok(())
}

#[tauri::command]
async fn fechar_aba_do_gerenciador(app: AppHandle, id: u64) -> Result<Retrato, String> {
    let janela = app
        .get_window("main")
        .ok_or("Janela principal indisponível.")?;
    fechar_aba(janela, id).await
}

#[tauri::command]
fn informacoes_versao(app: AppHandle) -> InformacoesVersao {
    InformacoesVersao {
        versao: app.package_info().version.to_string(),
        compilado_em_epoch: option_env!("CANOA_BUILD_EPOCH").and_then(|valor| valor.parse().ok()),
    }
}

#[tauri::command]
async fn armazenamento_atual(app: AppHandle) -> Result<Armazenamento, String> {
    let executavel = std::env::current_exe().map_err(|erro| erro.to_string())?;
    let local = app
        .path()
        .app_local_data_dir()
        .map_err(|erro| erro.to_string())?;
    let configuracoes = app
        .path()
        .app_config_dir()
        .map_err(|erro| erro.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        let (dados_locais_bytes, cache_identificado_bytes, parcial_local) =
            somar_arquivos(&local, true);
        let (configuracoes_bytes, _, parcial_config) = somar_arquivos(&configuracoes, false);
        let (executavel_bytes, parcial_executavel) = match std::fs::metadata(executavel) {
            Ok(valor) => (valor.len(), false),
            Err(_) => (0, true),
        };
        Armazenamento {
            executavel_bytes,
            dados_locais_bytes,
            cache_identificado_bytes,
            configuracoes_bytes,
            incompleto: parcial_local || parcial_config || parcial_executavel,
        }
    })
    .await
    .map_err(|erro| format!("Falha ao medir armazenamento: {erro}"))
}

#[tauri::command]
async fn nova_aba(window: Window) -> Result<Retrato, String> {
    let app = window.app_handle();
    parar(app)?;
    {
        let estado = window.state::<Estado>();
        let mut abas = estado.0.lock().map_err(|_| "Falha ao criar a aba.")?;
        let id = abas.proximo;
        abas.proximo += 1;
        abas.ativa = id;
        abas.itens.push(Aba {
            id,
            endereco: None,
            titulo: "Nova aba".into(),
            grupo: None,
            mutada: false,
        });
    }
    avisar(app);
    let _ = focar_interface(app.clone());
    println!("CANOA_ABA_NOVA");
    retrato(app)
}

#[tauri::command]
async fn trocar_aba(window: Window, id: u64) -> Result<Retrato, String> {
    let app = window.app_handle();
    let destino = {
        let estado = window.state::<Estado>();
        let abas = estado.0.lock().map_err(|_| "Falha ao ler as abas.")?;
        if abas.ativa == id {
            return Ok(abas.retrato());
        }
        abas.itens
            .iter()
            .find(|aba| aba.id == id)
            .ok_or("Aba não encontrada.")?
            .endereco
            .clone()
    };
    parar(app)?;
    window
        .state::<Estado>()
        .0
        .lock()
        .map_err(|_| "Falha ao trocar a aba.")?
        .ativa = id;
    if let Some(endereco) = destino.filter(|url| !eh_configuracoes(url)) {
        abrir(&window, id, &endereco)?;
    }
    avisar(app);
    println!("CANOA_ABA_ATIVA id={id}");
    retrato(app)
}

#[tauri::command]
fn mover_aba(window: Window, id: u64, destino: usize) -> Result<Retrato, String> {
    let app = window.app_handle();
    let estado = window.state::<Estado>();
    let mut abas = estado
        .0
        .lock()
        .map_err(|_| "Falha ao reorganizar as abas.")?;
    let origem = abas
        .itens
        .iter()
        .position(|aba| aba.id == id)
        .ok_or("Aba não encontrada.")?;
    let aba = abas.itens.remove(origem);
    let indice = destino.min(abas.itens.len());
    abas.itens.insert(indice, aba);
    drop(abas);
    avisar(app);
    retrato(app)
}

#[tauri::command]
fn definir_grupo_aba(app: AppHandle, id: u64, grupo: String) -> Result<Retrato, String> {
    let nome = grupo.trim();
    if nome.chars().count() > 32 {
        return Err("O nome do grupo pode ter até 32 caracteres.".into());
    }
    let estado = app.state::<Estado>();
    let mut abas = estado
        .0
        .lock()
        .map_err(|_| "Falha ao atualizar grupo da aba.")?;
    let aba = abas
        .itens
        .iter_mut()
        .find(|aba| aba.id == id)
        .ok_or("Aba não encontrada.")?;
    aba.grupo = if nome.is_empty() {
        None
    } else {
        Some(nome.to_string())
    };
    let resultado = abas.retrato();
    drop(abas);
    if !nome.is_empty() {
        let mut config = configuracoes::carregar(&app)?;
        if !config
            .grupos_abas
            .iter()
            .any(|item| item.nome.eq_ignore_ascii_case(nome))
        {
            config.grupos_abas.push(configuracoes::GrupoAba {
                nome: nome.to_string(),
                recolhido: false,
            });
            config
                .grupos_abas
                .sort_by_key(|item| item.nome.to_lowercase());
            configuracoes::salvar(&app, &config)?;
        }
    }
    avisar(&app);
    Ok(resultado)
}

#[tauri::command]
fn alternar_recolhimento_grupo(
    app: AppHandle,
    nome: String,
) -> Result<configuracoes::Configuracoes, String> {
    let nome = nome.trim();
    if nome.is_empty() || nome.chars().count() > 32 {
        return Err("Nome de grupo inválido.".into());
    }
    let mut config = configuracoes::carregar(&app)?;
    let grupo = config
        .grupos_abas
        .iter_mut()
        .find(|item| item.nome.eq_ignore_ascii_case(nome))
        .ok_or("Grupo não encontrado.")?;
    grupo.recolhido = !grupo.recolhido;
    configuracoes::salvar(&app, &config)?;
    Ok(config)
}

#[tauri::command]
fn alternar_mudo_atual(app: AppHandle) -> Result<Retrato, String> {
    #[cfg(windows)]
    {
        let (id, mudo_atual) = {
            let estado = app.state::<Estado>();
            let abas = estado
                .0
                .lock()
                .map_err(|_| "Falha ao ler o áudio da aba.")?;
            let aba = abas
                .itens
                .iter()
                .find(|aba| aba.id == abas.ativa)
                .ok_or("Aba ativa não encontrada.")?;
            if aba.endereco.as_deref().is_none_or(eh_configuracoes) {
                return Err("Abra uma página antes de alterar o áudio.".into());
            }
            (aba.id, aba.mutada)
        };
        let webview = app
            .get_webview("conteudo")
            .ok_or("Abra uma página antes de alterar o áudio.")?;
        audio_windows::definir_mudo(&webview, !mudo_atual)?;
        let estado = app.state::<Estado>();
        let mut abas = estado
            .0
            .lock()
            .map_err(|_| "Falha ao atualizar o áudio da aba.")?;
        if let Some(aba) = abas.itens.iter_mut().find(|aba| aba.id == id) {
            aba.mutada = !mudo_atual;
        }
        let retrato = abas.retrato();
        drop(abas);
        avisar(&app);
        return Ok(retrato);
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("O controle de áudio está disponível no WebView2 do Windows.".into())
    }
}

#[tauri::command]
fn alternar_mudo_aba(app: AppHandle, id: u64) -> Result<Retrato, String> {
    #[cfg(windows)]
    {
        let (ativa, atual) = {
            let estado = app.state::<Estado>();
            let abas = estado
                .0
                .lock()
                .map_err(|_| "Falha ao ler o áudio da aba.")?;
            let aba = abas
                .itens
                .iter()
                .find(|aba| aba.id == id)
                .ok_or("Aba não encontrada.")?;
            if aba.endereco.as_deref().is_none_or(eh_configuracoes) {
                return Err("Abra uma página antes de alterar o áudio.".into());
            }
            (abas.ativa == id, aba.mutada)
        };
        if ativa {
            let webview = app
                .get_webview("conteudo")
                .ok_or("Abra uma página antes de alterar o áudio.")?;
            audio_windows::definir_mudo(&webview, !atual)?;
        }
        let estado = app.state::<Estado>();
        let mut abas = estado
            .0
            .lock()
            .map_err(|_| "Falha ao atualizar o áudio da aba.")?;
        if let Some(aba) = abas.itens.iter_mut().find(|aba| aba.id == id) {
            aba.mutada = !atual;
        }
        let retrato = abas.retrato();
        drop(abas);
        avisar(&app);
        return Ok(retrato);
    }
    #[cfg(not(windows))]
    {
        let _ = (app, id);
        Err("O controle de áudio está disponível no WebView2 do Windows.".into())
    }
}

#[tauri::command]
async fn duplicar_aba(window: Window, id: u64) -> Result<Retrato, String> {
    let app = window.app_handle();
    let duplicada = {
        let estado = window.state::<Estado>();
        let abas = estado.0.lock().map_err(|_| "Falha ao duplicar a aba.")?;
        let posicao = abas
            .itens
            .iter()
            .position(|aba| aba.id == id)
            .ok_or("Aba não encontrada.")?;
        abas.itens[posicao].clone()
    };
    parar(app)?;
    let novo_id = {
        let estado = window.state::<Estado>();
        let mut abas = estado.0.lock().map_err(|_| "Falha ao duplicar a aba.")?;
        let posicao = abas
            .itens
            .iter()
            .position(|aba| aba.id == id)
            .ok_or("Aba não encontrada.")?;
        let novo_id = abas.proximo;
        abas.proximo += 1;
        let mut nova = duplicada;
        nova.id = novo_id;
        abas.itens.insert(posicao + 1, nova);
        abas.ativa = novo_id;
        novo_id
    };
    let endereco = window
        .state::<Estado>()
        .0
        .lock()
        .map_err(|_| "Falha ao ler a aba.")?
        .itens
        .iter()
        .find(|aba| aba.id == novo_id)
        .and_then(|aba| aba.endereco.clone());
    if let Some(endereco) = endereco.filter(|url| !eh_configuracoes(url)) {
        abrir(&window, novo_id, &endereco)?;
    } else {
        ajustar(&window);
    }
    avisar(app);
    retrato(app)
}

async fn fechar_conjunto(window: Window, id: u64, fechar: Vec<u64>) -> Result<Retrato, String> {
    let app = window.app_handle();
    {
        let estado = window.state::<Estado>();
        let abas = estado.0.lock().map_err(|_| "Falha ao fechar abas.")?;
        if !abas.itens.iter().any(|aba| aba.id == id) {
            return Err("Aba não encontrada.".into());
        }
    }
    parar(app)?;
    let endereco = {
        let estado = window.state::<Estado>();
        let mut abas = estado.0.lock().map_err(|_| "Falha ao fechar abas.")?;
        let mut removidas: Vec<(Aba, usize)> = abas
            .itens
            .iter()
            .enumerate()
            .filter(|(_, aba)| fechar.contains(&aba.id) && aba.id != id)
            .map(|(posicao, aba)| (aba.clone(), posicao))
            .collect();
        for item in removidas.drain(..) {
            abas.fechadas.push(item);
            if abas.fechadas.len() > 10 {
                abas.fechadas.remove(0);
            }
        }
        abas.itens
            .retain(|aba| !fechar.contains(&aba.id) || aba.id == id);
        abas.ativa = id;
        abas.itens
            .iter()
            .find(|aba| aba.id == id)
            .and_then(|aba| aba.endereco.clone())
    };
    if let Some(endereco) = endereco.filter(|url| !eh_configuracoes(url)) {
        abrir(&window, id, &endereco)?;
    } else {
        ajustar(&window);
    }
    avisar(app);
    retrato(app)
}

#[tauri::command]
async fn fechar_outras_abas(window: Window, id: u64) -> Result<Retrato, String> {
    let ids = window
        .state::<Estado>()
        .0
        .lock()
        .map_err(|_| "Falha ao ler as abas.")?
        .itens
        .iter()
        .map(|aba| aba.id)
        .filter(|aba_id| *aba_id != id)
        .collect();
    fechar_conjunto(window, id, ids).await
}

#[tauri::command]
async fn fechar_abas_a_direita(window: Window, id: u64) -> Result<Retrato, String> {
    let ids = {
        let estado = window.state::<Estado>();
        let abas = estado.0.lock().map_err(|_| "Falha ao ler as abas.")?;
        let posicao = abas
            .itens
            .iter()
            .position(|aba| aba.id == id)
            .ok_or("Aba não encontrada.")?;
        abas.itens
            .iter()
            .skip(posicao + 1)
            .map(|aba| aba.id)
            .collect()
    };
    fechar_conjunto(window, id, ids).await
}

#[tauri::command]
async fn fechar_aba(window: Window, id: u64) -> Result<Retrato, String> {
    let app = window.app_handle();
    let ativa = {
        let estado = window.state::<Estado>();
        let abas = estado.0.lock().map_err(|_| "Falha ao ler as abas.")?;
        if !abas.itens.iter().any(|aba| aba.id == id) {
            return Err("Aba não encontrada.".into());
        }
        abas.ativa == id
    };
    if ativa {
        parar(app)?;
    }
    let destino = {
        let estado = window.state::<Estado>();
        let mut abas = estado.0.lock().map_err(|_| "Falha ao fechar a aba.")?;
        let posicao = abas.itens.iter().position(|aba| aba.id == id).unwrap();
        let fechada = abas.itens[posicao].clone();
        abas.fechadas.push((fechada, posicao));
        if abas.fechadas.len() > 10 {
            abas.fechadas.remove(0);
        }
        abas.itens.retain(|aba| aba.id != id);
        if abas.itens.is_empty() {
            let novo = abas.proximo;
            abas.proximo += 1;
            abas.itens.push(Aba {
                id: novo,
                endereco: None,
                titulo: "Nova aba".into(),
                grupo: None,
                mutada: false,
            });
        }
        if ativa {
            abas.ativa = abas
                .itens
                .get(posicao)
                .or_else(|| abas.itens.last())
                .unwrap()
                .id;
        }
        if ativa {
            abas.itens
                .iter()
                .find(|aba| aba.id == abas.ativa)
                .and_then(|aba| aba.endereco.clone())
        } else {
            None
        }
    };
    if let Some(endereco) = destino.filter(|url| !eh_configuracoes(url)) {
        let id_ativo = window
            .state::<Estado>()
            .0
            .lock()
            .map_err(|_| "Falha ao ler a aba.")?
            .ativa;
        abrir(&window, id_ativo, &endereco)?;
    } else {
        ajustar(&window);
    }
    avisar(app);
    println!("CANOA_ABA_FECHADA id={id}");
    retrato(app)
}

#[tauri::command]
async fn reabrir_aba(window: Window) -> Result<Retrato, String> {
    let app = window.app_handle();
    if window
        .state::<Estado>()
        .0
        .lock()
        .map_err(|_| "Falha ao recuperar a aba.")?
        .fechadas
        .is_empty()
    {
        return Err("Não há abas fechadas recentemente nesta sessão.".into());
    }
    parar(app)?;
    let restaurada = {
        let estado = window.state::<Estado>();
        let mut abas = estado.0.lock().map_err(|_| "Falha ao recuperar a aba.")?;
        let Some((mut aba, posicao)) = abas.fechadas.pop() else {
            return Err("Não há abas fechadas recentemente nesta sessão.".into());
        };
        aba.id = abas.proximo;
        abas.proximo += 1;
        let id = aba.id;
        let endereco = aba.endereco.clone();
        let indice = posicao.min(abas.itens.len());
        abas.itens.insert(indice, aba);
        abas.ativa = id;
        (id, endereco)
    };
    if let Some(endereco) = restaurada.1.filter(|url| !eh_configuracoes(url)) {
        abrir(&window, restaurada.0, &endereco)?;
    } else {
        ajustar(&window);
    }
    avisar(app);
    retrato(app)
}

#[tauri::command]
async fn navegar(window: Window, entrada: String) -> Result<String, String> {
    let app = window.app_handle();
    let configuracoes = configuracoes::carregar(app)?;
    let url = interpretar_entrada(&entrada, configuracoes.buscador)?;
    navegar_url(&window, url)
}

#[tauri::command]
async fn pesquisar_fonte(
    window: Window,
    consulta: String,
    dominio: String,
    tema: String,
    filtrar: bool,
) -> Result<String, String> {
    let configuracoes = configuracoes::carregar(window.app_handle())?;
    let texto = fontes::consulta(&configuracoes, &consulta, &dominio, &tema, filtrar)?;
    let url = fontes::url_busca(configuracoes.buscador, &texto)?;
    navegar_url(&window, url)
}

fn navegar_url(window: &Window, url: Url) -> Result<String, String> {
    let app = window.app_handle();
    let id = {
        let estado = window.state::<Estado>();
        let mut abas = estado.0.lock().map_err(|_| "Falha ao ler a aba.")?;
        let id = abas.ativa;
        if let Some(aba) = abas.itens.iter_mut().find(|aba| aba.id == id) {
            aba.endereco = Some(url.to_string());
            aba.titulo = url.host_str().unwrap_or("Página").to_string();
        }
        id
    };
    if let Some(webview) = app.get_webview("conteudo") {
        webview
            .navigate(url.clone())
            .map_err(|erro| format!("Não foi possível navegar: {erro}"))?;
    } else {
        abrir(&window, id, url.as_str())?;
    }
    avisar(app);
    Ok(url.to_string())
}

#[tauri::command]
fn recarregar(app: AppHandle) -> Result<(), String> {
    app.get_webview("conteudo")
        .ok_or("Abra uma página primeiro.")?
        .reload()
        .map_err(|erro| erro.to_string())
}
#[tauri::command]
fn voltar(app: AppHandle) -> Result<(), String> {
    let webview = app
        .get_webview("conteudo")
        .ok_or("Abra uma página primeiro.")?;
    #[cfg(windows)]
    {
        navegacao_windows::voltar(&webview)
    }
    #[cfg(not(windows))]
    {
        webview
            .eval("window.history.go(-1)")
            .map_err(|erro| erro.to_string())
    }
}
#[tauri::command]
fn avancar(app: AppHandle) -> Result<(), String> {
    let webview = app
        .get_webview("conteudo")
        .ok_or("Abra uma página primeiro.")?;
    #[cfg(windows)]
    {
        navegacao_windows::avancar(&webview)
    }
    #[cfg(not(windows))]
    {
        webview
            .eval("window.history.go(1)")
            .map_err(|erro| erro.to_string())
    }
}
#[tauri::command]
fn imprimir_pagina(app: AppHandle) -> Result<(), String> {
    app.get_webview("conteudo")
        .ok_or("Abra uma página antes de imprimir.")?
        .eval("window.print()")
        .map_err(|erro| erro.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Faixa(Mutex::new(false)))
        .manage(TextoContexto(Mutex::new(String::new())))
        .manage(GerenciadorDownloads::default())
        .manage(Arc::new(bloqueio::Protecao::nova(true, Vec::new())))
        .manage(Estado(Mutex::new(Abas {
            itens: vec![Aba {
                id: 1,
                endereco: None,
                titulo: "Nova aba".into(),
                grupo: None,
                mutada: false,
            }],
            ativa: 1,
            proximo: 2,
            fechadas: Vec::new(),
        })))
        .on_window_event(|window, evento| {
            if window.label() == "main"
                && matches!(
                    evento,
                    WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }
                )
            {
                ajustar(window);
            }
        })
        .on_menu_event(|app, evento| {
            let id = evento.id().as_ref();
            if id.starts_with("canoa_") {
                let _ = app.emit_to(EventTarget::webview("main"), "menu-nativo", id.to_string());
            }
        })
        .invoke_handler(tauri::generate_handler![
            listar_abas,
            abrir_link_nova_aba,
            faixa_aviso,
            abrir_menu_nativo,
            mostrar_conteudo_pagina,
            copiar_texto_contexto,
            copiar_endereco_atual,
            ler_configuracoes,
            resumo_cofre,
            criar_cofre,
            desbloquear_cofre,
            bloquear_cofre,
            listar_senhas,
            salvar_senha,
            remover_senha,
            copiar_senha,
            resumo_bloqueador,
            atualizar_listas_bloqueio,
            reverter_listas_bloqueio,
            definir_historico_navegacao,
            remover_entrada_historico,
            limpar_historico_navegacao,
            definir_limite_site,
            remover_limite_site,
            registrar_tempo_site,
            definir_sites_foco,
            definir_duracao_foco,
            alternar_modo_foco,
            limpar_cookies_pagina,
            limpar_cookies_e_cache,
            definir_bloqueador,
            definir_limpeza_rastreamento,
            alternar_site_bloqueado,
            alternar_bloqueio_site_atual,
            alternar_excecao_bloqueador,
            definir_memoria,
            definir_buscador,
            definir_acoes_barra,
            adicionar_fonte,
            remover_fonte,
            alternar_favorito,
            salvar_para_leitura,
            alternar_lido,
            remover_item_leitura,
            salvar_pagina_em_salvos,
            solicitar_recursos_pagina,
            baixar_item_pagina,
            remover_item_salvo,
            remover_favorito,
            mover_favorito,
            exportar_favoritos,
            importar_favoritos,
            pasta_padrao,
            escolher_pasta_downloads,
            abrir_configuracoes,
            secao_configuracoes,
            focar_interface,
            abrir_downloads,
            limpar_historico_downloads,
            listar_downloads_ativos,
            cancelar_download,
            revelar_download,
            ler_leitura_offline,
            memoria_atual,
            diagnostico_atual,
            abrir_gerenciador,
            fechar_aba_do_gerenciador,
            informacoes_versao,
            armazenamento_atual,
            nova_aba,
            trocar_aba,
            mover_aba,
            definir_grupo_aba,
            alternar_recolhimento_grupo,
            alternar_mudo_atual,
            alternar_mudo_aba,
            duplicar_aba,
            fechar_aba,
            fechar_outras_abas,
            fechar_abas_a_direita,
            reabrir_aba,
            navegar,
            pesquisar_fonte,
            recarregar,
            voltar,
            avancar,
            imprimir_pagina
        ])
        .setup(|app| {
            let pasta_dados = app.path().app_data_dir().map_err(|erro| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Pasta de dados do Canoa indisponível: {erro}"),
                )
            })?;
            app.manage(senhas::Cofre::novo(
                pasta_dados.join("senhas").join("cofre.json"),
            ));
            if let Ok(config) = configuracoes::carregar(app.handle()) {
                app.state::<Arc<bloqueio::Protecao>>()
                    .inner()
                    .as_ref()
                    .atualizar(config.bloqueador_ativo, config.excecoes_bloqueador);
            }
            if let Ok(pasta) = app.path().app_data_dir() {
                if let Some(listas) =
                    atualizador_listas::carregar_ultima(&pasta.join("listas-bloqueador"))
                {
                    app.state::<Arc<bloqueio::Protecao>>()
                        .carregar_listas(&listas);
                }
            }
            #[cfg(not(debug_assertions))]
            let _ = app;
            #[cfg(debug_assertions)]
            if std::env::var("CANOA_TESTE_ABAS").as_deref() == Ok("1") {
                let window = app.get_window("main").expect("janela principal ausente");
                let janela = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let teste = || -> Result<(), String> {
                        tauri::async_runtime::block_on(navegar(
                            janela.clone(),
                            "https://example.com".into(),
                        ))?;
                        tauri::async_runtime::block_on(nova_aba(janela.clone()))?;
                        tauri::async_runtime::block_on(navegar(
                            janela.clone(),
                            "https://example.org".into(),
                        ))?;
                        tauri::async_runtime::block_on(nova_aba(janela.clone()))?;
                        tauri::async_runtime::block_on(navegar(
                            janela.clone(),
                            "https://www.iana.org/domains/reserved".into(),
                        ))?;
                        for rodada in 1..=30 {
                            let destino = (rodada % 3 + 1) as u64;
                            let estado = tauri::async_runtime::block_on(trocar_aba(
                                janela.clone(),
                                destino,
                            ))?;
                            if estado.ativa != destino || estado.abas.len() != 3 {
                                return Err(format!("Estado inesperado na troca {rodada}"));
                            }
                            println!("CANOA_TESTE_TROCA {rodada}/30 aba={destino}");
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        tauri::async_runtime::block_on(fechar_aba(janela.clone(), 2))?;
                        let final_estado =
                            tauri::async_runtime::block_on(fechar_aba(janela.clone(), 3))?;
                        if final_estado.abas.len() != 1 {
                            return Err("Fechamento de abas falhou".into());
                        }
                        println!("CANOA_TESTE_ABAS_CONCLUIDO");
                        Ok(())
                    };
                    if let Err(erro) = teste() {
                        eprintln!("CANOA_TESTE_ABAS_ERRO {erro}");
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o Canoa");
}

#[cfg(test)]
mod tests {
    use super::{
        interpretar_entrada, nome_arquivo_seguro, origem_cofre_confiavel, titulo_pagina, validar,
    };
    use crate::configuracoes::Buscador;
    use tauri::Url;
    #[test]
    fn aceita_dominio_sem_protocolo() {
        assert_eq!(
            validar("example.com").unwrap().as_str(),
            "https://example.com/"
        );
    }
    #[test]
    fn rejeita_enderecos_que_nao_sao_paginas_web() {
        assert!(validar("javascript:alert(1)").is_err());
        assert!(validar("file:///C:/teste.txt").is_err());
        assert!(validar(" ").is_err());
    }
    #[test]
    fn titulo_pagina_usa_titulo_e_alternativa_com_limite() {
        let titulo = serde_json::to_string("Artigo importante").unwrap();
        let vazio = serde_json::to_string("").unwrap();
        assert_eq!(titulo_pagina(&titulo, "example.com"), "Artigo importante");
        assert_eq!(titulo_pagina(&vazio, "example.com"), "example.com");
        assert_eq!(titulo_pagina("null", "example.com"), "example.com");
        let longo = serde_json::to_string(&"x".repeat(400)).unwrap();
        assert_eq!(titulo_pagina(&longo, "site").chars().count(), 300);
    }
    #[test]
    fn pesquisa_codificada_no_buscador_escolhido() {
        assert_eq!(
            interpretar_entrada("  canoa leve & rápido  ", Buscador::Duckduckgo)
                .unwrap()
                .as_str(),
            "https://duckduckgo.com/?q=canoa+leve+%26+r%C3%A1pido"
        );
        assert_eq!(
            interpretar_entrada("canoa", Buscador::Brave)
                .unwrap()
                .as_str(),
            "https://search.brave.com/search?q=canoa"
        );
        assert_eq!(
            interpretar_entrada("como abrir https://example.com", Buscador::Brave)
                .unwrap()
                .as_str(),
            "https://search.brave.com/search?q=como+abrir+https%3A%2F%2Fexample.com"
        );
    }
    #[test]
    fn enderecos_e_esquemas_nao_viram_pesquisas() {
        assert_eq!(
            interpretar_entrada("localhost:8766/teste", Buscador::Brave)
                .unwrap()
                .as_str(),
            "http://localhost:8766/teste"
        );
        assert_eq!(
            interpretar_entrada("127.0.0.1:8766", Buscador::Brave)
                .unwrap()
                .as_str(),
            "http://127.0.0.1:8766/"
        );
        assert_eq!(
            interpretar_entrada("example.com", Buscador::Brave)
                .unwrap()
                .as_str(),
            "https://example.com/"
        );
        assert!(interpretar_entrada("file:///C:/segredo.txt", Buscador::Brave).is_err());
        assert!(interpretar_entrada("javascript:alert(1)", Buscador::Brave).is_err());
        assert!(interpretar_entrada("C:\\segredo\\nota.txt", Buscador::Brave).is_err());
    }
    #[test]
    fn nome_de_download_nao_pode_injetar_caminho() {
        assert_eq!(
            nome_arquivo_seguro("../../segredo:foto?", "recurso"),
            "_.._segredo_foto_"
        );
        assert_eq!(nome_arquivo_seguro("...", "recurso"), "recurso");
        assert!(nome_arquivo_seguro(&"x".repeat(200), "recurso").len() <= 120);
    }

    #[test]
    fn cofre_so_aceita_origem_local_da_interface() {
        assert!(origem_cofre_confiavel(
            &Url::parse("http://tauri.localhost/index.html").unwrap()
        ));
        assert!(origem_cofre_confiavel(
            &Url::parse("tauri://localhost/index.html").unwrap()
        ));
        assert!(!origem_cofre_confiavel(
            &Url::parse("https://example.com").unwrap()
        ));
        assert!(!origem_cofre_confiavel(
            &Url::parse("https://tauri.localhost.example.com").unwrap()
        ));
    }
}
