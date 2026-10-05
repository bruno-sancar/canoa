use adblock::{engine::Engine, lists::FilterSet, request::Request};
use serde::Serialize;
use std::{cell::RefCell, collections::BTreeMap, sync::Mutex};
use tauri::Url;

const EASYLIST: &str = include_str!("../listas/easylist.txt");
const EASYPRIVACY: &str = include_str!("../listas/easyprivacy.txt");

#[derive(Clone)]
pub struct ListasAtualizadas {
    pub versao: String,
    pub atualizado_em: u64,
    pub easylist: String,
    pub easyprivacy: String,
    pub easylist_sha256: String,
    pub easyprivacy_sha256: String,
    pub tem_versao_anterior: bool,
}

struct Politica {
    ativo: bool,
    excecoes: Vec<String>,
}

struct Regras {
    revisao: u64,
    versao: String,
    atualizado_em: Option<u64>,
    easylist: String,
    easyprivacy: String,
    easylist_sha256: String,
    easyprivacy_sha256: String,
    tem_versao_anterior: bool,
}

#[derive(Default)]
struct Contagem {
    pagina: String,
    bloqueados_pagina: u64,
    total_sessao: u64,
    categorias: BTreeMap<String, u64>,
}

#[derive(Clone, Serialize)]
pub struct ResumoProtecao {
    pub pagina: String,
    pub bloqueados_pagina: u64,
    pub total_sessao: u64,
    pub categorias: BTreeMap<String, u64>,
    pub listas_versao: String,
    pub listas_atualizadas_em: Option<u64>,
    pub easylist_sha256: String,
    pub easyprivacy_sha256: String,
    pub tem_versao_anterior: bool,
}

pub struct Protecao {
    politica: Mutex<Politica>,
    regras: Mutex<Regras>,
    contagem: Mutex<Contagem>,
}

struct MotorLocal {
    revisao: u64,
    motor: Engine,
}

thread_local! { static MOTOR: RefCell<Option<MotorLocal>> = const { RefCell::new(None) }; }

fn novo_motor(easylist: &str, easyprivacy: &str) -> Engine {
    let mut filtros = FilterSet::new(false);
    filtros.add_filter_list(easylist.to_string(), Default::default());
    filtros.add_filter_list(easyprivacy.to_string(), Default::default());
    Engine::new_with_filter_set(filtros)
}

pub fn preparar_motor() {
    MOTOR.with(|slot| {
        if slot.borrow().is_none() {
            *slot.borrow_mut() = Some(MotorLocal {
                revisao: 0,
                motor: novo_motor(EASYLIST, EASYPRIVACY),
            });
        }
    });
}

impl Protecao {
    pub fn nova(ativo: bool, excecoes: Vec<String>) -> Self {
        Self {
            politica: Mutex::new(Politica { ativo, excecoes }),
            regras: Mutex::new(Regras {
                revisao: 0,
                versao: "listas incluídas na versão".into(),
                atualizado_em: None,
                easylist: EASYLIST.to_string(),
                easyprivacy: EASYPRIVACY.to_string(),
                easylist_sha256: sha256(EASYLIST),
                easyprivacy_sha256: sha256(EASYPRIVACY),
                tem_versao_anterior: false,
            }),
            contagem: Mutex::new(Contagem::default()),
        }
    }

    pub fn atualizar(&self, ativo: bool, excecoes: Vec<String>) {
        if let Ok(mut politica) = self.politica.lock() {
            politica.ativo = ativo;
            politica.excecoes = excecoes;
        }
    }

    pub fn carregar_listas(&self, listas: &ListasAtualizadas) {
        if let Ok(mut regras) = self.regras.lock() {
            regras.revisao = regras.revisao.wrapping_add(1);
            regras.versao = listas.versao.clone();
            regras.atualizado_em = Some(listas.atualizado_em);
            regras.easylist.clone_from(&listas.easylist);
            regras.easyprivacy.clone_from(&listas.easyprivacy);
            regras.easylist_sha256.clone_from(&listas.easylist_sha256);
            regras
                .easyprivacy_sha256
                .clone_from(&listas.easyprivacy_sha256);
            regras.tem_versao_anterior = listas.tem_versao_anterior;
        }
    }

    pub fn iniciar_pagina(&self, pagina: &str) {
        if let Ok(mut contagem) = self.contagem.lock() {
            if contagem.pagina != pagina {
                contagem.pagina = pagina.to_string();
                contagem.bloqueados_pagina = 0;
                contagem.categorias.clear();
            }
        }
    }

