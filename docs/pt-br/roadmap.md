# Roadmap

Ideias para as próximas versões do Memory Card. Nenhuma tem data ainda, e a
ordem pode mudar conforme os pedidos de quem usa.

## Ideias

### Importar a biblioteca da Steam

Trazer seus jogos da Steam como cópias digitais na sua conta Steam, cada um
ligado ao jogo do IGDB pelo ID da Steam, sem adivinhar pelo título. Usa a Web
API oficial da Steam com seu próprio ID Steam e chave de API.

### Partidas pelas integrações de jogos do Home Assistant

As integrações PlayStation Network, Steam e Xbox do Home Assistant já mostram o
que cada pessoa está jogando. Cada uma dessas entidades seria atribuída a uma
pessoa, como `tag_readers` faz com um leitor NFC, e o Memory Card registraria
uma partida quando aparecesse um jogo da biblioteca.

### Empréstimos

Anotar quem pegou uma cópia emprestada e desde quando, e depois marcar a
devolução. Um sensor no Home Assistant poderia listar os jogos emprestados.

### Notas e resenhas

Uma nota e uma resenha curta de cada pessoa, lado a lado na página do jogo.

### Retrospectiva do ano

O ano da casa em jogos: o que cada pessoa terminou, horas jogadas e os jogos
mais jogados.

### Próximos lançamentos

Datas de lançamento dos jogos da lista de desejos, vindas do IGDB, como um
calendário no Home Assistant.

### Etiquetas QR para a estante

Etiquetas QR para imprimir e colar na estante, que abrem a página do jogo. As
tags NFC continuam sendo o jeito de registrar partidas.

## Só APIs oficiais

O Memory Card usa apenas APIs oficiais e as integrações do próprio Home
Assistant. Lojas sem um jeito oficial de ler a biblioteca, como Nintendo,
PlayStation e GOG, continuam com a [importação CSV](./docs/import-export.md), a
[leitura de códigos de barras](./docs/barcodes.md) ou o cadastro manual.

## Apoie o desenvolvimento

O Memory Card é gratuito e de código aberto. Se você quer ver essas ideias
antes, pode apoiar o desenvolvimento no
[GitHub Sponsors](https://github.com/sponsors/claudiosanches) ou no
[Ko-fi](https://ko-fi.com/claudiosanches).

Ideias e pedidos são bem-vindos nas
[issues do GitHub](https://github.com/claudiosanches/ha-memory-card/issues).
