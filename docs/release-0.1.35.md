# Canoa 0.1.35 — escopo, reconstrução e validação

## Relação com a 0.1.28

A 0.1.28 é a última versão em que a busca comum por Enter foi observada funcionando. Os executáveis dessa versão foram preservados localmente, com estes SHA-256:

- Instalador: `91EA39E29D824D9177877E0282D71276A4941B4314BC45E8FFA20885BA5E5E40`
- Portátil: `8920A0B2777CBAAD106F4B013A51DDDF848DA29B5AA7186C628D6256E2E00E96`

Não foi encontrado snapshot-fonte 0.1.28 nem commit/tag correspondente no Git. O histórico do repositório remoto tinha código apenas até 0.1.6; a fonte local disponível chegou a 0.1.34. Portanto, a 0.1.35 é uma reconstrução seletiva guiada pelo comportamento e pela documentação da 0.1.28, não uma cópia exata da fonte daquela build.

## O que foi excluído da linha 0.1.35

- A WebView separada usada pela linha 0.1.29–0.1.33 para desenhar o painel de sugestões. O painel volta a pertencer à interface principal; durante a exibição, a página WebView2 é temporariamente ocultada para que não cubra os resultados. Isso deve ser confirmado com testes visuais de clique e teclado.
- O observador de respostas de mídia e o resolvedor externo para HLS/DASH/YouTube experimentais da candidata 0.1.34. A central mantém downloads HTTP(S) comuns, recursos expostos no DOM, texto e captura local de leitura.
- O ZIP de referência do baixador de vídeos; não há licença de redistribuição confirmada.

## Regressão que motivou a reconstrução

A comparação visual documentada entre 0.1.28, 0.1.29 e 0.1.33 encontrou a primeira falha de busca comum na 0.1.29, depois da mudança para uma WebView irmã de sugestões. Na 0.1.33, a lista ainda aparecia, mas busca, `@`, `#`, Nova aba e Voltar não executavam consistentemente. O histórico disponível não permite fazer `git bisect` entre essas versões, então a relação com o painel é uma hipótese técnica apoiada pela sequência e pelos testes, não uma prova isolada.

## Validação desta reconstrução

| Verificação | Resultado nesta branch |
| --- | --- |
| Instalação frontend limpa (`npm ci`) | Passou; auditoria NPM informou 0 vulnerabilidades conhecidas |
| TypeScript/Vite e 166 IDs, após o último ajuste | Passou |
| `cargo fmt --all -- --check` | Passou |
| `cargo test --locked --lib` | 38 passaram, 0 falharam |
| Compilação Windows/NSIS e portátil | Passou com `scripts/gerar-instalador.ps1` |
| Smoke test portátil: abertura e fechamento da janela | Passou; abriu janela principal e fechou com código 0 |
| Execução em tela: busca normal pelo DuckDuckGo | Passou; termo de teste não sensível exibiu resultados |
| `@` na Nova aba e numa página web | Passou; categoria e URL foram selecionadas por clique |
| `#` na Nova aba e numa página web | Passou; Enter abriu Configurações: Visão geral |
| Escape após abrir sugestões | Passou; a página carregada reapareceu |
| Voltar e Avançar | Passaram após criar navegações distintas; URL e conteúdo acompanharam o histórico |
| Janela e uma única aba | Passaram; abertura inicial mostrou uma aba e a janela encerrou normalmente |
| Acessibilidade com leitor de tela e varredura completa de teclado | Ainda pendente |
| Ponto de UX conhecido | A WebView da página fica temporariamente oculta enquanto sugestões aparecem; Escape/seleção restaura o conteúdo |
| Assinatura Authenticode e updater automático | Não incluídos nesta prévia |

## Pacotes Windows gerados localmente

| Arquivo | SHA-256 |
| --- | --- |
| `Canoa-0.1.35-Windows-instalador.exe` | `1C1D863C221BA64EDE8A6779916C099585E9984D1AAF3A9D27A6526173025AA9` |
| `Canoa-0.1.35-Windows-portatil.exe` | `0588DBAD928A11E5EDE4EBCE18C23D1D08E7825D6AB8607C6D6C7630DDE27DCA` |

Publicar esta compilação somente como prévia de teste, identificando claramente que a validação de UI e acessibilidade ainda está pendente. Não anunciá-la como estável até concluir essa etapa.

## Instalar, atualizar e configurar por terminal

Esta release oferece o instalador NSIS e o portátil. Instalação/atualização via WinGet poderá ser preparada depois que os endereços e hashes das Releases forem finais e o manifesto tiver passado pela validação do catálogo. Até lá, a instalação normal é pelo executável da Release; configurações de navegação permanecem na interface do Canoa. Não há CLI para editar configurações ou credenciais.

O manifesto WinGet é um mecanismo apropriado para instalar e atualizar uma aplicação Windows a partir de uma URL e versão declaradas. O envio ao catálogo é um trabalho separado da publicação do código-fonte. [Documentação oficial: manifestos WinGet](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest) e [comando install](https://learn.microsoft.com/en-us/windows/package-manager/winget/install).

## Critérios para liberar uma candidata

1. Compilação limpa por CI e script oficial em Windows.
2. Busca comum com Enter, URL direta e sugestão clicada.
3. `@` e `#` com clique, setas, Enter e Escape tanto numa nova aba quanto numa página carregada; o conteúdo retorna ao fechar o painel.
4. Uma única aba, Nova aba, Voltar, Avançar, Recarregar, menus e fechamento da janela.
5. Download comum de arquivo e conteúdo da página para texto/leitura offline.
6. Instalador/portátil abrem e fecham, hashes publicados e avisos conhecidos documentados.

## Próxima etapa de distribuição por terminal

Depois dos critérios acima, publicar a Release e preparar um manifesto WinGet com `PackageIdentifier`, versão, URL do instalador, arquitetura x64, hash e opções de instalação. Submeter esse manifesto ao catálogo oficial; só então anunciar comandos `winget install` e `winget upgrade`. Não habilitar instalação silenciosa nem atualização automática até validar assinatura, origem, hash e rollback.
