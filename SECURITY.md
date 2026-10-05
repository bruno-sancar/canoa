# Política de segurança

## Reportar vulnerabilidades

Use **Security → Advisories → Report a vulnerability** no GitHub. O formulário privado de reporte está habilitado neste repositório. Não publique detalhes exploráveis em uma issue pública antes de coordenar uma correção.

Ao reportar, inclua a versão do Canoa, o Windows e os passos mínimos para reproduzir. Não envie senhas, conteúdo do cofre, histórico pessoal, arquivos de perfil nem tokens. Apague URLs e dados pessoais dos logs antes de anexá-los.

## Escopo e limitações

- A senha mestra do cofre não é recuperável; mantenha uma cópia segura dos dados importantes.
- O Canoa não é um substituto de VPN, antivírus ou controle parental.
- A navegação usa o WebView2 Runtime do Windows. Vulnerabilidades desse runtime devem ser corrigidas também pela atualização do componente da Microsoft.
- As builds desta prévia não têm assinatura Authenticode. Baixe somente das Releases oficiais e valide o SHA-256 publicado.

## Alerta de dependência conhecido

O GitHub aponta `glib 0.18.5` por uma implementação insegura de iteradores, corrigida em `glib 0.20.0` ([RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), [correção upstream](https://github.com/gtk-rs/gtk-rs-core/pull/1343)). Essa entrada chega pelo stack GTK 3/Linux do Tauri. O alvo suportado nesta release é Windows x64 e `cargo tree --target x86_64-pc-windows-msvc -i glib` não inclui `glib`; ela não compõe o instalador Windows. Atualizar só `glib` para 0.20 quebra a compatibilidade com a geração GTK 3 atual. O projeto mantém o alerta visível e acompanhará a migração upstream antes de oferecer builds Linux.

## Resposta

Os mantenedores confirmarão o recebimento, avaliarão a reprodução e coordenarão a correção e a divulgação. O projeto ainda é mantido por uma equipe pequena; não há SLA de resposta ou correção nesta fase.
