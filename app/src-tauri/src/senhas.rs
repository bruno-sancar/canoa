use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305, XNonce};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::Url;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

const VERSAO: u8 = 1;
const SALT_BYTES: usize = 16;
const NONCE_BYTES: usize = 24;
const CHAVE_BYTES: usize = 32;
const ARQUIVO_MAX_BYTES: u64 = 16 * 1024 * 1024;
const TEMPO_DESBLOQUEADO: Duration = Duration::from_secs(5 * 60);

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    versao: u8,
    sal: Vec<u8>,
    nonce: Vec<u8>,
    cifra: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
struct Credencial {
    id: String,
    origem: String,
    titulo: String,
    usuario: String,
    senha: String,
    criado_em: u64,
    atualizado_em: u64,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
struct DadosCofre {
    versao: u8,
    credenciais: Vec<Credencial>,
}

struct Aberto {
    chave: Zeroizing<[u8; CHAVE_BYTES]>,
    sal: Zeroizing<Vec<u8>>,
    dados: DadosCofre,
    ultima_atividade: Instant,
}

#[derive(Clone, Serialize)]
pub struct ResumoCofre {
    pub existe: bool,
    pub desbloqueado: bool,
    pub quantidade: usize,
}

#[derive(Clone, Serialize)]
pub struct ResumoCredencial {
    pub id: String,
    pub origem: String,
    pub dominio: String,
    pub titulo: String,
    pub usuario: String,
    pub criado_em: u64,
    pub atualizado_em: u64,
}

pub struct Cofre {
    caminho: PathBuf,
    aberto: Mutex<Option<Aberto>>,
}

impl Cofre {
    pub fn novo(caminho: PathBuf) -> Self {
        Self {
            caminho,
            aberto: Mutex::new(None),
        }
    }

    pub fn resumo(&self) -> ResumoCofre {
        let existe = self.caminho.is_file();
        let Ok(mut aberto) = self.aberto.lock() else {
            return ResumoCofre {
                existe,
                desbloqueado: false,
                quantidade: 0,
            };
        };
        if aberto
            .as_ref()
            .is_some_and(|dados| dados.ultima_atividade.elapsed() >= TEMPO_DESBLOQUEADO)
        {
            *aberto = None;
        }
        ResumoCofre {
            existe,
            desbloqueado: aberto.is_some(),
            quantidade: aberto
                .as_ref()
                .map(|v| v.dados.credenciais.len())
                .unwrap_or(0),
        }
    }

    pub fn criar(&self, senha_mestra: String) -> Result<ResumoCofre, String> {
        let senha_mestra = Zeroizing::new(senha_mestra);
        if self.caminho.exists() {
            return Err("Já existe um cofre. Desbloqueie-o para continuar.".into());
        }
        validar_senha_mestra(&senha_mestra)?;
        let dados = DadosCofre {
            versao: VERSAO,
            credenciais: Vec::new(),
        };
        let (envelope, chave) = cifrar(&dados, &senha_mestra)?;
        let sal = Zeroizing::new(envelope.sal.clone());
        gravar_envelope(&self.caminho, &envelope)?;
        *self
            .aberto
            .lock()
            .map_err(|_| "O cofre está temporariamente indisponível.".to_string())? =
            Some(Aberto {
                chave,
                sal,
                dados,
                ultima_atividade: Instant::now(),
            });
        Ok(self.resumo())
    }

