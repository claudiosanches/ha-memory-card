# Página do jogo

Selecione um jogo na [Biblioteca](library.md) para abrir a página dele. Ela tem
quatro partes: o cabeçalho, seu progresso, os detalhes do jogo e os lançamentos
e cópias.

![A página de um jogo com seu progresso e o da casa](https://ha-memory-card.pages.dev/images/game.webp)

_Títulos de jogos e capas pertencem aos seus respectivos proprietários._

## Cabeçalho

O cabeçalho mostra a capa, o título e, para jogos vinculados ao IGDB, a data do
primeiro lançamento, desenvolvedoras e publicadoras. Também mostra quem
adicionou o jogo e quem o alterou por último.

Gestores têm estes botões:

- **Atualizar pelo IGDB** busca novamente os detalhes e a capa do jogo.
- **Trocar correspondência do IGDB** vincula o jogo a outro jogo do IGDB;
  **Vincular ao IGDB** aparece quando ainda não há vínculo. Veja
  [IGDB](igdb.md).
- **Editar** altera o título, os outros nomes em **Também conhecido como**, o
  resumo, os modos de jogo, a contagem de jogadores e as etiquetas.
- O botão de lixeira exclui o jogo com todas as cópias e as partidas de todo
  mundo para ele.

Para usar sua própria capa, um gestor seleciona a capa e envia uma imagem JPEG,
PNG ou WebP de até 10 MB e 100 megapixels. O IGDB nunca substitui uma capa
enviada, e removê-la traz de volta a do IGDB. Cada lançamento pode ter a própria
capa da mesma forma.

## Seu progresso

Esta parte é sua. Cada pessoa da casa tem a própria.

- **Status de jogo** coloca o jogo no seu [Backlog](backlog.md): backlog,
  jogando, pausado, zerado, abandonado, 100% ou um status criado pelos gestores.
- **Joguei hoje** registra uma sessão para hoje. Depois do registro, muda para
  **Registrado hoje**.
- **Jogado por último** mostra a sua sessão mais recente.
- **Histórico de jogo** lista suas sessões, da mais recente para a mais antiga.
  **Registrar uma sessão** adiciona uma com o dia, a cópia jogada e os minutos.
  Os botões de lápis e lixeira editam ou excluem uma sessão.
- Uma sessão para um jogo no backlog, ou sem status, muda o status para jogando.
  Jogos pausados e terminados mantêm o status.
- **Outros na casa** mostra o status de cada outra pessoa para o jogo, o número
  de sessões e quando jogou pela última vez. É somente leitura.

Gestores sempre mantêm as próprias partidas. Visualizadores mantêm as próprias
quando a opção `viewers_track_progress` do aplicativo está ligada; caso
contrário, veem as partidas da casa, mas não podem definir as próprias.

## Detalhes do jogo

Modos de jogo, gêneros, jogadores na mesma tela e online, outros nomes e o
resumo vêm do IGDB ou de **Editar**.

**Tempo para zerar** vem do IGDB: horas para a história principal, com extras e
para fazer tudo, conforme os jogadores informam no IGDB.

**Etiquetas** aparecem abaixo do resumo. Selecionar uma abre a biblioteca
filtrada por ela. Gestores criam etiquetas em [Gerenciar](manage.md).

**Outras versões** lista jogos da coleção que o IGDB identifica como versões
deste, como remaster, remake, port ou outra edição. Eles continuam como jogos
separados, com cópias e progresso próprios.

## Lançamentos e cópias

Um lançamento é o jogo em uma plataforma e uma região, como o lançamento
norte-americano de Switch. Uma cópia é um exemplar que você tem ou quer: física
ou digital, em um dispositivo, comprada em uma loja.

Gestores adicionam e alteram ambos:

1. **Adicionar cópia** pede a plataforma, a região e o formato, e se você já tem
   (**Eu tenho**) ou quer comprar (**Na lista de desejos**).
2. Preencha o que souber: código do produto, edição, o dispositivo onde está,
   para cópias digitais a conta dona da cópia, onde foi comprada, estado, data
   da compra, preço pago, quanto venderia hoje, código de barras e observações.
3. **Salvar e adicionar outra** mantém o formulário aberto para a próxima cópia.

Cada lançamento tem **Gerenciar lançamento**, para editar título, código do
produto e data de lançamento, ou adicionar outra cópia dele. Cada cópia tem um
botão de lápis para editar e um menu **⋯**:

- **Adicionar fotos** guarda fotos da caixa, do cartucho ou do manual. No
  celular, oferece a câmera. Uma cópia aceita até 20 fotos JPEG, PNG ou WebP de
  até 10 MB cada. O Memory Card remove a localização, detalhes da câmera e
  outros metadados que o celular grava na foto.
- **Adicionar jogos desta cópia** transforma a cópia em uma coletânea, como uma
  coleção em um só cartucho. Cada jogo incluído ganha uma cópia que acompanha
  esta: ao mudar plataforma, região, formato, dispositivo, conta, loja, estado
  ou status de lista de desejos da coletânea, eles mudam junto. Quando a
  coletânea está vinculada ao IGDB, o Memory Card sugere os jogos que o IGDB
  lista nela.
- **Vincular tag NFC** faz a leitura de uma tag registrar uma partida para esta
  cópia. Veja [Tags NFC](nfc-tags.md).
- **Excluir cópia** remove a cópia. Excluir a última cópia pergunta se você
  também quer excluir o jogo.

Um lançamento deixa de existir com sua última cópia, inclusive quando essa cópia
muda para outra plataforma, região ou código do produto.
