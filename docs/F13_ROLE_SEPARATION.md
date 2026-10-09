# F13 — Separação de Competências e Autoridade

Status: **contrato de engenharia implementado**, F13 continua IN_PROGRESS.
Prova: docs/PROOF_F13_ROLE_SEPARATION.md.

## Matriz de competência

| Papel | Faz | Não faz |
|---|---|---|
| Capacidade | Declara APIs, operações tecnicamente implementadas, versão, readiness, evidências e lacunas | Não escolhe negócio, decide o uso de plataformas ou autoriza pagamentos |
| Engenharia | Constrói conectores, SDKs, adapters, fluxos, testes e recuperação | Não decide estratégia nem assume autoridade de operador |
| Agente de decisões | Analisa oportunidades, risco, retorno e alternativas; produz proposta rastreável | Não concede a si mesmo autorização e não executa operações externas |
| Operador | Escolhe a plataforma e a estratégia; aprova, rejeita ou adia propostas; define escopo | Não substitui verificações técnicas, autenticação e autoridades do runtime |

## Consequências práticas

1. **99Freelas, HomeOcta e demais provedores são opções do operador.** Ausência de uma ação implementada significa `engineering_gap`, não proibição comercial. Uma opinião ou leitura pública do assistente não é usada para determinar permanentemente uso ou não uso de plataformas.
2. O inventário consome os manifests efetivos de `platform-adapters` para mostrar o que o sistema implementa de fato. Recursos em modo de simulação ou pendentes de qualificação não são declarados produtivos.
3. O agente de decisões gera apenas um `DecisionProposal` com evidências, dúvidas e estimativas separadas. A proposta começa com `decision=pending_operator`; não possui assinatura nem aprovação.
4. O operador autentica uma `OperatorDecision` vinculada ao hash da proposta, missão, origem e escopos exatos, com expiração máxima de 15 minutos e nonce individual.
5. O módulo de validação emite apenas `AuthorizationReceipt` e reserva uma vez o nonce. **Ele não executa navegador, envio de proposta, entrega ou pagamento.**
6. O dispatcher produtivo, ainda em engenharia, deverá consumir um recibo válido e verificar **independentemente** a autorização, a qualificação F02/F08, o estado F05/fencing, o contrato F09 e o consentimento humano quando requerido. Nenhum desses gates pode ser substituído por uma recomendação do agente.
7. Assinaturas dependem de chave de autoridade fornecida somente ao **processo do operador**, não ao agente de decisões. O teste usa chave sintética; não existe integração atual com um vault real ou identidade autenticada do operador.

## Limite desta implementação

A separação tem contratos e testes de lógica. A interface autenticada do operador, o processo de assinatura segregado, o dispatcher real por plataforma e sua homologação ainda precisam ser implementados. Não há autorização produtiva implícita nem receita comprovada. A política de simulação F09 permanece preservada até existir implementação/qualificação apropriada.

## API implementada

- `operator-control/src/capabilities.ts`: inventário técnico sem juízo comercial.
- `operator-control/src/decision_agent.ts`: propostas informativas, `pending_operator`.
- `operator-control/src/operator.ts`: verificação de autorização autenticada, escopo, tempo, identidade e nonce.
- `operator-control/test/roles.test.ts`: testes negativos de assinatura, escopo, origem, expiração, rejeição e repetição.

**Regra canônica:** CAPACIDADE descreve → ENGENHARIA constrói → AGENTE RECOMENDA → OPERADOR DECIDE → RUNTIME VALIDA → WORKER EXECUTA apenas quando todos os gates aplicáveis estiverem comprovados.
