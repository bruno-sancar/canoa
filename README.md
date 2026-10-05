# Canoa

**Canoa é um navegador desktop para Windows, em português, com foco em navegação organizada, privacidade local e controle do usuário.** Ele usa Tauri 2 e o WebView2 Runtime da Microsoft; não incorpora o Google Chrome nem baixa o navegador Edge completo.

> **Estado do projeto:** candidata 0.1.35 para testes públicos. A fonte exata que gerou o executável 0.1.28 não foi recuperada. Esta linha recomeça a partir do comportamento observado na 0.1.28, com fonte reconstruída e regressões de busca/sugestões tratadas nesta versão. Não é uma recompilação byte a byte da 0.1.28. Leia as limitações e os testes pendentes antes de usar.

## O que há nesta prévia

- Navegação por abas, pesquisa pela barra, fontes de pesquisa configuráveis e sugestões para abas abertas e itens locais. `@` filtra referências da biblioteca e `#` abre seções de Configurações.
- Favoritos com marcadores, lista de leitura, Baixados com histórico e capturas de texto/leitura offline.
- Download de arquivos e recursos diretamente expostos pela página, como imagens, áudio, vídeo e documentos HTTP(S). Streams HLS/DASH e resolução de vídeos por serviços externos não fazem parte desta versão.
- Bloqueio local de requisições conhecidas de anúncios e rastreadores, exceções por site, remoção opcional de parâmetros de rastreamento, sites bloqueados, limites de tempo e modo foco.
- Grupos de abas, controles de mudo, histórico local opcional, limpeza de cookies/cache e informações de memória/armazenamento.
- Cofre local cifrado para guardar e consultar credenciais. A senha mestra não é recuperável. Esta prévia não promete sincronização, preenchimento automático nem integração com PIN do Windows.
- Identificação indicativa de páginas que parecem exigir assinatura. O Canoa não remove paywalls, não contorna assinaturas e não desbloqueia conteúdo pago.

Consulte [o escopo e as limitações da 0.1.35](docs/release-0.1.35.md) antes de usar ou distribuir a prévia.

## Baixar e instalar

Os instaladores e executáveis portáteis devem ser baixados da seção **Releases** deste repositório. O instalador Windows é por usuário e pode solicitar conexão para instalar o WebView2 Runtime caso ele ainda não esteja presente. O pacote portátil também depende desse runtime no Windows.

Os executáveis são distribuídos sem assinatura Authenticode nesta fase; o Windows pode exibir um aviso de reputação. Confira os hashes SHA-256 publicados junto de cada release antes de executar o arquivo.

**Instalação via terminal/WinGet ainda não está publicada.** Primeiro vamos estabilizar a URL do instalador, os hashes e o processo de release; depois será possível solicitar a inclusão do manifesto do Canoa no catálogo WinGet. As configurações do navegador são feitas pela interface do Canoa. Não há CLI suportada para alterar configurações ou segredos.

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

Relatos, documentação e contribuições são bem-vindos. Leia [CONTRIBUTING.md](CONTRIBUTING.md) e [CHANGELOG.md](CHANGELOG.md). A licença do código do Canoa é MIT; algumas listas de filtros e dependências seguem termos próprios.

## Licença

Código e materiais originais deste projeto: [MIT](LICENSE). Dados e dependências de terceiros mantêm suas licenças originais, detalhadas em [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
