use super::{Diagnostico, Memoria, ProcessoMedido};
use std::collections::HashSet;
use std::mem::size_of;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

pub fn medir() -> Result<Memoria, String> {
    Ok(diagnosticar()?.memoria)
}

pub fn diagnosticar() -> Result<Diagnostico, String> {
    let caminho_atual = std::env::current_exe()
        .map_err(|erro| format!("Não foi possível identificar o Canoa: {erro}"))?
        .to_string_lossy()
        .to_lowercase();
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err("Não foi possível listar os processos.".into());
    }
    let snapshot = Handle(snapshot);
    let mut entrada = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut arvore = Vec::new();
    if unsafe { Process32FirstW(snapshot.0, &mut entrada) } == 0 {
        return Err("Não foi possível ler os processos.".into());
    }
    loop {
        let fim = entrada
            .szExeFile
            .iter()
            .position(|&caractere| caractere == 0)
            .unwrap_or(entrada.szExeFile.len());
        let nome = String::from_utf16_lossy(&entrada.szExeFile[..fim]).to_lowercase();
        arvore.push((entrada.th32ProcessID, entrada.th32ParentProcessID, nome));
        if unsafe { Process32NextW(snapshot.0, &mut entrada) } == 0 {
            break;
        }
    }

    let mut ids = HashSet::from([unsafe { GetCurrentProcessId() }]);
    for &(pid, _, ref nome) in &arvore {
        if !(nome == "canoa.exe" || (nome.starts_with("canoa-") && nome.ends_with(".exe")))
            || ids.contains(&pid)
        {
            continue;
        }
        let processo = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if processo.is_null() {
            continue;
        }
        let processo = Handle(processo);
        let mut caminho = vec![0u16; 32768];
        let mut tamanho = caminho.len() as u32;
        if unsafe { QueryFullProcessImageNameW(processo.0, 0, caminho.as_mut_ptr(), &mut tamanho) }
            != 0
        {
            let nome_completo =
                String::from_utf16_lossy(&caminho[..tamanho as usize]).to_lowercase();
            if nome_completo == caminho_atual || nome_completo.contains("\\canoa\\") {
                ids.insert(pid);
            }
        }
    }
    let instancias = ids.len();
    loop {
        let antes = ids.len();
        for &(pid, pai, _) in &arvore {
            if ids.contains(&pai) {
                ids.insert(pid);
            }
        }
        if ids.len() == antes {
            break;
        }
    }

    let mut resultado = Memoria {
        privado_bytes: 0,
        trabalho_bytes: 0,
        processos: 0,
        instancias,
    };
    let mut processos = Vec::new();
    for pid in ids {
        let processo = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if processo.is_null() {
            continue; // O processo pode ter terminado após o instantâneo.
        }
        let processo = Handle(processo);
        let mut contadores = PROCESS_MEMORY_COUNTERS_EX {
            cb: size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            ..Default::default()
        };
        let sucesso = unsafe {
            GetProcessMemoryInfo(
                processo.0,
                (&mut contadores as *mut PROCESS_MEMORY_COUNTERS_EX)
                    .cast::<PROCESS_MEMORY_COUNTERS>(),
                size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            )
        };
        if sucesso != 0 {
            let nome = arvore
                .iter()
                .find(|(id, _, _)| *id == pid)
                .map(|(_, _, nome)| nome.clone())
                .unwrap_or_else(|| "processo".into());
            let categoria = if nome.starts_with("canoa") {
                "Aplicativo Canoa"
            } else if nome.contains("webview") || nome.contains("msedge") {
                "Motor WebView2"
            } else {
                "Processo auxiliar"
            };
            resultado.privado_bytes = resultado
                .privado_bytes
                .saturating_add(contadores.PrivateUsage as u64);
            resultado.trabalho_bytes = resultado
                .trabalho_bytes
                .saturating_add(contadores.WorkingSetSize as u64);
            resultado.processos += 1;
            processos.push(ProcessoMedido {
                id: pid,
                nome,
                categoria: categoria.into(),
                privado_bytes: contadores.PrivateUsage as u64,
                trabalho_bytes: contadores.WorkingSetSize as u64,
            });
        }
    }
    if resultado.processos == 0 {
        return Err("Não foi possível medir os processos do Canoa.".into());
    }
    processos.sort_by(|a, b| b.privado_bytes.cmp(&a.privado_bytes));
    Ok(Diagnostico {
        memoria: resultado,
        processos,
    })
}
