import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type Aba = { id: number; endereco: string | null; titulo: string; grupo: string | null; mutada: boolean };
type Retrato = { abas: Aba[]; ativa: number };
type Memoria = { privado_bytes: number; trabalho_bytes: number; processos: number; instancias: number };
type DownloadFinalizado = { id: string; nome: string; caminho: string | null; sucesso: boolean; tipo: string };
type DownloadInfo = DownloadFinalizado & { registrado_em: number };
type DownloadAtivo = { id: string; nome: string; bytes_recebidos: number; total_bytes: number | null; cancelavel: boolean; tipo: string };
type RecursoPagina = { endereco: string; tipo: "imagem" | "video" | "audio" | "documento"; nome: string };
type PacoteRecursosPagina = { endereco: string; titulo: string; itens: RecursoPagina[]; instante: number };
type Buscador = "duckduckgo" | "brave";
type Fonte = { dominio: string; tema: string };
type Favorito = { endereco: string; titulo: string; pastas: string[]; marcadores: string[] };
type ResultadoImportacao = { configuracoes: Configuracoes; adicionados: number; ignorados: number };
type ItemLeitura = { endereco: string; titulo: string; adicionado_em: number; lido: boolean };
type ItemSalvo = { endereco: string; titulo: string; tipo: string; salvo_em: number; caminho: string | null };
type AcoesBarra = { favorito: boolean; protecao: boolean; leitura: boolean; baixar: boolean };
type GrupoAba = { nome: string; recolhido: boolean };
type EntradaHistorico = { endereco: string; titulo: string; visitado_em: number };
type LimiteSite = { dominio: string; minutos_diarios: number };
type UsoSite = { dominio: string; dia: string; segundos: number };
type Configuracoes = { mostrar_memoria: boolean; bloqueador_ativo: boolean; excecoes_bloqueador: string[]; sites_bloqueados: string[]; limpar_rastreamento: boolean; historico_downloads: DownloadInfo[]; pasta_downloads: string | null; buscador: Buscador; fontes: Fonte[]; favoritos: Favorito[]; lista_leitura: ItemLeitura[]; salvos: ItemSalvo[]; acoes_barra: AcoesBarra; grupos_abas: GrupoAba[]; historico_ativo: boolean; dias_historico: number; historico_navegacao: EntradaHistorico[]; limites_sites: LimiteSite[]; uso_diario_sites: UsoSite[]; sites_foco: string[]; foco_ate_epoch: number | null; duracao_foco_minutos: number };
type EstadoUsoSite = { dominio: string; segundos: number; limite_segundos: number; atingido: boolean };
type ResultadoLimpeza = { sucesso: boolean; mensagem: string };
type ResumoProtecao = { pagina: string; bloqueados_pagina: number; total_sessao: number; categorias: Record<string, number>; listas_versao: string; listas_atualizadas_em: number | null; easylist_sha256: string; easyprivacy_sha256: string; tem_versao_anterior: boolean };
type InformacoesVersao = { versao: string; compilado_em_epoch: number | null };
type Armazenamento = { executavel_bytes: number; dados_locais_bytes: number; cache_identificado_bytes: number; configuracoes_bytes: number; incompleto: boolean };
type ContextoMenuPagina = { x: number; y: number; endereco: string; titulo: string; link: string | null; imagem: string | null; texto: string };
type AcessoArtigo = { endereco: string; titulo: string; restrito: boolean; sinais: string[]; links: string[] };
type ResumoCofre = { existe: boolean; desbloqueado: boolean; quantidade: number };
type CredencialResumo = { id: string; origem: string; dominio: string; titulo: string; usuario: string; criado_em: number; atualizado_em: number };

const aviso = document.querySelector<HTMLElement>("#aviso")!;
const avisoTexto = document.querySelector<HTMLElement>("#aviso-texto")!;
const avisoIcone = document.querySelector<HTMLElement>("#aviso-icone")!;
const avisoAcao = document.querySelector<HTMLButtonElement>("#aviso-acao")!;
const endereco = document.querySelector<HTMLInputElement>("#endereco")!;
const sugestoes = document.querySelector<HTMLElement>("#sugestoes")!;
const formulario = document.querySelector<HTMLFormElement>("#navegacao")!;
const lista = document.querySelector<HTMLElement>("#abas")!;
const nova = document.querySelector<HTMLButtonElement>("#nova-aba")!;
const botaoFavorito = document.querySelector<HTMLButtonElement>("#favorito-atual")!;
const botaoProtecaoSite = document.querySelector<HTMLButtonElement>("#protecao-site")!;
const botaoSalvarLeitura = document.querySelector<HTMLButtonElement>("#salvar-leitura")!;
const botaoBaixarItens = document.querySelector<HTMLButtonElement>("#baixar-itens")!;
const listaFavoritos = document.querySelector<HTMLUListElement>("#favoritos-lista")!;
const importarFavoritos = document.querySelector<HTMLButtonElement>("#importar-favoritos")!;
const exportarFavoritos = document.querySelector<HTMLButtonElement>("#exportar-favoritos")!;
const exportarHtml = document.querySelector<HTMLButtonElement>("#exportar-html")!;
const dialogoEditarFavorito = document.querySelector<HTMLDialogElement>("#editar-favorito")!;
const formularioEditarFavorito = document.querySelector<HTMLFormElement>("#form-editar-favorito")!;
const tituloEditarFavorito = document.querySelector<HTMLInputElement>("#editar-favorito-titulo")!;
const pastasEditarFavorito = document.querySelector<HTMLInputElement>("#editar-favorito-pastas")!;
const menuFavoritos = document.querySelector<HTMLElement>("#menu-favoritos")!;
const botaoMenuFavoritos = document.querySelector<HTMLButtonElement>("#abrir-menu-favoritos")!;
const botaoBuscaFavoritos = document.querySelector<HTMLButtonElement>("#alternar-busca-favoritos")!;
const buscaFavoritos = document.querySelector<HTMLInputElement>("#buscar-favoritos")!;
const indicador = document.querySelector<HTMLButtonElement>("#memoria")!;
const botaoConfiguracoes = document.querySelector<HTMLButtonElement>("#abrir-configuracoes")!;
const menuPrincipal = document.querySelector<HTMLElement>("#menu-principal")!;
const menuMemoria = document.querySelector<HTMLButtonElement>("#menu-memoria")!;
const painelConfiguracoes = document.querySelector<HTMLElement>("#configuracoes")!;
const controleMemoria = document.querySelector<HTMLInputElement>("#config-mostrar-memoria")!;
const controleBuscador = document.querySelector<HTMLSelectElement>("#config-buscador")!;
const controleBloqueador = document.querySelector<HTMLInputElement>("#config-bloqueador-ativo")!;
const listaExcecoesBloqueador = document.querySelector<HTMLUListElement>("#lista-excecoes-bloqueador")!;
const excecoesVazias = document.querySelector<HTMLElement>("#excecoes-bloqueador-vazias")!;
const controleLimpezaRastreamento = document.querySelector<HTMLInputElement>("#config-limpar-rastreamento")!;
const listaSitesBloqueados = document.querySelector<HTMLUListElement>("#lista-sites-bloqueados")!;
const sitesBloqueadosVazios = document.querySelector<HTMLElement>("#sites-bloqueados-vazios")!;
const controleHistorico = document.querySelector<HTMLInputElement>("#config-historico-ativo")!;
const diasHistorico = document.querySelector<HTMLSelectElement>("#config-historico-dias")!;
const buscaHistorico = document.querySelector<HTMLInputElement>("#buscar-historico")!;
const listaHistoricoNavegacao = document.querySelector<HTMLUListElement>("#historico-navegacao")!;
const historicoVazio = document.querySelector<HTMLElement>("#historico-vazio")!;
const formularioLimiteSite = document.querySelector<HTMLFormElement>("#form-limite-site")!;
const limitesSitesEl = document.querySelector<HTMLUListElement>("#limites-sites")!;
const limitesSitesVazios = document.querySelector<HTMLElement>("#limites-sites-vazios")!;
const formularioSiteFoco = document.querySelector<HTMLFormElement>("#form-site-foco")!;
const sitesFocoEl = document.querySelector<HTMLUListElement>("#sites-foco")!;
const sitesFocoVazios = document.querySelector<HTMLElement>("#sites-foco-vazios")!;
const duracaoFoco = document.querySelector<HTMLSelectElement>("#duracao-foco")!;
const estadoFoco = document.querySelector<HTMLElement>("#estado-foco")!;
const statusLimpezaDados = document.querySelector<HTMLElement>("#limpeza-dados-status")!;
const bloqueiosPagina = document.querySelector<HTMLElement>("#bloqueios-pagina")!;
const bloqueiosSessao = document.querySelector<HTMLElement>("#bloqueios-sessao")!;
const bloqueiosCategorias = document.querySelector<HTMLElement>("#bloqueios-categorias")!;
const statusListasBloqueador = document.querySelector<HTMLElement>("#status-listas-bloqueador")!;
const botaoAtualizarListas = document.querySelector<HTMLButtonElement>("#atualizar-listas-bloqueador")!;
const formCriarCofre = document.querySelector<HTMLFormElement>("#form-criar-cofre")!;
const formDesbloquearCofre = document.querySelector<HTMLFormElement>("#form-desbloquear-cofre")!;
const cofreFechado = document.querySelector<HTMLElement>("#cofre-fechado")!;
const cofreAberto = document.querySelector<HTMLElement>("#cofre-aberto")!;
const statusCofre = document.querySelector<HTMLElement>("#status-cofre")!;
const formularioCredencial = document.querySelector<HTMLFormElement>("#form-credencial")!;
const listaSenhas = document.querySelector<HTMLUListElement>("#senhas-salvas")!;
const vaziasSenhas = document.querySelector<HTMLElement>("#senhas-vazias")!;
const enderecoSenha = document.querySelector<HTMLInputElement>("#senha-site")!;
const tituloSenha = document.querySelector<HTMLInputElement>("#senha-titulo")!;
const usuarioSenha = document.querySelector<HTMLInputElement>("#senha-usuario")!;
const valorSenha = document.querySelector<HTMLInputElement>("#senha-valor")!;
const cancelarEdicaoSenha = document.querySelector<HTMLButtonElement>("#cancelar-edicao-credencial")!;
let idCredencialEmEdicao: string | null = null;
const botaoReverterListas = document.querySelector<HTMLButtonElement>("#reverter-listas-bloqueador")!;
const fontesSalvas = document.querySelector<HTMLUListElement>("#fontes-salvas")!;
const caminhoDownloads = document.querySelector<HTMLElement>("#config-pasta")!;
const listaDownloadsRecentes = document.querySelector<HTMLUListElement>("#downloads-recentes")!;
const listaDownloadsAtivos = document.querySelector<HTMLUListElement>("#downloads-ativos")!;
const valorMemoria = document.querySelector<HTMLElement>("#config-memoria-valor")!;
const listaLeituraEl = document.querySelector<HTMLUListElement>("#lista-leitura")!;
const listaSalvosEl = document.querySelector<HTMLUListElement>("#lista-salvos")!;
const listaFontesBiblioteca = document.querySelector<HTMLUListElement>("#fontes-biblioteca")!;
const painelFavoritos = document.querySelector<HTMLElement>("#painel-favoritos")!;
const painelLeitura = document.querySelector<HTMLElement>("#painel-leitura")!;
const marcadorFavoritos = document.createComment("posição original dos favoritos");
const marcadorLeitura = document.createComment("posição original da lista de leitura");
painelFavoritos.parentNode?.insertBefore(marcadorFavoritos, painelFavoritos);
painelLeitura.parentNode?.insertBefore(marcadorLeitura, painelLeitura);
let medindo = false;
let ultimaMedicao = 0;
const downloadsAtivos = new Map<string, DownloadAtivo>();
let downloadsRecentes: DownloadInfo[] = [];
let recursosPagina: RecursoPagina[] = [];
let tituloRecursosPagina = "pagina";
let filtroTipoRecursos = "todos";
let avisoTemporizador = 0;
let mostrarMemoria = true;
let configurando = false;
let buscador: Buscador = "duckduckgo";
let fontes: Fonte[] = [];
let favoritos: Favorito[] = [];
let listaLeitura: ItemLeitura[] = [];
let salvos: ItemSalvo[] = [];
let abasAbertas: Aba[] = [];
let retratoAtual: Retrato | null = null;
let gruposAba: GrupoAba[] = [];
let idAbaAnterior: number | null = null;
let enderecoAtual: string | null = null;
let bloqueadorAtivo = true;
let excecoesBloqueador: string[] = [];
let sitesBloqueadosAtuais: string[] = [];
let focoAteEpoch: number | null = null;
let alterandoFavorito = false;
let enderecoFavoritoEmEdicao: string | null = null;
let consultaFavoritos = "";
const filtrosBiblioteca: Record<string, string> = { leitura: "", salvos: "", fontes: "" };
let fontePendente: Fonte | null = null;
let abaContextual: Aba | null = null;
let contextoMenuPagina: ContextoMenuPagina | null = null;
let acessoArtigo: AcessoArtigo | null = null;
const URL_CONFIGURACOES = "canoa://configuracoes";
type Sugestao = { rotulo: string; detalhe: string; executar: () => Promise<void>; apenasSelecionar?: boolean };
let sugestoesAtuais: Sugestao[] = [];
let sugestaoSelecionada = -1;
let filaVisibilidadePagina = Promise.resolve();
let historicoReferencia: EntradaHistorico[] = [];
let credenciaisReferencia: CredencialResumo[] = [];
let cofreReferenciaExiste = false;
let cofreReferenciaDesbloqueado = false;

const categoriasReferencia = [
  { chave: "favoritos", nome: "Favoritos" },
  { chave: "leitura", nome: "Lista de leitura" },
  { chave: "baixados", nome: "Baixados" },
  { chave: "historico", nome: "Histórico" },
  { chave: "senhas", nome: "Senhas" },
  { chave: "fontes", nome: "Fontes" },
] as const;
const categoriasConfiguracao = [
  { secao: "inicio", nome: "Visão geral" },
  { secao: "pesquisa", nome: "Pesquisa e fontes" },
  { secao: "favoritos", nome: "Favoritos" },
  { secao: "leitura", nome: "Lista de leitura" },
  { secao: "barra", nome: "Barra de endereço" },
  { secao: "memoria", nome: "Memória e armazenamento" },
  { secao: "downloads", nome: "Baixados" },
  { secao: "historico", nome: "Histórico" },
  { secao: "foco", nome: "Tempo e foco" },
  { secao: "limpeza", nome: "Cookies e cache" },
  { secao: "privacidade", nome: "Proteção e bloqueios" },
  { secao: "senhas", nome: "Senhas" },
  { secao: "atalhos", nome: "Atalhos" },
  { secao: "sobre", nome: "Sobre o Canoa" },
] as const;

function ocultarSugestoes() {
  sugestoes.hidden = true;
  sugestoes.replaceChildren();
  sugestoesAtuais = [];
  sugestaoSelecionada = -1;
  endereco.setAttribute("aria-expanded", "false");
  endereco.removeAttribute("aria-activedescendant");
  ajustarVisibilidadePagina(true);
}

function ajustarVisibilidadePagina(mostrar: boolean) {
  if (mostrar && !enderecoAtual?.startsWith("http")) return;
  filaVisibilidadePagina = filaVisibilidadePagina.then(async () => {
    try { await invoke("mostrar_conteudo_pagina", { mostrar }); }
    catch { /* Não existe WebView de página na Nova Aba ou em Configurações. */ }
  });
}

async function navegarSugestao(indice: number) {
  const sugestao = sugestoesAtuais[indice];
  if (!sugestao) return;
  ocultarSugestoes();
  await filaVisibilidadePagina;
  try { await sugestao.executar(); if (!sugestao.apenasSelecionar) informar("Carregando…"); }
  catch (erro) { informar(String(erro), true); }
}

