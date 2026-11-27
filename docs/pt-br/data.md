# Dados e backups

Tudo o que o aplicativo armazena fica na pasta de dados dele, que o Home
Assistant inclui nos backups:

- `db/memory-card.sqlite`: a coleção e as partidas de todas as pessoas.
- `db/pre-migration/`: uma cópia do banco de dados feita antes de cada
  atualização que altera sua estrutura. As cinco últimas são mantidas.
- `covers/`: capas baixadas e enviadas, além de fotos de cópias e hardware.

Os backups param o aplicativo por um momento para que o banco de dados seja
copiado em estado consistente. O aplicativo volta a iniciar sozinho quando o
backup termina.

Para manter uma cópia fora do Home Assistant, **Exportar coleção** em
[Importar / exportar](import-export.md) baixa todas as cópias em um arquivo
CSV, e a aba **Hardware** baixa todos os itens de hardware. Esses arquivos não
incluem sessões de jogo, histórico de status, fotos nem capas.
