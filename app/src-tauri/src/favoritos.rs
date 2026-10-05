use std::collections::BTreeMap;

use tauri::Url;

use crate::configuracoes::{Configuracoes, Favorito};

const LIMITE_FAVORITOS: usize = 2_000;
pub const LIMITE_ARQUIVO: usize = 10_485_760;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ArquivoFavoritos {
    formato: String,
    versao: u8,
    favoritos: Vec<Favorito>,
}

fn validar_endereco(endereco: &str) -> Result<Url, String> {
    let url = Url::parse(endereco).map_err(|_| "Endereço de favorito inválido.")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || url.host_str() == Some("canoa.invalid")
        || !url.username().is_empty()
        || url.password().is_some()
        || endereco.len() > 2048
    {
        return Err("Endereço de favorito inválido.".into());
    }
    Ok(url)
}

pub fn exportar(configuracoes: &Configuracoes) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(&ArquivoFavoritos {
        formato: "canoa-favoritos".into(),
        versao: 1,
        favoritos: configuracoes.favoritos.clone(),
    })
    .map_err(|erro| format!("Não foi possível preparar os favoritos: {erro}"))
}

fn escapar_html(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn exportar_html(configuracoes: &Configuracoes) -> Vec<u8> {
    let mut html = String::from("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n<TITLE>Favoritos do Canoa</TITLE>\n<H1>Favoritos do Canoa</H1>\n<DL><p>\n");
    let mut raiz = NoeudFavoritos::default();
    for item in &configuracoes.favoritos {
        let mut noeud = &mut raiz;
        for pasta in &item.pastas {
            noeud = noeud.subpastas.entry(pasta.clone()).or_default();
        }
        noeud.itens.push(item);
    }
    escrever_noeud(&mut html, &raiz, 1);
    html.push_str("</DL><p>\n");
    html.into_bytes()
}

#[derive(Default)]
struct NoeudFavoritos<'a> {
    subpastas: BTreeMap<String, NoeudFavoritos<'a>>,
    itens: Vec<&'a Favorito>,
}

fn escrever_noeud(html: &mut String, noeud: &NoeudFavoritos<'_>, nivel: usize) {
    let recuo = "    ".repeat(nivel);
    for item in &noeud.itens {
        html.push_str(&recuo);
        html.push_str("<DT><A HREF=\"");
        html.push_str(&escapar_html(&item.endereco));
        html.push_str("\">");
        html.push_str(&escapar_html(&item.titulo));
        html.push_str("</A>\n");
    }
    for (nome, subpasta) in &noeud.subpastas {
        html.push_str(&recuo);
        html.push_str("<DT><H3>");
        html.push_str(&escapar_html(nome));
        html.push_str("</H3>\n");
        html.push_str(&recuo);
        html.push_str("<DL><p>\n");
        escrever_noeud(html, subpasta, nivel + 1);
        html.push_str(&recuo);
        html.push_str("</DL><p>\n");
    }
}

pub fn importar(configuracoes: &mut Configuracoes, bytes: &[u8]) -> Result<usize, String> {
    if bytes.len() > LIMITE_ARQUIVO {
        return Err("O arquivo de favoritos excede 10 MiB.".into());
    }
    let arquivo: ArquivoFavoritos = serde_json::from_slice(bytes)
        .map_err(|_| "Arquivo de favoritos inválido ou incompatível.".to_string())?;
    if arquivo.formato != "canoa-favoritos" || arquivo.versao != 1 {
        return Err("Formato de favoritos não reconhecido.".into());
    }
    if arquivo.favoritos.len() > LIMITE_FAVORITOS {
        return Err("O arquivo contém mais de 2.000 favoritos.".into());
    }
    let mut novos = Vec::new();
    for item in arquivo.favoritos {
        validar_endereco(&item.endereco)?;
        if item.titulo.trim().is_empty() || item.titulo.chars().count() > 100 {
            return Err("Título de favorito inválido.".into());
        }
        if !configuracoes
            .favoritos
            .iter()
            .chain(novos.iter())
            .any(|atual: &Favorito| atual.endereco == item.endereco)
        {
            novos.push(item);
        }
    }
    if configuracoes.favoritos.len() + novos.len() > LIMITE_FAVORITOS {
        return Err("A importação ultrapassaria o limite de 2.000 favoritos.".into());
    }
    let quantidade = novos.len();
    configuracoes.favoritos.extend(novos);
    Ok(quantidade)
}

