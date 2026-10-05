use crate::bloqueio::{sha256, ListasAtualizadas};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const URL_EASYLIST: &str = "https://easylist.to/easylist/easylist.txt";
const URL_EASYPRIVACY: &str = "https://easylist.to/easylist/easyprivacy.txt";
const LIMITE_BYTES: u64 = 12 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
struct Manifesto {
    versao: String,
    atualizado_em: u64,
    easylist_sha256: String,
    easyprivacy_sha256: String,
}

fn agora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|tempo| tempo.as_secs())
        .unwrap_or_default()
}

fn epoca_da_pasta(pasta: &Path) -> u64 {
    pasta
        .file_name()
        .and_then(|nome| nome.to_str())
        .and_then(|nome| nome.strip_prefix("listas-"))
        .and_then(|nome| nome.split('-').next())
        .and_then(|valor| valor.parse().ok())
        .unwrap_or(0)
}

fn proxima_epoca(diretorio: &Path) -> u64 {
    let mais_recente = fs::read_dir(diretorio)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entrada| epoca_da_pasta(&entrada.path()))
        .max()
        .unwrap_or(0);
    agora().max(mais_recente.saturating_add(1))
}

fn ordenar_por_epoca(pastas: &mut [PathBuf]) {
    pastas.sort_by(|a, b| {
        epoca_da_pasta(a)
            .cmp(&epoca_da_pasta(b))
            .then_with(|| a.cmp(b))
    });
}

fn ler_lista(url: &str, titulo_esperado: &str) -> Result<String, String> {
    let cliente = reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(45))
        .redirect(reqwest::redirect::Policy::limited(3))
        .user_agent("Canoa/0.1.26 atualizador de filtros")
        .build()
        .map_err(|erro| format!("Não foi possível iniciar a conexão segura: {erro}"))?;
    let resposta = cliente
        .get(url)
        .send()
        .map_err(|erro| format!("Falha ao baixar a lista oficial: {erro}"))?;
    if !resposta.status().is_success() {
        return Err(format!(
            "A origem respondeu com HTTP {}.",
            resposta.status()
        ));
    }
    if resposta.url().host_str() != Some("easylist.to") {
        return Err("A lista foi redirecionada para um domínio inesperado.".into());
    }
    if resposta
        .content_length()
        .is_some_and(|tamanho| tamanho > LIMITE_BYTES)
    {
        return Err("A lista excede o limite de tamanho permitido.".into());
    }
    let mut bytes = Vec::new();
    resposta
        .take(LIMITE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|erro| format!("Falha ao ler a lista: {erro}"))?;
    if bytes.len() as u64 > LIMITE_BYTES {
        return Err("A lista excede o limite de tamanho permitido.".into());
    }
    let texto = String::from_utf8(bytes)
        .map_err(|_| "A origem enviou texto que não está em UTF-8.".to_string())?;
    validar_lista(&texto, titulo_esperado)?;
    Ok(texto)
}

fn validar_lista(texto: &str, titulo_esperado: &str) -> Result<(), String> {
    if texto.len() < 32 || !texto.starts_with("[Adblock Plus") {
        return Err("O formato da lista não corresponde ao padrão ABP.".into());
    }
    let titulo = texto
        .lines()
        .take(16)
        .any(|linha| linha.eq_ignore_ascii_case(&format!("! Title: {titulo_esperado}")));
    if !titulo {
        return Err(format!(
            "O cabeçalho não identifica a lista {titulo_esperado} esperada."
        ));
    }
    let regras = texto
        .lines()
        .filter(|linha| {
            let linha = linha.trim();
            !linha.is_empty() && !linha.starts_with('!') && !linha.starts_with('[')
        })
        .count();
    if regras < 1_000 {
        return Err(format!(
            "A lista {titulo_esperado} parece incompleta ({regras} regras)."
        ));
    }
    Ok(())
}

fn versao_lista(texto: &str) -> String {
    texto
        .lines()
        .take(16)
        .find_map(|linha| {
            linha
                .strip_prefix("! Version:")
                .map(str::trim)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "versão sem rótulo".into())
}

fn gravar_seguro(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut arquivo = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|erro| format!("Não foi possível preparar o arquivo de atualização: {erro}"))?;
    arquivo
        .write_all(bytes)
        .map_err(|erro| format!("Não foi possível salvar a lista: {erro}"))?;
    arquivo
        .sync_all()
        .map_err(|erro| format!("Não foi possível finalizar a lista: {erro}"))
}