function desenharSugestoes() {
  const texto = endereco.value.trim();
  if (configurando) { ocultarSugestoes(); return; }
  const configuracao = texto.match(/^#([^\s]*)(?:\s+(.*))?$/);
  if (configuracao) {
    const termo = (configuracao[1] ?? "").toLocaleLowerCase("pt-BR");
    const encontrados: Sugestao[] = categoriasConfiguracao
      .filter((item) => !termo || item.nome.toLocaleLowerCase("pt-BR").includes(termo) || item.secao.startsWith(termo))
      .map((item) => ({
        rotulo: item.nome,
        detalhe: `Configuração · ${item.secao}`,
        executar: async () => { await abrirConfiguracoes(item.secao); },
      }));
    desenharListaSugestoes(encontrados);
    return;
  }
  const referencia = texto.match(/^@([^\s]*)(?:\s+(.*))?$/);
  if (referencia) {
    const categoriaDigitada = referencia[1].toLocaleLowerCase("pt-BR");
    const termo = (referencia[2] ?? "").trim().toLocaleLowerCase("pt-BR");
    const categoria = categoriasReferencia.find((item) => item.chave === categoriaDigitada || item.nome.toLocaleLowerCase("pt-BR") === categoriaDigitada);
    let encontrados: Sugestao[];
    if (!categoria) {
      encontrados = categoriasReferencia
        .filter((item) => !categoriaDigitada || item.nome.toLocaleLowerCase("pt-BR").includes(categoriaDigitada) || item.chave.startsWith(categoriaDigitada))
        .map((item) => ({
          rotulo: item.nome,
          detalhe: item.chave === "senhas" && cofreReferenciaExiste && !cofreReferenciaDesbloqueado
            ? "Cofre bloqueado · desbloqueie para consultar os sites"
            : `Referência · ${contagemReferencia(item.chave)} item(ns)`,
          apenasSelecionar: true,
          executar: async () => { endereco.value = `@${item.chave} `; desenharSugestoes(); endereco.focus(); },
        }));
    } else {
      if (categoria.chave === "senhas" && cofreReferenciaExiste && !cofreReferenciaDesbloqueado) {
        encontrados = [{ rotulo: "Desbloquear Cofre de senhas", detalhe: "As URLs associadas ficam ocultas enquanto o cofre está bloqueado.", executar: async () => { await abrirConfiguracoes("senhas"); } }];
      } else encontrados = itensReferencia(categoria.chave)
        .filter((item) => !termo || `${item.rotulo} ${item.detalhe} ${item.url}`.toLocaleLowerCase("pt-BR").includes(termo))
        .map((item) => ({
          rotulo: item.rotulo,
          detalhe: item.detalhe,
          executar: async () => {
            if (item.abrirLocal) await invoke("revelar_download", { caminho: item.abrirLocal });
            else endereco.value = await invoke<string>("navegar", { entrada: item.url });
          },
        }));
      if (encontrados.length === 0) encontrados.push({ rotulo: "Nenhum item nesta referência", detalhe: categoria.nome, apenasSelecionar: true, executar: async () => {} });
    }
    desenharListaSugestoes(encontrados);
    return;
  }
  if (texto.length < 2) { ocultarSugestoes(); return; }
  const busca = texto.toLocaleLowerCase("pt-BR");
  const abasUnicas = new Map<string, Aba>();
  for (const aba of abasAbertas) if (aba.endereco && !aba.endereco.startsWith(URL_CONFIGURACOES) && `${aba.titulo} ${aba.endereco}`.toLocaleLowerCase("pt-BR").includes(busca) && !abasUnicas.has(aba.endereco)) abasUnicas.set(aba.endereco, aba);
  const encontrados: Sugestao[] = [...abasUnicas.values()].slice(0, 3).map((aba) => ({
    rotulo: aba.titulo, detalhe: `Aba aberta · ${aba.endereco}`,
    executar: async () => { desenhar(await invoke<Retrato>("trocar_aba", { id: aba.id })); },
  }));
  encontrados.push(...favoritos.filter((item) => `${item.titulo} ${item.endereco}`.toLocaleLowerCase("pt-BR").includes(busca)).slice(0, 3).map((item) => ({
    rotulo: item.titulo, detalhe: `Favorito · ${item.endereco}`,
    executar: async () => { endereco.value = await invoke<string>("navegar", { entrada: item.endereco }); },
  })));
  encontrados.push(...listaLeitura.filter((item) => `${item.titulo} ${item.endereco}`.toLocaleLowerCase("pt-BR").includes(busca)).slice(0, 3).map((item) => ({
    rotulo: item.titulo, detalhe: `Lista de leitura · ${item.lido ? "lido" : "não lido"}`,
    executar: async () => { endereco.value = await invoke<string>("navegar", { entrada: item.endereco }); },
  })));
  encontrados.push(...salvos.filter((item) => `${item.titulo} ${item.endereco} ${item.tipo}`.toLocaleLowerCase("pt-BR").includes(busca)).slice(0, 3).map((item) => ({
    rotulo: item.titulo, detalhe: `Baixado · ${item.tipo}`,
    executar: async () => {
      if (item.caminho) await invoke("revelar_download", { caminho: item.caminho });
      else endereco.value = await invoke<string>("navegar", { entrada: item.endereco });
    },
  })));
  const nomesGrupos = new Set([...gruposAba.map((grupo) => grupo.nome), ...abasAbertas.flatMap((aba) => aba.grupo ? [aba.grupo] : [])]);
  encontrados.push(...[...nomesGrupos].filter((nome) => nome.toLocaleLowerCase("pt-BR").includes(busca)).slice(0, 3).map((nome) => ({
    rotulo: nome, detalhe: `Grupo de abas · ${abasAbertas.filter((aba) => aba.grupo === nome).length} abertas`,
    executar: async () => {
      const membro = abasAbertas.find((aba) => aba.grupo === nome);
      if (membro) { desenhar(await invoke<Retrato>("trocar_aba", { id: membro.id })); return; }
      const novo = await invoke<Retrato>("nova_aba");
      const aba = novo.abas.find((item) => item.id === novo.ativa);
      if (aba) await atribuirGrupo(aba.id, nome);
    },
  })));
  const pareceEnderecoWeb = /^https?:\/\//i.test(texto);
  const pareceConsulta = !pareceEnderecoWeb && (texto.includes(" ") || !texto.includes("."));
  if (pareceConsulta) {
    for (const fonte of fontes.filter((item) => `${item.tema} ${item.dominio}`.toLocaleLowerCase("pt-BR").includes(busca) || texto.includes(" ")).slice(0, 3)) {
      encontrados.push({ rotulo: `Buscar em ${fonte.dominio}`, detalhe: `Fonte · ${fonte.tema}`,
        executar: async () => { endereco.value = await invoke<string>("pesquisar_fonte", { consulta: texto, dominio: fonte.dominio, tema: fonte.tema, filtrar: true }); fontePendente = null; },
      });
    }
  }
  encontrados.push({
    rotulo: pareceEnderecoWeb ? "Abrir endereço" : `Abrir ou pesquisar “${texto}”`,
    detalhe: fontePendente && !pareceEnderecoWeb ? `Pesquisar em ${fontePendente.dominio}` : pareceEnderecoWeb ? "Ir diretamente para esta página" : `Pelo ${buscador === "brave" ? "Brave Search" : "DuckDuckGo"} se não for endereço`,
    executar: async () => {
      if (fontePendente && !pareceEnderecoWeb) {
        endereco.value = await invoke<string>("pesquisar_fonte", { consulta: texto, dominio: fontePendente.dominio, tema: fontePendente.tema, filtrar: true });
        fontePendente = null;
      } else endereco.value = await invoke<string>("navegar", { entrada: texto });
    },
  });
  desenharListaSugestoes(encontrados);
}

function contagemReferencia(chave: typeof categoriasReferencia[number]["chave"]): number {
  switch (chave) {
    case "favoritos": return favoritos.length;
    case "leitura": return listaLeitura.length;
    case "baixados": return salvos.length;
    case "historico": return historicoReferencia.length;
    case "senhas": return credenciaisReferencia.length;
    case "fontes": return fontes.length;
  }
}

function itensReferencia(chave: typeof categoriasReferencia[number]["chave"]): Array<{ rotulo: string; detalhe: string; url: string; abrirLocal?: string }> {
  switch (chave) {
    case "favoritos": return favoritos.map((item) => ({ rotulo: item.titulo || item.endereco, detalhe: `Favorito · ${item.endereco}`, url: item.endereco }));
    case "leitura": return listaLeitura.map((item) => ({ rotulo: item.titulo || item.endereco, detalhe: `Lista de leitura · ${item.lido ? "lido" : "não lido"}`, url: item.endereco }));
    case "baixados": return salvos.map((item) => ({ rotulo: item.titulo || item.endereco, detalhe: `Baixado · ${item.tipo} · ${item.endereco}`, url: item.endereco, ...(item.caminho ? { abrirLocal: item.caminho } : {}) }));
    case "historico": return historicoReferencia.map((item) => ({ rotulo: item.titulo || item.endereco, detalhe: `Histórico · ${new Date(item.visitado_em * 1000).toLocaleString("pt-BR")} · ${item.endereco}`, url: item.endereco }));
    case "senhas": return credenciaisReferencia.map((item) => ({ rotulo: `${item.titulo} · ${item.usuario}`, detalhe: `Senha associada · ${item.dominio} · ${item.origem}`, url: item.origem }));
    case "fontes": return fontes.map((item) => ({ rotulo: `${item.tema} · ${item.dominio}`, detalhe: `Fonte de pesquisa · https://${item.dominio}`, url: `https://${item.dominio}` }));
  }
}

function desenharListaSugestoes(encontrados: Sugestao[]) {
  sugestoesAtuais = encontrados;
  sugestaoSelecionada = -1;
  sugestoes.replaceChildren();
  for (const [indice, item] of encontrados.entries()) {
    const botao = document.createElement("button");
    botao.type = "button";
    botao.id = `sugestao-${indice}`;
    botao.setAttribute("role", "option");
    botao.setAttribute("aria-selected", "false");
    const rotulo = document.createElement("strong"); rotulo.textContent = item.rotulo;
    const detalhe = document.createElement("small"); detalhe.textContent = item.detalhe;
    botao.append(rotulo, detalhe);
    botao.addEventListener("mousedown", (evento) => evento.preventDefault());
    botao.addEventListener("click", () => void navegarSugestao(indice));
    sugestoes.append(botao);
  }
  sugestoes.hidden = encontrados.length === 0;
  if (encontrados.length === 0) { endereco.setAttribute("aria-expanded", "false"); return; }
  endereco.setAttribute("aria-expanded", "true");
  ajustarVisibilidadePagina(false);
}

endereco.addEventListener("input", desenharSugestoes);
// O painel fica na interface principal; um clique em outra área o dispensa.
endereco.addEventListener("blur", () => window.setTimeout(() => {
  if (document.activeElement !== endereco && sugestoesAtuais.length > 0) ocultarSugestoes();
}, 500));
endereco.addEventListener("keydown", (evento) => {
  if (sugestoes.hidden) return;
  if (["ArrowDown", "PageDown", "ArrowUp", "PageUp"].includes(evento.key) && !evento.ctrlKey && !evento.altKey) {
    evento.preventDefault();
    const direcao = evento.key.endsWith("Down") ? 1 : -1;
    sugestaoSelecionada = sugestaoSelecionada < 0
      ? (direcao > 0 ? 0 : sugestoesAtuais.length - 1)
      : (sugestaoSelecionada + direcao + sugestoesAtuais.length) % sugestoesAtuais.length;
    for (const [indice, botao] of [...sugestoes.querySelectorAll<HTMLButtonElement>("button")].entries()) botao.setAttribute("aria-selected", String(indice === sugestaoSelecionada));
    endereco.setAttribute("aria-activedescendant", `sugestao-${sugestaoSelecionada}`);
    sugestoes.querySelector<HTMLButtonElement>(`#sugestao-${sugestaoSelecionada}`)?.scrollIntoView({ block: "nearest" });
  } else if (evento.key === "Enter") {
    if (!sugestoes.hidden && sugestoesAtuais.length > 0) {
      // Enter confirma a opção ativa; sem seleção explícita, comandos @/# escolhem
      // a primeira categoria, enquanto texto/URL abre a ação padrão no fim da lista.
      const indice = sugestaoSelecionada >= 0
        ? sugestaoSelecionada
        : /^[@#]/.test(endereco.value.trim()) ? 0 : sugestoesAtuais.length - 1;
      evento.preventDefault();
      evento.stopPropagation();
      void navegarSugestao(indice);
    } else if (endereco.value.trim()) {
      evento.preventDefault();
      evento.stopPropagation();
      formulario.requestSubmit();
    }
  } else if (evento.key === "Escape") {
    evento.preventDefault(); ocultarSugestoes();
  }
});

function desenharFavoritos() {
  listaFavoritos.replaceChildren();
  const consulta = consultaFavoritos.trim().toLocaleLowerCase("pt-BR");
  const exibidos = favoritos.filter((item) => {
    if (!consulta) return true;
    const dominio = (() => { try { return new URL(item.endereco).host; } catch { return item.endereco; } })();
    return `${item.titulo} ${dominio} ${(item.pastas ?? []).join(" ")} ${(item.marcadores ?? []).join(" ")}`.toLocaleLowerCase("pt-BR").includes(consulta);
  }).sort((a, b) => {
    const pastaA = (a.pastas ?? []).join(" / ");
    const pastaB = (b.pastas ?? []).join(" / ");
    if (!!pastaA !== !!pastaB) return pastaA ? -1 : 1;
    return pastaA.localeCompare(pastaB, "pt-BR") || a.titulo.localeCompare(b.titulo, "pt-BR");
  });
  const vazios = document.querySelector<HTMLElement>("#favoritos-vazios")!;
  vazios.hidden = exibidos.length > 0;
  vazios.textContent = consulta && favoritos.length ? "Nenhum favorito corresponde à pesquisa." : "Guarde uma página com Ctrl+D para encontrá-la aqui.";
  document.querySelector<HTMLElement>("#favoritos-contagem")!.textContent = favoritos.length
    ? (consulta ? `${exibidos.length} de ${favoritos.length}` : String(favoritos.length)) : "";
  document.querySelector<HTMLElement>("#favoritos-aba-contagem")!.textContent = `(${favoritos.length})`;
  exportarFavoritos.disabled = favoritos.length === 0;
  exportarHtml.disabled = favoritos.length === 0;
  const grupos = new Map<string, Favorito[]>();
  for (const item of exibidos) {
    const caminho = JSON.stringify(item.pastas ?? []);
    const grupo = grupos.get(caminho) ?? [];
    grupo.push(item);
    grupos.set(caminho, grupo);
  }
  for (const [caminho, itens] of grupos) {
    const pastas = JSON.parse(caminho) as string[];
    if (pastas.length) {
      const cabecalho = document.createElement("li");
      cabecalho.className = "favorito-pasta-titulo";
      cabecalho.textContent = `▸ ${pastas[pastas.length - 1]}`;
      cabecalho.title = pastas.join(" / ");
      listaFavoritos.append(cabecalho);
    }
    for (const item of itens) {
      const linha = document.createElement("li");
      const abrir = document.createElement("button");
      abrir.type = "button";
      abrir.className = "favorito-abrir";
      abrir.textContent = item.titulo;
      abrir.title = item.endereco;
      abrir.addEventListener("click", async () => {
        try {
          const url = await invoke<string>("navegar", { entrada: item.endereco });
          endereco.value = url;
          informar("Carregando…");
        } catch (erro) { informar(String(erro), true); }
      });
      const dominio = document.createElement("span");
      dominio.className = "favorito-dominio";
      try { dominio.textContent = new URL(item.endereco).host; } catch { dominio.textContent = item.endereco; }
      const marcadorSenha = (item.marcadores ?? []).includes("Senha associada") ? document.createElement("span") : null;
      if (marcadorSenha) { marcadorSenha.className = "favorito-marcador"; marcadorSenha.textContent = "Senha associada"; }
      const editar = document.createElement("button");
      editar.type = "button";
      editar.className = "favorito-editar";
      editar.textContent = "✎";
      editar.title = "Editar nome ou pasta";
      editar.setAttribute("aria-label", `Editar ${item.titulo}`);
      editar.addEventListener("click", () => {
        enderecoFavoritoEmEdicao = item.endereco;
        tituloEditarFavorito.value = item.titulo;
        pastasEditarFavorito.value = (item.pastas ?? []).join(" / ");
        dialogoEditarFavorito.showModal();
        tituloEditarFavorito.focus();
      });
      const remover = document.createElement("button");
      remover.type = "button";
      remover.className = "favorito-remover";
      remover.textContent = "×";
      remover.title = "Remover favorito";
      remover.setAttribute("aria-label", `Remover ${item.titulo} dos favoritos`);
      remover.addEventListener("click", async () => {
        remover.disabled = true;
        try { aplicarConfiguracoes(await invoke<Configuracoes>("remover_favorito", { endereco: item.endereco })); }
        catch (erro) { remover.disabled = false; informar(String(erro), true); }
      });
      linha.append(abrir, dominio);
      if (marcadorSenha) linha.append(marcadorSenha);
      linha.append(editar, remover);
      listaFavoritos.append(linha);
    }
  }
  const marcado = !!enderecoAtual && favoritos.some((item) => item.endereco === enderecoAtual);
  let podeGuardar = false;
  try {
    if (enderecoAtual) {
      const url = new URL(enderecoAtual);
      podeGuardar = ["http:", "https:"].includes(url.protocol) && !url.username && !url.password && url.hostname !== "canoa.invalid";
    }
  } catch { /* A barra ainda pode estar sincronizando o endereço. */ }
  botaoFavorito.disabled = !podeGuardar || alterandoFavorito;
  botaoFavorito.textContent = marcado ? "★" : "☆";
  botaoFavorito.setAttribute("aria-label", marcado ? "Remover página dos favoritos" : "Adicionar página aos favoritos");
  botaoFavorito.title = `${marcado ? "Remover dos" : "Adicionar aos"} favoritos (Ctrl+D)`;
}

function limparFormularioSenha() {
  idCredencialEmEdicao = null;
  formularioCredencial.reset();
  enderecoSenha.disabled = false;
  cancelarEdicaoSenha.hidden = true;
  document.querySelector<HTMLButtonElement>("#salvar-credencial")!.textContent = "Salvar senha";
}

async function atualizarCofre(carregarLista = true) {
  try {
    const estado = await invoke<ResumoCofre>("resumo_cofre");
    cofreReferenciaExiste = estado.existe;
    cofreReferenciaDesbloqueado = estado.desbloqueado;
    cofreFechado.hidden = estado.desbloqueado;
    cofreAberto.hidden = !estado.desbloqueado;
    formCriarCofre.hidden = estado.existe;
    formDesbloquearCofre.hidden = !estado.existe;
    document.querySelector<HTMLElement>("#cofre-quantidade")!.textContent = `${estado.quantidade} senha${estado.quantidade === 1 ? "" : "s"} salva${estado.quantidade === 1 ? "" : "s"}`;
    if (!estado.desbloqueado) {
      credenciaisReferencia = [];
      if (!cofreAberto.hidden) limparFormularioSenha();
      listaSenhas.replaceChildren();
      vaziasSenhas.hidden = true;
      statusCofre.textContent = estado.existe ? "Cofre bloqueado. Desbloqueie para acessar os dados." : "Crie um cofre para começar. Guarde a senha mestra: não há recuperação.";
      return;
    }
    statusCofre.textContent = "Cofre desbloqueado neste computador.";
    if (!carregarLista) return;
    const credenciais = await invoke<CredencialResumo[]>("listar_senhas");
    credenciaisReferencia = credenciais;
    listaSenhas.replaceChildren();
    vaziasSenhas.hidden = credenciais.length > 0;
    for (const credencial of credenciais) {
      const linha = document.createElement("li");
      const descricao = document.createElement("span");
      descricao.textContent = `${credencial.titulo} · ${credencial.usuario} · ${credencial.dominio}`;
      const copiar = document.createElement("button");
      copiar.type = "button"; copiar.textContent = "Copiar senha";
      copiar.setAttribute("aria-label", `Copiar senha de ${credencial.titulo} para ${credencial.usuario}`);
      copiar.addEventListener("click", async () => {
        copiar.disabled = true;
        try { await invoke("copiar_senha", { id: credencial.id }); notificar("Senha copiada; a área de transferência será limpa após 30 segundos se não mudar."); }
        catch (erro) { informar(String(erro), true); }
        finally { copiar.disabled = false; }
      });
      const editar = document.createElement("button");
      editar.type = "button"; editar.textContent = "Editar";
      editar.setAttribute("aria-label", `Editar senha de ${credencial.titulo} para ${credencial.usuario}`);
      editar.addEventListener("click", () => {
        idCredencialEmEdicao = credencial.id;
        enderecoSenha.value = credencial.origem;
        enderecoSenha.disabled = true;
        tituloSenha.value = credencial.titulo;
        usuarioSenha.value = credencial.usuario;
        valorSenha.value = "";
        document.querySelector<HTMLButtonElement>("#salvar-credencial")!.textContent = "Atualizar senha";
        cancelarEdicaoSenha.hidden = false;
        valorSenha.focus();
      });
      const remover = document.createElement("button");
      remover.type = "button"; remover.textContent = "Remover";
      remover.setAttribute("aria-label", `Remover senha de ${credencial.titulo} para ${credencial.usuario}`);
      remover.addEventListener("click", async () => {
        if (!window.confirm(`Remover a senha salva para ${credencial.dominio} (${credencial.usuario})?`)) return;
        remover.disabled = true;
        try {
          await invoke("remover_senha", { id: credencial.id });
          aplicarConfiguracoes(await invoke<Configuracoes>("ler_configuracoes"));
          await atualizarCofre();
          notificar("Senha removida do cofre");
        } catch (erro) { remover.disabled = false; informar(String(erro), true); }
      });
      linha.append(descricao, copiar, editar, remover);
      listaSenhas.append(linha);
    }
  } catch (erro) { statusCofre.textContent = `Não foi possível acessar o cofre: ${String(erro)}`; }
}

formCriarCofre.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  const senhaMestra = document.querySelector<HTMLInputElement>("#senha-mestra-nova")!;
  const confirmar = document.querySelector<HTMLInputElement>("#senha-mestra-confirmar")!;
  if (senhaMestra.value !== confirmar.value) { statusCofre.textContent = "As senhas mestras não conferem."; confirmar.focus(); return; }
  try {
    await invoke("criar_cofre", { senhaMestra: senhaMestra.value });
    senhaMestra.value = ""; confirmar.value = "";
    await atualizarCofre();
    notificar("Cofre criado e desbloqueado");
  } catch (erro) { senhaMestra.value = ""; confirmar.value = ""; statusCofre.textContent = String(erro); }
});

