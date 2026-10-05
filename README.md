# Canoa

**Canoa é um navegador desktop para Windows, em português, com foco em navegação organizada, privacidade local e controle do usuário.** Ele usa Tauri 2 e o WebView2 Runtime da Microsoft; não incorpora o Google Chrome nem baixa o navegador Edge completo.

> **Estado do projeto:** prévia pública 0.1.36 para testes. A fonte exata que gerou o executável 0.1.28 não foi recuperada. Esta linha recomeça a partir do comportamento observado na 0.1.28, com fonte reconstruída e regressões de busca/sugestões tratadas na 0.1.35. Não é uma recompilação byte a byte da 0.1.28. Leia as limitações e os testes pendentes antes de usar.

## Recursos desta prévia

O Canoa reúne navegação por abas e uma biblioteca local com Favoritos, Lista de leitura e Baixados. A barra pesquisa na web, sugere abas e itens locais, usa `@` para escolher uma coleção e `#` para localizar uma seção de Configurações. A lista de fontes pode restringir uma consulta ao domínio selecionado.

Configurações organiza pesquisa, barra de endereço, Favoritos, Lista de leitura, memória e armazenamento, Baixados, Histórico, Tempo e foco, Cookies e cache, Proteção e bloqueios, Senhas, Atalhos e informações do Canoa. O inventário completo, o que cada opção faz, os atalhos confirmados e as limitações estão em [Recursos e atalhos](docs/recursos-e-atalhos.md).

Esta versão inclui bloqueio local de anúncios e rastreadores conhecidos, exceções por site, limpeza opcional de parâmetros de rastreamento, bloqueio de domínios, limites de tempo e modo foco. Também inclui grupos de abas, mudo por aba/página, cofre local cifrado, histórico opcional, limpeza de cookies/cache e medição agregada de memória do Canoa/WebView2.

**Limites relevantes:** o seletor Baixar lista arquivos HTTP(S) diretamente expostos pela página e permite guardar texto/captura HTML estática; não resolve streams HLS/DASH, DRM ou conteúdo que o site não expõe. Downloads comuns são iniciados pelo WebView2 e têm menos informações de progresso/controle. A indicação de acesso a artigos é informativa: não remove paywalls, autenticação ou controles de acesso. O cofre não sincroniza nem preenche formulários automaticamente. Há recursos cujo funcionamento foi compilado/testado, mas ainda precisa de mais validação em sites e computadores diferentes; o status está descrito no inventário.

Leia também [a nota da versão 0.1.36](docs/release-0.1.36.md), o [histórico de mudanças](CHANGELOG.md) e [o histórico disponível](docs/historico-de-versoes.md). A linha do tempo explica o que foi observado e preservado, e distingue isso de versões com código-fonte reproduzível.

## Baixar e instalar

Os instaladores e executáveis portáteis devem ser baixados da seção **Releases** deste repositório. O instalador Windows é por usuário e pode solicitar conexão para instalar o WebView2 Runtime caso ele ainda não esteja presente. O pacote portátil também depende desse runtime no Windows.

Os executáveis são distribuídos sem assinatura Authenticode nesta fase; o Windows pode exibir um aviso de reputação. Confira os hashes SHA-256 publicados junto de cada release antes de executar o arquivo.

**Instalação via terminal/WinGet ainda não está publicada.** Até que um manifesto seja aceito no catálogo WinGet, instale pelo executável na página Releases. As configurações do navegador são feitas pela interface do Canoa; não há CLI suportada para alterar configurações ou segredos.

## Compilar no Windows

### Requisitos

- Windows 10 ou 11 x64.
- Node.js 22 e npm.
- Rust stable com o alvo MSVC e ferramentas de compilação C++ do Visual Studio.
- WebView2 Runtime para executar o aplicativo compilado.

As instruções oficiais de ambiente do Tauri estão em [Prerequisites](https://v2.tauri.app/start/prerequisites/). A compilação de produção usa os comandos do Tauri presentes nas dependências locais e gera o instalador NSIS e o executável portátil.

### Baixar o código e validar

```powershell
git clone https://github.com/bruno-sancar/canoa.git
cd canoa
cd app
npm ci
npm run build
cd src-tauri
cargo fmt --all -- --check
cargo test --locked --lib
```

Para compilar os pacotes Windows usando o script isolado do projeto, volte à raiz e execute:

```powershell
cd ..\..
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\gerar-instalador.ps1
```

O script usa uma cópia de compilação em `%LOCALAPPDATA%\Canoa\build-test`, evitando instalar dependências dentro da pasta de trabalho do aplicativo. A saída local vai para `output/`; executáveis e artefatos locais não são versionados.

## Privacidade e dados

- Preferências, biblioteca, histórico e cofre são armazenados localmente no perfil do Windows. O histórico é opcional.
- O Canoa não envia telemetria própria nem exige uma conta.
- O bloqueador usa regras locais EasyList e EasyPrivacy; as listas têm licenças próprias e podem bloquear funções de sites. O usuário pode pausar a proteção por domínio.
- A busca envia os termos ao provedor escolhido nas configurações. Navegar em um site também transmite a ele os dados normais da requisição HTTP.
- A área de trabalho usa WebView2 fornecido pela Microsoft, que acompanha o Edge Runtime. Atualizações de segurança desse runtime dependem da Microsoft.
- O cofre não sincroniza e não pode ser recuperado se a senha mestra for perdida. Faça exportações seguras dos dados não secretos que o Canoa permitir exportar; nunca compartilhe o arquivo de configuração do perfil.

Mais detalhes e canais de reporte estão em [SECURITY.md](SECURITY.md) e [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Contribuir

Relatos, documentação e contribuições são bem-vindos. Leia [CONTRIBUTING.md](CONTRIBUTING.md), [CHANGELOG.md](CHANGELOG.md) e [o histórico](docs/historico-de-versoes.md). Para enviar um relato, use a aba Issues deste repositório e inclua versão, Windows, passos e resultado esperado; remova URLs privadas, credenciais e dados do perfil.

## Licença

Código e materiais originais deste projeto: [MIT](LICENSE). Dados e dependências de terceiros mantêm suas licenças originais, detalhadas em [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
