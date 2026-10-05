use crate::configuracoes::{Configuracoes, EntradaHistorico, UsoSite};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn agora_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

pub fn dia_local() -> String {
    #[cfg(windows)]
    {
        let agora = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
        return format!("{:04}-{:02}-{:02}", agora.wYear, agora.wMonth, agora.wDay);
    }
    #[cfg(not(windows))]
    {
        let dia = agora_epoch() / 86_400;
        // Civil date from days since the Unix epoch.
        let z = dia as i64 + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let mut ano = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let mes = mp + if mp < 10 { 3 } else { -9 };
        ano += i64::from(mes <= 2);
        format!("{ano:04}-{mes:02}-{:02}", doy - (153 * mp + 2) / 5 + 1)
    }
}

pub fn dominio_corresponde(host: &str, dominio: &str) -> bool {
    host.eq_ignore_ascii_case(dominio)
        || host
            .to_ascii_lowercase()
            .strip_suffix(&format!(".{dominio}"))
            .is_some()
}

pub fn limite(config: &Configuracoes, host: &str) -> Option<(u64, u64)> {
    let hoje = dia_local();
    let limite = config
        .limites_sites
        .iter()
        .find(|item| dominio_corresponde(host, &item.dominio))?;
    let segundos = config
        .uso_diario_sites
        .iter()
        .find(|uso| uso.dia == hoje && dominio_corresponde(host, &uso.dominio))
        .map(|uso| uso.segundos)
        .unwrap_or_default();
    Some((segundos, u64::from(limite.minutos_diarios) * 60))
}

pub fn foco_bloqueia(config: &Configuracoes, host: &str, agora: u64) -> bool {
    config.foco_ate_epoch.is_some_and(|ate| ate > agora)
        && config
            .sites_foco
            .iter()
            .any(|dominio| dominio_corresponde(host, dominio))
}

pub fn registrar_uso(config: &mut Configuracoes, host: &str, segundos: u64) -> Option<(u64, u64)> {
    let host = crate::privacidade::normalizar_dominio(host).ok()?;
    let hoje = dia_local();
    let (dominio_limite, limite) = config
        .limites_sites
        .iter()
        .find(|item| dominio_corresponde(&host, &item.dominio))
        .map(|item| (item.dominio.clone(), u64::from(item.minutos_diarios) * 60))?;
    config.uso_diario_sites.retain(|uso| uso.dia == hoje);
    let uso = if let Some(uso) = config
        .uso_diario_sites
        .iter_mut()
        .find(|uso| uso.dominio == dominio_limite && uso.dia == hoje)
    {
        uso
    } else {
        config.uso_diario_sites.push(UsoSite {
            dominio: dominio_limite,
            dia: hoje,
            segundos: 0,
        });
        config.uso_diario_sites.last_mut()?
    };
    // The UI reports in short intervals; clamp the delta so a stale tab cannot
    // charge hours of usage in a single update.
    uso.segundos = uso.segundos.saturating_add(segundos.min(30));
    Some((uso.segundos, limite))
}

pub fn registrar_visita(config: &mut Configuracoes, endereco: &str, titulo: &str, agora: u64) {
    if !config.historico_ativo {
        return;
    }
    let Some(url) = tauri::Url::parse(endereco)
        .ok()
        .filter(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some())
    else {
        return;
    };
    let endereco = url.to_string();
    config.historico_navegacao.insert(
        0,
        EntradaHistorico {
            endereco,
            titulo: titulo.chars().take(300).collect(),
            visitado_em: agora,
        },
    );
    let manter_desde =
        agora.saturating_sub(u64::from(config.dias_historico.clamp(1, 365)) * 86_400);
    config
        .historico_navegacao
        .retain(|entrada| entrada.visitado_em >= manter_desde);
    config.historico_navegacao.truncate(3_000);
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::configuracoes::{LimiteSite, UsoSite};

    #[test]
    fn dominio_inclui_subdominios_sem_confundir_sufixos() {
        assert!(dominio_corresponde("m.rede.com", "rede.com"));
        assert!(!dominio_corresponde("falsarede.com", "rede.com"));
    }

    #[test]
    fn foco_so_bloqueia_dominios_cadastrados_enquanto_ativo() {
        let mut config = Configuracoes::default();
        config.sites_foco.push("rede.com".into());
        config.foco_ate_epoch = Some(200);
        assert!(foco_bloqueia(&config, "www.rede.com", 100));
        assert!(!foco_bloqueia(&config, "www.rede.com", 200));
        assert!(!foco_bloqueia(&config, "outra.com", 100));
    }

    #[test]
    fn limite_e_uso_sao_persistiveis_e_delta_e_limitado() {
        let mut config = Configuracoes::default();
        config.limites_sites.push(LimiteSite {
            dominio: "rede.com".into(),
            minutos_diarios: 1,
        });
        let (uso, max) = registrar_uso(&mut config, "www.rede.com", 999).unwrap();
        assert_eq!((uso, max), (30, 60));
        assert!(limite(&config, "www.rede.com").unwrap().0 >= 30);
        assert!(!config.uso_diario_sites.is_empty());
        let (uso_www, _) = registrar_uso(&mut config, "www.rede.com", 15).unwrap();
        let (uso_app, _) = registrar_uso(&mut config, "app.rede.com", 15).unwrap();
        assert_eq!(uso_app, uso_www + 15);
        assert_eq!(config.uso_diario_sites[0].dominio, "rede.com");
        let _: &UsoSite = &config.uso_diario_sites[0];
    }

    #[test]
    fn historico_e_opt_in_e_aceita_apenas_http_https() {
        let mut config = Configuracoes::default();
        registrar_visita(&mut config, "https://example.com", "Título", 1_800_000_000);
        assert!(config.historico_navegacao.is_empty());
        config.historico_ativo = true;
        registrar_visita(&mut config, "file:///segredo", "Arquivo", 1_800_000_001);
        registrar_visita(
            &mut config,
            "https://example.com/path",
            "Título",
            1_800_000_002,
        );
        assert_eq!(config.historico_navegacao.len(), 1);
        assert_eq!(config.historico_navegacao[0].titulo, "Título");
    }
}