formDesbloquearCofre.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  const senhaMestra = document.querySelector<HTMLInputElement>("#senha-mestra")!;
  try {
    await invoke("desbloquear_cofre", { senhaMestra: senhaMestra.value });
    senhaMestra.value = "";
    await atualizarCofre();
    notificar("Cofre desbloqueado");
  } catch (erro) { senhaMestra.value = ""; senhaMestra.focus(); statusCofre.textContent = String(erro); }
});

formularioCredencial.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  const botao = document.querySelector<HTMLButtonElement>("#salvar-credencial")!;
  botao.disabled = true;
  try {
    await invoke<CredencialResumo>("salvar_senha", {
      id: idCredencialEmEdicao,
      endereco: enderecoSenha.value,
      titulo: tituloSenha.value,
      usuario: usuarioSenha.value,
      senha: valorSenha.value,
    });
    limparFormularioSenha();
    aplicarConfiguracoes(await invoke<Configuracoes>("ler_configuracoes"));
    await atualizarCofre();
    notificar("Senha protegida no cofre; site associado aos favoritos");
  } catch (erro) { valorSenha.value = ""; informar(String(erro), true); }
  finally { botao.disabled = false; }
});

document.querySelector<HTMLButtonElement>("#bloquear-cofre")!.addEventListener("click", async () => {
  try { await invoke("bloquear_cofre"); limparFormularioSenha(); await atualizarCofre(); }
  catch (erro) { informar(String(erro), true); }
});
cancelarEdicaoSenha.addEventListener("click", limparFormularioSenha);
document.querySelector<HTMLButtonElement>("#usar-site-atual")!.addEventListener("click", () => {
  try {
    if (!enderecoAtual) throw new Error("Abra uma página HTTPS antes de usar esta ação.");
    const url = new URL(enderecoAtual);
    if (url.protocol !== "https:") throw new Error("A página atual não usa HTTPS.");
    enderecoSenha.value = url.origin;
    if (!tituloSenha.value) tituloSenha.value = url.hostname;
  } catch (erro) { informar(erro instanceof Error ? erro.message : "Abra uma página HTTPS antes de usar esta ação.", true); }
});
void atualizarCofre();
window.setInterval(() => { if (!cofreAberto.hidden || !cofreFechado.hidden) void atualizarCofre(false); }, 15_000);

botaoBuscaFavoritos.addEventListener("click", () => {
  buscaFavoritos.hidden = !buscaFavoritos.hidden;
  if (buscaFavoritos.hidden) { buscaFavoritos.value = ""; consultaFavoritos = ""; desenharFavoritos(); }
  else { buscaFavoritos.focus(); }
});
buscaFavoritos.addEventListener("input", () => { consultaFavoritos = buscaFavoritos.value; desenharFavoritos(); });

document.querySelector<HTMLButtonElement>("#cancelar-edicao-favorito")!.addEventListener("click", () => dialogoEditarFavorito.close());
dialogoEditarFavorito.addEventListener("click", (evento) => {
  if (evento.target === dialogoEditarFavorito) dialogoEditarFavorito.close();
});
formularioEditarFavorito.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  if (!enderecoFavoritoEmEdicao) return;
  const botao = document.querySelector<HTMLButtonElement>("#salvar-edicao-favorito")!;
  botao.disabled = true;
  const pastas = pastasEditarFavorito.value.split("/").map((pasta) => pasta.trim()).filter(Boolean);
  try {
    aplicarConfiguracoes(await invoke<Configuracoes>("mover_favorito", {
      endereco: enderecoFavoritoEmEdicao,
      titulo: tituloEditarFavorito.value,
      pastas,
    }));
    dialogoEditarFavorito.close();
    notificar("Favorito atualizado");
  } catch (erro) { informar(String(erro), true); }
  finally { botao.disabled = false; }
});

function desenharFontes() {
  fontesSalvas.replaceChildren();
  document.querySelector<HTMLElement>("#fontes-vazias")!.hidden = fontes.length > 0;
  const ordenadas = fontes.map((fonte, indice) => ({ fonte, indice })).sort((a, b) => a.fonte.tema.localeCompare(b.fonte.tema, "pt-BR") || a.fonte.dominio.localeCompare(b.fonte.dominio, "pt-BR"));
  for (const { fonte } of ordenadas) {
    const linha = document.createElement("li");
    const rotulo = document.createElement("span");
    rotulo.textContent = `${fonte.tema} · ${fonte.dominio}`;
    const remover = document.createElement("button");
    remover.type = "button";
    remover.textContent = "Remover";
    remover.setAttribute("aria-label", `Remover ${fonte.dominio} do tema ${fonte.tema}`);
    remover.addEventListener("click", async () => {
      remover.disabled = true;
      try { aplicarConfiguracoes(await invoke<Configuracoes>("remover_fonte", fonte)); }
      catch (erro) { remover.disabled = false; informar(String(erro), true); }
    });
    linha.append(rotulo, remover);
    fontesSalvas.append(linha);
  }
}

function dataCurta(epoch: number) {
  return new Intl.DateTimeFormat("pt-BR", { dateStyle: "medium" }).format(new Date(epoch * 1000));
}

function desenharBiblioteca() {
  listaLeituraEl.replaceChildren();
  document.querySelector<HTMLElement>("#leitura-contagem")!.textContent = `(${listaLeitura.length})`;
  document.querySelector<HTMLElement>("#leitura-resumo")!.textContent = String(listaLeitura.length);
  document.querySelector<HTMLElement>("#leitura-vazia")!.hidden = listaLeitura.length > 0;
  const buscaLeitura = filtrosBiblioteca.leitura.trim().toLocaleLowerCase("pt-BR");
  for (const item of listaLeitura.filter((i) => !buscaLeitura || `${i.titulo} ${i.endereco} ${i.lido ? "lido" : "não lido"}`.toLocaleLowerCase("pt-BR").includes(buscaLeitura))) {
    const linha = document.createElement("li");
    const abrir = document.createElement("button"); abrir.type = "button"; abrir.className = "biblioteca-abrir";
    abrir.textContent = item.titulo; abrir.title = item.endereco;
    abrir.addEventListener("click", () => void invoke<string>("navegar", { entrada: item.endereco }).then((url) => { endereco.value = url; informar("Carregando…"); }).catch((erro) => informar(String(erro), true)));
    const detalhe = document.createElement("small"); detalhe.textContent = `${item.lido ? "Lido" : "Não lido"} · ${dataCurta(item.adicionado_em)}`;
    const lido = document.createElement("button"); lido.type = "button"; lido.className = "biblioteca-acao"; lido.textContent = item.lido ? "↶" : "✓"; lido.title = item.lido ? "Marcar como não lido" : "Marcar como lido";
    lido.addEventListener("click", () => void invoke<Configuracoes>("alternar_lido", { endereco: item.endereco }).then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true)));
    const remover = document.createElement("button"); remover.type = "button"; remover.className = "biblioteca-acao"; remover.textContent = "×"; remover.title = "Remover da Lista de leitura";
    remover.addEventListener("click", () => void invoke<Configuracoes>("remover_item_leitura", { endereco: item.endereco }).then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true)));
    const texto = document.createElement("div"); texto.className = "biblioteca-texto"; texto.append(abrir, detalhe);
    linha.append(texto, lido, remover); listaLeituraEl.append(linha);
  }
  { const vazio = document.querySelector<HTMLElement>("#leitura-vazia")!; const semResultados = !!buscaLeitura && listaLeituraEl.childElementCount === 0; vazio.textContent = semResultados ? "Nenhum item corresponde à busca." : "Artigos que você salvar para ler depois aparecerão aqui."; vazio.hidden = listaLeitura.length > 0 && !semResultados; }
  listaSalvosEl.replaceChildren();
  document.querySelector<HTMLElement>("#salvos-contagem")!.textContent = `(${salvos.length})`;
  document.querySelector<HTMLElement>("#salvos-resumo")!.textContent = String(salvos.length);
  document.querySelector<HTMLElement>("#salvos-vazios")!.hidden = salvos.length > 0;
  const buscaSalvos = filtrosBiblioteca.salvos.trim().toLocaleLowerCase("pt-BR");
  for (const item of salvos.filter((i) => !buscaSalvos || `${i.titulo} ${i.endereco} ${i.tipo} ${i.caminho ?? ""}`.toLocaleLowerCase("pt-BR").includes(buscaSalvos))) {
    const linha = document.createElement("li");
    const abrir = document.createElement("button"); abrir.type = "button"; abrir.className = "biblioteca-abrir";
    abrir.textContent = item.titulo; abrir.title = item.caminho ?? item.endereco;
    abrir.addEventListener("click", () => {
      const acao = item.tipo === "leitura-offline" && item.caminho
        ? abrirLeituraOffline(item.caminho)
        : item.caminho ? invoke("revelar_download", { caminho: item.caminho }) : invoke<string>("navegar", { entrada: item.endereco }).then((url) => { endereco.value = url; informar("Carregando…"); });
      void acao.catch((erro) => informar(String(erro), true));
    });
    const detalhe = document.createElement("small"); detalhe.textContent = `${item.tipo} · ${dataCurta(item.salvo_em)}${item.caminho ? " · arquivo baixado" : " · link da página"}`;
    const remover = document.createElement("button"); remover.type = "button"; remover.className = "biblioteca-acao"; remover.textContent = "×"; remover.title = "Remover dos itens baixados";
    remover.addEventListener("click", () => void invoke<Configuracoes>("remover_item_salvo", { endereco: item.endereco, caminho: item.caminho }).then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true)));
    const texto = document.createElement("div"); texto.className = "biblioteca-texto"; texto.append(abrir, detalhe);
    linha.append(texto, remover); listaSalvosEl.append(linha);
  }
  { const vazio = document.querySelector<HTMLElement>("#salvos-vazios")!; const semResultados = !!buscaSalvos && listaSalvosEl.childElementCount === 0; vazio.textContent = semResultados ? "Nenhum item corresponde à busca." : "Baixados registra arquivos obtidos pelo navegador e links de página guardados. Favoritos serve para acesso rápido; a seleção de recursos individuais ficará no fluxo Baixar."; vazio.hidden = salvos.length > 0 && !semResultados; }
  listaFontesBiblioteca.replaceChildren();
  document.querySelector<HTMLElement>("#fontes-contagem")!.textContent = `(${fontes.length})`;
  document.querySelector<HTMLElement>("#fontes-resumo")!.textContent = String(fontes.length);
  document.querySelector<HTMLElement>("#fontes-vazias-inicio")!.hidden = fontes.length > 0;
  const buscaFontes = filtrosBiblioteca.fontes.trim().toLocaleLowerCase("pt-BR");
  for (const fonte of fontes.filter((i) => !buscaFontes || `${i.tema} ${i.dominio}`.toLocaleLowerCase("pt-BR").includes(buscaFontes))) {
    const linha = document.createElement("li");
    const abrir = document.createElement("button"); abrir.type = "button"; abrir.className = "biblioteca-abrir"; abrir.textContent = `${fonte.tema} · ${fonte.dominio}`;
    abrir.title = `Pesquisar nesta fonte: ${fonte.dominio}`;
    abrir.addEventListener("click", () => {
      fontePendente = fonte;
      endereco.placeholder = `Pesquisar em ${fonte.dominio}`;
      ativarColecao("favoritos");
      void focarEndereco();
      informar(`Digite a pesquisa para ${fonte.tema} · ${fonte.dominio}`);
    });
    linha.append(abrir); listaFontesBiblioteca.append(linha);
  }
  { const vazio = document.querySelector<HTMLElement>("#fontes-vazias-inicio")!; const semResultados = !!buscaFontes && listaFontesBiblioteca.childElementCount === 0; vazio.textContent = semResultados ? "Nenhuma fonte corresponde à busca." : "Fontes organizadas por tema aparecerão aqui."; vazio.hidden = fontes.length > 0 && !semResultados; }
}

function ativarColecao(nome: string) {
  const paineis: Record<string, string> = { favoritos: "#painel-favoritos", leitura: "#painel-leitura", salvos: "#painel-salvos", fontes: "#painel-colecao-fontes" };
  for (const botao of document.querySelectorAll<HTMLButtonElement>("[data-colecao]")) {
    const ativa = botao.dataset.colecao === nome;
    botao.setAttribute("aria-selected", String(ativa));
  }
  for (const [colecao, seletor] of Object.entries(paineis)) document.querySelector<HTMLElement>(seletor)!.hidden = colecao !== nome;
}

for (const botao of document.querySelectorAll<HTMLButtonElement>("[data-colecao]")) botao.addEventListener("click", () => ativarColecao(botao.dataset.colecao ?? "favoritos"));
document.querySelector<HTMLButtonElement>("#gerenciar-fontes-biblioteca")?.addEventListener("click", () => void abrirConfiguracoes("pesquisa"));
for (const botao of document.querySelectorAll<HTMLButtonElement>(".alternar-busca-biblioteca")) {
  botao.addEventListener("click", () => {
    const nome = botao.dataset.busca!;
    const campo = document.querySelector<HTMLInputElement>(`#buscar-${nome}`)!;
    campo.hidden = !campo.hidden;
    if (campo.hidden) { campo.value = ""; filtrosBiblioteca[nome] = ""; desenharBiblioteca(); }
    else campo.focus();
  });
}
for (const campo of document.querySelectorAll<HTMLInputElement>(".busca-biblioteca")) {
  campo.addEventListener("input", () => { filtrosBiblioteca[campo.id.replace("buscar-", "")] = campo.value; desenharBiblioteca(); });
}

