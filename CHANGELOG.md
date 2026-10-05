# Histórico de mudanças

Mudanças relevantes, agrupadas por versão. Versões intermediárias sem fonte/tags públicas não são tratadas como releases reproduzíveis. A evolução antes da linha pública está resumida em [Histórico e estado das versões](docs/historico-de-versoes.md).

## [0.1.36] — 2026-10-05 — prévia pública

- Corrige o texto do botão do modo foco para exibir o atalho real (`Ctrl+Shift+F`).
- Acrescenta o catálogo detalhado de recursos, configurações, atalhos e limitações, além do histórico público disponível.
- Atualiza instruções de contribuição e inclui formulários para relatos de defeitos e Pull Requests.
- A navegação e os recursos são os mesmos da 0.1.35; esta revisão não amplia a cobertura de compatibilidade com sites ou acessibilidade.
- Instalador e portátil Windows x64, hashes e nota de validação acompanham a [Release 0.1.36](https://github.com/bruno-sancar/canoa/releases/tag/v0.1.36).

## [0.1.35] — 2026-10-05 — prévia pública

- Reconstrói seletivamente a navegação a partir do comportamento observado na 0.1.28; não é o fonte original dessa versão.
- Restaura e valida busca comum, referências `@`, seções `#`, ações por clique, Escape, navegação Voltar/Avançar, aba única e ciclo básico da janela.
- Mantém biblioteca local, seletor Baixar, recursos de privacidade, grupos, cofre de senhas e ferramentas descritos no [catálogo de recursos e atalhos](docs/recursos-e-atalhos.md).
- Exclui o experimento de resolver streams HLS/DASH/vídeos de serviços externos que apareceu na candidata 0.1.34.
- Publica instalador por usuário e pacote portátil Windows x64 com hashes SHA-256 na [Release 0.1.35](https://github.com/bruno-sancar/canoa/releases/tag/v0.1.35).
- Limitações conhecidas: sem assinatura Authenticode ou atualização automática; acessibilidade com leitor de tela e compatibilidade ampla com sites aguardam mais validação; ver [nota de release](docs/release-0.1.35.md).

## Convenção para próximas versões

Adicionar mudanças ao topo deste arquivo, agrupadas em **Adicionado**, **Alterado**, **Corrigido**, **Removido** e **Limitações**. Registre apenas o que está presente na tag correspondente. A nota GitHub deve linkar a mesma entrada e declarar prévia/estável, testes, hashes e problemas conhecidos.