    pub fn registrar_bloqueio(&self, tipo: &str) -> ResumoProtecao {
        if let Ok(mut contagem) = self.contagem.lock() {
            contagem.bloqueados_pagina = contagem.bloqueados_pagina.saturating_add(1);
            contagem.total_sessao = contagem.total_sessao.saturating_add(1);
            *contagem.categorias.entry(tipo.to_string()).or_default() += 1;
        }
        self.resumo()
    }

    pub fn resumo(&self) -> ResumoProtecao {
        let contagem = self
            .contagem
            .lock()
            .map(|valor| {
                (
                    valor.pagina.clone(),
                    valor.bloqueados_pagina,
                    valor.total_sessao,
                    valor.categorias.clone(),
                )
            })
            .unwrap_or_default();
        let regras = self.regras.lock().ok();
        let (versao, atualizado_em, easylist_sha256, easyprivacy_sha256, tem_versao_anterior) =
            regras
                .as_ref()
                .map(|r| {
                    (
                        r.versao.clone(),
                        r.atualizado_em,
                        r.easylist_sha256.clone(),
                        r.easyprivacy_sha256.clone(),
                        r.tem_versao_anterior,
                    )
                })
                .unwrap_or_else(|| {
                    (
                        "indisponível".into(),
                        None,
                        String::new(),
                        String::new(),
                        false,
                    )
                });
        ResumoProtecao {
            pagina: contagem.0,
            bloqueados_pagina: contagem.1,
            total_sessao: contagem.2,
            categorias: contagem.3,
            listas_versao: versao,
            listas_atualizadas_em: atualizado_em,
            easylist_sha256,
            easyprivacy_sha256,
            tem_versao_anterior,
        }
    }

    pub fn bloquear(&self, url: &str, origem: &str, tipo: &str) -> bool {
        let Ok(politica) = self.politica.lock() else {
            return false;
        };
        if !politica.ativo || em_excecao(origem, &politica.excecoes) {
            return false;
        }
        let Ok(pedido) = Request::new(url, origem, tipo, "GET") else {
            return false;
        };
        let Ok(regras) = self.regras.lock() else {
            return false;
        };
        MOTOR.with(|slot| {
            let mut motor = slot.borrow_mut();
            if motor
                .as_ref()
                .map_or(true, |atual| atual.revisao != regras.revisao)
            {
                *motor = Some(MotorLocal {
                    revisao: regras.revisao,
                    motor: novo_motor(&regras.easylist, &regras.easyprivacy),
                });
            }
            motor
                .as_ref()
                .is_some_and(|atual| atual.motor.check_network_request(&pedido).should_block())
        })
    }
}

pub fn sha256(conteudo: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(conteudo.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn em_excecao(origem: &str, excecoes: &[String]) -> bool {
    let Ok(url) = Url::parse(origem) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    excecoes
        .iter()
        .any(|excecao| host == excecao || host.ends_with(&format!(".{excecao}")))
}

#[cfg(test)]
mod tests {
    use super::{em_excecao, Protecao};

    #[test]
    fn excecao_cobre_subdominios_sem_cobrir_sufixos_enganosos() {
        let lista = vec!["example.com".to_string()];
        assert!(em_excecao("https://example.com", &lista));
        assert!(em_excecao("https://news.example.com", &lista));
        assert!(!em_excecao("https://notexample.com", &lista));
    }

    #[test]
    fn bloqueia_rastreamento_com_lista_e_permite_origem_em_excecao() {
        super::preparar_motor();
        let protecao = Protecao::nova(true, vec![]);
        let bloqueado = protecao.bloquear(
            "https://www.google-analytics.com/analytics.js",
            "https://loja.example/",
            "script",
        );
        assert!(bloqueado);
        protecao.atualizar(true, vec!["loja.example".into()]);
        assert!(!protecao.bloquear(
            "https://www.google-analytics.com/analytics.js",
            "https://loja.example/",
            "script"
        ));
        protecao.atualizar(false, vec![]);
        assert!(!protecao.bloquear(
            "https://www.google-analytics.com/analytics.js",
            "https://loja.example/",
            "script"
        ));
    }

    #[test]
    fn contadores_por_pagina_reiniciam_sem_apagar_total_da_sessao() {
        let protecao = Protecao::nova(true, vec![]);
        protecao.iniciar_pagina("https://example.com/a");
        protecao.registrar_bloqueio("script");
        protecao.registrar_bloqueio("image");
        let primeira = protecao.resumo();
        assert_eq!(primeira.bloqueados_pagina, 2);
        assert_eq!(primeira.total_sessao, 2);
        assert_eq!(primeira.categorias.get("script"), Some(&1));
        protecao.iniciar_pagina("https://example.com/b");
        let segunda = protecao.resumo();
        assert_eq!(segunda.bloqueados_pagina, 0);
        assert_eq!(segunda.total_sessao, 2);
    }
}