function desenharProtecao() {
  let dominio = "";
  try { if (enderecoAtual) dominio = new URL(enderecoAtual).hostname.toLowerCase(); } catch { /* Página interna */ }
  const excecaoAtual = !!dominio && excecoesBloqueador.some((item) => dominio === item || dominio.endsWith(`.${item}`));
  const disponivel = !!dominio && /^https?:/i.test(enderecoAtual ?? "");
  botaoProtecaoSite.disabled = !disponivel || !bloqueadorAtivo;
  botaoProtecaoSite.classList.toggle("protecao-pausada", excecaoAtual);
  botaoProtecaoSite.setAttribute("aria-pressed", String(excecaoAtual));
  botaoProtecaoSite.setAttribute("aria-label", excecaoAtual ? `Retomar proteção em ${dominio}` : `Pausar proteção em ${dominio || "este site"}`);
  botaoProtecaoSite.title = `${excecaoAtual ? "Retomar" : "Pausar"} anúncios e rastreadores neste site (Ctrl+Alt+P)`;
  const status = document.querySelector<HTMLElement>("#protecao-site-atual")!;
  status.textContent = !bloqueadorAtivo
    ? "A proteção global está desativada."
    : excecaoAtual
      ? `Proteção pausada em ${dominio}; ative-a pelo escudo da barra ou remova a exceção abaixo.`
      : disponivel
        ? `Proteção ativa em ${dominio}. O escudo pausa as regras para este domínio e seus subdomínios.`
        : "A proteção vale para todas as páginas HTTP/HTTPS, exceto as incluídas na lista de exceções.";
  controleBloqueador.checked = bloqueadorAtivo;
  listaExcecoesBloqueador.replaceChildren();
  for (const host of excecoesBloqueador) {
    const li = document.createElement("li");
    const texto = document.createElement("span"); texto.textContent = host;
    const remover = document.createElement("button"); remover.type = "button"; remover.textContent = "Retomar"; remover.title = `Retomar proteção em ${host}`;
    remover.addEventListener("click", async () => {
      try { aplicarConfiguracoes(await invoke<Configuracoes>("alternar_excecao_bloqueador", { dominio: host })); notificar(`Proteção retomada em ${host}`); }
      catch (erro) { informar(String(erro), true); }
    });
    li.append(texto, remover); listaExcecoesBloqueador.append(li);
  }
  excecoesVazias.hidden = excecoesBloqueador.length > 0;
}

function desenharResumoProtecao(resumo: ResumoProtecao) {
  botaoReverterListas.disabled = !resumo.tem_versao_anterior;
  bloqueiosPagina.textContent = `Bloqueados nesta página: ${resumo.bloqueados_pagina}`;
  bloqueiosSessao.textContent = `Nesta sessão: ${resumo.total_sessao}`;
  const categorias = Object.entries(resumo.categorias ?? {}).sort(([a], [b]) => a.localeCompare(b));
  bloqueiosCategorias.textContent = categorias.length
    ? `Categorias: ${categorias.map(([tipo, quantidade]) => `${tipo} ${quantidade}`).join(" · ")}`
    : "Categorias: sem bloqueios registrados";
  if (resumo.listas_atualizadas_em) {
    statusListasBloqueador.textContent = `${resumo.listas_versao} · ${new Date(resumo.listas_atualizadas_em * 1000).toLocaleString("pt-BR")}`;
  } else {
    statusListasBloqueador.textContent = "Usando as listas incluídas no aplicativo; nenhuma atualização manual foi instalada.";
  }
}

async function atualizarResumoProtecao() {
  try { desenharResumoProtecao(await invoke<ResumoProtecao>("resumo_bloqueador")); }
  catch { /* Proteção indisponível em builds sem suporte nativo. */ }
}

function aplicarConfiguracoes(valor: Configuracoes) {
  mostrarMemoria = valor.mostrar_memoria;
  bloqueadorAtivo = valor.bloqueador_ativo ?? true;
  excecoesBloqueador = valor.excecoes_bloqueador ?? [];
  sitesBloqueadosAtuais = valor.sites_bloqueados ?? [];
  controleLimpezaRastreamento.checked = valor.limpar_rastreamento ?? false;
  downloadsRecentes = valor.historico_downloads ?? [];
  desenharDownloadsRecentes();
  desenharSitesBloqueados(valor.sites_bloqueados ?? []);
  indicador.hidden = !mostrarMemoria;
  menuMemoria.firstChild!.textContent = mostrarMemoria ? "Ocultar memória " : "Mostrar memória ";
  controleMemoria.checked = mostrarMemoria;
  controleBuscador.value = valor.buscador;
  buscador = valor.buscador;
  fontes = valor.fontes;
  favoritos = valor.favoritos;
  desenharProtecao();
  listaLeitura = valor.lista_leitura ?? [];
  salvos = valor.salvos ?? [];
  gruposAba = valor.grupos_abas ?? [];
  desenharHistorico(valor);
  desenharControlesSessao(valor);
  const acoesBarra = valor.acoes_barra ?? { favorito: true, protecao: true, leitura: true, baixar: true };
  botaoFavorito.hidden = !acoesBarra.favorito;
  botaoProtecaoSite.hidden = !acoesBarra.protecao;
  botaoSalvarLeitura.hidden = !acoesBarra.leitura;
  botaoBaixarItens.hidden = !acoesBarra.baixar;
  for (const [id, chave] of [["config-barra-favorito", "favorito"], ["config-barra-protecao", "protecao"], ["config-barra-leitura", "leitura"], ["config-barra-baixar", "baixar"]] as const) {
    const controle = document.querySelector<HTMLInputElement>(`#${id}`);
    if (controle) controle.checked = acoesBarra[chave];
  }
  desenharFontes();
  desenharFavoritos();
  desenharBiblioteca();
  document.querySelector<HTMLElement>("#config-contagem-favoritos")!.textContent = `(${favoritos.length})`;
  document.querySelector<HTMLElement>("#config-contagem-leitura")!.textContent = `(${listaLeitura.length})`;
  caminhoDownloads.textContent = valor.pasta_downloads ?? "Pasta Downloads do Windows";
  document.querySelector<HTMLButtonElement>("#config-pasta-padrao")!.disabled = valor.pasta_downloads === null;
  if (retratoAtual) queueMicrotask(() => { if (retratoAtual) desenhar(retratoAtual); });
}

function desenharHistorico(config: Configuracoes) {
  historicoReferencia = config.historico_navegacao ?? [];
  controleHistorico.checked = config.historico_ativo;
  diasHistorico.value = String(config.dias_historico || 90);
  const termo = buscaHistorico.value.trim().toLocaleLowerCase("pt-BR");
  const entradas = (config.historico_navegacao ?? []).filter((item) => !termo || `${item.titulo} ${item.endereco}`.toLocaleLowerCase("pt-BR").includes(termo));
  listaHistoricoNavegacao.replaceChildren();
  for (const item of entradas) {
    const li = document.createElement("li");
    const abrir = document.createElement("button"); abrir.type = "button"; abrir.className = "item-biblioteca-abrir";
    abrir.textContent = item.titulo || item.endereco; abrir.title = item.endereco;
    abrir.addEventListener("click", () => void invoke<string>("navegar", { entrada: item.endereco }).then((url) => { endereco.value = url; }).catch((erro) => informar(String(erro), true)));
    const data = document.createElement("small"); data.textContent = new Date(item.visitado_em * 1000).toLocaleString("pt-BR");
    const remover = document.createElement("button"); remover.type = "button"; remover.textContent = "Remover"; remover.setAttribute("aria-label", `Remover ${item.titulo || item.endereco} do histórico`);
    remover.addEventListener("click", () => void invoke<Configuracoes>("remover_entrada_historico", { endereco: item.endereco }).then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true)));
    li.append(abrir, data, remover); listaHistoricoNavegacao.append(li);
  }
  historicoVazio.hidden = entradas.length > 0;
  if (!config.historico_ativo) historicoVazio.textContent = "Ative o histórico para registrar futuras visitas.";
  else historicoVazio.textContent = termo ? "Nenhum resultado para esta busca." : "Nenhuma visita registrada neste período.";
}

function desenharControlesSessao(config: Configuracoes) {
  const dataAtual = new Date();
  const usoDia = `${dataAtual.getFullYear()}-${String(dataAtual.getMonth() + 1).padStart(2, "0")}-${String(dataAtual.getDate()).padStart(2, "0")}`;
  limitesSitesEl.replaceChildren();
  for (const limite of config.limites_sites ?? []) {
    const li = document.createElement("li"); const texto = document.createElement("span");
    const uso = (config.uso_diario_sites ?? []).find((item) => item.dominio === limite.dominio && item.dia === usoDia)?.segundos ?? 0;
    texto.textContent = `${limite.dominio} — ${Math.floor(uso / 60)}/${limite.minutos_diarios} min hoje`;
    const remover = document.createElement("button"); remover.type = "button"; remover.textContent = "Remover";
    remover.addEventListener("click", () => void invoke<Configuracoes>("remover_limite_site", { dominio: limite.dominio }).then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true)));
    li.append(texto, remover); limitesSitesEl.append(li);
  }
  limitesSitesVazios.hidden = (config.limites_sites ?? []).length > 0;
  sitesFocoEl.replaceChildren();
  for (const dominio of config.sites_foco ?? []) {
    const li = document.createElement("li"); const texto = document.createElement("span"); texto.textContent = dominio;
    const remover = document.createElement("button"); remover.type = "button"; remover.textContent = "Remover";
    remover.addEventListener("click", () => void invoke<Configuracoes>("definir_sites_foco", { dominio, adicionar: false }).then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true)));
    li.append(texto, remover); sitesFocoEl.append(li);
  }
  sitesFocoVazios.hidden = (config.sites_foco ?? []).length > 0;
  duracaoFoco.value = String(config.duracao_foco_minutos || 25);
  const restante = config.foco_ate_epoch ? config.foco_ate_epoch - Math.floor(Date.now() / 1000) : 0;
  focoAteEpoch = config.foco_ate_epoch;
  const ativo = restante > 0;
  estadoFoco.textContent = ativo ? `Modo foco ativo — ${Math.ceil(restante / 60)} min restantes.` : "Modo foco inativo.";
  const alternar = document.querySelector<HTMLButtonElement>("#alternar-foco")!;
  alternar.textContent = ativo ? "Encerrar modo foco" : `Iniciar modo foco (Ctrl+Alt+F)`;
}

function desenharSitesBloqueados(sites: string[]) {
  listaSitesBloqueados.replaceChildren();
  for (const dominio of sites) {
    const linha = document.createElement("li");
    const nome = document.createElement("span"); nome.textContent = dominio;
    const remover = document.createElement("button"); remover.type = "button"; remover.textContent = "Desbloquear";
    remover.addEventListener("click", async () => {
      try { aplicarConfiguracoes(await invoke<Configuracoes>("alternar_site_bloqueado", { dominio })); notificar(`Bloqueio removido de ${dominio}`); }
      catch (erro) { informar(String(erro), true); }
    });
    linha.append(nome, remover); listaSitesBloqueados.append(linha);
  }
  sitesBloqueadosVazios.hidden = sites.length > 0;
}

async function abrirConfiguracoes(secao = "") {
  try {
    desenhar(await invoke<Retrato>("abrir_configuracoes"));
    if (secao) desenhar(await invoke<Retrato>("secao_configuracoes", { secao }));
  } catch (erro) { informar(String(erro), true); }
}

async function carregarDetalhesConfiguracoes() {
  try {
    aplicarConfiguracoes(await invoke<Configuracoes>("ler_configuracoes"));
    void carregarDownloadsAtivos();
    void atualizarMemoria(true);
    document.querySelector<HTMLElement>("#config-armazenamento")!.textContent = "Medindo dados locais…";
    void invoke<Armazenamento>("armazenamento_atual").then((valor) => {
      const formato = (bytes: number) => `${(bytes / 1048576).toLocaleString("pt-BR", { maximumFractionDigits: 1 })} MiB`;
      document.querySelector<HTMLElement>("#config-armazenamento")!.textContent = `Dados locais: ${formato(valor.dados_locais_bytes)}`;
      document.querySelector<HTMLElement>("#config-cache")!.textContent = `Cache identificado: ${formato(valor.cache_identificado_bytes)} (incluído nos dados locais)`;
      document.querySelector<HTMLElement>("#config-executavel")!.textContent = `Executável: ${formato(valor.executavel_bytes)} (sem o motor WebView2 instalado no sistema)`;
      document.querySelector<HTMLElement>("#config-configuracoes")!.textContent = `Configurações: ${formato(valor.configuracoes_bytes)}`;
      document.querySelector<HTMLElement>("#config-armazenamento-aviso")!.textContent = valor.incompleto ? "Alguns arquivos estavam indisponíveis; os valores podem estar abaixo do total." : "";
    }).catch(() => {
      document.querySelector<HTMLElement>("#config-armazenamento")!.textContent = "Armazenamento indisponível";
    });
    void invoke<InformacoesVersao>("informacoes_versao").then((informacoes) => {
      document.querySelector<HTMLElement>("#config-versao")!.textContent = `Versão ${informacoes.versao}`;
      document.querySelector<HTMLElement>("#config-compilacao")!.textContent = informacoes.compilado_em_epoch === null
        ? "Data da compilação indisponível"
        : `Compilado em ${new Date(informacoes.compilado_em_epoch * 1000).toLocaleString("pt-BR")}`;
    }).catch(() => {
      document.querySelector<HTMLElement>("#config-versao")!.textContent = "Versão indisponível";
    });
  } catch (erro) { informar(String(erro), true); }
}

async function carregarDownloadsAtivos() {
  try {
    downloadsAtivos.clear();
    for (const item of await invoke<DownloadAtivo[]>("listar_downloads_ativos")) downloadsAtivos.set(item.id, item);
    desenharDownloadsAtivos();
  } catch (erro) { console.warn("Não foi possível listar downloads ativos", erro); }
}

async function fecharConfiguracoes() {
  if (!configurando) return;
  try {
    const retrato = await invoke<Retrato>("listar_abas");
    desenhar(await invoke<Retrato>("fechar_aba", { id: retrato.ativa }));
  } catch (erro) { informar(String(erro), true); }
}

function fecharAviso() {
  window.clearTimeout(avisoTemporizador);
  aviso.hidden = true;
  document.body.classList.remove("com-aviso");
  void invoke("faixa_aviso", { visivel: false });
}

function notificar(mensagem: string, tipo: "download" | "erro" | "info" = "info", pasta = false) {
  window.clearTimeout(avisoTemporizador);
  avisoTexto.textContent = mensagem;
  avisoIcone.textContent = tipo === "download" ? "↓" : tipo === "erro" ? "!" : "i";
  aviso.dataset.tipo = tipo;
  avisoAcao.hidden = !pasta;
  aviso.hidden = false;
  document.body.classList.add("com-aviso");
  void invoke("faixa_aviso", { visivel: true });
  avisoTemporizador = window.setTimeout(fecharAviso, tipo === "erro" ? 12000 : 8000);
}

async function abrirDownloads() {
  try { await invoke("abrir_downloads"); }
  catch (erro) { notificar(String(erro), "erro"); }
}

function desenharDownloadsRecentes() {
  listaDownloadsRecentes.replaceChildren();
  desenharDownloadsAtivos();
  const recentes = document.querySelector<HTMLElement>("#downloads-vazios")!;
  recentes.hidden = downloadsRecentes.length > 0;
  document.querySelector<HTMLButtonElement>("#limpar-downloads")!.disabled = downloadsRecentes.length === 0;
  for (const item of downloadsRecentes) {
    const linha = document.createElement("li");
    const detalhes = document.createElement("span");
    detalhes.className = "download-recente-detalhes";
    const nome = document.createElement("strong");
    nome.textContent = item.nome;
    const estado = document.createElement("small");
    estado.textContent = `${item.sucesso ? "Concluído" : "Falhou ou foi cancelado"} · ${new Date(item.registrado_em).toLocaleString("pt-BR", { dateStyle: "short", timeStyle: "short" })}`;
    detalhes.append(nome, estado);
    linha.append(detalhes);
    if (item.sucesso && item.caminho) {
      if (item.tipo === "leitura-offline") {
        const ler = document.createElement("button");
        ler.type = "button";
        ler.textContent = "Ler offline";
        ler.addEventListener("click", () => void abrirLeituraOffline(item.caminho!));
        linha.append(ler);
      }
      const revelar = document.createElement("button");
      revelar.type = "button";
      revelar.className = "download-revelar";
      revelar.textContent = "Mostrar";
      revelar.title = "Mostrar arquivo na pasta";
      revelar.addEventListener("click", async () => {
        try { await invoke("revelar_download", { caminho: item.caminho }); }
        catch (erro) { informar(String(erro), true); }
      });
      linha.append(revelar);
    }
    listaDownloadsRecentes.append(linha);
  }
}