    pub fn desbloquear(&self, senha_mestra: String) -> Result<ResumoCofre, String> {
        let senha_mestra = Zeroizing::new(senha_mestra);
        if !self.caminho.is_file() {
            return Err("Crie um cofre antes de desbloqueá-lo.".into());
        }
        let metadata =
            fs::metadata(&self.caminho).map_err(|_| "Não foi possível ler o cofre.".to_string())?;
        if metadata.len() > ARQUIVO_MAX_BYTES {
            return Err("O arquivo do cofre excede o limite seguro.".into());
        }
        let bytes =
            fs::read(&self.caminho).map_err(|_| "Não foi possível ler o cofre.".to_string())?;
        let envelope: Envelope = serde_json::from_slice(&bytes)
            .map_err(|_| "O arquivo do cofre está inválido ou corrompido.".to_string())?;
        let sal = Zeroizing::new(envelope.sal.clone());
        let (dados, chave) = decifrar(envelope, &senha_mestra)?;
        if dados.versao != VERSAO || dados.credenciais.len() > 10_000 {
            return Err("O formato do cofre não é compatível.".into());
        }
        *self
            .aberto
            .lock()
            .map_err(|_| "O cofre está temporariamente indisponível.".to_string())? =
            Some(Aberto {
                chave,
                sal,
                dados,
                ultima_atividade: Instant::now(),
            });
        Ok(self.resumo())
    }

    pub fn travar(&self) -> Result<ResumoCofre, String> {
        *self
            .aberto
            .lock()
            .map_err(|_| "O cofre está temporariamente indisponível.".to_string())? = None;
        Ok(self.resumo())
    }

    pub fn listar(&self) -> Result<Vec<ResumoCredencial>, String> {
        self.com_dados(|aberto| Ok(aberto.dados.credenciais.iter().map(resumir).collect()))
    }

    pub fn metadado(&self, id: &str) -> Result<ResumoCredencial, String> {
        self.com_dados(|aberto| {
            aberto
                .dados
                .credenciais
                .iter()
                .find(|item| item.id == id)
                .map(resumir)
                .ok_or_else(|| "A credencial não foi encontrada.".into())
        })
    }

    pub fn salvar(
        &self,
        id: Option<&str>,
        endereco: &str,
        titulo: &str,
        usuario: &str,
        senha: String,
    ) -> Result<ResumoCredencial, String> {
        let senha = Zeroizing::new(senha);
        let origem = normalizar_origem(endereco)?;
        let titulo = limitar_texto(titulo, 120, "nome do site")?;
        let usuario = limitar_texto(usuario, 512, "usuário")?;
        if senha.is_empty() || senha.len() > 4096 {
            return Err("A senha precisa ter entre 1 e 4.096 bytes.".into());
        }
        self.alterar(|dados| {
            let agora = agora();
            if let Some(id) = id {
                if dados.credenciais.iter().any(|item| item.id != id && item.origem == origem && item.usuario == usuario) {
                    return Err("Já existe outra senha para esse site e usuário.".into());
                }
                let credencial = dados.credenciais.iter_mut().find(|item| item.id == id).ok_or_else(|| "A credencial não foi encontrada.".to_string())?;
                if credencial.origem != origem { return Err("A edição mantém o site original. Remova e crie outra credencial para mudar de site.".into()); }
                credencial.titulo = titulo.clone();
                credencial.usuario = usuario.clone();
                credencial.senha = senha.to_string();
                credencial.atualizado_em = agora;
                Ok(credencial.id.clone())
            } else {
                if dados.credenciais.iter().any(|item| item.origem == origem && item.usuario == usuario) { return Err("Já existe uma senha para esse site e usuário.".into()); }
                let id = novo_id();
                dados.credenciais.push(Credencial { id: id.clone(), origem: origem.clone(), titulo: titulo.clone(), usuario: usuario.clone(), senha: senha.to_string(), criado_em: agora, atualizado_em: agora });
                Ok(id)
            }
        })?;
        let id_salvo = self.com_dados(|aberto| {
            Ok(aberto
                .dados
                .credenciais
                .iter()
                .find(|item| item.origem == origem && item.usuario == usuario)
                .map(|item| item.id.clone())
                .unwrap_or_default())
        })?;
        self.metadado(&id_salvo)
    }

