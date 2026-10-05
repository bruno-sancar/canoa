# Canoa 0.1.36 — revisão de documentação e atalho

Estado: prévia pública para testes. Esta revisão conserva a navegação e os recursos da 0.1.35 e não deve ser tratada como versão estável.

## Alterações

- Corrige no botão de modo foco o rótulo do atalho: `Ctrl+Shift+F` (a combinação já era a implementada).
- Publica o [inventário de recursos, configurações e atalhos](recursos-e-atalhos.md) e o [histórico de versões disponível](historico-de-versoes.md), deixando explícitos os estados, limitações e limites de evidência.
- Melhora as orientações para contribuição e adiciona modelos GitHub para defeitos e Pull Requests.

## Verificações desta revisão

- `npm ci`: passou; 0 vulnerabilidades conhecidas na auditoria npm.
- `npm run build`: passou; TypeScript/Vite e verificação dos 166 IDs da interface.
- `cargo fmt --all -- --check`: passou.
- `cargo test --locked --lib`: 38 passaram, 0 falharam.
- `scripts/gerar-instalador.ps1`: gerou instalador NSIS e executável portátil para Windows x64.
- Os principais cenários de navegação da 0.1.35 permanecem os já registrados na [nota de validação 0.1.35](release-0.1.35.md); não foram repetidos nesta correção textual.
- Leitor de tela, navegação integral por teclado, instalação/desinstalação limpa e compatibilidade ampla entre sites seguem fora da cobertura desta rodada.

## Arquivos e hashes SHA-256

| Arquivo | SHA-256 |
| --- | --- |
| `Canoa-0.1.36-Windows-instalador.exe` | `1E9C38E485E15A1689BA77CA28E226467BCE9589BCF01170E2A1C2FE4CC03931` |
| `Canoa-0.1.36-Windows-portatil.exe` | `2EB2A0DBBC97A476E86B16B1D60331940EF4B8E040CC312CE52D6611A4B84B25` |

Os mesmos valores estão no arquivo `Canoa-0.1.36-SHA256SUMS.txt` anexado à Release.

## Limitações

Sem assinatura Authenticode, atualização automática, sincronização, WinGet, autofill de senhas, captura completa de sites offline ou resolução HLS/DASH. O acesso a artigos é somente informativo e não remove paywalls. Veja [recursos e atalhos](recursos-e-atalhos.md) e [segurança](../SECURITY.md).