fn decodificar_entidades(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len());
    let mut restante = texto;
    while let Some(indice) = restante.find('&') {
        saida.push_str(&restante[..indice]);
        restante = &restante[indice..];
        let Some(fim) = restante.find(';').filter(|fim| *fim <= 12) else {
            saida.push('&');
            restante = &restante[1..];
            continue;
        };
        let entidade = &restante[1..fim];
        let caractere = match entidade {
            "amp" => Some('&'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "nbsp" => Some(' '),
            _ if entidade.starts_with("#x") || entidade.starts_with("#X") => {
                u32::from_str_radix(&entidade[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            }
            _ if entidade.starts_with('#') => {
                entidade[1..].parse::<u32>().ok().and_then(char::from_u32)
            }
            _ => None,
        };
        if let Some(caractere) = caractere {
            saida.push(caractere);
        } else {
            saida.push_str(&restante[..=fim]);
        }
        restante = &restante[fim + 1..];
    }
    saida.push_str(restante);
    saida
}

fn atributo_href(tag: &str) -> Option<&str> {
    let bytes = tag.as_bytes();
    let mut posicao = 0;
    while posicao < bytes.len() {
        while posicao < bytes.len() && bytes[posicao].is_ascii_whitespace() {
            posicao += 1;
        }
        let inicio = posicao;
        while posicao < bytes.len()
            && (bytes[posicao].is_ascii_alphanumeric() || bytes[posicao] == b'-')
        {
            posicao += 1;
        }
        if inicio == posicao {
            posicao += 1;
            continue;
        }
        let nome = &tag[inicio..posicao];
        while posicao < bytes.len() && bytes[posicao].is_ascii_whitespace() {
            posicao += 1;
        }
        if posicao >= bytes.len() || bytes[posicao] != b'=' {
            continue;
        }
        posicao += 1;
        while posicao < bytes.len() && bytes[posicao].is_ascii_whitespace() {
            posicao += 1;
        }
        if posicao >= bytes.len() {
            break;
        }
        let aspas = bytes[posicao];
        let inicio_valor;
        if aspas == b'"' || aspas == b'\'' {
            posicao += 1;
            inicio_valor = posicao;
            while posicao < bytes.len() && bytes[posicao] != aspas {
                posicao += 1;
            }
        } else {
            inicio_valor = posicao;
            while posicao < bytes.len() && !bytes[posicao].is_ascii_whitespace() {
                posicao += 1;
            }
        }
        let valor = &tag[inicio_valor..posicao];
        if aspas == b'"' || aspas == b'\'' {
            posicao = (posicao + 1).min(bytes.len());
        }
        if nome.eq_ignore_ascii_case("href") {
            return Some(valor);
        }
    }
    None
}

fn atualizar_pastas(
    minusculas: &str,
    texto: &str,
    inicio: usize,
    fim: usize,
    profundidade_dl: &mut usize,
    pastas: &mut Vec<String>,
) {
    let mut posicao = inicio;
    while posicao < fim {
        let fechamento = minusculas[posicao..fim]
            .find("</dl>")
            .map(|i| (i + posicao, 0));
        let abertura = minusculas[posicao..fim]
            .find("<dl")
            .map(|i| (i + posicao, 1));
        let titulo = minusculas[posicao..fim]
            .find("<h3")
            .map(|i| (i + posicao, 2));
        let Some((indice, tipo)) = [fechamento, abertura, titulo]
            .into_iter()
            .flatten()
            .min_by_key(|(indice, _)| *indice)
        else {
            break;
        };
        match tipo {
            0 => {
                *profundidade_dl = profundidade_dl.saturating_sub(1);
                if *profundidade_dl > 0 {
                    pastas.pop();
                }
                posicao = indice + 5;
            }
            1 => {
                *profundidade_dl = profundidade_dl.saturating_add(1);
                posicao = indice + 3;
            }
            _ => {
                let Some(inicio_titulo) = minusculas[indice..fim].find('>') else {
                    break;
                };
                let inicio_titulo = indice + inicio_titulo + 1;
                let Some(fim_relativo) = minusculas[inicio_titulo..fim].find("</h3>") else {
                    break;
                };
                let fim_titulo = inicio_titulo + fim_relativo;
                let nome = decodificar_entidades(texto[inicio_titulo..fim_titulo].trim());
                let nome = nome.trim();
                if !nome.is_empty() && pastas.len() < 12 {
                    pastas.push(nome.chars().take(80).collect());
                }
                posicao = fim_titulo + 5;
            }
        }
    }
}

pub fn importar_html(
    configuracoes: &mut Configuracoes,
    bytes: &[u8],
) -> Result<(usize, usize), String> {
    if bytes.len() > LIMITE_ARQUIVO {
        return Err("O arquivo de favoritos excede 10 MiB.".into());
    }
    let texto =
        std::str::from_utf8(bytes).map_err(|_| "O HTML precisa estar em UTF-8.".to_string())?;
    if !texto.contains("NETSCAPE-Bookmark-file-1") {
        return Err("Formato HTML de favoritos não reconhecido.".into());
    }
    let minusculas = texto.to_ascii_lowercase();
    let mut posicao = 0;
    let mut cursor_contexto = 0;
    let mut profundidade_dl = 0;
    let mut pastas = Vec::new();
    let mut novos = Vec::<Favorito>::new();
    let mut ignorados = 0;
    while let Some(relativo) = minusculas[posicao..].find("<a") {
        let inicio = posicao + relativo;
        if !minusculas
            .as_bytes()
            .get(inicio + 2)
            .is_some_and(u8::is_ascii_whitespace)
        {
            posicao = inicio + 2;
            continue;
        }
        atualizar_pastas(
            &minusculas,
            texto,
            cursor_contexto,
            inicio,
            &mut profundidade_dl,
            &mut pastas,
        );
        let Some(fim_relativo) = minusculas[inicio..].find('>') else {
            break;
        };
        let fim_tag = inicio + fim_relativo;
        let Some(fim_titulo_relativo) = minusculas[fim_tag + 1..].find("</a>") else {
            break;
        };
        let fim_titulo = fim_tag + 1 + fim_titulo_relativo;
        let tag = &texto[inicio + 2..fim_tag];
        let titulo = decodificar_entidades(texto[fim_tag + 1..fim_titulo].trim());
        if let Some(href) = atributo_href(tag) {
            let endereco = decodificar_entidades(href);
            if let Ok(url) = validar_endereco(&endereco) {
                let titulo = if titulo.trim().is_empty() {
                    url.host_str().unwrap_or("Página").to_string()
                } else {
                    titulo.trim().chars().take(100).collect()
                };
                if configuracoes
                    .favoritos
                    .iter()
                    .chain(novos.iter())
                    .any(|item| item.endereco == endereco)
                {
                    ignorados += 1;
                } else {
                    novos.push(Favorito {
                        endereco,
                        titulo,
                        pastas: pastas.clone(),
                        marcadores: Vec::new(),
                    });
                }
            } else {
                ignorados += 1;
            }
        } else {
            ignorados += 1;
        }
        posicao = fim_titulo + 4;
        cursor_contexto = posicao;
    }
    if novos.is_empty() && ignorados == 0 {
        return Err("Nenhum favorito encontrado no HTML.".into());
    }
    if configuracoes.favoritos.len() + novos.len() > LIMITE_FAVORITOS {
        return Err("A importação ultrapassaria o limite de 2.000 favoritos.".into());
    }
    let adicionados = novos.len();
    configuracoes.favoritos.extend(novos);
    Ok((adicionados, ignorados))
}

pub fn alternar(
    configuracoes: &mut Configuracoes,
    endereco: &str,
    titulo: &str,
) -> Result<bool, String> {
    let url = validar_endereco(endereco)
        .map_err(|_| "Esta página não pode ser adicionada aos favoritos.".to_string())?;
    if let Some(indice) = configuracoes
        .favoritos
        .iter()
        .position(|item| item.endereco == endereco)
    {
        if configuracoes.favoritos[indice]
            .marcadores
            .iter()
            .any(|marcador| marcador == "Senha associada")
        {
            return Err("Este favorito tem uma senha associada. Remova a senha no cofre antes de excluir o favorito.".into());
        }
        configuracoes.favoritos.remove(indice);
        return Ok(false);
    }
    if configuracoes.favoritos.len() >= LIMITE_FAVORITOS {
        return Err("Limite de 2.000 favoritos atingido.".into());
    }
    let titulo = titulo.trim();
    configuracoes.favoritos.push(Favorito {
        endereco: endereco.to_string(),
        titulo: if titulo.is_empty() {
            url.host_str().unwrap_or("Página").to_string()
        } else {
            titulo.chars().take(100).collect()
        },
        pastas: Vec::new(),
        marcadores: Vec::new(),
    });
    Ok(true)
}

pub fn marcar_senha_associada(
    configuracoes: &mut Configuracoes,
    origem: &str,
    associado: bool,
) -> Result<(), String> {
    let url = validar_endereco(origem)
        .map_err(|_| "O endereço associado à senha é inválido.".to_string())?;
    if url.scheme() != "https" {
        return Err("Favoritos associados a senhas precisam usar HTTPS.".into());
    }
    let origem = url.origin().ascii_serialization();
    let indices = configuracoes
        .favoritos
        .iter()
        .enumerate()
        .filter_map(|(indice, favorito)| {
            Url::parse(&favorito.endereco)
                .is_ok_and(|favorito| favorito.origin().ascii_serialization() == origem)
                .then_some(indice)
        })
        .collect::<Vec<_>>();
    if associado {
        if !indices.is_empty() {
            for indice in indices {
                let marcadores = &mut configuracoes.favoritos[indice].marcadores;
                if !marcadores
                    .iter()
                    .any(|marcador| marcador == "Senha associada")
                {
                    marcadores.push("Senha associada".into());
                }
            }
        } else {
            if configuracoes.favoritos.len() >= LIMITE_FAVORITOS {
                return Err(
                    "Limite de favoritos atingido; remova um item antes de associar a senha."
                        .into(),
                );
            }
            configuracoes.favoritos.push(Favorito {
                endereco: format!("{origem}/"),
                titulo: url.host_str().unwrap_or("Site").to_string(),
                pastas: Vec::new(),
                marcadores: vec!["Senha associada".into()],
            });
        }
    } else {
        for favorito in configuracoes.favoritos.iter_mut().filter(|favorito| {
            Url::parse(&favorito.endereco)
                .is_ok_and(|favorito| favorito.origin().ascii_serialization() == origem)
        }) {
            favorito
                .marcadores
                .retain(|marcador| marcador != "Senha associada");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        alternar, exportar, exportar_html, importar, importar_html, marcar_senha_associada,
    };
    use crate::configuracoes::Configuracoes;

    #[test]
    fn adicionar_e_remover_sem_duplicar() {
        let mut configuracoes = Configuracoes::default();
        assert!(alternar(&mut configuracoes, "https://example.com/a", "Exemplo").unwrap());
        assert_eq!(configuracoes.favoritos.len(), 1);
        assert!(!alternar(&mut configuracoes, "https://example.com/a", "Exemplo").unwrap());
        assert!(configuracoes.favoritos.is_empty());
    }

    #[test]
    fn nao_guardar_credenciais_na_url() {
        let mut configuracoes = Configuracoes::default();
        assert!(alternar(&mut configuracoes, "https://nome:senha@example.com", "").is_err());
        assert!(configuracoes.favoritos.is_empty());
    }

    #[test]
    fn exportar_e_importar_sem_duplicar() {
        let mut origem = Configuracoes::default();
        alternar(&mut origem, "https://example.com/1", "Primeiro").unwrap();
        let arquivo = exportar(&origem).unwrap();
        let mut destino = Configuracoes::default();
        assert_eq!(importar(&mut destino, &arquivo).unwrap(), 1);
        assert_eq!(importar(&mut destino, &arquivo).unwrap(), 0);
        assert_eq!(destino.favoritos[0].titulo, "Primeiro");
    }

    #[test]
    fn arquivo_invalido_nao_altera_favoritos() {
        let mut destino = Configuracoes::default();
        let invalido = br#"{"formato":"canoa-favoritos","versao":1,"favoritos":[{"endereco":"https://example.com","titulo":"Bom"},{"endereco":"file:///segredo","titulo":"Ruim"}]}"#;
        assert!(importar(&mut destino, invalido).is_err());
        assert!(destino.favoritos.is_empty());
    }

    #[test]
    fn importa_html_netscape_e_ignora_esquema_inseguro() {
        let html = br#"<!DOCTYPE NETSCAPE-Bookmark-file-1><DL><p><DT><A HREF="https://example.com/?a=1&amp;b=2">Exemplo &amp; teste</A><DT><A HREF="javascript:alert(1)">Ignorar</A></DL>"#;
        let mut destino = Configuracoes::default();
        assert_eq!(importar_html(&mut destino, html).unwrap(), (1, 1));
        assert_eq!(
            destino.favoritos[0].endereco,
            "https://example.com/?a=1&b=2"
        );
        assert_eq!(destino.favoritos[0].titulo, "Exemplo & teste");
    }

    #[test]
    fn html_incompativel_nao_altera_favoritos() {
        let mut destino = Configuracoes::default();
        assert!(importar_html(
            &mut destino,
            b"<html><a href='https://example.com'>x</a></html>"
        )
        .is_err());
        assert!(destino.favoritos.is_empty());
    }

    #[test]
    fn html_exportado_pode_ser_reimportado() {
        let mut origem = Configuracoes::default();
        alternar(&mut origem, "https://example.com/?a=1&b=2", "A & B").unwrap();
        let mut destino = Configuracoes::default();
        assert_eq!(
            importar_html(&mut destino, &exportar_html(&origem)).unwrap(),
            (1, 0)
        );
        assert_eq!(destino.favoritos[0].titulo, "A & B");
    }

    #[test]
    fn senha_associada_marca_todas_as_pastas_do_site_e_protege_remocao() {
        let mut configuracoes = Configuracoes::default();
        alternar(
            &mut configuracoes,
            "https://example.com/marketing",
            "Marketing",
        )
        .unwrap();
        alternar(&mut configuracoes, "https://example.com/pessoal", "Pessoal").unwrap();
        marcar_senha_associada(&mut configuracoes, "https://example.com", true).unwrap();
        assert!(configuracoes.favoritos.iter().all(|item| item
            .marcadores
            .iter()
            .any(|marcador| marcador == "Senha associada")));
        assert!(alternar(
            &mut configuracoes,
            "https://example.com/marketing",
            "Marketing"
        )
        .is_err());
        marcar_senha_associada(&mut configuracoes, "https://example.com", false).unwrap();
        assert!(configuracoes
            .favoritos
            .iter()
            .all(|item| item.marcadores.is_empty()));
    }
}