    pub fn remover(&self, id: &str) -> Result<(ResumoCredencial, bool), String> {
        let mut removido = None;
        self.alterar(|dados| {
            let indice = dados
                .credenciais
                .iter()
                .position(|item| item.id == id)
                .ok_or_else(|| "A credencial não foi encontrada.".to_string())?;
            removido = Some(dados.credenciais.remove(indice));
            Ok(())
        })?;
        let credencial = removido.ok_or_else(|| "A credencial não foi encontrada.".to_string())?;
        let origem = credencial.origem.clone();
        let restante = self.com_dados(|aberto| {
            Ok(aberto
                .dados
                .credenciais
                .iter()
                .any(|item| item.origem == origem))
        })?;
        let resultado = resumir(&credencial);
        drop(credencial);
        Ok((resultado, !restante))
    }

    pub fn senha_para_copiar(&self, id: &str) -> Result<Zeroizing<String>, String> {
        self.com_dados(|aberto| {
            aberto
                .dados
                .credenciais
                .iter()
                .find(|item| item.id == id)
                .map(|item| Zeroizing::new(item.senha.clone()))
                .ok_or_else(|| "A credencial não foi encontrada.".into())
        })
    }

    fn com_dados<R>(
        &self,
        operacao: impl FnOnce(&mut Aberto) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut guard = self
            .aberto
            .lock()
            .map_err(|_| "O cofre está temporariamente indisponível.".to_string())?;
        let aberto = guard
            .as_mut()
            .ok_or_else(|| "Desbloqueie o cofre para continuar.".to_string())?;
        if aberto.ultima_atividade.elapsed() >= TEMPO_DESBLOQUEADO {
            *guard = None;
            return Err("O cofre foi bloqueado por inatividade. Desbloqueie-o novamente.".into());
        }
        aberto.ultima_atividade = Instant::now();
        operacao(aberto)
    }

    fn alterar<R>(
        &self,
        operacao: impl FnOnce(&mut DadosCofre) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut guard = self
            .aberto
            .lock()
            .map_err(|_| "O cofre está temporariamente indisponível.".to_string())?;
        let aberto = guard
            .as_mut()
            .ok_or_else(|| "Desbloqueie o cofre para continuar.".to_string())?;
        if aberto.ultima_atividade.elapsed() >= TEMPO_DESBLOQUEADO {
            *guard = None;
            return Err("O cofre foi bloqueado por inatividade. Desbloqueie-o novamente.".into());
        }
        let mut novos_dados = aberto.dados.clone();
        let valor = operacao(&mut novos_dados)?;
        let envelope = cifrar_com_chave(&novos_dados, &aberto.chave, &aberto.sal)?;
        gravar_envelope(&self.caminho, &envelope)?;
        aberto.dados = novos_dados;
        aberto.ultima_atividade = Instant::now();
        Ok(valor)
    }
}

fn resumir(item: &Credencial) -> ResumoCredencial {
    ResumoCredencial {
        id: item.id.clone(),
        origem: item.origem.clone(),
        dominio: Url::parse(&item.origem)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string))
            .unwrap_or_default(),
        titulo: item.titulo.clone(),
        usuario: item.usuario.clone(),
        criado_em: item.criado_em,
        atualizado_em: item.atualizado_em,
    }
}

fn limitar_texto(valor: &str, maximo: usize, nome: &str) -> Result<String, String> {
    let valor = valor.trim();
    if valor.is_empty() || valor.len() > maximo || valor.chars().any(char::is_control) {
        return Err(format!(
            "O campo {nome} está vazio ou excede o limite permitido."
        ));
    }
    Ok(valor.to_string())
}

pub fn normalizar_origem(endereco: &str) -> Result<String, String> {
    if endereco.len() > 2_048 {
        return Err("O endereço do site excede o limite permitido.".into());
    }
    let url = Url::parse(endereco).map_err(|_| "Informe um endereço HTTPS válido.".to_string())?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(
            "Senhas só podem ser associadas a sites HTTPS sem credenciais no endereço.".into(),
        );
    }
    Ok(url.origin().ascii_serialization())
}

