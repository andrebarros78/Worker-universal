# Rastreabilidade de construção F → produto M

Este arquivo é **histórico de engenharia**, não contrato de runtime nem cadastro de módulos.

| Etapa de construção | Módulos construídos ou alterados | Estado histórico |
|---|---|---|
| F00 | M2, M5, M6, M9, M10, M11, M14 | CLOSED |
| F01 | M1, M2, M16 | CLOSED |
| F02 | M3, M14 | CLOSED |
| F03 | M6, M8 | CLOSED |
| F04 | M5 | CLOSED |
| F05 | M2, M8, M9 | CLOSED |
| F06 | M3, M4, M8 | CLOSED |
| F07 | M2, M7 | CLOSED |
| F08 | M15 | CLOSED |
| F09 | M13 | CLOSED |
| F10 | M12 | CLOSED |
| F11 | M9, M10 | CLOSED |
| F12 | M11, M14 | CLOSED |
| F13 | M1, M5, M8, M13, M16 | IN_PROGRESS |

Fonte histórica do estado das fases: `docs/PHASE_LEDGER.md`. O registro não significa que todas as capacidades de um módulo estão homologadas.

## Nomes F14/F15/F16 utilizados nas correções recentes

Esses nomes foram usados na conversa e em comprovantes técnicos, mas **não existem como novas fases ativas no livro canônico**:

- **F14** (marcador histórico): correção da leitura do Edge em segundo plano; pertence ao produto **M5**, com evidência armazenada em **M8**.
- **F15** (marcador histórico): incorporação de runtimes e preparação portátil; melhora a entrega de **M5** e infraestrutura do produto.
- **F16** (marcador histórico): reconciliação e distribuição portátil de **M1–M16**.

Referências antigas continuam verificáveis e não são renomeadas retroativamente. No produto o nome do componente é sempre **M**, e o nome da atividade de construção é **F**. Nenhum `Fxx` pode ser registrado como módulo.

**F13 permanece a única fase de construção produtiva aberta; MISSION_PROVEN=false.**