function desenharDownloadsAtivos() {
  listaDownloadsAtivos.replaceChildren();
  for (const item of downloadsAtivos.values()) {
    const linha = document.createElement("li");
    const detalhes = document.createElement("span"); detalhes.className = "download-recente-detalhes";
    const nome = document.createElement("strong"); nome.textContent = item.nome;
    const estado = document.createElement("small");
    if (item.total_bytes && item.total_bytes > 0) {
      const percentual = Math.min(100, Math.floor(item.bytes_recebidos / item.total_bytes * 100));
      estado.textContent = `Baixando · ${percentual}% · ${(item.bytes_recebidos / 1048576).toFixed(1)} / ${(item.total_bytes / 1048576).toFixed(1)} MiB`;
      const progresso = document.createElement("progress"); progresso.max = item.total_bytes; progresso.value = item.bytes_recebidos;
      progresso.setAttribute("aria-label", `Progresso de ${item.nome}`); detalhes.append(nome, estado, progresso);
    } else {
      estado.textContent = item.cancelavel ? `Baixando · ${(item.bytes_recebidos / 1048576).toFixed(1)} MiB recebidos` : "Gerenciado pelo WebView2 · progresso não informado";
      detalhes.append(nome, estado);
    }
    linha.append(detalhes);
    if (item.cancelavel) {
      const cancelar = document.createElement("button"); cancelar.type = "button"; cancelar.textContent = "Cancelar";
      cancelar.setAttribute("aria-label", `Cancelar download de ${item.nome}`);
      cancelar.addEventListener("click", async () => {
        cancelar.disabled = true;
        try { await invoke("cancelar_download", { id: item.id }); }
        catch (erro) { cancelar.disabled = false; informar(String(erro), true); }
      });
      linha.append(cancelar);
    }
    listaDownloadsAtivos.append(linha);
  }
  listaDownloadsAtivos.hidden = downloadsAtivos.size === 0;
  document.querySelector<HTMLElement>("#downloads-ativos-vazios")!.hidden = downloadsAtivos.size > 0;
}

async function abrirLeituraOffline(caminho: string) {
  try {
    const html = await invoke<string>("ler_leitura_offline", { caminho });
    document.querySelector<HTMLIFrameElement>("#leitura-offline-frame")!.srcdoc = html;
    document.querySelector<HTMLDialogElement>("#dialogo-leitura-offline")!.showModal();
  } catch (erro) { informar(String(erro), true); }
}

// The persisted Rust history is canonical; refresh it after each completed download.

async function focarEndereco() {
  try {
    await invoke("focar_interface");
    endereco.focus();
    endereco.select();
  } catch (erro) { informar(String(erro), true); }
}

async function atualizarMemoria(forcar = false) {
  if (medindo || (!mostrarMemoria && !configurando) || document.visibilityState === "hidden" || (!forcar && Date.now() - ultimaMedicao < 10000)) return;
  medindo = true;
  try {
    const valor = await invoke<Memoria>("memoria_atual");
    const rotulo = valor.instancias > 1 ? "Mem. total" : "Mem.";
    indicador.textContent = `${rotulo} ${Math.round(valor.privado_bytes / 1048576)} MiB`;
    const grupo = valor.instancias > 1 ? ` em ${valor.instancias} instâncias abertas do Canoa` : "";
    indicador.title = `Abrir tarefas (Shift+Esc). Memória privada do Canoa e WebView2${grupo}: ${(valor.privado_bytes / 1048576).toFixed(1)} MiB em ${valor.processos} processos. Não equivale à RAM física exclusiva.`;
    valorMemoria.textContent = `${(valor.privado_bytes / 1048576).toFixed(1)} MiB em ${valor.processos} processos`;
  } catch {
    indicador.textContent = "Mem. —";
    indicador.title = "Abrir tarefas (Shift+Esc). Medição de memória indisponível";
    valorMemoria.textContent = "Medição indisponível";
  } finally {
    ultimaMedicao = Date.now();
    medindo = false;
  }
}

function informar(mensagem: string, falha = false) {
  if (falha) notificar(mensagem, "erro");
  document.body.classList.toggle("carregando", mensagem === "Carregando…" || mensagem === "Abrindo página…");
}

function avisarDownload(mensagem: string, falha = false) {
  notificar(mensagem, falha ? "erro" : "download", true);
}