pub fn baixar_e_salvar(diretorio: &Path) -> Result<ListasAtualizadas, String> {
    salvar_baseline_se_necessario(diretorio)?;
    let easylist = ler_lista(URL_EASYLIST, "EasyList")?;
    let easyprivacy = ler_lista(URL_EASYPRIVACY, "EasyPrivacy")?;
    let epoch = proxima_epoca(diretorio);
    fs::create_dir_all(diretorio)
        .map_err(|erro| format!("Não foi possível criar a pasta de listas: {erro}"))?;
    let sufixo = format!("{}-{}", std::process::id(), epoch);
    let temporario = diretorio.join(format!(".baixando-{sufixo}"));
    let destino = diretorio.join(format!("listas-{epoch}-{}", std::process::id()));
    fs::create_dir(&temporario)
        .map_err(|erro| format!("Não foi possível reservar a atualização: {erro}"))?;
    let resultado = (|| {
        gravar_seguro(&temporario.join("easylist.txt"), easylist.as_bytes())?;
        gravar_seguro(&temporario.join("easyprivacy.txt"), easyprivacy.as_bytes())?;
        let manifesto = Manifesto {
            versao: format!(
                "EasyList {} · EasyPrivacy {}",
                versao_lista(&easylist),
                versao_lista(&easyprivacy)
            ),
            atualizado_em: epoch,
            easylist_sha256: sha256(&easylist),
            easyprivacy_sha256: sha256(&easyprivacy),
        };
        let json = serde_json::to_vec_pretty(&manifesto).map_err(|erro| erro.to_string())?;
        gravar_seguro(&temporario.join("manifesto.json"), &json)?;
        fs::rename(&temporario, &destino)
            .map_err(|erro| format!("Não foi possível ativar a atualização validada: {erro}"))?;
        Ok::<Manifesto, String>(manifesto)
    })();
    let manifesto = match resultado {
        Ok(valor) => valor,
        Err(erro) => {
            let _ = fs::remove_dir_all(&temporario);
            return Err(erro);
        }
    };
    manter_duas_versoes(diretorio, &destino);
    let tem_versao_anterior = contar_validas(diretorio) > 1;
    Ok(ListasAtualizadas {
        versao: manifesto.versao,
        atualizado_em: manifesto.atualizado_em,
        easylist,
        easyprivacy,
        easylist_sha256: manifesto.easylist_sha256,
        easyprivacy_sha256: manifesto.easyprivacy_sha256,
        tem_versao_anterior,
    })
}

fn manifesto_valido(pasta: &Path) -> Option<(Manifesto, String, String)> {
    let manifesto: Manifesto =
        serde_json::from_slice(&fs::read(pasta.join("manifesto.json")).ok()?).ok()?;
    let easylist = fs::read_to_string(pasta.join("easylist.txt")).ok()?;
    let easyprivacy = fs::read_to_string(pasta.join("easyprivacy.txt")).ok()?;
    if sha256(&easylist) != manifesto.easylist_sha256
        || sha256(&easyprivacy) != manifesto.easyprivacy_sha256
    {
        return None;
    }
    if validar_lista(&easylist, "EasyList").is_err()
        || validar_lista(&easyprivacy, "EasyPrivacy").is_err()
    {
        return None;
    }
    Some((manifesto, easylist, easyprivacy))
}

pub fn carregar_ultima(diretorio: &Path) -> Option<ListasAtualizadas> {
    let mut pastas = fs::read_dir(diretorio)
        .ok()?
        .filter_map(Result::ok)
        .map(|entrada| entrada.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .is_some_and(|nome| nome.to_string_lossy().starts_with("listas-"))
        })
        .collect::<Vec<_>>();
    ordenar_por_epoca(&mut pastas);
    let quantidade = pastas
        .iter()
        .filter(|pasta| manifesto_valido(pasta).is_some())
        .count();
    pastas.into_iter().rev().find_map(|pasta| {
        let (manifesto, easylist, easyprivacy) = manifesto_valido(&pasta)?;
        Some(ListasAtualizadas {
            versao: manifesto.versao,
            atualizado_em: manifesto.atualizado_em,
            easylist,
            easyprivacy,
            easylist_sha256: manifesto.easylist_sha256,
            easyprivacy_sha256: manifesto.easyprivacy_sha256,
            tem_versao_anterior: quantidade > 1,
        })
    })
}

