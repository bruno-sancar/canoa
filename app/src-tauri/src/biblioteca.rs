use tauri::Url;

use crate::configuracoes::{Configuracoes, ItemLeitura, ItemSalvo};

const LIMITE_ITENS: usize = 2_000;

fn validar_endereco(endereco: &str) -> Result<String, String> {
    let url = Url::parse(endereco).map_err(|_| "Endereço da página inválido.")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || url.host_str() == Some("canoa.invalid")
        || !url.username().is_empty()
        || url.password().is_some()
        || endereco.len() > 2_048
    {
        return Err("Endereço da página inválido.".into());
    }
    Ok(url.to_string())
}

fn agora() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|tempo| tempo.as_secs())
        .unwrap_or_default()
}

pub fn salvar_para_leitura(
    configuracoes: &mut Configuracoes,
    endereco: &str,
    titulo: &str,
) -> Result<bool, String> {
    let endereco = validar_endereco(endereco)?;
    if let Some(item) = configuracoes
        .lista_leitura
        .iter_mut()
        .find(|item| item.endereco == endereco)
    {
        item.titulo = titulo.chars().take(300).collect();
        return Ok(false);
    }
    if configuracoes.lista_leitura.len() >= LIMITE_ITENS {
        return Err("A Lista de leitura atingiu o limite de 2.000 itens.".into());
    }
    configuracoes.lista_leitura.insert(
        0,
        ItemLeitura {
            endereco,
            titulo: titulo.chars().take(300).collect(),
            adicionado_em: agora(),
            lido: false,
        },
    );
    Ok(true)
}

pub fn alternar_lido(configuracoes: &mut Configuracoes, endereco: &str) -> Result<(), String> {
    let endereco = validar_endereco(endereco)?;
    let item = configuracoes
        .lista_leitura
        .iter_mut()
        .find(|item| item.endereco == endereco)
        .ok_or("Página não encontrada na Lista de leitura.")?;
    item.lido = !item.lido;
    Ok(())
}

pub fn remover_leitura(configuracoes: &mut Configuracoes, endereco: &str) -> Result<(), String> {
    let endereco = validar_endereco(endereco)?;
    configuracoes
        .lista_leitura
        .retain(|item| item.endereco != endereco);
    Ok(())
}

pub fn salvar_pagina(
    configuracoes: &mut Configuracoes,
    endereco: &str,
    titulo: &str,
) -> Result<bool, String> {
    let endereco = validar_endereco(endereco)?;
    if let Some(item) = configuracoes
        .salvos
        .iter_mut()
        .find(|item| item.endereco == endereco && item.caminho.is_none())
    {
        item.titulo = titulo.chars().take(300).collect();
        return Ok(false);
    }
    if configuracoes.salvos.len() >= LIMITE_ITENS {
        return Err("A coleção Salvos atingiu o limite de 2.000 itens.".into());
    }
    configuracoes.salvos.insert(
        0,
        ItemSalvo {
            endereco,
            titulo: titulo.chars().take(300).collect(),
            tipo: "pagina".into(),
            salvo_em: agora(),
            caminho: None,
        },
    );
    Ok(true)
}

pub fn registrar_download(
    configuracoes: &mut Configuracoes,
    endereco: &str,
    titulo: &str,
    caminho: std::path::PathBuf,
) -> Result<bool, String> {
    let endereco = validar_endereco(endereco)?;
    if configuracoes
        .salvos
        .iter()
        .any(|item| item.caminho.as_ref() == Some(&caminho))
    {
        return Ok(false);
    }
    if configuracoes.salvos.len() >= LIMITE_ITENS {
        configuracoes.salvos.pop();
    }
    let extensao = caminho
        .extension()
        .and_then(|valor| valor.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let tipo = match extensao.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "avif" => "imagem",
        "mp4" | "webm" | "mov" | "mkv" => "video",
        "pdf" => "pdf",
        "md" | "txt" | "html" | "htm" | "json" | "js" | "css" | "rs" | "ts" => "texto",
        _ => "arquivo",
    };
    configuracoes.salvos.insert(
        0,
        ItemSalvo {
            endereco,
            titulo: titulo.chars().take(300).collect(),
            tipo: tipo.into(),
            salvo_em: agora(),
            caminho: Some(caminho),
        },
    );
    Ok(true)
}

pub fn remover_salvo(configuracoes: &mut Configuracoes, endereco: &str) -> Result<(), String> {
    let endereco = validar_endereco(endereco)?;
    configuracoes
        .salvos
        .retain(|item| !(item.endereco == endereco && item.caminho.is_none()));
    Ok(())
}

pub fn remover_item_salvo(configuracoes: &mut Configuracoes, caminho: &str) {
    configuracoes.salvos.retain(|item| {
        item.caminho
            .as_ref()
            .map(|p| p.to_string_lossy() != caminho)
            .unwrap_or(true)
    });
}

#[cfg(test)]
mod tests {
    use super::{
        alternar_lido, registrar_download, remover_leitura, salvar_pagina, salvar_para_leitura,
    };
    use crate::configuracoes::Configuracoes;

    #[test]
    fn leitura_e_salvos_persistem_sem_duplicar_url() {
        let mut config = Configuracoes::default();
        assert!(salvar_para_leitura(&mut config, "https://example.com/a", "Artigo").unwrap());
        assert!(
            !salvar_para_leitura(&mut config, "https://example.com/a", "Título atualizado")
                .unwrap()
        );
        assert_eq!(config.lista_leitura.len(), 1);
        assert_eq!(config.lista_leitura[0].titulo, "Título atualizado");
        alternar_lido(&mut config, "https://example.com/a").unwrap();
        assert!(config.lista_leitura[0].lido);
        remover_leitura(&mut config, "https://example.com/a").unwrap();
        assert!(config.lista_leitura.is_empty());
        assert!(salvar_pagina(&mut config, "https://example.com/a", "Página").unwrap());
        assert!(!salvar_pagina(&mut config, "https://example.com/a", "Página").unwrap());
        assert_eq!(config.salvos[0].tipo, "pagina");
    }

    #[test]
    fn rejeita_esquemas_nao_web_e_classifica_downloads() {
        let mut config = Configuracoes::default();
        assert!(salvar_para_leitura(&mut config, "file:///C:/segredo", "Arquivo").is_err());
        assert!(registrar_download(
            &mut config,
            "https://example.com/photo",
            "Foto",
            "C:/tmp/foto.webp".into()
        )
        .unwrap());
        assert_eq!(config.salvos[0].tipo, "imagem");
    }
}