fn validar_senha_mestra(senha: &str) -> Result<(), String> {
    if senha.chars().count() < 12 || senha.len() > 1_024 {
        return Err(
            "Use uma senha mestra com pelo menos 12 caracteres e no máximo 1.024 bytes.".into(),
        );
    }
    Ok(())
}

fn agora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|tempo| tempo.as_secs())
        .unwrap_or_default()
}

fn novo_id() -> String {
    let mut bytes = [0_u8; 16];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parametros_argon() -> Result<Params, String> {
    Params::new(19 * 1024, 2, 1, Some(CHAVE_BYTES))
        .map_err(|_| "Não foi possível preparar a criptografia do cofre.".into())
}

fn cifrar(
    dados: &DadosCofre,
    senha: &str,
) -> Result<(Envelope, Zeroizing<[u8; CHAVE_BYTES]>), String> {
    let mut sal = [0_u8; SALT_BYTES];
    OsRng.fill_bytes(&mut sal);
    let mut chave = Zeroizing::new([0_u8; CHAVE_BYTES]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, parametros_argon()?)
        .hash_password_into(senha.as_bytes(), &sal, chave.as_mut())
        .map_err(|_| "Não foi possível derivar a chave do cofre.".to_string())?;
    Ok((cifrar_com_chave_salt(dados, &chave, sal.to_vec())?, chave))
}

fn cifrar_com_chave(
    dados: &DadosCofre,
    chave: &[u8; CHAVE_BYTES],
    sal: &[u8],
) -> Result<Envelope, String> {
    cifrar_com_chave_salt(dados, chave, sal.to_vec())
}

fn cifrar_com_chave_salt(
    dados: &DadosCofre,
    chave: &[u8; CHAVE_BYTES],
    sal: Vec<u8>,
) -> Result<Envelope, String> {
    let mut nonce = [0_u8; NONCE_BYTES];
    OsRng.fill_bytes(&mut nonce);
    let cifra = XChaCha20Poly1305::new_from_slice(chave)
        .map_err(|_| "Não foi possível preparar a criptografia do cofre.".to_string())?;
    let texto = Zeroizing::new(
        serde_json::to_vec(dados)
            .map_err(|_| "Não foi possível preparar os dados do cofre.".to_string())?,
    );
    let cifra = cifra
        .encrypt(XNonce::from_slice(&nonce), texto.as_slice())
        .map_err(|_| "Não foi possível proteger os dados do cofre.".to_string())?;
    Ok(Envelope {
        versao: VERSAO,
        sal,
        nonce: nonce.to_vec(),
        cifra,
    })
}

fn decifrar(
    envelope: Envelope,
    senha: &str,
) -> Result<(DadosCofre, Zeroizing<[u8; CHAVE_BYTES]>), String> {
    if envelope.versao != VERSAO
        || envelope.sal.len() != SALT_BYTES
        || envelope.nonce.len() != NONCE_BYTES
        || envelope.cifra.len() > ARQUIVO_MAX_BYTES as usize
    {
        return Err("O cofre não pôde ser desbloqueado. Confira a senha mestra ou a integridade do arquivo.".into());
    }
    let mut chave = Zeroizing::new([0_u8; CHAVE_BYTES]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, parametros_argon()?)
        .hash_password_into(senha.as_bytes(), &envelope.sal, chave.as_mut())
        .map_err(|_| {
            "O cofre não pôde ser desbloqueado. Confira a senha mestra ou a integridade do arquivo."
                .to_string()
        })?;
    let cipher = XChaCha20Poly1305::new_from_slice(chave.as_ref())
        .map_err(|_| "O cofre não pôde ser desbloqueado.".to_string())?;
    let texto = Zeroizing::new(cipher.decrypt(XNonce::from_slice(&envelope.nonce), envelope.cifra.as_ref())
        .map_err(|_| "O cofre não pôde ser desbloqueado. Confira a senha mestra ou a integridade do arquivo.".to_string())?);
    let dados = serde_json::from_slice(&texto).map_err(|_| {
        "O cofre não pôde ser desbloqueado. Confira a senha mestra ou a integridade do arquivo."
            .to_string()
    })?;
    Ok((dados, chave))
}

fn gravar_envelope(caminho: &Path, envelope: &Envelope) -> Result<(), String> {
    let diretorio = caminho
        .parent()
        .ok_or_else(|| "A pasta do cofre é inválida.".to_string())?;
    fs::create_dir_all(diretorio)
        .map_err(|_| "Não foi possível preparar a pasta do cofre.".to_string())?;
    let bytes = serde_json::to_vec(envelope)
        .map_err(|_| "Não foi possível preparar o arquivo cifrado.".to_string())?;
    if bytes.len() as u64 > ARQUIVO_MAX_BYTES {
        return Err("O cofre excede o limite de armazenamento seguro.".into());
    }
    let mut sufixo = [0_u8; 8];
    OsRng.fill_bytes(&mut sufixo);
    let nome_temp = format!(
        ".canoa-cofre-{}-{}.tmp",
        std::process::id(),
        sufixo
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    let temporario = diretorio.join(nome_temp);
    let resultado = (|| {
        let mut arquivo = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporario)
            .map_err(|_| "Não foi possível criar o arquivo temporário do cofre.".to_string())?;
        arquivo
            .write_all(&bytes)
            .map_err(|_| "Não foi possível salvar o cofre.".to_string())?;
        arquivo
            .sync_all()
            .map_err(|_| "Não foi possível finalizar a gravação do cofre.".to_string())?;
        fs::rename(&temporario, caminho)
            .map_err(|_| "Não foi possível ativar o arquivo atualizado do cofre.".to_string())
    })();
    if resultado.is_err() {
        let _ = fs::remove_file(temporario);
    }
    resultado
}

#[cfg(test)]
mod tests {
    use super::{normalizar_origem, Cofre};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn caminho_teste() -> PathBuf {
        PathBuf::from(std::env::temp_dir()).join(format!(
            "canoa-cofre-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn cofre_cifra_persiste_bloqueia_e_rejeita_senha_incorreta() {
        let caminho = caminho_teste();
        let cofre = Cofre::novo(caminho.clone());
        cofre.criar("frase mestra para teste 123".into()).unwrap();
        let credencial = cofre
            .salvar(
                None,
                "https://example.test/login",
                "Exemplo",
                "teste",
                "segredo sintético".into(),
            )
            .unwrap();
        assert!(cofre
            .salvar(
                None,
                "https://example.test/outro",
                "Exemplo",
                "teste",
                "outro segredo sintético".into()
            )
            .is_err());
        cofre
            .salvar(
                Some(&credencial.id),
                "https://example.test",
                "Exemplo",
                "teste editado",
                "senha nova sintética".into(),
            )
            .unwrap();
        let arquivo = fs::read_to_string(&caminho).unwrap();
        assert!(!arquivo.contains("segredo sintético"));
        assert!(!arquivo.contains("senha nova sintética"));
        cofre.travar().unwrap();
        assert!(cofre
            .desbloquear("senha incorreta de teste".into())
            .is_err());
        let reiniciado = Cofre::novo(caminho.clone());
        reiniciado
            .desbloquear("frase mestra para teste 123".into())
            .unwrap();
        let lista = reiniciado.listar().unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].usuario, "teste editado");
        assert_eq!(
            reiniciado.senha_para_copiar(&lista[0].id).unwrap().as_str(),
            "senha nova sintética"
        );
        let _ = fs::remove_file(caminho);
    }

    #[test]
    fn endereco_exige_https_e_descarta_caminho() {
        assert_eq!(
            normalizar_origem("https://example.com/login?utm=1").unwrap(),
            "https://example.com"
        );
        assert!(normalizar_origem("http://example.com").is_err());
        assert!(normalizar_origem("https://usuario:senha@example.com").is_err());
    }
}
