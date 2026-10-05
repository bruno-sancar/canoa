use tauri::Url;

const PARAMETROS_RASTREAMENTO: &[&str] = &[
    "gclid",
    "dclid",
    "fbclid",
    "msclkid",
    "ttclid",
    "yclid",
    "_hsenc",
    "_hsmi",
    "mc_cid",
    "mc_eid",
    "igshid",
    "vero_id",
    "oly_anon_id",
    "oly_enc_id",
];

pub fn normalizar_dominio(entrada: &str) -> Result<String, String> {
    let entrada = entrada.trim().trim_end_matches('.').to_ascii_lowercase();
    let host = if entrada.contains("://") {
        Url::parse(&entrada)
            .ok()
            .and_then(|url| url.host_str().map(str::to_owned))
    } else {
        Some(entrada)
    }
    .ok_or("Digite um domínio válido, como exemplo.com.")?;
    if host.len() > 253
        || !host.contains('.')
        || host.split('.').any(|parte| {
            parte.is_empty()
                || parte.len() > 63
                || parte.starts_with('-')
                || parte.ends_with('-')
                || !parte.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
    {
        return Err("Digite um domínio válido, como exemplo.com.".into());
    }
    Ok(host)
}

pub fn dominio_bloqueado(host: &str, lista: &[String]) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    lista
        .iter()
        .any(|dominio| host == *dominio || host.ends_with(&format!(".{dominio}")))
}

pub fn limpar_parametros_rastreamento(url: &mut Url) -> bool {
    let pares: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(chave, _)| {
            let chave = chave.to_ascii_lowercase();
            !(chave.starts_with("utm_") || PARAMETROS_RASTREAMENTO.contains(&chave.as_str()))
        })
        .map(|(chave, valor)| (chave.into_owned(), valor.into_owned()))
        .collect();
    let alterado = url.query().is_some() && pares.len() != url.query_pairs().count();
    if alterado {
        url.set_query(None);
        if !pares.is_empty() {
            url.query_pairs_mut().extend_pairs(pares);
        }
    }
    alterado
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limpa_rastreamento_e_preserva_consulta_legitima() {
        let mut url =
            Url::parse("https://exemplo.com/busca?q=canoa&utm_source=email&gclid=123&page=2")
                .unwrap();
        assert!(limpar_parametros_rastreamento(&mut url));
        assert_eq!(url.as_str(), "https://exemplo.com/busca?q=canoa&page=2");
        assert!(!limpar_parametros_rastreamento(&mut url));
    }

    #[test]
    fn bloqueio_cobre_subdominio_sem_bloquear_sufixo_parecido() {
        let lista = vec!["exemplo.com".to_string()];
        assert!(dominio_bloqueado("conta.exemplo.com", &lista));
        assert!(!dominio_bloqueado("naoexemplo.com", &lista));
    }

    #[test]
    fn normaliza_url_e_rejeita_entrada_invalida() {
        assert_eq!(
            normalizar_dominio(" HTTPS://News.Exemplo.com/path ").unwrap(),
            "news.exemplo.com"
        );
        assert!(normalizar_dominio("localhost").is_err());
    }
}