fn listar_validas(diretorio: &Path) -> Vec<PathBuf> {
    let mut pastas = fs::read_dir(diretorio)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entrada| entrada.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .is_some_and(|nome| nome.to_string_lossy().starts_with("listas-"))
                && manifesto_valido(path).is_some()
        })
        .collect::<Vec<_>>();
    ordenar_por_epoca(&mut pastas);
    pastas
}

fn contar_validas(diretorio: &Path) -> usize {
    listar_validas(diretorio).len()
}

fn salvar_baseline_se_necessario(diretorio: &Path) -> Result<(), String> {
    if contar_validas(diretorio) > 0 {
        return Ok(());
    }
    let easylist = include_str!("../listas/easylist.txt").to_string();
    let easyprivacy = include_str!("../listas/easyprivacy.txt").to_string();
    validar_lista(&easylist, "EasyList")?;
    validar_lista(&easyprivacy, "EasyPrivacy")?;
    fs::create_dir_all(diretorio)
        .map_err(|erro| format!("Não foi possível preparar a pasta de listas: {erro}"))?;
    let temporario = diretorio.join(format!(".baseline-{}", std::process::id()));
    let destino = diretorio.join(format!("listas-0000000001-{}", std::process::id()));
    if destino.exists() {
        return Ok(());
    }
    fs::create_dir(&temporario)
        .map_err(|erro| format!("Não foi possível preparar a versão incluída: {erro}"))?;
    let resultado = (|| {
        gravar_seguro(&temporario.join("easylist.txt"), easylist.as_bytes())?;
        gravar_seguro(&temporario.join("easyprivacy.txt"), easyprivacy.as_bytes())?;
        let manifesto = Manifesto {
            versao: "Versão incluída no Canoa".into(),
            atualizado_em: 0,
            easylist_sha256: sha256(&easylist),
            easyprivacy_sha256: sha256(&easyprivacy),
        };
        let json = serde_json::to_vec_pretty(&manifesto).map_err(|erro| erro.to_string())?;
        gravar_seguro(&temporario.join("manifesto.json"), &json)?;
        fs::rename(&temporario, &destino)
            .map_err(|erro| format!("Não foi possível registrar a versão incluída: {erro}"))
    })();
    if resultado.is_err() {
        let _ = fs::remove_dir_all(&temporario);
    }
    resultado
}

pub fn reverter_para_anterior(diretorio: &Path) -> Result<ListasAtualizadas, String> {
    let pastas = listar_validas(diretorio);
    if pastas.len() < 2 {
        return Err("Ainda não há uma versão anterior válida para restaurar.".into());
    }
    let (manifesto, easylist, easyprivacy) = manifesto_valido(&pastas[pastas.len() - 2])
        .ok_or_else(|| "A versão anterior não passou na verificação de integridade.".to_string())?;
    let epoch = proxima_epoca(diretorio);
    let temporario = diretorio.join(format!(".reversao-{}-{epoch}", std::process::id()));
    let destino = diretorio.join(format!("listas-{epoch}-{}-reversao", std::process::id()));
    fs::create_dir(&temporario)
        .map_err(|erro| format!("Não foi possível preparar a reversão: {erro}"))?;
    let resultado = (|| {
        gravar_seguro(&temporario.join("easylist.txt"), easylist.as_bytes())?;
        gravar_seguro(&temporario.join("easyprivacy.txt"), easyprivacy.as_bytes())?;
        let manifesto_reversao = Manifesto {
            atualizado_em: epoch,
            ..manifesto.clone()
        };
        let json =
            serde_json::to_vec_pretty(&manifesto_reversao).map_err(|erro| erro.to_string())?;
        gravar_seguro(&temporario.join("manifesto.json"), &json)?;
        fs::rename(&temporario, &destino)
            .map_err(|erro| format!("Não foi possível ativar a reversão: {erro}"))
    })();
    if let Err(erro) = resultado {
        let _ = fs::remove_dir_all(&temporario);
        return Err(erro);
    }
    manter_duas_versoes(diretorio, &destino);
    Ok(ListasAtualizadas {
        versao: manifesto.versao,
        atualizado_em: epoch,
        easylist,
        easyprivacy,
        easylist_sha256: manifesto.easylist_sha256,
        easyprivacy_sha256: manifesto.easyprivacy_sha256,
        tem_versao_anterior: contar_validas(diretorio) > 1,
    })
}

