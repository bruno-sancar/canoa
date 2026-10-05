# Histórico e estado das versões

Este projeto foi desenvolvido em iterações frequentes entre as versões 0.1.1 e 0.1.36. A linha pública do Git começa em um snapshot limpo da 0.1.35. Isso foi intencional: versões experimentais antigas e seu histórico de autoria não foram copiados para o repositório público. Portanto, o Git público não é um histórico completo de desenvolvimento.

## Linha do tempo documentada

As faixas abaixo agrupam marcos registrados nas notas locais de desenvolvimento. Nem toda versão intermediária foi uma release pública ou tem um código-fonte reproduzível; não associe esses números a tags ou binários públicos que não existam.

| Versões/marcos | Evolução registrada |
| --- | --- |
| `0.1.1–0.1.6` | Base Tauri/WebView2 para Windows, abas e navegação, busca por fontes, configurações iniciais, medição de memória e primeiros instaladores/testes. O histórico público antigo do código termina em `0.1.6`. |
| `0.1.7–0.1.14` | Favoritos locais, organização/importação/exportação, recuperação de abas, downloads recentes, grupos de abas, atalhos e refinamentos de navegação/instalação. |
| `0.1.15–0.1.21` | Correções de inicialização e interface; biblioteca pessoal, lista de leitura, Baixados, sugestões e seletor de recursos da página. |
| `0.1.22–0.1.28` | Bloqueio de anúncios/rastreadores, acesso informativo a artigos, limpeza de rastreamento, cofre local de senhas e refinamentos de navegação/referências. A `0.1.28` foi preservada como executável e referência comportamental para busca comum; o seu fonte correspondente não foi localizado. |
| `0.1.29–0.1.33` | Experimentos de sobreposição/seleção de sugestões e tentativas de correção. Comparações dos executáveis registraram regressões em busca, seleção de sugestões e comandos de navegação. A relação causal com a implementação de sobreposição é uma hipótese, não uma conclusão provada por `git bisect`. |
| `0.1.34` | Experimento de detecção/resolução de mídia, HLS/DASH e serviço externo. Foi excluído da reconstrução pública 0.1.35; o ZIP de referência não foi distribuído por falta de licença confirmada. |
| `0.1.35` | Reconstrução seletiva que retoma comportamento observado na `0.1.28`, recupera busca e ações de sugestões, mantém os recursos locais confirmados e publica um inventário explícito de estado e limitações. É uma prévia de testes, não uma recuperação exata do código 0.1.28. |
| `0.1.36` | Corrige o rótulo visível do atalho de modo foco, detalha inventário/histórico e melhora os modelos de contribuição. Sem mudanças na navegação. |

## Linha detalhada da regressão `0.1.28–0.1.34`

As versões abaixo não são releases Git reproduzíveis. O resumo usa notas de desenvolvimento e testes visuais dos executáveis que foram preservados.

| Versão | Mudança ou evidência preservada | Resultado conhecido |
| --- | --- | --- |
| `0.1.28` | Revisão de aba única, navegação e sugestões `@`/`#`; incluía Favoritos, Leitura, Baixados, Histórico, Senhas e Fontes na referência `@`. | Busca comum com Enter observada funcionando. `#` mostrava opções, mas Enter ainda podia submeter `#` ao buscador; não é evidência de que todas as sugestões estavam corretas. Fonte não recuperada. |
| `0.1.29` | Introduziu uma WebView nativa separada para colocar o painel de sugestões acima da página. | Teste comparativo observou a busca comum sem navegar; interação de URL não foi confirmada. A camada visual melhorou num teste parcial, mas o fluxo completo não foi validado. |
| `0.1.30` | Adicionou caminho IPC para seleção no painel nativo e preservação temporária da lista durante a transferência de foco. | Build e 38 testes passaram; houve teste visual parcial herdado da 0.1.29. A instalação visual e a seleção completa da 0.1.30 ficaram sem confirmação. |
| `0.1.31` | Não há nota independente nem fonte versionada preservada para esta iteração. | Mudanças e comportamento próprios não podem ser afirmados com confiança. |
| `0.1.32` | Diagnóstico relatou sugestões que renderizavam sem executar ações; tentou corrigir Enter, foco, IPC e ordem das WebViews. | Build e testes passaram; automação não conseguiu confirmar ações de busca, `@`, `#`, navegação ou fechamento no executável. |
| `0.1.33` | Hotfix ajustou inicialização/dimensionamento do painel e instrumentação de erro. | Teste real voltou a observar busca comum, escolha de URL em `@`, seção em `#`, Nova aba e Voltar sem resposta confiável. O hotfix não resolveu a regressão. |
| `0.1.34` | Experimento posterior adicionou observação de respostas de rede e resolução externa de vídeos/streams HLS/DASH. | Código dessa candidata foi descartado da linha pública 0.1.35; dependências externas e licença do ZIP de referência não foram adotadas. |

Não houve releases oficiais/tag Git de `0.1.28–0.1.34` neste repositório público. Esta tabela preserva a sequência útil para entender a reconstrução, sem transformar testes incompletos em promessa de funcionamento nem simular um histórico de commits que não existe.

## O que a evidência permite afirmar

- Há executáveis preservados da `0.1.28`, mas não foi localizado o snapshot-fonte que os gerou nem um commit/tag correspondente.
- Testes comparativos registrados observaram busca comum por Enter funcionando em `0.1.28` e falhando em `0.1.29` e `0.1.33`. As falhas de `@`, `#`, Nova aba e Voltar também foram registradas na `0.1.33`.
- Os registros disponíveis não provam em qual alteração exata a regressão foi introduzida. Não foi possível executar `git bisect` nas fontes de `0.1.29–0.1.34`, pois elas não formavam uma sequência pública de commits.
- A `0.1.35` foi compilada de uma fonte reconstruída a partir do estado local posterior, com componentes experimentais seletivamente removidos. “Baseada no comportamento de 0.1.28” não quer dizer “mesmo fonte” nem equivalência integral de recursos.
- As notas de manutenção e comparação que sustentam esse resumo são preservadas no histórico local do mantenedor, mas não foram copiadas integralmente para o repositório público. Só os fatos necessários para interpretar a release foram resumidos aqui.

## Política de histórico daqui em diante

Cada versão distribuída deve ter uma entrada em `CHANGELOG.md` com data, mudanças visíveis, correções, incompatibilidades e limitações. A página GitHub Releases deve apontar para a nota de release versionada e identificar claramente prévia ou estável. Tags devem apontar para o fonte realmente usado para compilar o binário. Para releases futuras, anexar hashes e resultados de CI; não criar notas retroativas ou tags para versões sem fonte verificável.

Use issues para defeitos e propostas, Pull Requests para mudanças de código, e a seção de segurança privada para vulnerabilidades. Consulte [Como contribuir](../CONTRIBUTING.md) e [a política de segurança](../SECURITY.md).
