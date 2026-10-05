# Canoa 0.1.35 — escopo, reconstrução e validação

Estado: **prévia pública para testes**, publicada em [GitHub Releases](https://github.com/bruno-sancar/canoa/releases/tag/v0.1.35). Não classificar como estável.

## Relação com a 0.1.28

A 0.1.28 é a última versão em que a busca comum por Enter foi observada funcionando. Os executáveis dessa versão foram preservados localmente, com estes SHA-256:

- Instalador: `91EA39E29D824D9177877E0282D71276A4941B4314BC45E8FFA20885BA5E5E40`
- Portátil: `8920A0B2777CBAAD106F4B013A51DDDF848DA29B5AA7186C628D6256E2E00E96`

Não foi encontrado snapshot-fonte 0.1.28 nem commit/tag correspondente no Git. O histórico do repositório remoto tinha código apenas até 0.1.6; a fonte local disponível chegou a 0.1.34. Portanto, a 0.1.35 é uma reconstrução seletiva guiada pelo comportamento e pela documentação da 0.1.28, não uma cópia exata da fonte daquela build.

## O que foi excluído da linha 0.1.35

- A WebView separada usada pela linha 0.1.29–0.1.33 para desenhar o painel de sugestões. O painel volta a pertencer à interface principal; durante a exibição, a página WebView2 é temporariamente ocultada para que não cubra os resultados.
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
| Execução em tela: busca normal pelo DuckDuckGo | Passou; consulta não sensível exibiu resultados |
| `@` na Nova aba e numa página web | Passou; categoria e URL foram selecionadas por clique |
| `#` na Nova aba e numa página web | Passou; Enter abriu Configurações: Visão geral |
| Escape após abrir sugestões | Passou; a página carregada reapareceu |
| Voltar e Avançar | Passaram após criar navegações distintas; URL e conteúdo acompanharam o histórico |
| Janela e aba única | Passaram; abertura inicial mostrou uma aba e o fechamento encerrou normalmente |
| Leitor de tela, teclado integral, instalação/desinstalação limpa e compatibilidade entre sites | Ainda não cobertos nesta rodada |
| Ponto de UX conhecido | A WebView da página fica temporariamente oculta enquanto sugestões aparecem; Escape ou seleção restaura o conteúdo |
| Assinatura Authenticode e updater automático | Não incluídos nesta prévia |

## Pacotes Windows gerados localmente

| Arquivo | SHA-256 |
| --- | --- |
| `Canoa-0.1.35-Windows-instalador.exe` | `1C1D863C221BA64EDE8A6779916C099585E9984D1AAF3A9D27A6526173025AA9` |
| `Canoa-0.1.35-Windows-portatil.exe` | `0588DBAD928A11E5EDE4EBCE18C23D1D08E7825D6AB8607C6D6C7630DDE27DCA` |

Esta compilação foi publicada como prévia de teste. A validação visual dos principais fluxos acima passou; isso não equivale a validar todas as funções, sites, combinações de foco ou acessibilidade. Consulte [o inventário de recursos, estado e atalhos](recursos-e-atalhos.md) antes de assumir cobertura integral.

## Instalar, atualizar e configurar por terminal

Esta release oferece o instalador NSIS e o portátil. Instalação/atualização via WinGet poderá ser preparada depois que os endereços e hashes das Releases forem finais e o manifesto tiver passado pela validação do catálogo. Até lá, a instalação normal é pelo executável da Release; configurações de navegação permanecem na interface do Canoa. Não há CLI para editar configurações ou credenciais.

O manifesto WinGet é um mecanismo apropriado para instalar e atualizar uma aplicação Windows a partir de uma URL e versão declaradas. O envio ao catálogo é um trabalho separado da publicação do código-fonte. [Documentação oficial: manifestos WinGet](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest) e [comando install](https://learn.microsoft.com/en-us/windows/package-manager/winget/install).

## Cobertura de testes desta prévia

Os resultados abaixo distinguem verificações feitas nesta prévia da validação futura recomendada:

- **Feito:** frontend, formatação/testes Rust, compilação NSIS/portátil, CI Windows, smoke test de abertura/fechamento portátil e cenários de UI listados na tabela acima.
- **Feito parcialmente:** central de downloads, bibliotecas, privacidade, grupos, cofre e configurações têm implementação e testes de código; compatibilidade visual completa e persistência ponta a ponta de cada fluxo não foram cobertas por esta rodada.
- **Próximo para testadores:** instalação/remoção em ambiente limpo; busca, referências e atalhos com diferentes estados de foco; download HTTP comum e captura offline em páginas diversas; importação/exportação de Favoritos; bloqueio e exceções; cofre com credenciais sintéticas; leitor de tela e navegação integral por teclado.

## Próxima etapa de distribuição por terminal

Uma distribuição por terminal ainda exige URL permanente, assinatura/verificação de origem e validação de instalação/rollback. Só depois preparar e submeter manifesto WinGet com `PackageIdentifier`, versão, arquitetura x64, hash e opções de instalação. Até existir publicação aceita no catálogo, não anunciar `winget install` ou `winget upgrade`. Não habilitar atualização automática nesta prévia.
