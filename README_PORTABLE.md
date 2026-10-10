# TIMED-MISSION-AGENT — Distribuição portátil F16

**Estado:** candidato de distribuição integrada, não release produtiva homologada.

## Como executar no Windows x64

1. Extraia a pasta `TIMED-MISSION-AGENT` integralmente ou mantenha-a em mídia removível íntegra.
2. Execute `START_TMA.cmd` para preparar o adaptador de navegador e seu estado local.
3. Execute `START_TMA.cmd verify` para validar as camadas núcleo Rust, supervisão Go, Python e Elixir/OTP sem instalações adicionais dessas linguagens.
4. A extensão Edge da HomeOcta precisa de autorização nativa do navegador no perfil do computador. O instalador prepara a extensão, mas **não declara que ela foi autorizada**.
5. O modo `START_TMA.cmd supervisor-health` inicia o servidor de saúde do supervisor Go em loopback. Ele não equivale ao despacho produtivo de missões.

## Componentes incluídos

- Nove executáveis Rust: core, foundation, planner, validation, economic, security-verify, production, availability, qualification
- Supervisor Go em executável Windows x64
- Node.js 24.18.0 e dependências Playwright + Chromium isolado
- Python 3.14.4, Pillow 11.3.0 e Tesseract OCR com modelo de idioma inglês
- Elixir 1.20.4 / Erlang OTP 29 em release autocontido
- Adapter SDK, contratos JSON Schema, adaptadores, código-fonte, testes e documentação
- Fontes Ada/SPARK; prova formal GNATprove não incluída

## Segurança e portabilidade

O conteúdo do ZIP não contém perfis Edge, cookies, senhas, chaves locais de instalação, bancos operacionais ou histórico de missões. Identidade da extensão e memória operacional são criadas separadamente no diretório local do usuário em cada computador. O programa não depende de uma letra específica de unidade.

A fonte canônica do desenvolvimento continua `D:\TIMED-MISSION-AGENT`, mas os binários do pacote usam caminhos relativos ao local de extração. O ZIP é um artefato de distribuição, não uma segunda raiz canônica de desenvolvimento.

## Limites comprovados e não comprovados

**Comprovado em ambiente de teste:** preparação portátil, inicialização offline, autotestes Rust/Go/Python/OTP, regressões Python/Node, navegador Chromium isolado e transporte de aula sintética em aba de segundo plano.

**Não comprovado:** extensão instalada no Edge autenticado do operador, leitura de aula real, aprendizagem independente após reinicialização, missão comercial produtiva, recuperação econômica real, instalação em um segundo PC físico, prova formal Ada/SPARK e revisão completa das licenças de redistribuição dos runtimes.

Não marcar `MISSION_PROVEN` antes de execução produtiva com evidência e aceitação independente.
