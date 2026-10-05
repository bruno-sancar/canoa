# Recursos, configurações e atalhos

Este inventário descreve o que está implementado no código público da 0.1.36. “Implementado” não significa compatibilidade universal com todos os sites ou uma certificação completa de acessibilidade. Os estados de validação abaixo separam testes observados de recursos que ainda precisam de uma rodada dedicada.

## Navegação e barra de endereço

- **Abas:** criar, fechar, alternar, reabrir a última aba fechada na sessão e mover a aba atual. O conteúdo WebView2 é mantido para a aba selecionada e pode ser recarregado ao alternar.
- **Pesquisa e fontes:** pesquisar com DuckDuckGo ou Brave Search, escolhidos em Configurações → Pesquisa e fontes. As fontes são domínios guardados localmente; ao escolher uma, a consulta é limitada usando `site:dominio` no buscador selecionado. O Canoa não opera um índice nem um buscador próprio.
- **Sugestões comuns:** a barra apresenta abas e itens locais correspondentes; `Enter` executa a busca ou abre o endereço. `↑`/`↓` percorrem opções e `Escape` fecha a lista.
- **Referências `@`:** escolha Favoritos, Lista de leitura, Baixados, Histórico, Senhas ou Fontes e filtre os itens daquela coleção. Senhas só aparecem quando o cofre está desbloqueado. `Enter` abre o endereço escolhido.
- **Configurações `#`:** escolha uma seção pelo nome e pressione `Enter` para abri-la.
- **Navegação:** Voltar, Avançar, Recarregar e abrir link em nova aba.

Os fluxos centrais de busca, `@`, `#`, Voltar/Avançar, uma aba, Escape e abertura/fechamento da janela foram exercitados visualmente na 0.1.35 e o código de navegação não mudou na 0.1.36. A lista permanece clicável sobre uma página carregada, mas enquanto sugestões estão abertas a página WebView fica temporariamente oculta. Leitor de tela, navegação completa só por teclado e uma matriz ampla de sites ainda precisam de validação.

## Biblioteca

### Favoritos

Guardar/remover a página atual, pesquisar, organizar em pastas, editar, apagar, importar HTML/JSON e exportar HTML/JSON. Credenciais salvas podem associar o domínio a Favoritos com o marcador “Senha associada”. Um favorito é uma organização local, não uma sincronização de conta.

### Lista de leitura

Guardar uma página para depois, pesquisar, marcar como lida e remover. É uma lista de URLs e metadados locais; salvar nela não contorna assinatura nem mantém necessariamente o conteúdo completo acessível.

### Baixados e Central de Baixados

É a área que reúne o histórico dos downloads do navegador e o seletor **Baixar itens desta página**. O seletor identifica imagens, vídeo, áudio e documentos disponíveis em URLs HTTP(S) expostas no DOM; permite filtrar por tipo e escolher itens. Também oferece texto, captura HTML estática, impressão/salvamento em PDF e abertura da Central.

Downloads iniciados pela central registram progresso e podem ser cancelados. Downloads comuns entregues ao WebView2 são geridos pelo motor e podem não informar progresso ou permitir cancelamento dentro do Canoa. O histórico persistente mantém até 200 resultados concluídos/com falha; apagá-lo não apaga os arquivos. A captura offline não inclui os recursos externos como um pacote completo de site.

**Não incluído:** resolver HLS/DASH, juntar áudio e vídeo, baixar mídia protegida, contornar DRM/autenticação ou obter conteúdo que a página não disponibilize ao navegador.

### Histórico

Opcional; quando ativo, registra navegações localmente e permite pesquisar, definir retenção de 7, 30, 90 ou 365 dias e apagar os registros. Desativar interrompe novos registros; os já existentes permanecem até remoção ou expiração.

### Fontes de pesquisa

Cadastro local de domínio e tema, pesquisa/filtro e remoção. A consulta restrita é uma busca normal no provedor escolhido com `site:`; a apresentação e os resultados dependem desse provedor.

## Privacidade, controle e ferramentas

- **Anúncios e rastreadores:** bloqueio local baseado em EasyList/EasyPrivacy, contadores por página/sessão/categoria, atualização e reversão de listas, opção global e exceções por domínio. Regras podem quebrar recursos legítimos; pausar por site é a opção de compatibilidade.
- **Parâmetros de rastreamento:** remoção opcional de parâmetros conhecidos na URL. Não é anonimização: não apaga identificadores/cookies já armazenados nem impede todos os métodos de rastreamento.
- **Sites bloqueados:** bloqueia o domínio cadastrado e subdomínios durante navegação.
- **Tempo e foco:** limite diário por domínio contabilizado quando a janela Canoa está em foco e o domínio está na página ativa; modo foco temporizado bloqueia os domínios configurados.
- **Cookies e cache:** apagar cookies que correspondem ao site atual ou cookies/cache do perfil. A limpeza geral pode encerrar sessões. Não é uma ferramenta de limpeza seletiva de todo tipo de armazenamento web.
- **Senhas:** cofre local cifrado, desbloqueio por senha mestra, cadastrar/editar/remover e copiar credenciais; associação visual com Favoritos. Senhas mestras perdidas não podem ser recuperadas. Não há autofill, captura automática de login, sincronização, PIN do Windows ou importação de credenciais nesta prévia.
- **Grupos de abas:** atribuir nome, mover, recolher/expandir e remover do grupo. Não são espaços de trabalho sincronizados.
- **Áudio:** mutar/reativar o áudio da página ou aba. Não há centro de mídia do sistema nem promessa de reprodução em segundo plano depois de fechar/trocar a aba.
- **Tarefas e memória:** mostra a memória privada estimada do Canoa e dos processos WebView2 associados, além de tarefas/abas. Esse total não é RAM física exclusiva por aba; a memória pode não ser atribuível com precisão a cada página.
- **Acesso ao artigo:** análise local indicativa de sinais comuns de assinatura e atalhos para o site, página oficial de acesso ou Lista de leitura. Não remove paywall, não altera acesso de assinatura e não busca cópias não autorizadas.
- **Sobre:** versão e data de compilação.

