# F13 — Separação entre Engenharia e Homologação Produtiva

Status: **F13 permanece IN_PROGRESS**. Este documento separa o que já está tecnicamente entregue do que ainda falta para homologação produtiva real.

## F13-ENG — Engenharia técnica

**Estado:** COMPLETE / CANDIDATE.

A engenharia técnica compreende arquitetura, integração local, contratos, validações, gates e provas reproduzíveis sem declarar trabalho produtivo externo. Inclui:

- `production-rust` candidato `v1.0.0-rc.1`;
- execução local determinística com F05, F06, F07 e F10;
- recuperação por lease/fencing e rejeição de resultado stale;
- evidência local persistida em disco e validada por SHA-256;
- custo sintético registrado sem receita/pagamento;
- separação de papéis em `operator-control`;
- gate `F13_ENGINEERING_VERIFY=PASS`;
- regressão integral `BASELINE_VERIFY=PASS`.

Essa camada **não** precisa decidir se uma plataforma deve ser usada. Ela apenas diz o que existe, o que falta e quais gates técnicos foram satisfeitos.

## F13-HOMOLOG — Homologação produtiva

**Estado:** PENDING / BLOCKED.

A homologação produtiva é outra autoridade. Ela exige uma missão real, autorizada pelo operador, executada em plataforma/fluxo permitido, com evidência independente e resultado econômico comprovável quando aplicável. Inclui:

- autorização explícita do operador;
- adapter implementado e qualificado para a ação produtiva específica;
- F08/F02 promovidos com evidência, não por decisão do agente;
- F05 idempotency/fencing/recovery na operação real;
- F07 validação independente do resultado externo;
- F10 registro econômico real, sem receita fictícia;
- runbook operacional e rollback;
- aceite final para `MISSION_PROVEN`.

Enquanto esses itens não existirem, o resultado correto é:

```text
F13_ENGINEERING_VERIFY=PASS
F13_PRODUCTIVE_HOMOLOGATION=PENDING
MISSION_PROVEN=PENDING
```

## Regra canônica

A conclusão de engenharia **não fecha F13**. F13 só fecha quando a homologação produtiva passar.

O agente de decisões pode recomendar. O operador decide. A engenharia implementa e prova. O runtime valida. Nenhuma dessas camadas substitui as outras.

## Relação com plataformas externas

Se uma ação em 99Freelas, HomeOcta ou outra plataforma não existe tecnicamente, ela deve ser registrada como **engineering_gap**. Isso não é veto comercial. A decisão de priorizar ou utilizar uma plataforma cabe ao operador, apoiado pelo agente de decisões.

Se o operador autorizar uma ação produtiva, a engenharia ainda deve implementar ou habilitar o caminho técnico correspondente, com autenticação, autorização, idempotência, validação e auditoria.
