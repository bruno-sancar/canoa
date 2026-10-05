use crate::configuracoes::{Buscador, Configuracoes, Fonte};
use tauri::Url;

const LIMITE_FONTES: usize = 100;
const LIMITE_CONSULTA: usize = 500;

fn dominio_valido(entrada: &str) -> Result<String, String> {
    let dominio = entrada.trim().trim_end_matches('.').to_ascii_lowercase();
    let partes: Vec<&str> = dominio.split('.').collect();
    if dominio.len() > 253
        || partes.len() < 2
        || dominio.parse::<std::net::IpAddr>().is_ok()
        || partes.iter().any(|parte| {
            parte.is_empty()
                || parte.len() > 63
                || !parte
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                || !parte
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
                || !parte
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
    {
        return Err("Digite apenas um domínio público, como exemplo.com.br.".into());
    }
    Ok(dominio)
}

fn tema_valido(entrada: &str) -> Result<String, String> {
    let tema = entrada.split_whitespace().collect::<Vec<_>>().join(" ");
    if tema.is_empty() || tema.chars().count() > 40 || tema.chars().any(char::is_control) {
        return Err("Dê um nome de até 40 caracteres para o tema.".into());
    }
    Ok(tema)
}

pub fn adicionar(
    configuracoes: &mut Configuracoes,
    dominio: &str,
    tema: &str,
) -> Result<(), String> {
    let dominio = dominio_valido(dominio)?;
    let tema = tema_valido(tema)?;
    if configuracoes.fontes.len() >= LIMITE_FONTES {
        return Err("Limite de 100 fontes atingido.".into());
    }
    if configuracoes.fontes.iter().any(|fonte| {
        fonte.dominio.eq_ignore_ascii_case(&dominio)
            && fonte.tema.to_lowercase() == tema.to_lowercase()
    }) {
        return Err("Essa fonte já está cadastrada neste tema.".into());
    }
    configuracoes.fontes.push(Fonte { dominio, tema });
    Ok(())
}

pub fn remover(configuracoes: &mut Configuracoes, dominio: &str, tema: &str) -> Result<(), String> {
    let antes = configuracoes.fontes.len();
    configuracoes
        .fontes
        .retain(|fonte| !(fonte.dominio.eq_ignore_ascii_case(dominio) && fonte.tema == tema));
    if antes == configuracoes.fontes.len() {
        return Err("Fonte não encontrada.".into());
    }
    Ok(())
}

pub fn consulta(
    configuracoes: &Configuracoes,
    texto: &str,
    dominio: &str,
    tema: &str,
    filtrar: bool,
) -> Result<String, String> {
    let texto = texto.trim();
    if texto.is_empty() || texto.chars().count() > LIMITE_CONSULTA {
        return Err("Digite uma pesquisa de até 500 caracteres.".into());
    }
    let fonte = configuracoes
        .fontes
        .iter()
        .find(|fonte| fonte.dominio == dominio && fonte.tema == tema)
        .ok_or("Escolha uma fonte cadastrada nas configurações.")?;
    Ok(if filtrar {
        format!("{texto} site:{}", fonte.dominio)
    } else {
        texto.to_string()
    })
}

pub fn url_busca(buscador: Buscador, consulta: &str) -> Result<Url, String> {
    let base = match buscador {
        Buscador::Duckduckgo => "https://duckduckgo.com/",
        Buscador::Brave => "https://search.brave.com/search",
    };
    let mut url = Url::parse(base).map_err(|erro| erro.to_string())?;
    url.query_pairs_mut().append_pair("q", consulta);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::{adicionar, consulta, remover, url_busca};
    use crate::configuracoes::{Buscador, Configuracoes};

    #[test]
    fn fontes_sao_locais_e_nao_aceitam_url_ou_duplicata() {
        let mut configuracoes = Configuracoes::default();
        adicionar(&mut configuracoes, " Exemplo.COM.BR ", "Notícias").unwrap();
        assert_eq!(configuracoes.fontes[0].dominio, "exemplo.com.br");
        assert!(adicionar(&mut configuracoes, "exemplo.com.br", "notícias").is_err());
        assert!(adicionar(&mut configuracoes, "https://exemplo.com", "Estudo").is_err());
        assert!(adicionar(&mut configuracoes, "127.0.0.1", "Estudo").is_err());
        assert!(adicionar(&mut configuracoes, "exemplo.com:443", "Estudo").is_err());
        adicionar(&mut configuracoes, "exemplo.com.br", "Estudo").unwrap();
        assert_eq!(configuracoes.fontes.len(), 2);
        remover(&mut configuracoes, "exemplo.com.br", "Notícias").unwrap();
        assert_eq!(configuracoes.fontes.len(), 1);
    }

    #[test]
    fn busca_filtrada_exige_fonte_salva_e_mostra_consulta_enviada() {
        let mut configuracoes = Configuracoes::default();
        adicionar(&mut configuracoes, "exemplo.com.br", "Estudo").unwrap();
        let comum = consulta(
            &configuracoes,
            " energia solar ",
            "exemplo.com.br",
            "Estudo",
            false,
        )
        .unwrap();
        let filtrada = consulta(
            &configuracoes,
            " energia solar ",
            "exemplo.com.br",
            "Estudo",
            true,
        )
        .unwrap();
        assert_eq!(comum, "energia solar");
        assert_eq!(filtrada, "energia solar site:exemplo.com.br");
        assert_eq!(
            url_busca(Buscador::Brave, &filtrada).unwrap().as_str(),
            "https://search.brave.com/search?q=energia+solar+site%3Aexemplo.com.br"
        );
        assert!(consulta(&configuracoes, "energia solar", "outro.com", "Estudo", true).is_err());
    }
}
