# Avisos de terceiros

Este arquivo identifica os materiais de terceiros distribuídos com o Canoa. Os arquivos `Cargo.lock` e `app/package-lock.json` registram as versões das dependências usadas para compilar. Cada biblioteca e pacote mantém seus próprios avisos e licença; a licença MIT do Canoa não substitui esses termos.

## Listas de filtros

O aplicativo inclui os dados EasyList e EasyPrivacy em `app/src-tauri/listas/`.

- [EasyList](https://easylist.to/easylist/easylist.txt) — regras comunitárias de anúncios.
- [EasyPrivacy](https://easylist.to/easylist/easyprivacy.txt) — regras comunitárias de rastreadores.
- Licenças indicadas pelo projeto: GPL-3.0-or-later ou CC BY-SA 3.0. Consulte a [licença oficial](https://easylist.to/pages/licence.html) e os cabeçalhos das cópias distribuídas.
- As listas são atualizáveis pelo recurso do aplicativo; novas cópias devem manter cabeçalhos, versão e atribuição.

## Dependências principais

- [Tauri](https://github.com/tauri-apps/tauri) e [tauri-plugin-dialog](https://github.com/tauri-apps/plugins-workspace) — MIT ou Apache-2.0, conforme os avisos de cada componente.
- [adblock-rust](https://github.com/brave/adblock-rust) — mecanismo de filtragem mantido pelo Brave, sob MIT ou Apache-2.0.
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) — runtime fornecido e atualizado pela Microsoft; não é incorporado como código do Canoa.
- Outras crates Rust e dependências NPM estão fixadas nos arquivos de lock. Consulte os metadados e avisos publicados pelos respectivos projetos antes de redistribuir binários alterados.

## Referência recebida para o recurso Baixar

O ZIP `baixador-de-videos-web.zip` foi usado somente como referência local. Ele não foi copiado para este repositório nem integrado ao código, pois a licença de redistribuição não foi confirmada.