## Seções de Configurações

| Seção | O que é possível gerir |
| --- | --- |
| Visão geral | Estado/resumo da configuração e acesso às funções principais. |
| Pesquisa e fontes | Provedor (DuckDuckGo/Brave Search) e cadastro/gestão de domínios temáticos. |
| Favoritos | Lista integral, pesquisa, pastas, edição, remoção, importação/exportação. |
| Lista de leitura | Lista integral, pesquisa, marcar lido e remover. |
| Barra de endereço | Mostrar/ocultar ações visíveis de Favoritos, Proteção, Leitura e Baixar itens. |
| Memória e armazenamento | Estimativa de memória e armazenamento local, abrir Tarefas e exibir/ocultar indicador. |
| Baixados | Pasta de destino, abrir/restaurar pasta padrão, downloads ativos, histórico persistente e limpeza do histórico. |
| Histórico | Ativar/desativar, pesquisa, retenção e limpeza. |
| Tempo e foco | Limites diários, duração do foco e domínios bloqueados durante o foco. |
| Cookies e cache | Limpeza no site atual ou no perfil. |
| Proteção e bloqueios | Bloqueador, listas, exceções, remoção de parâmetros, sites bloqueados e explicação de acesso a artigos. |
| Senhas | Desbloquear/criar cofre e gerir credenciais locais. |
| Atalhos | Lista de atalhos desta versão; a personalização ainda não existe. |
| Sobre o Canoa | Versão, compilação e informações da arquitetura. |

## Atalhos de teclado confirmados no código

| Atalho | Ação |
| --- | --- |
| `Ctrl+T` | Nova aba |
| `Ctrl+W` ou `Ctrl+F4` | Fechar aba atual |
| `Ctrl+Shift+T` | Reabrir a última aba fechada nesta sessão |
| `Ctrl+L` ou `Alt+D` | Focar barra de endereço/pesquisa |
| `Ctrl+R` ou `F5` | Recarregar |
| `Alt+←` / `Alt+→` | Voltar / avançar |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Próxima / aba anterior |
| `Ctrl+1` a `Ctrl+8` / `Ctrl+9` | Aba pela posição / última aba |
| `Ctrl+PageUp` / `Ctrl+PageDown` | Mover aba atual para esquerda / direita |
| `Ctrl+D` | Adicionar/remover Favorito |
| `Ctrl+J` | Abrir a pasta de downloads no Explorador de Arquivos do Windows |
| `Ctrl+Shift+O` | Abrir Favoritos |
| `Ctrl+Alt+R` | Abrir Lista de leitura |
| `Ctrl+Alt+S` | Abrir Central de Baixados |
| `Ctrl+Alt+K` | Abrir Senhas |
| `Ctrl+Alt+F` | Abrir Fontes |
| `Ctrl+Shift+S` | Baixar itens da página atual |
| `Ctrl+Alt+A` | Abrir opções legítimas de acesso ao artigo |
| `Ctrl+Alt+P` | Pausar/retomar proteção no site atual |
| `Ctrl+Alt+B` | Bloquear/desbloquear site atual |
| `Ctrl+Alt+M` | Mutar/reativar áudio da página |
| `Ctrl+Shift+G` | Adicionar aba atual a um grupo |
| `Ctrl+Shift+F` | Alternar modo foco |
| `Ctrl+Shift+M` | Mostrar/ocultar indicador de memória |
| `Ctrl+,` | Abrir Configurações |
| `Ctrl+H` | Abrir Histórico |
| `Shift+Esc` | Abrir Tarefas |
| `Shift+F10` | Abrir ações da aba ativa |

Os atalhos não são personalizáveis nesta versão. Alguns atalhos também aparecem nos menus de contexto; a tabela acima descreve as combinações registradas no código, não substitui testes em todas as combinações de foco e layout de teclado.

## Não incluído nesta versão

Instalação por WinGet/CLI, atualização automática, assinatura Authenticode, sincronização de perfil, extensões, VPN, velocidade de rede, controles completos de mídia em segundo plano, alocação confiável de RAM por aba, autofill de senhas e desbloqueio de paywalls não fazem parte da 0.1.36. A ordem de implementação futura deve ser decidida por feedback e validação, não por uma promessa de prazo.
