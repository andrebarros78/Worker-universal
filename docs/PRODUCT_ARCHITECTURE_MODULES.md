# Arquitetura permanente TIMED-MISSION-AGENT

**M = módulo funcional do produto. F = fase de construção ou homologação.**

Fonte única do catálogo: `architecture/PRODUCT_MODULES.json`.
Validação nativa: `START_TMA.cmd verify`. Consulta: `START_TMA.cmd modules` e `START_TMA.cmd module M1`.

| Módulo | Nome e responsabilidade |
|---|---|
| M1 | Cérebro e Decisão: objetivo e propostas; não equivale a IA plenamente autônoma comprovada |
| M2 | Core de Missões: autoridade Rust, prazos, transições e aceitação |
| M3 | Capacidades e Ferramentas: SDKs, descritores e catálogo técnico |
| M4 | Planejamento e Roteamento: plano e seleção de capacidades |
| M5 | Execução Web e Computador: Worker e Edge/HomeOcta |
| M6 | Visão e Documentos: Python, OCR, extração |
| M7 | Validação e Recuperação: evidência e consistência |
| M8 | Memória e Evidências: persistência; aprendizagem semântica ainda não provada |
| M9 | Supervisão e Concorrência: Go, processos e reinício |
| M10 | Disponibilidade: Erlang/Elixir OTP e Rust |
| M11 | Segurança e Invariantes: Rust/Ada, prova formal pendente |
| M12 | Controlador Econômico: custo, valor e risco |
| M13 | Adaptadores de Plataformas: acesso a serviços autorizados |
| M14 | Contratos Intermódulos: JSON Schema e versões de interfaces |
| M15 | Qualificação e Benchmark: critérios, medição e regressão |
| M16 | Gate de Produção: evidência e aceitação produtiva |

A autoridade de estado pertence ao M2; os demais módulos comunicam resultados por contratos M14.
A validação do catálogo exige IDs únicos e contínuos, caminhos de código existentes e dependências sem ciclos. Identificadores F nunca são aceitos como módulos do produto.

As fases históricas estão no `docs/PHASE_LEDGER.md` e sua relação com os módulos está documentada separadamente em `docs/engineering/PHASE_MODULE_TRACEABILITY.md`. F13 continua em andamento. As referências F14/F15/F16 usadas em correções são marcadores de construção, não unidades produtivas.

**Testes locais de engenharia não equivalem a homologação produtiva. MISSION_PROVEN=false.**
