use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{AppHandle, Manager};

const NOME_ARQUIVO: &str = "configuracoes.json";
const LIMITE_CONFIGURACOES_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Buscador {
    #[default]
    Duckduckgo,
    Brave,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fonte {
    pub dominio: String,
    pub tema: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrupoAba {
    pub nome: String,
    #[serde(default)]
    pub recolhido: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntradaHistorico {
    pub endereco: String,
    pub titulo: String,
    pub visitado_em: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LimiteSite {
    pub dominio: String,
    pub minutos_diarios: u16,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsoSite {
    pub dominio: String,
    pub dia: String,
    pub segundos: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AcoesBarra {
    pub favorito: bool,
    pub protecao: bool,
    pub leitura: bool,
    pub baixar: bool,
}

impl Default for AcoesBarra {
    fn default() -> Self {
        Self {
            favorito: true,
            protecao: true,
            leitura: true,
            baixar: true,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Configuracoes {
    pub mostrar_memoria: bool,
    #[serde(default = "protecoes_ativas_por_padrao")]
    pub bloqueador_ativo: bool,
    #[serde(default)]
    pub excecoes_bloqueador: Vec<String>,
    pub pasta_downloads: Option<PathBuf>,
    pub buscador: Buscador,
    pub fontes: Vec<Fonte>,
    pub favoritos: Vec<Favorito>,
    pub lista_leitura: Vec<ItemLeitura>,
    pub salvos: Vec<ItemSalvo>,
    #[serde(default)]
    pub sites_bloqueados: Vec<String>,
    #[serde(default)]
    pub limpar_rastreamento: bool,
    #[serde(default)]
    pub historico_downloads: Vec<ItemDownload>,
    #[serde(default)]
    pub acoes_barra: AcoesBarra,
    #[serde(default)]
    pub grupos_abas: Vec<GrupoAba>,
    #[serde(default)]
    pub historico_ativo: bool,
    #[serde(default = "dias_historico_padrao")]
    pub dias_historico: u16,
    #[serde(default)]
    pub historico_navegacao: Vec<EntradaHistorico>,
    #[serde(default)]
    pub limites_sites: Vec<LimiteSite>,
    #[serde(default)]
    pub uso_diario_sites: Vec<UsoSite>,
    #[serde(default)]
    pub sites_foco: Vec<String>,
    #[serde(default)]
    pub foco_ate_epoch: Option<u64>,
    #[serde(default = "duracao_foco_padrao")]
    pub duracao_foco_minutos: u16,
}

fn dias_historico_padrao() -> u16 {
    90
}
fn duracao_foco_padrao() -> u16 {
    25
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemDownload {
    pub nome: String,
    pub caminho: Option<PathBuf>,
    pub sucesso: bool,
    pub registrado_em: u64,
    #[serde(default)]
    pub tipo: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemLeitura {
    pub endereco: String,
    pub titulo: String,
    pub adicionado_em: u64,
    #[serde(default)]
    pub lido: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemSalvo {
    pub endereco: String,
    pub titulo: String,
    pub tipo: String,
    pub salvo_em: u64,
    #[serde(default)]
    pub caminho: Option<PathBuf>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Favorito {
    pub endereco: String,
    pub titulo: String,
    #[serde(default)]
    pub pastas: Vec<String>,
    #[serde(default)]
    pub marcadores: Vec<String>,
}

impl Default for Configuracoes {
    fn default() -> Self {
        Self {
            mostrar_memoria: true,
            bloqueador_ativo: true,
            excecoes_bloqueador: Vec::new(),
            pasta_downloads: None,
            buscador: Buscador::default(),
            fontes: Vec::new(),
            favoritos: Vec::new(),
            lista_leitura: Vec::new(),
            salvos: Vec::new(),
            sites_bloqueados: Vec::new(),
            limpar_rastreamento: false,
            historico_downloads: Vec::new(),
            acoes_barra: AcoesBarra::default(),
            grupos_abas: Vec::new(),
            historico_ativo: false,
            dias_historico: dias_historico_padrao(),
            historico_navegacao: Vec::new(),
            limites_sites: Vec::new(),
            uso_diario_sites: Vec::new(),
            sites_foco: Vec::new(),
            foco_ate_epoch: None,
            duracao_foco_minutos: duracao_foco_padrao(),
        }
    }
}

fn protecoes_ativas_por_padrao() -> bool {
    true
}

fn caminho(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|pasta| pasta.join(NOME_ARQUIVO))
        .map_err(|erro| format!("Não foi possível localizar as configurações: {erro}"))
}

pub fn carregar(app: &AppHandle) -> Result<Configuracoes, String> {
    let arquivo = caminho(app)?;
    if let Ok(metadata) = fs::metadata(&arquivo) {
        if metadata.len() > LIMITE_CONFIGURACOES_BYTES {
            return Err("O arquivo de configurações excede o limite seguro de 20 MiB.".into());
        }
    }
    let conteudo = match fs::read(&arquivo) {
        Ok(conteudo) => conteudo,
        Err(erro) if erro.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Configuracoes::default());
        }
        Err(erro) => return Err(format!("Não foi possível ler as configurações: {erro}")),
    };
    serde_json::from_slice(&conteudo)
        .map_err(|erro| format!("O arquivo de configurações está inválido: {erro}"))
}

pub fn salvar(app: &AppHandle, configuracoes: &Configuracoes) -> Result<(), String> {
    let arquivo = caminho(app)?;
    let diretorio = arquivo
        .parent()
        .ok_or("Não foi possível localizar a pasta de configurações.")?;
    fs::create_dir_all(diretorio)
        .map_err(|erro| format!("Não foi possível criar a pasta de configurações: {erro}"))?;
    let bytes = serde_json::to_vec_pretty(configuracoes)
        .map_err(|erro| format!("Não foi possível preparar as configurações: {erro}"))?;
    let tempo = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|erro| format!("Não foi possível salvar as configurações: {erro}"))?
        .as_nanos();
    let temporario = diretorio.join(format!(".{NOME_ARQUIVO}.{}.{tempo}.tmp", process::id()));

    let escrita = (|| -> Result<(), String> {
        let mut destino = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporario)
            .map_err(|erro| format!("Não foi possível preparar o salvamento: {erro}"))?;
        destino
            .write_all(&bytes)
            .and_then(|_| destino.sync_all())
            .map_err(|erro| format!("Não foi possível gravar as configurações: {erro}"))?;
        drop(destino);
        substituir_atomico(&temporario, &arquivo)
            .map_err(|erro| format!("Não foi possível concluir o salvamento: {erro}"))
    })();
    if escrita.is_err() {
        let _ = fs::remove_file(&temporario);
    }
    escrita
}

pub fn validar_pasta_downloads(pasta: &Path) -> Result<(), String> {
    if !pasta.is_absolute() {
        return Err("Escolha um caminho absoluto para a pasta de downloads.".into());
    }
    let metadados = fs::metadata(pasta)
        .map_err(|_| "A pasta de downloads não existe ou não está acessível.".to_string())?;
    if !metadados.is_dir() {
        return Err("O caminho de downloads precisa ser uma pasta.".into());
    }
    verificar_permissao_de_criar_arquivo(pasta)
}

#[cfg(windows)]
fn verificar_permissao_de_criar_arquivo(pasta: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;

    type Handle = *mut std::ffi::c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(
            nome: *const u16,
            acesso: u32,
            compartilhamento: u32,
            seguranca: *mut std::ffi::c_void,
            criacao: u32,
            atributos: u32,
            modelo: Handle,
        ) -> Handle;
        fn CloseHandle(handle: Handle) -> i32;
    }

    const FILE_ADD_FILE: u32 = 0x0002;
    const SHARE_ALL: u32 = 0x0007;
    const OPEN_EXISTING: u32 = 3;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let nome: Vec<u16> = pasta.as_os_str().encode_wide().chain(Some(0)).collect();
    // Consulta a permissão de criar arquivos na pasta sem criar um arquivo de teste.
    let handle = unsafe {
        CreateFileW(
            nome.as_ptr(),
            FILE_ADD_FILE,
            SHARE_ALL,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            std::ptr::null_mut(),
        )
    };
    if handle == (-1_isize) as Handle {
        return Err("O Canoa não tem permissão para salvar arquivos nessa pasta.".into());
    }
    unsafe { CloseHandle(handle) };
    Ok(())
}

#[cfg(not(windows))]
fn verificar_permissao_de_criar_arquivo(_pasta: &Path) -> Result<(), String> {
    // A permissão efetiva será novamente conferida ao abrir o arquivo de destino.
    Ok(())
}

#[cfg(windows)]
fn substituir_atomico(origem: &Path, destino: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(origem: *const u16, destino: *const u16, opcoes: u32) -> i32;
    }

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;
    let origem: Vec<u16> = origem.as_os_str().encode_wide().chain(Some(0)).collect();
    let destino: Vec<u16> = destino.as_os_str().encode_wide().chain(Some(0)).collect();
    let sucesso = unsafe {
        MoveFileExW(
            origem.as_ptr(),
            destino.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if sucesso == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn substituir_atomico(origem: &Path, destino: &Path) -> std::io::Result<()> {
    fs::rename(origem, destino)
}

#[cfg(test)]
mod tests {
    use super::{Buscador, Configuracoes, GrupoAba};

    #[test]
    fn configuracao_antiga_recebe_buscador_padrao() {
        let anterior = br#"{"mostrar_memoria":false,"pasta_downloads":null}"#;
        let configuracao: Configuracoes = serde_json::from_slice(anterior).unwrap();
        assert!(!configuracao.mostrar_memoria);
        assert_eq!(configuracao.buscador, Buscador::Duckduckgo);
        assert!(configuracao.fontes.is_empty());
        assert!(configuracao.favoritos.is_empty());
        assert!(configuracao.lista_leitura.is_empty());
        assert!(configuracao.salvos.is_empty());
        assert!(configuracao.sites_bloqueados.is_empty());
        assert!(!configuracao.limpar_rastreamento);
        assert!(configuracao.historico_downloads.is_empty());
        assert!(configuracao.acoes_barra.favorito);
        assert!(configuracao.acoes_barra.protecao);
        assert!(configuracao.acoes_barra.leitura);
        assert!(configuracao.acoes_barra.baixar);
        assert!(configuracao.grupos_abas.is_empty());
    }

    #[test]
    fn historico_de_download_antigo_continua_compatível() {
        let anterior = br#"{"nome":"arquivo.pdf","caminho":"C:/Downloads/arquivo.pdf","sucesso":true,"registrado_em":123}"#;
        let item: super::ItemDownload = serde_json::from_slice(anterior).unwrap();
        assert_eq!(item.nome, "arquivo.pdf");
        assert!(item.sucesso);
        assert!(item.tipo.is_empty());
    }

    #[test]
    fn grupos_salvam_nome_e_estado_de_recolhimento() {
        let mut configuracao = Configuracoes::default();
        configuracao.grupos_abas.push(GrupoAba {
            nome: "Pesquisa".into(),
            recolhido: true,
        });
        let bytes = serde_json::to_vec(&configuracao).unwrap();
        let reaberta: Configuracoes = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(reaberta.grupos_abas.len(), 1);
        assert_eq!(reaberta.grupos_abas[0].nome, "Pesquisa");
        assert!(reaberta.grupos_abas[0].recolhido);
    }
}
