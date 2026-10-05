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

## O que a evidência permite afirmar

- Há executáveis preservados da `0.1.28`, mas não foi localizado o snapshot-fonte que os gerou nem um commit/tag correspondente.
- Testes comparativos registrados observaram busca comum por Enter funcionando em `0.1.28` e falhando em `0.1.29` e `0.1.33`. As falhas de `@`, `#`, Nova aba e Voltar também foram registradas na `0.1.33`.
- Os registros disponíveis não provam em qual alteração exata a regressão foi introduzida. Não foi possível executar `git bisect` nas fontes de `0.1.29–0.1.34`, pois elas não formavam uma sequência pública de commits.
- A `0.1.35` foi compilada de uma fonte reconstruída a partir do estado local posterior, com componentes experimentais seletivamente removidos. “Baseada no comportamento de 0.1.28” não quer dizer “mesmo fonte” nem equivalência integral de recursos.
- As notas de manutenção e comparação que sustentam esse resumo são preservadas no histórico local do mantenedor, mas não foram copiadas integralmente para o repositório público. Só os fatos necessários para interpretar a release foram resumidos aqui.

## Política de histórico daqui em diante

Cada versão distribuída deve ter uma entrada em `CHANGELOG.md` com data, mudanças visíveis, correções, incompatibilidades e limitações. A página GitHub Releases deve apontar para a nota de release versionada e identificar claramente prévia ou estável. Tags devem apontar para o fonte realmente usado para compilar o binário. Para releases futuras, anexar hashes e resultados de CI; não criar notas retroativas ou tags para versões sem fonte verificável.

Use issues para defeitos e propostas, Pull Requests para mudanças de código, e a seção de segurança privada para vulnerabilidades. Consulte [Como contribuir](../CONTRIBUTING.md) e [a política de segurança](../SECURITY.md).
