# Prova F13 — Separação de Capacidade, Engenharia, Agente e Operador

PC Vendas: `D:\TIMED-MISSION-AGENT`.
F13 continua `IN_PROGRESS` e `MISSION_PROVEN=PENDING`.

Implementação: `operator-control` (Node/TypeScript), dependente somente dos manifests técnicos da F09 e de módulos Node padrão.

## Testes executados

- Inventário das capacidades reais inclui 99Freelas sem proibição comercial inferida.
- Operações como enviar propostas/entregar trabalho, quando ausentes do código, são `engineering_gap`.
- Agente produz `DecisionProposal` determinístico, com `decision=pending_operator`, nunca assinatura.
- Operador pode aprovar, rejeitar ou adiar; apenas uma aprovação autenticada e específica pode gerar recibo.
- Assinatura HMAC-SHA256 inválida, identidade não autorizada, escopo ampliado, outra missão e prazo expirado são rejeitados.
- Nonce de autorização só pode ser reservado uma vez; repetição é rejeitada.
- Os testes não geram side effects externos nem credenciais reais.

**Resultado inicial:** 9/9 testes TypeScript PASS.

## Restrições e limites

Separação de papéis no contrato **não equivale** a uma autenticação completa de operador ou autorização real de API. A chave de assinatura deve ser guardada fora do ambiente do agente de decisões. Antes de qualquer execução externa, é obrigatório implementar UI/CLI autenticada do operador, integração do despacho com F05/F08/F09 e trilha de auditoria com evidências. F13 não deve ser encerrada por esta prova isolada.

## Regressão

CONTINUITY_OK current=F13 next=NONE terminal=F13
F13_ROLE_BOUNDARIES=PASS tests=9 commercial_policy_not_in_engineering=true operator_decision_separate=true
F13_ENGINEERING_VERIFY=PASS
F13_PRODUCTION_ACCEPTANCE=BLOCKED EXTERNAL_ADAPTER_NOT_QUALIFIED
BASELINE_VERIFY=PASS
