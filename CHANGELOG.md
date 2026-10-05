# Histórico de versões

As versões públicas seguem SemVer durante a fase `0.x`; recursos e formatos ainda podem mudar.

## 0.1.35 — candidata de reconstrução para testes públicos

- Reinicia a linha pública sobre o comportamento observado na 0.1.28. O snapshot-fonte exato da 0.1.28 não foi localizado; a reconstrução e seus limites estão registrados em `docs/release-0.1.35.md`.
- Retira a camada separada de WebView usada nas sugestões e a resolução externa de vídeo/HLS/DASH da candidata 0.1.34.
- Mantém downloads comuns e arquivos expostos no DOM, além de histórico local de downloads e leitura offline.
- Busca normal pelo DuckDuckGo, comandos `@` e `#`, clique em sugestões, Voltar/Avançar e ciclo básico da janela foram testados no executável 0.1.35.
- A página fica temporariamente oculta enquanto o painel de sugestões está aberto. A validação de leitores de tela e a revisão integral de acessibilidade permanecem pendentes.
- Esta candidata não deve ser descrita como estável; issues de regressão devem incluir sistema, passos e URL de teste sem dados pessoais.

## Histórico anterior

O repositório inicia a linha pública a partir de um snapshot limpo de 0.1.35. O histórico de protótipos 0.1.1–0.1.34 permanece fora da linha pública para evitar expor dados de autoria pessoais e versões instáveis. A 0.1.28 é referência comportamental, não fonte reproduzível.
