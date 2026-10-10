# F14 — Engenharia permanente do Worker (portabilidade)

## Separação obrigatória

- **Produto:** adaptador de navegador, bootstrap portátil, autenticação delegada ao navegador, recepção local assinada, registros de aulas e memória. Estes módulos devem acompanhar a distribuição do Worker.
- **Homologação:** testes e scripts de prova; nunca são usados como substitutos de um recurso produtivo nem adicionam acesso artificial ao navegador do operador.

## Estrutura para cada computador

- Arquivos do programa: relativos ao local em que o Worker for aberto, inclusive em unidade removível. A implementação não fixa nenhuma letra de unidade.
- Configuração do adaptador: criada no diretório local de dados do usuário, em `TMA/browser-bridge/homeocta`.
- A chave aleatória de 256 bits pertence exclusivamente à instalação daquele usuário/PC e **não é distribuída no pendrive**.
- Atualizações usam os arquivos da versão do produto e preservam a identidade local, salvo rotação explícita de chave.
- Evidências do curso: sob a raiz local autorizada de evidências do Worker, não em caminho fixo de desenvolvimento.
- Permissões da extensão: só HomeOcta e o canal de loopback local, nunca leitura ampla de todos os sites.
- O nome do site identifica somente este adapter. Outras plataformas precisam de adapters e contratos de origem próprios.

## Sequência de instalação por máquina

1. Executar o iniciador portátil `web-worker/PREPARE_PORTABLE_BROWSER.cmd`; ele localiza Node integrado, se presente, ou um Node compatível instalado.
2. Preparar/verificar a extensão local, incluindo credencial única do PC.
3. Verificar se a extensão já está autorizada no perfil do Edge. Se não estiver, solicitar a aprovação nativa no Edge e registrar a identidade dessa instalação.
4. Executar a missão do Worker na sessão do usuário que possui o Edge autenticado. **A autenticação do navegador é uma capacidade do usuário/PC**, não algo transportável automaticamente no pendrive.
5. Capturar conteúdo autenticado, persistir fonte, reiniciar o Worker e provar leitura da memória; só depois executar teste de aprendizagem e homologação final.

## Não equivalem a instalação produtiva

- Preparar a pasta de extensão **não** instala a extensão no Edge.
- Declarar um extensionId **não** comprova que o navegador a carregou.
- Carregar o navegador em um perfil isolado e com aula fictícia **não** comprova acesso à conta do usuário.
- Capturar uma página **não** comprova que o Worker aprendeu.
- Nenhuma rotina deve copiar cookies, perfis ou senhas para outra máquina.

## Dependências e pendências de distribuição

- O launcher procura `runtime/win-x64/node.exe` dentro do produto e aceita Node do sistema como alternativa. O runtime binário ainda precisa ser incorporado ao release realmente offline.
- A concessão inicial de permissões da extensão no Edge não pode ser falsificada nem contornada. Distribuição gerenciada via Edge Add-ons/políticas empresariais pode ser adotada nos ambientes que a permitirem.
- A promoção ao repositório canônico só deve ocorrer após segurança de armazenamento e integração com a baseline produtiva; a unidade D apresentou eventos NTFS de corrupção.
- A homologação de verdade tem de usar o Edge autenticado e uma aula real, com aprendizagem e recuperação independentes; essa prova ainda está pendente.