function mostrarSecaoConfiguracoes(url: string) {
  const secaoOriginal = url.slice(URL_CONFIGURACOES.length).replace(/^\//, "");
  const secao = !secaoOriginal ? "inicio" : secaoOriginal === "armazenamento" ? "memoria" : secaoOriginal === "biblioteca" ? "favoritos" : secaoOriginal;
  posicionarColecoesConfiguracoes(secao);
  for (const botao of document.querySelectorAll<HTMLButtonElement>("[data-ir-secao]")) {
    botao.setAttribute("aria-current", botao.dataset.irSecao === secao ? "page" : "false");
  }
  for (const grupo of document.querySelectorAll<HTMLElement>("[data-config-secao]")) {
    grupo.hidden = !!secao && grupo.dataset.configSecao !== secao;
  }
  document.querySelector<HTMLElement>("#configuracoes-titulo")!.textContent = secao
    ? `Configurações: ${({ inicio: "Visão geral", pesquisa: "Pesquisa e fontes", favoritos: "Favoritos", leitura: "Lista de leitura", barra: "Barra de endereço", memoria: "Memória e armazenamento", downloads: "Baixados", historico: "Histórico", foco: "Tempo e foco", limpeza: "Cookies e cache", privacidade: "Proteção e bloqueios", senhas: "Senhas", atalhos: "Atalhos", sobre: "Sobre" } as Record<string, string>)[secao] ?? ""}`
    : "Configurações";
}

function posicionarColecoesConfiguracoes(secao: string) {
  const hospedeiroFavoritos = document.querySelector<HTMLElement>("#config-favoritos-host")!;
  const hospedeiroLeitura = document.querySelector<HTMLElement>("#config-leitura-host")!;
  const mostrarFavoritos = !secao || secao === "favoritos";
  const mostrarLeitura = !secao || secao === "leitura";
  if (mostrarFavoritos) {
    hospedeiroFavoritos.append(painelFavoritos);
    painelFavoritos.hidden = false;
  } else if (marcadorFavoritos.parentNode) {
    marcadorFavoritos.parentNode.insertBefore(painelFavoritos, marcadorFavoritos.nextSibling);
    painelFavoritos.hidden = document.querySelector<HTMLButtonElement>('[data-colecao="favoritos"]')?.getAttribute("aria-selected") !== "true";
  }
  if (mostrarLeitura) {
    hospedeiroLeitura.append(painelLeitura);
    painelLeitura.hidden = false;
  } else if (marcadorLeitura.parentNode) {
    marcadorLeitura.parentNode.insertBefore(painelLeitura, marcadorLeitura.nextSibling);
    painelLeitura.hidden = document.querySelector<HTMLButtonElement>('[data-colecao="leitura"]')?.getAttribute("aria-selected") !== "true";
  }
}

function desenhar(valor: Retrato) {
  retratoAtual = valor;
  const atual = valor.abas.find((aba) => aba.id === valor.ativa)!;
  if (idAbaAnterior !== atual.id && atual.endereco === null) {
    consultaFavoritos = "";
    buscaFavoritos.value = "";
    buscaFavoritos.hidden = true;
    for (const nome of Object.keys(filtrosBiblioteca)) {
      filtrosBiblioteca[nome] = "";
      const campo = document.querySelector<HTMLInputElement>(`#buscar-${nome}`);
      if (campo) { campo.value = ""; campo.hidden = true; }
    }
    fontePendente = null;
    endereco.placeholder = "Endereço ou pesquisa";
    desenharBiblioteca();
  }
  idAbaAnterior = atual.id;
  abasAbertas = valor.abas;
  if (document.activeElement !== endereco) ocultarSugestoes();
  const estavaConfigurando = configurando;
  const editandoEndereco = document.activeElement === endereco;
  configurando = !!atual.endereco && atual.endereco.startsWith(URL_CONFIGURACOES);
  painelConfiguracoes.hidden = !configurando;
  document.body.classList.toggle("configurando", configurando);
  if (!configurando) posicionarColecoesConfiguracoes("__fora__");
  for (const id of ["voltar", "avancar"]) document.querySelector<HTMLButtonElement>(`#${id}`)!.disabled = configurando;
  if (configurando) {
    mostrarSecaoConfiguracoes(atual.endereco!);
    if (!estavaConfigurando) void carregarDetalhesConfiguracoes();
  }
  enderecoAtual = atual.endereco;
  desenharProtecao();
  botaoSalvarLeitura.disabled = configurando || !atual.endereco || !/^https?:\/\//i.test(atual.endereco);
  botaoBaixarItens.disabled = configurando || !atual.endereco || !/^https?:\/\//i.test(atual.endereco);
  desenharFavoritos();
  document.body.classList.toggle("multiplas", valor.abas.length > 0);
  document.body.classList.toggle("pagina-inicial", atual.endereco === null && !configurando);
  if (!editandoEndereco) {
    endereco.value = atual.endereco ?? "";
    endereco.placeholder = atual.endereco === null ? "Endereço ou pesquisa" : "";
  }
  lista.replaceChildren();
  let abaAtiva: HTMLElement | null = null;
  const gruposInseridos = new Set<string>();
  const gruposRecolhidos = new Set(gruposAba.filter((grupo) => grupo.recolhido).map((grupo) => grupo.nome.toLocaleLowerCase("pt-BR")));
  for (const aba of valor.abas) {
    if (aba.grupo && !gruposInseridos.has(aba.grupo.toLocaleLowerCase("pt-BR"))) {
      gruposInseridos.add(aba.grupo.toLocaleLowerCase("pt-BR"));
      const recolhido = gruposRecolhidos.has(aba.grupo.toLocaleLowerCase("pt-BR"));
      const alternar = document.createElement("button");
      alternar.type = "button";
      alternar.className = "aba-grupo-toggle";
      alternar.textContent = `${recolhido ? "▸" : "▾"} ${aba.grupo} (${valor.abas.filter((item) => item.grupo?.toLocaleLowerCase("pt-BR") === aba.grupo!.toLocaleLowerCase("pt-BR")).length})`;
      alternar.title = `${recolhido ? "Expandir" : "Recolher"} grupo ${aba.grupo}`;
      alternar.setAttribute("aria-expanded", String(!recolhido));
      alternar.setAttribute("aria-current", String(valor.abas.some((item) => item.id === valor.ativa && item.grupo?.toLocaleLowerCase("pt-BR") === aba.grupo!.toLocaleLowerCase("pt-BR"))));
      alternar.addEventListener("dragover", (evento) => { evento.preventDefault(); alternar.classList.add("destino-arraste"); });
      alternar.addEventListener("dragleave", () => alternar.classList.remove("destino-arraste"));
      alternar.addEventListener("drop", (evento) => {
        evento.preventDefault(); alternar.classList.remove("destino-arraste");
        const origemId = Number(evento.dataTransfer?.getData("text/plain"));
        if (origemId && valor.abas.some((item) => item.id === origemId && item.grupo?.toLocaleLowerCase("pt-BR") !== aba.grupo!.toLocaleLowerCase("pt-BR"))) void atribuirGrupo(origemId, aba.grupo!).catch((erro) => informar(String(erro), true));
      });
      alternar.addEventListener("click", async () => {
        try { aplicarConfiguracoes(await invoke<Configuracoes>("alternar_recolhimento_grupo", { nome: aba.grupo })); }
        catch (erro) { informar(String(erro), true); }
      });
      if (recolhido && valor.abas.some((item) => item.id === valor.ativa && item.grupo?.toLocaleLowerCase("pt-BR") === aba.grupo!.toLocaleLowerCase("pt-BR"))) abaAtiva = alternar;
      lista.append(alternar);
    }
    if (aba.grupo && gruposRecolhidos.has(aba.grupo.toLocaleLowerCase("pt-BR"))) {
      continue;
    }
    const grupo = document.createElement("div");
    grupo.className = "aba";
    const escolher = document.createElement("button");
    escolher.type = "button";
    escolher.className = "aba-titulo";
    escolher.textContent = aba.grupo ? `${aba.grupo} · ${aba.titulo}` : aba.titulo;
    grupo.classList.toggle("aba-agrupada", !!aba.grupo);
    grupo.dataset.grupo = aba.grupo ?? "";
    escolher.title = aba.endereco ?? "Nova aba";
    escolher.setAttribute("aria-label", `Abrir aba ${aba.titulo}`);
    escolher.setAttribute("aria-haspopup", "menu");
    escolher.setAttribute("aria-current", aba.id === valor.ativa ? "page" : "false");
    escolher.draggable = true;
    escolher.title = `${aba.endereco ?? "Nova aba"} · Arraste para reorganizar`;
    escolher.addEventListener("click", () => void acionar("trocar_aba", { id: aba.id }, aba.id));
    escolher.addEventListener("contextmenu", (evento) => { evento.preventDefault(); evento.stopPropagation(); abrirMenuAba(evento.clientX, evento.clientY, aba, valor.abas); });
    escolher.addEventListener("keydown", (evento) => {
      if (evento.key === "ContextMenu" || (evento.shiftKey && evento.key === "F10")) {
        evento.preventDefault(); evento.stopPropagation(); const caixa = escolher.getBoundingClientRect(); abrirMenuAba(caixa.left, caixa.bottom, aba, valor.abas);
      }
    });
    escolher.addEventListener("dragstart", (evento) => {
      evento.dataTransfer?.setData("text/plain", String(aba.id));
      if (evento.dataTransfer) evento.dataTransfer.effectAllowed = "move";
      grupo.classList.add("arrastando");
    });
    escolher.addEventListener("dragend", () => {
      for (const item of lista.querySelectorAll(".aba")) item.classList.remove("arrastando", "destino-arraste");
    });
    grupo.addEventListener("dragover", (evento) => { evento.preventDefault(); grupo.classList.add("destino-arraste"); });
    grupo.addEventListener("dragleave", (evento) => {
      if (!grupo.contains(evento.relatedTarget as Node | null)) grupo.classList.remove("destino-arraste");
    });
    grupo.addEventListener("drop", (evento) => {
      evento.preventDefault();
      grupo.classList.remove("destino-arraste");
      const origemId = Number(evento.dataTransfer?.getData("text/plain"));
      const origem = valor.abas.findIndex((item) => item.id === origemId);
      const alvo = valor.abas.findIndex((item) => item.id === aba.id);
      if (origem < 0 || alvo < 0 || origem === alvo) return;
      const caixa = grupo.getBoundingClientRect();
      let destino = alvo + (evento.clientX > caixa.left + caixa.width / 2 ? 1 : 0);
      if (origem < destino) destino -= 1;
      if (origem !== destino) void invoke<Retrato>("mover_aba", { id: origemId, destino }).then(desenhar).catch((erro) => informar(String(erro), true));
    });
    const fechar = document.createElement("button");
    fechar.type = "button";
    fechar.className = "aba-fechar";
    fechar.textContent = "×";
    fechar.title = "Fechar aba (Ctrl+W)";
    fechar.setAttribute("aria-label", `Fechar aba ${aba.titulo}`);
    fechar.addEventListener("click", () => void acionar("fechar_aba", { id: aba.id }));
    grupo.append(escolher, fechar);
    lista.append(grupo);
    if (aba.id === valor.ativa) abaAtiva = grupo;
  }
  if (valor.abas.length > 1) abaAtiva?.scrollIntoView({ block: "nearest", inline: "nearest" });
  if (abaContextual && !valor.abas.some((aba) => aba.id === abaContextual!.id)) fecharMenuAbas();
}

function abrirMenuAba(x: number, y: number, aba: Aba, abas: Aba[]) {
  ocultarSugestoes();
  fecharMenu();
  contextoMenuPagina = null;
  abaContextual = aba;
  const posicao = abas.findIndex((item) => item.id === aba.id);
  const host = obterDominio(aba.endereco);
  void invoke("abrir_menu_nativo", {
    tipo: "abas", x, y,
    podeFecharOutras: abas.length > 1,
    podeFecharDireita: posicao >= 0 && posicao < abas.length - 1,
    temEndereco: !!aba.endereco && !aba.endereco.startsWith(URL_CONFIGURACOES),
    temLink: false, temImagem: false, favorito: false, mutada: aba.mutada,
    textoSelecionado: null, grupoAtual: aba.grupo,
    siteBloqueado: !!host && dominioEstaNaLista(host, sitesBloqueadosAtuais),
    protecaoPausada: !!host && dominioEstaNaLista(host, excecoesBloqueador),
    // Tauri requires every command argument, even for the principal menu.
  }).catch((erro) => { abaContextual = null; informar(String(erro), true); });
}

function fecharMenuAbas(restaurarFoco = false) {
  abaContextual = null;
  if (restaurarFoco) lista.querySelector<HTMLButtonElement>(".aba-titulo[aria-current='page']")?.focus();
}

async function atribuirGrupo(id: number, nome: string) {
  const retrato = await invoke<Retrato>("definir_grupo_aba", { id, grupo: nome });
  const limpo = nome.trim();
  if (limpo && !gruposAba.some((grupo) => grupo.nome.toLocaleLowerCase("pt-BR") === limpo.toLocaleLowerCase("pt-BR"))) gruposAba = [...gruposAba, { nome: limpo, recolhido: false }];
  desenhar(retrato);
}

async function executarAcaoMenuAba(acao: string) {
  const aba = abaContextual;
  fecharMenuAbas();
  if (!aba) return;
  if (acao === "mudo") {
    try { desenhar(await invoke<Retrato>("alternar_mudo_aba", { id: aba.id })); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (acao === "recarregar") {
    if (!aba.endereco) return;
    try {
      const retrato = await invoke<Retrato>("listar_abas");
      if (retrato.ativa !== aba.id) desenhar(await invoke<Retrato>("trocar_aba", { id: aba.id }));
      if (aba.endereco.startsWith(URL_CONFIGURACOES)) await carregarDetalhesConfiguracoes();
      else await invoke("recarregar");
    } catch (erro) { informar(String(erro), true); }
    return;
  }
  const comandos: Record<string, string> = {
    duplicar: "duplicar_aba",
    fechar: "fechar_aba",
    fechar_outras: "fechar_outras_abas",
    fechar_direita: "fechar_abas_a_direita",
  };
  const comando = comandos[acao];
  if (comando) void acionar(comando, { id: aba.id }, aba.id);
}

async function acionar(comando: string, args?: Record<string, unknown>, idAnterior?: number) {
  try {
    const proximo = await invoke<Retrato>(comando, args);
    desenhar(proximo);
    if (idAnterior !== undefined && idAnterior !== proximo.ativa) {
      informar("Abrindo página…");
    }
    if (proximo.abas.find((aba) => aba.id === proximo.ativa)?.endereco === null) {
      informar("Nova aba");
      void focarEndereco();
    }
    if (downloadsAtivos.size > 0 && ["nova_aba", "trocar_aba", "fechar_aba", "duplicar_aba", "fechar_outras_abas", "fechar_abas_a_direita"].includes(comando)) avisarDownload("Os downloads continuam; consulte a central Baixados");
  } catch (erro) {
    informar(String(erro), true);
  }
}

formulario.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  ocultarSugestoes();
  try {
    const url = fontePendente && endereco.value.trim() && !/^https?:\/\//i.test(endereco.value.trim())
      ? await invoke<string>("pesquisar_fonte", { consulta: endereco.value, dominio: fontePendente.dominio, tema: fontePendente.tema, filtrar: true })
      : await invoke<string>("navegar", { entrada: endereco.value });
    fontePendente = null;
    endereco.placeholder = "Endereço ou pesquisa";
    endereco.value = url;
    informar("Carregando…");
  } catch (erro) {
    informar(String(erro), true);
  }
});


for (const comando of ["voltar", "avancar", "recarregar"]) {
  document.querySelector<HTMLButtonElement>(`#${comando}`)!.addEventListener("click", async () => {
    if (configurando) { if (comando === "recarregar") void carregarDetalhesConfiguracoes(); return; }
    try { await invoke(comando); } catch (erro) { informar(String(erro), true); }
  });
}

nova.addEventListener("click", () => void acionar("nova_aba"));
botaoFavorito.addEventListener("click", () => void alternarFavorito());
async function alternarExcecaoProtecao() {
  if (!enderecoAtual || !/^https?:/i.test(enderecoAtual)) return;
  try {
    const eraExcecao = (() => { try { const host = new URL(enderecoAtual!).hostname.toLowerCase(); return excecoesBloqueador.some((item) => host === item || host.endsWith(`.${item}`)); } catch { return false; } })();
    aplicarConfiguracoes(await invoke<Configuracoes>("alternar_excecao_bloqueador", { dominio: null }));
    notificar(eraExcecao ? "Proteção retomada neste site" : "Proteção pausada neste site");
    await invoke("recarregar");
  } catch (erro) { informar(String(erro), true); }
}
botaoProtecaoSite.addEventListener("click", () => void alternarExcecaoProtecao());
controleBloqueador.addEventListener("change", async () => {
  controleBloqueador.disabled = true;
  try {
    aplicarConfiguracoes(await invoke<Configuracoes>("definir_bloqueador", { ativo: controleBloqueador.checked }));
    notificar(bloqueadorAtivo ? "Bloqueio de anúncios e rastreadores ativado" : "Bloqueio de anúncios e rastreadores desativado");
    if (enderecoAtual && /^https?:/i.test(enderecoAtual)) await invoke("recarregar");
  } catch (erro) { informar(String(erro), true); }
  finally { controleBloqueador.disabled = false; }
});
botaoAtualizarListas.addEventListener("click", async () => {
  botaoAtualizarListas.disabled = true;
  statusListasBloqueador.textContent = "Baixando e validando EasyList e EasyPrivacy…";
  try {
    desenharResumoProtecao(await invoke<ResumoProtecao>("atualizar_listas_bloqueio"));
    notificar("Listas EasyList e EasyPrivacy atualizadas");
  } catch (erro) {
    statusListasBloqueador.textContent = "A atualização falhou; as listas anteriores continuam ativas.";
    informar(String(erro), true);
    void atualizarResumoProtecao();
  } finally { botaoAtualizarListas.disabled = false; }
});
botaoReverterListas.addEventListener("click", async () => {
  botaoReverterListas.disabled = true;
  statusListasBloqueador.textContent = "Restaurando a versão anterior…";
  try {
    desenharResumoProtecao(await invoke<ResumoProtecao>("reverter_listas_bloqueio"));
    notificar("Versão anterior das listas restaurada");
  } catch (erro) {
    informar(String(erro), true);
    void atualizarResumoProtecao();
  }
});
controleLimpezaRastreamento.addEventListener("change", async () => {
  controleLimpezaRastreamento.disabled = true;
  try {
    aplicarConfiguracoes(await invoke<Configuracoes>("definir_limpeza_rastreamento", { ativo: controleLimpezaRastreamento.checked }));
    notificar(controleLimpezaRastreamento.checked ? "Limpeza de rastreamento ativada" : "Limpeza de rastreamento desativada");
  } catch (erro) { informar(String(erro), true); }
  finally { controleLimpezaRastreamento.disabled = false; }
});
document.querySelector<HTMLFormElement>("#form-site-bloqueado")!.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  const campo = document.querySelector<HTMLInputElement>("#site-bloqueado-dominio")!;
  try {
    const config = await invoke<Configuracoes>("alternar_site_bloqueado", { dominio: campo.value });
    aplicarConfiguracoes(config); campo.value = ""; notificar("Site bloqueado");
  } catch (erro) { informar(String(erro), true); }
});
function fecharMenuFavoritos() { menuFavoritos.hidden = true; botaoMenuFavoritos.setAttribute("aria-expanded", "false"); }
botaoMenuFavoritos.addEventListener("click", () => {
  menuFavoritos.hidden = !menuFavoritos.hidden;
  botaoMenuFavoritos.setAttribute("aria-expanded", String(!menuFavoritos.hidden));
});
document.addEventListener("click", (evento) => {
  if (!menuFavoritos.contains(evento.target as Node) && evento.target !== botaoMenuFavoritos) fecharMenuFavoritos();
});
menuFavoritos.addEventListener("keydown", (evento) => { if (evento.key === "Escape") { fecharMenuFavoritos(); botaoMenuFavoritos.focus(); } });
importarFavoritos.addEventListener("click", async () => {
  fecharMenuFavoritos();
  importarFavoritos.disabled = true;
  try {
    const resultado = await invoke<ResultadoImportacao | null>("importar_favoritos");
    if (resultado) {
      aplicarConfiguracoes(resultado.configuracoes);
      notificar(`${resultado.adicionados} importado(s)${resultado.ignorados ? ` · ${resultado.ignorados} ignorado(s)` : ""}`);
    }
  } catch (erro) { informar(String(erro), true); }
  finally { importarFavoritos.disabled = false; }
});
async function exportar(formato: "html" | "json") {
  fecharMenuFavoritos();
  const botao = formato === "html" ? exportarHtml : exportarFavoritos;
  botao.disabled = true;
  try {
    const quantidade = await invoke<number | null>("exportar_favoritos", { formato });
    if (quantidade !== null) notificar(`${quantidade} favorito(s) exportado(s)`);
  } catch (erro) { informar(String(erro), true); }
  finally { botao.disabled = favoritos.length === 0; }
}
exportarFavoritos.addEventListener("click", () => void exportar("json"));
exportarHtml.addEventListener("click", () => void exportar("html"));
botaoSalvarLeitura.addEventListener("click", async () => {
  botaoSalvarLeitura.disabled = true;
  try { aplicarConfiguracoes(await invoke<Configuracoes>("salvar_para_leitura")); notificar("Adicionado à Lista de leitura"); }
  catch (erro) { informar(String(erro), true); }
  finally { botaoSalvarLeitura.disabled = !enderecoAtual || !/^https?:\/\//i.test(enderecoAtual); }
});
botaoBaixarItens.addEventListener("click", () => {
  if (!botaoBaixarItens.disabled) void invoke("solicitar_recursos_pagina").catch((erro) => informar(String(erro), true));
});
async function alternarFavorito() {
  if (alterandoFavorito || botaoFavorito.disabled) return;
  alterandoFavorito = true;
  botaoFavorito.disabled = true;
  try {
    const marcado = favoritos.some((item) => item.endereco === enderecoAtual);
    aplicarConfiguracoes(await invoke<Configuracoes>("alternar_favorito"));
    notificar(marcado ? "Removido dos favoritos" : "Adicionado aos favoritos");
  } catch (erro) { informar(String(erro), true); }
  finally { alterandoFavorito = false; desenharFavoritos(); }
}
async function abrirTarefas() {
  try { await invoke("abrir_gerenciador"); }
  catch (erro) { informar(String(erro), true); }
}
indicador.addEventListener("click", () => void abrirTarefas());
indicador.addEventListener("contextmenu", (evento) => { evento.preventDefault(); evento.stopPropagation(); abrirMenu(); });
document.querySelector<HTMLButtonElement>("#config-abrir-tarefas")!.addEventListener("click", () => void abrirTarefas());
function abrirMenu() {
  ocultarSugestoes();
  contextoMenuPagina = null;
  abaContextual = null;
  const caixa = botaoConfiguracoes.getBoundingClientRect();
  const temEndereco = !!enderecoAtual && /^https?:\/\//i.test(enderecoAtual);
  const dominio = obterDominio(enderecoAtual);
  void invoke("abrir_menu_nativo", {
    tipo: "principal", x: caixa.left, y: caixa.bottom,
    podeFecharOutras: false, podeFecharDireita: false, temEndereco,
    temLink: false, temImagem: false, favorito: false, mutada: false,
    textoSelecionado: null, grupoAtual: null,
    siteBloqueado: !!dominio && dominioEstaNaLista(dominio, sitesBloqueadosAtuais),
    protecaoPausada: !!dominio && dominioEstaNaLista(dominio, excecoesBloqueador),
  })
    .catch((erro) => informar(String(erro), true));
}
function fecharMenu() { menuPrincipal.hidden = true; botaoConfiguracoes.setAttribute("aria-expanded", "false"); }
botaoConfiguracoes.addEventListener("click", abrirMenu);
document.addEventListener("click", (evento) => { if (!menuPrincipal.contains(evento.target as Node) && evento.target !== botaoConfiguracoes) fecharMenu(); });
document.addEventListener("contextmenu", (evento) => {
  const alvo = evento.target as HTMLElement;
  if (alvo.closest("input, textarea, [contenteditable='true']")) return;
  evento.preventDefault();
  evento.stopPropagation();
});
document.querySelector<HTMLButtonElement>("#menu-configuracoes")!.addEventListener("click", () => { fecharMenu(); void abrirConfiguracoes(); });
document.querySelector<HTMLButtonElement>("#menu-tarefas")!.addEventListener("click", () => { fecharMenu(); void abrirTarefas(); });
document.querySelector<HTMLButtonElement>("#menu-historico")!.addEventListener("click", () => { fecharMenu(); void abrirConfiguracoes("historico"); });
document.querySelector<HTMLButtonElement>("#menu-foco")!.addEventListener("click", () => { fecharMenu(); void invoke<Configuracoes>("alternar_modo_foco").then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true)); });
menuMemoria.addEventListener("click", () => { fecharMenu(); void alternarIndicadorMemoria(); });
void listen<string>("menu-nativo", async (evento) => {
  const acoesAbas: Record<string, string> = {
    canoa_aba_recarregar: "recarregar",
    canoa_aba_mudo: "mudo",
    canoa_aba_duplicar: "duplicar",
    canoa_aba_fechar: "fechar",
    canoa_aba_fechar_outras: "fechar_outras",
    canoa_aba_fechar_direita: "fechar_direita",
  };
  const acaoAba = acoesAbas[evento.payload];
  if (evento.payload === "canoa_aba_grupo" || evento.payload === "canoa_aba_desagrupar") {
    const aba = abaContextual;
    fecharMenuAbas();
    if (!aba) return;
    const grupo = evento.payload === "canoa_aba_desagrupar" ? "" : window.prompt("Nome do grupo de abas (até 32 caracteres)", aba.grupo ?? "");
    if (grupo === null) return;
    try { await atribuirGrupo(aba.id, grupo); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (acaoAba) { await executarAcaoMenuAba(acaoAba); return; }
  if (evento.payload === "canoa_principal_configuracoes") { fecharMenu(); await abrirConfiguracoes(); return; }
  if (evento.payload === "canoa_principal_tarefas") { fecharMenu(); await abrirTarefas(); return; }
  if (evento.payload === "canoa_principal_memoria") { fecharMenu(); await alternarIndicadorMemoria(); return; }
  if (evento.payload === "canoa_limpar_cookies_site") {
    if (!window.confirm("Remover cookies correspondentes à página atual? Isso pode encerrar a sessão neste site.")) return;
    try { await invoke("limpar_cookies_pagina"); statusLimpezaDados.textContent = "Limpeza solicitada para a página atual…"; }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (evento.payload === "canoa_principal_baixados") { fecharMenuAbas(); await abrirConfiguracoes("downloads"); return; }
  if (evento.payload === "canoa_acesso_artigo") { await abrirAcessoArtigo(); return; }
  if (evento.payload === "canoa_privacidade_bloquear" || evento.payload === "canoa_privacidade_protecao") {
    const host = obterDominio(contextoMenuPagina?.endereco ?? abaContextual?.endereco ?? enderecoAtual);
    if (!host) return;
    try {
      if (evento.payload === "canoa_privacidade_bloquear") {
        const config = await invoke<Configuracoes>("alternar_site_bloqueado", { dominio: host });
        aplicarConfiguracoes(config);
        const bloqueado = dominioEstaNaLista(host, config.sites_bloqueados ?? []);
        notificar(bloqueado ? `Bloqueio ativo para ${host}; vale para as próximas navegações` : `Bloqueio removido de ${host}`);
      } else {
        const config = await invoke<Configuracoes>("alternar_excecao_bloqueador", { dominio: host });
        aplicarConfiguracoes(config);
        const pausada = dominioEstaNaLista(host, config.excecoes_bloqueador ?? []);
        notificar(pausada ? `Proteção pausada em ${host}` : `Proteção retomada em ${host}`);
        if (obterDominio(enderecoAtual) === host) await invoke("recarregar");
      }
    } catch (erro) { informar(String(erro), true); }
    return;
  }
  if (evento.payload === "canoa_principal_ler_depois") {
    try { aplicarConfiguracoes(await invoke<Configuracoes>("salvar_para_leitura")); notificar("Adicionado à Lista de leitura"); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (evento.payload === "canoa_principal_salvar_pagina") {
    try { aplicarConfiguracoes(await invoke<Configuracoes>("salvar_pagina_em_salvos")); notificar("Link da página guardado em Baixados"); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (evento.payload === "canoa_principal_baixar_recursos") {
    try { await invoke("solicitar_recursos_pagina"); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (evento.payload === "canoa_pagina_favorito") { await alternarFavorito(); return; }
  if (evento.payload === "canoa_pagina_fontes") {
    await abrirConfiguracoes("pesquisa");
    const campo = document.querySelector<HTMLInputElement>("#fonte-dominio")!;
    try { campo.value = new URL(contextoMenuPagina?.endereco ?? enderecoAtual ?? "").hostname; } catch { campo.value = ""; }
    campo.focus();
    return;
  }
  if (evento.payload === "canoa_pagina_abrir_link" || evento.payload === "canoa_pagina_abrir_imagem") {
    const alvo = evento.payload.endsWith("imagem") ? contextoMenuPagina?.imagem : contextoMenuPagina?.link;
    if (!alvo) return;
    void invoke<Retrato>("abrir_link_nova_aba", { endereco: alvo }).then(desenhar).catch((erro) => informar(String(erro), true));
    return;
  }
  if (evento.payload === "canoa_pagina_copiar_texto") {
    try { await invoke("copiar_texto_contexto"); notificar("Texto copiado"); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (evento.payload === "canoa_pagina_voltar") { void invoke("voltar").catch((erro) => informar(String(erro), true)); return; }
  if (evento.payload === "canoa_pagina_avancar") { void invoke("avancar").catch((erro) => informar(String(erro), true)); return; }
  if (evento.payload === "canoa_pagina_salvar_como") {
    try { await invoke("solicitar_recursos_pagina"); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (evento.payload === "canoa_pagina_copiar_link") {
    try { await invoke("copiar_endereco_atual"); notificar("Link da página copiado"); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
  if (evento.payload === "canoa_pagina_recarregar") { void invoke("recarregar").catch((erro) => informar(String(erro), true)); return; }
  if (evento.payload === "canoa_pagina_mudo") { void invoke<Retrato>("alternar_mudo_atual").then(desenhar).catch((erro) => informar(String(erro), true)); return; }
  if (evento.payload === "canoa_pagina_imprimir") { void invoke("imprimir_pagina").catch((erro) => informar(String(erro), true)); return; }
  if (evento.payload === "canoa_principal_copiar_link") {
    try { await invoke("copiar_endereco_atual"); informar("Link da página copiado"); }
    catch (erro) { informar(String(erro), true); }
    return;
  }
});
void listen<string>("menu-contexto-pagina", (evento) => {
  try {
    const contexto = JSON.parse(evento.payload) as ContextoMenuPagina;
    if (!/^https?:\/\//i.test(contexto.endereco)) return;
    contextoMenuPagina = contexto;
    const deslocamento = 52 + (document.body.classList.contains("multiplas") ? 32 : 0) + (aviso.hidden ? 0 : 40);
    void invoke("abrir_menu_nativo", {
      tipo: "pagina", x: contexto.x, y: contexto.y + deslocamento,
      podeFecharOutras: false, podeFecharDireita: false, temEndereco: true,
      temLink: !!contexto.link, temImagem: !!contexto.imagem,
      favorito: favoritos.some((item) => item.endereco === contexto.endereco),
      mutada: abasAbertas.find((aba) => aba.id === idAbaAnterior)?.mutada ?? false,
      textoSelecionado: contexto.texto,
      grupoAtual: null,
      siteBloqueado: (() => { const host = obterDominio(contexto.endereco); return !!host && dominioEstaNaLista(host, sitesBloqueadosAtuais); })(),
      protecaoPausada: (() => { const host = obterDominio(contexto.endereco); return !!host && dominioEstaNaLista(host, excecoesBloqueador); })(),
    }).catch((erro) => informar(String(erro), true));
  } catch (erro) { informar(`Não foi possível abrir o menu da página: ${String(erro)}`, true); }
});
async function abrirAcessoArtigo() {
  const atual = enderecoAtual;
  if (!atual || !/^https?:\/\//i.test(atual)) return;
  const dialogo = document.querySelector<HTMLDialogElement>("#dialogo-acesso-artigo")!;
  const titulo = document.querySelector<HTMLElement>("#acesso-artigo-titulo")!;
  const resumo = document.querySelector<HTMLElement>("#acesso-artigo-resumo")!;
  const sinais = document.querySelector<HTMLUListElement>("#acesso-artigo-sinais")!;
  const abrir = document.querySelector<HTMLButtonElement>("#acesso-artigo-abrir")!;
  const status = acessoArtigo?.endereco === atual ? acessoArtigo : null;
  let origem = atual;
  try { origem = new URL(atual).origin; } catch {}
  const linkAcesso = status?.links.find((link) => { try { return new URL(link).origin === origem; } catch { return false; } });
  titulo.textContent = status?.titulo || "Acesso ao artigo";
  resumo.textContent = status?.restrito
    ? "Há sinais de que esta página pode exigir assinatura ou autenticação. A detecção é indicativa."
    : status
      ? "Não encontramos sinais comuns de restrição nesta página. Isso não garante que o conteúdo esteja livre."
      : "A página ainda está sendo verificada. Você pode abrir o site ou guardar o artigo enquanto isso.";
  sinais.replaceChildren();
  for (const texto of status?.sinais ?? []) { const item = document.createElement("li"); item.textContent = texto; sinais.append(item); }
  abrir.hidden = !linkAcesso;
  abrir.dataset.endereco = linkAcesso ?? "";
  const site = document.querySelector<HTMLButtonElement>("#acesso-artigo-site")!;
  site.dataset.endereco = origem;
  if (!dialogo.open) {
    dialogo.showModal();
    if (enderecoAtual) {
      try { await invoke("mostrar_conteudo_pagina", { mostrar: false }); }
      catch (erro) { informar(`Não foi possível exibir o painel de acesso: ${String(erro)}`, true); }
    }
  }
}
document.querySelector<HTMLButtonElement>("#acesso-artigo-abrir")!.addEventListener("click", async (evento) => {
  const alvo = (evento.currentTarget as HTMLButtonElement).dataset.endereco;
  if (!alvo || !enderecoAtual) return;
  try {
    if (new URL(alvo).origin !== new URL(enderecoAtual).origin) throw new Error("O link precisa pertencer ao mesmo site do artigo.");
    const retrato = await invoke<Retrato>("abrir_link_nova_aba", { endereco: alvo });
    desenhar(retrato);
    document.querySelector<HTMLDialogElement>("#dialogo-acesso-artigo")!.close();
  } catch (erro) { informar(String(erro), true); }
});
document.querySelector<HTMLButtonElement>("#acesso-artigo-site")!.addEventListener("click", async (evento) => {
  const alvo = (evento.currentTarget as HTMLButtonElement).dataset.endereco;
  if (!alvo) return;
  try {
    const retrato = await invoke<Retrato>("abrir_link_nova_aba", { endereco: alvo });
    desenhar(retrato);
    document.querySelector<HTMLDialogElement>("#dialogo-acesso-artigo")!.close();
  } catch (erro) { informar(String(erro), true); }
});
document.querySelector<HTMLButtonElement>("#acesso-artigo-leitura")!.addEventListener("click", async () => {
  try {
    aplicarConfiguracoes(await invoke<Configuracoes>("salvar_para_leitura"));
    document.querySelector<HTMLDialogElement>("#dialogo-acesso-artigo")!.close();
    notificar("Artigo salvo na Lista de leitura");
  } catch (erro) { informar(String(erro), true); }
});
void listen<string>("acesso-artigo-detectado", (evento) => {
  try {
    const pacote = JSON.parse(evento.payload) as AcessoArtigo;
    if (!/^https?:\/\//i.test(pacote.endereco) || pacote.endereco !== enderecoAtual) return;
    acessoArtigo = { ...pacote, sinais: Array.isArray(pacote.sinais) ? pacote.sinais.slice(0, 4) : [], links: Array.isArray(pacote.links) ? pacote.links.slice(0, 4) : [] };
  } catch { /* pacote de verificação inválido */ }
});
document.querySelector<HTMLDialogElement>("#dialogo-acesso-artigo")!.addEventListener("close", () => {
  if (enderecoAtual) void invoke("mostrar_conteudo_pagina", { mostrar: true }).catch((erro) => informar(String(erro), true));
});
document.querySelector<HTMLButtonElement>("#fechar-acesso-artigo")!.addEventListener("click", () => document.querySelector<HTMLDialogElement>("#dialogo-acesso-artigo")!.close());
void listen<string>("recursos-pagina", async (evento) => {
  try {
    const pacote = JSON.parse(evento.payload) as PacoteRecursosPagina;
    if (!/^https?:\/\//i.test(pacote.endereco) || !Array.isArray(pacote.itens)) return;
    filtroTipoRecursos = "todos";
    tituloRecursosPagina = new URL(pacote.endereco).hostname || "pagina";
    const combinados = new Map<string, RecursoPagina>();
    for (const item of pacote.itens) {
      if (!/^https?:\/\//i.test(item.endereco) || !["imagem", "video", "audio", "documento"].includes(item.tipo)) continue;
      if (!combinados.has(item.endereco)) combinados.set(item.endereco, item);
      if (combinados.size >= 80) break;
    }
    recursosPagina = Array.from(combinados.values());
    const rotulos: Record<RecursoPagina["tipo"], string> = { imagem: "Imagem", video: "Vídeo", audio: "Áudio", documento: "Documento" };
    const filtros = document.querySelector<HTMLElement>("#recursos-filtros")!;
    filtros.replaceChildren();
    const tipos = ["todos", ...new Set(recursosPagina.map((item) => item.tipo))];
    for (const tipoFiltro of tipos) {
      const quantidade = tipoFiltro === "todos" ? recursosPagina.length : recursosPagina.filter((item) => item.tipo === tipoFiltro).length;
      const botao = document.createElement("button"); botao.type = "button"; botao.className = "recurso-filtro";
      botao.dataset.tipo = tipoFiltro;
      botao.textContent = `${tipoFiltro === "todos" ? "Todos" : rotulos[tipoFiltro as RecursoPagina["tipo"]]} (${quantidade})`;
      botao.setAttribute("aria-pressed", String(filtroTipoRecursos === tipoFiltro));
      botao.addEventListener("click", () => { filtroTipoRecursos = tipoFiltro; desenharRecursosPagina(); });
      filtros.append(botao);
    }
    document.querySelector<HTMLElement>("#recursos-resumo")!.textContent = `${recursosPagina.length} recurso(s) encontrado(s).`;
    document.querySelector<HTMLElement>("#recursos-vazio")!.hidden = recursosPagina.length > 0;
    desenharRecursosPagina();
    document.querySelector<HTMLElement>("#recursos-titulo")!.textContent = pacote.titulo ? `Itens de ${pacote.titulo}` : "Itens desta página";
    const dialogo = document.querySelector<HTMLDialogElement>("#dialogo-recursos")!;
    if (!dialogo.open) {
      await invoke("mostrar_conteudo_pagina", { mostrar: false });
      dialogo.showModal();
    }
  } catch (erro) { informar(`Não foi possível listar os itens desta página: ${String(erro)}`, true); }
});
function desenharRecursosPagina() {
    const lista = document.querySelector<HTMLUListElement>("#recursos-lista")!;
    for (const botao of document.querySelectorAll<HTMLButtonElement>("#recursos-filtros .recurso-filtro")) {
      botao.setAttribute("aria-pressed", String(botao.dataset.tipo === filtroTipoRecursos));
    }
    lista.replaceChildren();
    const visiveis = recursosPagina.map((item, indice) => ({ item, indice })).filter(({ item }) => filtroTipoRecursos === "todos" || item.tipo === filtroTipoRecursos);
    document.querySelector<HTMLElement>("#recursos-sem-filtro")!.hidden = visiveis.length > 0 || recursosPagina.length === 0;
    const rotulos: Record<RecursoPagina["tipo"], string> = { imagem: "Imagem", video: "Vídeo", audio: "Áudio", documento: "Documento" };
    visiveis.forEach(({ item, indice }) => {
      const linha = document.createElement("li");
      const detalhes = document.createElement("div");
      const tipo = document.createElement("span"); tipo.className = "recurso-tipo"; tipo.textContent = rotulos[item.tipo];
      const nome = document.createElement("strong"); nome.textContent = item.nome || new URL(item.endereco).pathname.split("/").pop() || rotulos[item.tipo];
      const origem = document.createElement("small"); origem.textContent = new URL(item.endereco).hostname;
      detalhes.append(tipo, nome, origem);
      const baixar = document.createElement("button"); baixar.type = "button"; baixar.textContent = "Baixar";
      baixar.addEventListener("click", async () => {
        baixar.disabled = true;
        try { await invoke("baixar_item_pagina", { endereco: item.endereco, nome: item.nome || `recurso-${indice + 1}`, tipo: item.tipo }); notificar("Download iniciado"); }
        catch (erro) { informar(String(erro), true); baixar.disabled = false; }
      });
      linha.append(detalhes, baixar); lista.append(linha);
    });
}
document.querySelector<HTMLDialogElement>("#dialogo-recursos")!.addEventListener("close", () => {
    void invoke("mostrar_conteudo_pagina", { mostrar: true }).catch((erro) => console.debug("A página ativa já foi trocada ao fechar o seletor de recursos.", erro));
});
for (const botao of document.querySelectorAll<HTMLButtonElement>("[data-baixar-pagina]")) {
  botao.addEventListener("click", async () => {
    try {
      await invoke("baixar_item_pagina", { endereco: "", nome: tituloRecursosPagina, tipo: botao.dataset.baixarPagina });
      notificar("Download iniciado");
    } catch (erro) { informar(String(erro), true); }
  });
}
document.querySelector<HTMLButtonElement>("#abrir-central-baixados")!.addEventListener("click", () => {
  document.querySelector<HTMLDialogElement>("#dialogo-recursos")!.close();
  void abrirConfiguracoes("downloads");
});
document.querySelector<HTMLDialogElement>("#dialogo-leitura-offline")!.addEventListener("close", () => {
  document.querySelector<HTMLIFrameElement>("#leitura-offline-frame")!.srcdoc = "";
});
document.querySelector<HTMLButtonElement>("#imprimir-pagina")!.addEventListener("click", () => {
  document.querySelector<HTMLDialogElement>("#dialogo-recursos")!.close();
  void invoke("imprimir_pagina").catch((erro) => informar(String(erro), true));
});
document.querySelector<HTMLButtonElement>("#fechar-configuracoes")!.addEventListener("click", () => void fecharConfiguracoes());
for (const botao of document.querySelectorAll<HTMLButtonElement>("[data-ir-secao]")) {
  botao.addEventListener("click", () => {
    void invoke<Retrato>("secao_configuracoes", { secao: botao.dataset.irSecao ?? "" })
      .then(desenhar).catch((erro) => informar(String(erro), true));
  });
}
controleMemoria.addEventListener("change", async () => {
  controleMemoria.disabled = true;
  try { aplicarConfiguracoes(await invoke<Configuracoes>("definir_memoria", { mostrar: controleMemoria.checked })); }
  catch (erro) { controleMemoria.checked = mostrarMemoria; informar(String(erro), true); }
  finally { controleMemoria.disabled = false; }
});
for (const id of ["config-barra-favorito", "config-barra-protecao", "config-barra-leitura", "config-barra-baixar"]) {
  document.querySelector<HTMLInputElement>(`#${id}`)!.addEventListener("change", async () => {
    const acoes_barra: AcoesBarra = {
      favorito: document.querySelector<HTMLInputElement>("#config-barra-favorito")!.checked,
      protecao: document.querySelector<HTMLInputElement>("#config-barra-protecao")!.checked,
      leitura: document.querySelector<HTMLInputElement>("#config-barra-leitura")!.checked,
      baixar: document.querySelector<HTMLInputElement>("#config-barra-baixar")!.checked,
    };
    try { aplicarConfiguracoes(await invoke<Configuracoes>("definir_acoes_barra", { acoesBarra: acoes_barra })); }
    catch (erro) { void invoke<Configuracoes>("ler_configuracoes").then(aplicarConfiguracoes); informar(String(erro), true); }
  });
}
async function alternarIndicadorMemoria() {
  try {
    aplicarConfiguracoes(await invoke<Configuracoes>("definir_memoria", { mostrar: !mostrarMemoria }));
  } catch (erro) { informar(String(erro), true); }
}
controleBuscador.addEventListener("change", async () => {
  controleBuscador.disabled = true;
  try {
    aplicarConfiguracoes(await invoke<Configuracoes>("definir_buscador", { buscador: controleBuscador.value }));
    notificar("Buscador padrão atualizado");
  } catch (erro) {
    try { aplicarConfiguracoes(await invoke<Configuracoes>("ler_configuracoes")); } catch { /* O erro original é mostrado abaixo. */ }
    informar(String(erro), true);
  } finally { controleBuscador.disabled = false; }
});
document.querySelector<HTMLFormElement>("#adicionar-fonte")!.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  const dominio = document.querySelector<HTMLInputElement>("#fonte-dominio")!;
  const tema = document.querySelector<HTMLInputElement>("#fonte-tema")!;
  try {
    aplicarConfiguracoes(await invoke<Configuracoes>("adicionar_fonte", { dominio: dominio.value, tema: tema.value }));
    dominio.value = "";
    dominio.focus();
  } catch (erro) { informar(String(erro), true); }
});
document.querySelector<HTMLButtonElement>("#config-escolher-pasta")!.addEventListener("click", async () => {
  try {
    const valor = await invoke<Configuracoes | null>("escolher_pasta_downloads");
    if (valor) { aplicarConfiguracoes(valor); notificar("Pasta de downloads atualizada"); }
  } catch (erro) { informar(String(erro), true); }
});
document.querySelector<HTMLButtonElement>("#config-pasta-padrao")!.addEventListener("click", async () => {
  try { aplicarConfiguracoes(await invoke<Configuracoes>("pasta_padrao")); notificar("Pasta padrão restaurada"); }
  catch (erro) { informar(String(erro), true); }
});
document.querySelector<HTMLButtonElement>("#config-abrir-pasta")!.addEventListener("click", () => void abrirDownloads());
document.querySelector<HTMLButtonElement>("#limpar-downloads")!.addEventListener("click", async () => {
  try { aplicarConfiguracoes(await invoke<Configuracoes>("limpar_historico_downloads")); notificar("Histórico de downloads limpo; arquivos mantidos"); }
  catch (erro) { informar(String(erro), true); }
});
controleHistorico.addEventListener("change", async () => {
  try { aplicarConfiguracoes(await invoke<Configuracoes>("definir_historico_navegacao", { ativo: controleHistorico.checked, dias: Number(diasHistorico.value) })); }
  catch (erro) { informar(String(erro), true); }
});
diasHistorico.addEventListener("change", async () => {
  try { aplicarConfiguracoes(await invoke<Configuracoes>("definir_historico_navegacao", { ativo: controleHistorico.checked, dias: Number(diasHistorico.value) })); }
  catch (erro) { informar(String(erro), true); }
});
buscaHistorico.addEventListener("input", () => { void invoke<Configuracoes>("ler_configuracoes").then(desenharHistorico).catch((erro) => informar(String(erro), true)); });
document.querySelector<HTMLButtonElement>("#limpar-historico-navegacao")!.addEventListener("click", async () => {
  if (!window.confirm("Apagar todo o histórico de navegação salvo neste perfil?")) return;
  try { aplicarConfiguracoes(await invoke<Configuracoes>("limpar_historico_navegacao")); notificar("Histórico de navegação apagado"); }
  catch (erro) { informar(String(erro), true); }
});
formularioLimiteSite.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  const dominio = document.querySelector<HTMLInputElement>("#limite-site-dominio")!;
  const minutos = document.querySelector<HTMLInputElement>("#limite-site-minutos")!;
  try { aplicarConfiguracoes(await invoke<Configuracoes>("definir_limite_site", { dominio: dominio.value, minutos: Number(minutos.value) })); dominio.value = ""; notificar("Limite diário salvo"); }
  catch (erro) { informar(String(erro), true); }
});
formularioSiteFoco.addEventListener("submit", async (evento) => {
  evento.preventDefault();
  const campo = document.querySelector<HTMLInputElement>("#site-foco-dominio")!;
  try { aplicarConfiguracoes(await invoke<Configuracoes>("definir_sites_foco", { dominio: campo.value, adicionar: true })); campo.value = ""; }
  catch (erro) { informar(String(erro), true); }
});
duracaoFoco.addEventListener("change", async () => {
  try { aplicarConfiguracoes(await invoke<Configuracoes>("definir_duracao_foco", { minutos: Number(duracaoFoco.value) })); }
  catch (erro) { informar(String(erro), true); }
});
document.querySelector<HTMLButtonElement>("#alternar-foco")!.addEventListener("click", async () => {
  try { aplicarConfiguracoes(await invoke<Configuracoes>("alternar_modo_foco")); }
  catch (erro) { informar(String(erro), true); }
});
document.querySelector<HTMLButtonElement>("#limpar-cookies-site")!.addEventListener("click", async () => {
  if (!window.confirm("Remover os cookies associados à página atual? Isso pode encerrar sua sessão nesse site. O cache e outros dados do site serão mantidos.")) return;
  try { await invoke("limpar_cookies_pagina"); statusLimpezaDados.textContent = "Limpeza solicitada para a página atual…"; }
  catch (erro) { informar(String(erro), true); }
});
document.querySelector<HTMLButtonElement>("#limpar-cookies-cache")!.addEventListener("click", async () => {
  if (!window.confirm("Remover cookies e cache de todos os sites neste perfil do Canoa? Isso pode desconectar você dos sites abertos.")) return;
  try { await invoke("limpar_cookies_e_cache"); statusLimpezaDados.textContent = "Limpeza solicitada para o perfil…"; }
  catch (erro) { informar(String(erro), true); }
});
desenharDownloadsRecentes();
document.querySelector<HTMLButtonElement>("#aviso-fechar")!.addEventListener("click", fecharAviso);
avisoAcao.addEventListener("click", () => void abrirDownloads());
function executarAtalho(acao: string) {
  if (acao === "tarefas") { void abrirTarefas(); return; }
  if (acao === "configuracoes") { void abrirConfiguracoes(); return; }
  if (acao === "historico") { void abrirConfiguracoes("historico"); return; }
  if (acao === "alternar_foco") { void invoke<Configuracoes>("alternar_modo_foco").then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true)); return; }
  if (acao === "alternar_memoria") { void alternarIndicadorMemoria(); return; }
  if (acao === "alternar_bloqueio_site") { void alternarBloqueioSiteAtual(); return; }
  if (acao === "mutar_pagina") { void invoke<Retrato>("alternar_mudo_atual").then(desenhar).catch((erro) => informar(String(erro), true)); return; }
  if (acao === "grupar_aba") {
    void invoke<Retrato>("listar_abas").then(async (retrato) => {
      const aba = retrato.abas.find((item) => item.id === retrato.ativa);
      if (!aba) return;
      const nome = window.prompt("Nome do grupo de abas (até 32 caracteres)", aba.grupo ?? "");
      if (nome !== null) await atribuirGrupo(aba.id, nome);
    }).catch((erro) => informar(String(erro), true));
    return;
  }
  if (acao === "baixar_recursos") {
    void invoke("solicitar_recursos_pagina").catch((erro) => informar(String(erro), true));
    return;
  }
  if (acao === "reabrir_aba") { void invoke<Retrato>("reabrir_aba").then(desenhar).catch((erro) => informar(String(erro), true)); return; }
  if (acao === "menu_contexto_aba") {
    void invoke<Retrato>("listar_abas").then((retrato) => {
      const aba = retrato.abas.find((item) => item.id === retrato.ativa);
      const titulo = lista.querySelector<HTMLButtonElement>(".aba-titulo[aria-current='page']");
      if (!aba || !titulo) return;
      const caixa = titulo.getBoundingClientRect();
      abrirMenuAba(caixa.left, caixa.bottom, aba, retrato.abas);
    }).catch((erro) => informar(String(erro), true));
    return;
  }
  if (acao === "mover_aba_anterior" || acao === "mover_aba_proxima") {
    void invoke<Retrato>("listar_abas").then((retrato) => {
      const atual = retrato.abas.findIndex((aba) => aba.id === retrato.ativa);
      const destino = Math.max(0, Math.min(retrato.abas.length - 1, atual + (acao === "mover_aba_anterior" ? -1 : 1)));
      if (atual !== destino) return invoke<Retrato>("mover_aba", { id: retrato.ativa, destino }).then(desenhar);
      return undefined;
    }).catch((erro) => informar(String(erro), true));
    return;
  }
  if (acao === "endereco") { void focarEndereco(); return; }
  if (acao === "favorito") { void alternarFavorito(); return; }
  if (acao === "alternar_protecao_site") { void alternarExcecaoProtecao(); return; }
  if (acao === "acesso_artigo") { void abrirAcessoArtigo(); return; }
  if (acao === "ver_favoritos") {
    void invoke<Retrato>("nova_aba").then((valor) => {
      desenhar(valor);
      document.querySelector<HTMLElement>("#favoritos-titulo")!.focus();
    }).catch((erro) => informar(String(erro), true));
    return;
  }
  if (acao === "proxima_aba" || acao === "aba_anterior" || /^aba_[1-9]$/.test(acao)) {
    void invoke<Retrato>("listar_abas").then((retrato) => {
      const atual = retrato.abas.findIndex((aba) => aba.id === retrato.ativa);
      const numero = /^aba_([1-9])$/.exec(acao);
      const destino = numero
        ? (numero[1] === "9" ? retrato.abas.length - 1 : Math.min(Number(numero[1]) - 1, retrato.abas.length - 1))
        : (atual + (acao === "proxima_aba" ? 1 : -1) + retrato.abas.length) % retrato.abas.length;
      if (destino !== atual) void acionar("trocar_aba", { id: retrato.abas[destino].id }, retrato.ativa);
    }).catch((erro) => informar(String(erro), true));
    return;
  }
  switch (acao) {
    case "nova_aba": void acionar("nova_aba"); break;
    case "fechar_aba": void invoke<Retrato>("listar_abas").then((retrato) => acionar("fechar_aba", { id: retrato.ativa })).catch((erro) => informar(String(erro), true)); break;
    case "endereco": void focarEndereco(); break;
    case "recarregar": case "voltar": case "avancar":
      if (configurando) { if (acao === "recarregar") void carregarDetalhesConfiguracoes(); }
      else void invoke(acao).catch((erro) => informar(String(erro), true));
      break;
    case "downloads": void abrirDownloads(); break;
  }
}
void listen<void>("historico-atualizado", () => { if (configurando) void invoke<Configuracoes>("ler_configuracoes").then(aplicarConfiguracoes).catch(() => {}); });
void listen<ResultadoLimpeza>("limpeza-dados-concluida", (evento) => {
  statusLimpezaDados.textContent = evento.payload.mensagem;
  if (evento.payload.sucesso) notificar(evento.payload.mensagem); else notificar(evento.payload.mensagem, "erro");
});
void listen<string>("site-bloqueado-foco", (evento) => informar(`${evento.payload} está bloqueado durante o modo foco.`));
void listen<string>("site-limite-atingido", (evento) => informar(`Limite diário atingido para ${evento.payload}.`, true));
void listen<ResumoProtecao>("contador-bloqueador-atualizado", (evento) => desenharResumoProtecao(evento.payload));
setInterval(async () => {
  const faltam = focoAteEpoch ? focoAteEpoch - Math.floor(Date.now() / 1000) : 0;
  const botaoFoco = document.querySelector<HTMLButtonElement>("#alternar-foco");
  if (botaoFoco) {
    const ativo = faltam > 0;
    estadoFoco.textContent = ativo ? `Modo foco ativo — ${Math.ceil(faltam / 60)} min restantes.` : "Modo foco inativo.";
    botaoFoco.textContent = ativo ? "Encerrar modo foco" : "Iniciar modo foco (Ctrl+Shift+F)";
  }
  if (document.visibilityState !== "visible" || !document.hasFocus() || configurando || !enderecoAtual) return;
  const dominio = obterDominio(enderecoAtual);
  if (!dominio) return;
  try {
    const config = await invoke<Configuracoes>("ler_configuracoes");
    if (!(config.limites_sites ?? []).some((item) => dominioEstaNaLista(dominio, [item.dominio]))) return;
    const estado = await invoke<EstadoUsoSite | null>("registrar_tempo_site", { dominio, segundos: 15 });
    if (estado?.atingido) informar(`Limite diário atingido para ${dominio}.`, true);
    else if (estado) { estadoFoco.textContent = `${dominio}: ${Math.floor(estado.segundos / 60)}/${Math.floor(estado.limite_segundos / 60)} min hoje.`; }
  } catch { /* A navegação pode ter trocado enquanto a medição era enviada. */ }
}, 15_000);
async function alternarBloqueioSiteAtual() {
  try {
    const config = await invoke<Configuracoes>("alternar_bloqueio_site_atual");
    aplicarConfiguracoes(config);
    const host = enderecoAtual ? new URL(enderecoAtual).hostname : "site";
    const bloqueado = (config.sites_bloqueados ?? []).some((item) => host === item || host.endsWith(`.${item}`));
    notificar(bloqueado ? `${host} foi bloqueado` : `Bloqueio removido de ${host}`);
    if (bloqueado) await invoke("nova_aba");
  } catch (erro) { informar(String(erro), true); }
}
function obterDominio(url: string | null | undefined): string | null {
  try { return url && /^https?:\/\//i.test(url) ? new URL(url).hostname.toLowerCase() : null; } catch { return null; }
}
function dominioEstaNaLista(host: string, dominios: string[]): boolean {
  return dominios.some((dominio) => host === dominio || host.endsWith(`.${dominio}`));
}
document.addEventListener("keydown", (evento) => {
  if (!evento.repeat && !evento.ctrlKey && !evento.altKey && evento.shiftKey && evento.key.toLowerCase() === "f10") {
    const alvo = evento.target as HTMLElement;
    if (alvo.closest("input, textarea, [contenteditable='true']")) return;
    evento.preventDefault();
    executarAtalho("menu_contexto_aba");
    return;
  }
  if (!evento.ctrlKey && !evento.altKey && evento.shiftKey && evento.key === "Escape") {
    evento.preventDefault(); void abrirTarefas(); return;
  }
  if (evento.repeat) return;
  const tecla = evento.key.toLowerCase();
  if (evento.ctrlKey && evento.altKey && !evento.shiftKey) {
    if (tecla === "s") { evento.preventDefault(); void abrirConfiguracoes("downloads"); return; }
    if (tecla === "k") { evento.preventDefault(); void abrirConfiguracoes("senhas"); return; }
    const colecao = ({ r: "leitura", f: "fontes" } as Record<string, string>)[tecla];
    if (colecao) {
      evento.preventDefault();
      void invoke<Retrato>("nova_aba").then((retrato) => { desenhar(retrato); ativarColecao(colecao); }).catch((erro) => informar(String(erro), true));
      return;
    }
  }
  if (evento.ctrlKey && !evento.altKey && !evento.shiftKey && tecla === "h") { evento.preventDefault(); executarAtalho("historico"); return; }
  if (evento.ctrlKey && evento.shiftKey && !evento.altKey && tecla === "f") { evento.preventDefault(); executarAtalho("alternar_foco"); return; }
  let acao = "";
  if (evento.ctrlKey && !evento.altKey && !evento.shiftKey) {
    acao = ({ t: "nova_aba", w: "fechar_aba", l: "endereco", r: "recarregar", j: "downloads", d: "favorito", ",": "configuracoes" } as Record<string, string>)[tecla] ?? "";
    if (tecla === "tab" || tecla === "pagedown") acao = "proxima_aba";
    if (tecla === "pageup") acao = "aba_anterior";
    if (/^[1-9]$/.test(tecla)) acao = `aba_${tecla}`;
    if (tecla === "f4") acao = "fechar_aba";
  } else if (evento.ctrlKey && evento.shiftKey && !evento.altKey) {
    acao = tecla === "tab" ? "aba_anterior" : tecla === "pageup" ? "mover_aba_anterior" : tecla === "pagedown" ? "mover_aba_proxima" : tecla === "o" ? "ver_favoritos" : tecla === "m" ? "alternar_memoria" : tecla === "t" ? "reabrir_aba" : tecla === "s" ? "baixar_recursos" : tecla === "g" ? "grupar_aba" : "";
  } else if (evento.altKey && !evento.ctrlKey && !evento.shiftKey) {
    acao = tecla === "arrowleft" ? "voltar" : tecla === "arrowright" ? "avancar" : tecla === "d" ? "endereco" : "";
  } else if (evento.ctrlKey && evento.altKey && !evento.shiftKey) {
    acao = tecla === "a" ? "acesso_artigo" : tecla === "p" ? "alternar_protecao_site" : tecla === "b" ? "alternar_bloqueio_site" : tecla === "m" ? "mutar_pagina" : "";
  } else if (tecla === "f5" && !evento.ctrlKey && !evento.altKey && !evento.shiftKey) {
    acao = "recarregar";
  }
  if (acao) { evento.preventDefault(); executarAtalho(acao); }
});
void listen<string>("atalho", (evento) => executarAtalho(evento.payload));
void listen<string>("site-bloqueado", (evento) => informar(`Navegação bloqueada para ${evento.payload}. Gerencie a lista em Configurações > Privacidade.`, true));
void listen<string>("limpar-url-rastreamento", (evento) => { void invoke<string>("navegar", { entrada: evento.payload }).catch((erro) => informar(String(erro), true)); });

void listen<Retrato>("abas-atualizadas", (evento) => desenhar(evento.payload));
void listen<string>("abrir-link-nova-aba", (evento) => {
  void invoke<Retrato>("abrir_link_nova_aba", { endereco: evento.payload })
    .then(desenhar).catch((erro) => informar(String(erro), true));
});
void listen<void>("biblioteca-atualizada", () => {
  void invoke<Configuracoes>("ler_configuracoes").then(aplicarConfiguracoes).catch((erro) => informar(String(erro), true));
});
void listen<string>("pagina-iniciada", (evento) => {
  if (enderecoAtual !== evento.payload) acessoArtigo = null;
  endereco.value = evento.payload;
  informar("Carregando…");
  void atualizarResumoProtecao();
});
void listen<string>("pagina-endereco", (evento) => {
  if (enderecoAtual !== evento.payload) acessoArtigo = null;
  if (document.activeElement !== endereco) endereco.value = evento.payload;
  enderecoAtual = evento.payload;
  desenharProtecao();
  botaoSalvarLeitura.disabled = configurando || !/^https?:\/\//i.test(evento.payload);
  desenharFavoritos();
});
void listen<string>("pagina-encerrada", (evento) => {
  endereco.value = evento.payload;
  informar("Pronto");
  void atualizarMemoria();
});
void listen<string>("pagina-erro", (evento) => {
  endereco.value = evento.payload;
  informar("Não foi possível abrir a página", true);
});
void listen<DownloadAtivo>("download-ativo", (evento) => {
  downloadsAtivos.set(evento.payload.id, evento.payload);
  desenharDownloadsAtivos();
  avisarDownload(`Baixando: ${evento.payload.nome}`);
});
void listen<DownloadAtivo>("download-progresso", (evento) => {
  downloadsAtivos.set(evento.payload.id, evento.payload);
  desenharDownloadsAtivos();
});
void listen<DownloadFinalizado>("download-finalizado", (evento) => {
  downloadsAtivos.delete(evento.payload.id);
  desenharDownloadsAtivos();
  const { nome, sucesso } = evento.payload;
  void invoke<Configuracoes>("ler_configuracoes").then(aplicarConfiguracoes).catch((erro) => console.warn("Falha ao atualizar histórico de downloads", erro));
  avisarDownload(sucesso ? `Salvo: ${nome}` : `Falha ao baixar: ${nome}`, !sucesso);
});
void listen<string>("download-pasta-falha", (evento) => notificar(evento.payload, "erro"));

const detalharErroInicializacao = (erro: unknown) => erro instanceof Error && erro.stack
  ? erro.stack.split("\n").slice(0, 3).join(" · ")
  : String(erro);
void invoke<Retrato>("listar_abas").then(desenhar).catch((erro) => informar(detalharErroInicializacao(erro), true));
void invoke<Configuracoes>("ler_configuracoes").then(aplicarConfiguracoes).catch((erro) => informar(detalharErroInicializacao(erro), true));
void atualizarResumoProtecao();
void carregarDownloadsAtivos();
void atualizarMemoria();
window.setInterval(() => void atualizarMemoria(), 15000);
document.addEventListener("visibilitychange", () => {
  if (document.visibilityState === "visible") void atualizarMemoria();
});