fn manter_duas_versoes(diretorio: &Path, atual: &Path) {
    let mut pastas = match fs::read_dir(diretorio) {
        Ok(entradas) => entradas
            .filter_map(Result::ok)
            .map(|entrada| entrada.path())
            .filter(|path| {
                path.is_dir()
                    && path
                        .file_name()
                        .is_some_and(|nome| nome.to_string_lossy().starts_with("listas-"))
            })
            .collect::<Vec<PathBuf>>(),
        Err(_) => return,
    };
    ordenar_por_epoca(&mut pastas);
    pastas.reverse();
    let mut mantidas = 0;
    for pasta in pastas {
        if !pasta.join("manifesto.json").exists() {
            continue;
        }
        if &pasta == atual || mantidas == 0 {
            mantidas += 1;
            continue;
        }
        if mantidas < 2 {
            mantidas += 1;
        } else {
            let _ = fs::remove_dir_all(pasta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{reverter_para_anterior, salvar_baseline_se_necessario, validar_lista, Manifesto};
    use crate::bloqueio::sha256;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn rejeita_resposta_html_e_lista_incompleta() {
        assert!(validar_lista("<!doctype html><title>Denied</title>", "EasyList").is_err());
        assert!(validar_lista("[Adblock Plus 2.0]\n! Title: EasyList\n/a/", "EasyList").is_err());
    }

    #[test]
    fn valida_lista_com_cabecalho_e_volume_minimo() {
        let mut lista = String::from("[Adblock Plus 2.0]\n! Title: EasyList\n");
        for numero in 0..1_001 {
            lista.push_str(&format!("||ads{numero}.example^\n"));
        }
        assert!(validar_lista(&lista, "EasyList").is_ok());
        assert!(validar_lista(&lista, "EasyPrivacy").is_err());
    }

    #[test]
    fn restaura_a_lista_anterior_e_preserva_a_versao_atual_para_nova_reversao() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pasta = PathBuf::from(std::env::temp_dir()).join(format!("canoa-listas-{id}"));
        salvar_baseline_se_necessario(&pasta).unwrap();
        let atual_epoch = super::agora().saturating_sub(100);
        let atual = pasta.join(format!("listas-{atual_epoch}-999"));
        fs::create_dir(&atual).unwrap();
        let mut nova = String::from("[Adblock Plus 2.0]\n! Title: EasyList\n");
        let mut privacidade = String::from("[Adblock Plus 2.0]\n! Title: EasyPrivacy\n");
        for numero in 0..1_001 {
            nova.push_str(&format!("||ads{numero}.example^\n"));
            privacidade.push_str(&format!("||track{numero}.example^\n"));
        }
        fs::write(atual.join("easylist.txt"), &nova).unwrap();
        fs::write(atual.join("easyprivacy.txt"), &privacidade).unwrap();
        let manifesto = Manifesto {
            versao: "teste atual".into(),
            atualizado_em: atual_epoch,
            easylist_sha256: sha256(&nova),
            easyprivacy_sha256: sha256(&privacidade),
        };
        fs::write(
            atual.join("manifesto.json"),
            serde_json::to_vec(&manifesto).unwrap(),
        )
        .unwrap();

        let restaurada = reverter_para_anterior(&pasta).unwrap();
        assert_eq!(restaurada.versao, "Versão incluída no Canoa");
        assert!(restaurada.tem_versao_anterior);
        assert_eq!(restaurada.easylist, include_str!("../listas/easylist.txt"));
        assert_eq!(
            reverter_para_anterior(&pasta).unwrap().versao,
            "teste atual"
        );
        let _ = fs::remove_dir_all(pasta);
    }
}
