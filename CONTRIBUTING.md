# Contribuir com o Canoa

Obrigado por ajudar. O foco imediato é tornar a navegação básica previsível e as funções locais compreensíveis antes de ampliar o roadmap.

## Antes de abrir uma alteração

1. Procure issues existentes e descreva o problema, a versão, o Windows e os passos de reprodução.
2. Para mudanças grandes, proponha primeiro o escopo numa issue/discussão.
3. Não inclua segredos, dados de perfil, instaladores, capturas pessoais ou materiais recebidos sem licença de redistribuição.

## Desenvolvimento

- Interface: TypeScript, HTML e CSS em `app/`.
- Núcleo e integração Windows: Rust/Tauri em `app/src-tauri/`.
- Uma mudança deve manter os rótulos e mensagens em português brasileiro e respeitar o armazenamento local e a privacidade descritos no README.
- Não contorne autenticação, DRM, paywalls ou controles de acesso. Não adicione captura de cookies ou sincronização sem uma especificação de segurança aprovada.

Antes do pull request, execute:

```powershell
cd app
npm ci
npm run build
cd src-tauri
cargo fmt --all -- --check
cargo test --locked --lib
```

O pipeline também compila a interface e roda a suíte Rust no Windows. Para mudanças de interface, descreva os cenários visuais testados e inclua capturas sem dados pessoais.

## Pull requests

- Use um título claro no imperativo, descreva motivação, implementação e validação.
- Mantenha o escopo pequeno e evite commits com arquivos gerados.
- Cada alteração aceita é licenciada sob MIT, salvo indicação expressa em contrário. Dependências e listas de terceiros conservam suas licenças próprias.
- A equipe pode pedir alterações, testes adicionais ou dividir uma contribuição em etapas.
