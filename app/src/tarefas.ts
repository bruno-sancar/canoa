import { invoke } from "@tauri-apps/api/core";

type Aba = { id: number; endereco: string | null; titulo: string };
type Retrato = { abas: Aba[]; ativa: number };
type Memoria = { privado_bytes: number; trabalho_bytes: number; processos: number; instancias: number };
type Processo = { id: number; nome: string; categoria: string; privado_bytes: number; trabalho_bytes: number };
type Diagnostico = { memoria: Memoria; processos: Processo[] };

const pegar = (seletor: string) => document.querySelector<HTMLElement>(seletor)!;
const mebibytes = (bytes: number) => `${(bytes / 1048576).toLocaleString("pt-BR", { maximumFractionDigits: 1 })} MiB`;
let medindo = false;

function desenharAbas(retrato: Retrato) {
  pegar("#total-abas").textContent = `${retrato.abas.length} ${retrato.abas.length === 1 ? "aba" : "abas"}`;
  pegar("#total-pausadas").textContent = String(retrato.abas.filter((aba) => aba.id !== retrato.ativa).length);
  const lista = pegar("#lista-abas");
  lista.replaceChildren();
  for (const aba of retrato.abas) {
    const linha = document.createElement("div");
    linha.className = "tarefa-aba";
    const descricao = document.createElement("div");
    descricao.className = "tarefa-aba-descricao";
    const titulo = document.createElement("strong");
    titulo.textContent = aba.titulo;
    const endereco = document.createElement("small");
    endereco.textContent = aba.endereco ?? "Sem página carregada";
    const estado = document.createElement("span");
    estado.className = "tarefa-estado";
    estado.textContent = aba.id === retrato.ativa ? "Selecionada" : aba.endereco === null ? "Nova aba" : "Pausada";
    const fechar = document.createElement("button");
    fechar.type = "button";
    fechar.textContent = "Fechar";
    fechar.setAttribute("aria-label", `Fechar aba ${aba.titulo}`);
    fechar.addEventListener("click", async () => {
      fechar.disabled = true;
      try {
        const proximo = await invoke<Retrato>("fechar_aba_do_gerenciador", { id: aba.id });
        desenharAbas(proximo);
        void atualizar();
      } catch (erro) {
        pegar("#mensagem").textContent = `Não foi possível fechar a aba: ${String(erro)}`;
        fechar.disabled = false;
      }
    });
    descricao.append(titulo, endereco);
    linha.append(descricao, estado, fechar);
    lista.append(linha);
  }
}

function desenharProcessos(diagnostico: Diagnostico) {
  pegar("#total-privado").textContent = mebibytes(diagnostico.memoria.privado_bytes);
  pegar("#total-trabalho").textContent = mebibytes(diagnostico.memoria.trabalho_bytes);
  pegar("#total-processos").textContent = String(diagnostico.memoria.processos);
  const lista = pegar("#lista-processos");
  lista.replaceChildren();
  for (const processo of diagnostico.processos) {
    const linha = document.createElement("tr");
    const nome = document.createElement("td");
    const principal = document.createElement("strong");
    principal.textContent = processo.categoria;
    principal.title = processo.categoria === "Aplicativo Canoa" ? "Janela e comandos do aplicativo." : processo.categoria === "Motor WebView2" ? "Processo do motor de páginas da Microsoft. A função individual ainda não foi identificada." : "Processo associado ao aplicativo; função individual não identificada.";
    const arquivo = document.createElement("small");
    arquivo.textContent = processo.nome;
    arquivo.title = `Arquivo executável: ${processo.nome}`;
    nome.append(principal, arquivo);
    const id = document.createElement("td");
    id.textContent = String(processo.id);
    id.title = "PID: identificador temporário do processo nesta execução.";
    const privado = document.createElement("td");
    privado.textContent = mebibytes(processo.privado_bytes);
    privado.title = "Memória privada comprometida; não equivale à RAM física exclusiva.";
    const trabalho = document.createElement("td");
    trabalho.textContent = mebibytes(processo.trabalho_bytes);
    trabalho.title = "Conjunto de trabalho residente em RAM; pode incluir páginas compartilhadas.";
    linha.append(nome, id, privado, trabalho);
    lista.append(linha);
  }
  pegar("#atualizado-em").textContent = `Atualizado às ${new Date().toLocaleTimeString("pt-BR")}`;
}

async function atualizar() {
  if (medindo || document.visibilityState === "hidden") return;
  medindo = true;
  try {
    const [retrato, diagnostico] = await Promise.all([
      invoke<Retrato>("listar_abas"),
      invoke<Diagnostico>("diagnostico_atual"),
    ]);
    desenharAbas(retrato);
    desenharProcessos(diagnostico);
    pegar("#mensagem").textContent = diagnostico.memoria.instancias > 1
      ? `Total inclui ${diagnostico.memoria.instancias} instâncias abertas do Canoa.`
      : "";
  } catch (erro) {
    pegar("#mensagem").textContent = `Medição indisponível: ${String(erro)}`;
  } finally {
    medindo = false;
  }
}

pegar("#atualizar").addEventListener("click", () => void atualizar());
document.addEventListener("visibilitychange", () => { if (document.visibilityState === "visible") void atualizar(); });
void atualizar();
window.setInterval(() => void atualizar(), 5000);
