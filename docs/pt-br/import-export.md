# Importar / exportar

**Importar / exportar** na barra lateral leva a coleção entre o Memory Card e
uma planilha em formato CSV. Só gestores veem esta página.

![Importando um arquivo de outra planilha](https://ha-memory-card.pages.dev/images/import.webp)

## Exportar

- **Exportar coleção** baixa todas as cópias, uma por linha.
- **Baixar modelo em branco** baixa só as colunas, sem linhas, para preencher em
  uma planilha.
- **Exportar CSV** na [Biblioteca](library.md) baixa apenas as cópias que
  correspondem aos filtros atuais.

A coluna `play_status` guarda o seu próprio status. As partidas das outras
pessoas ficam fora do arquivo.

## Importar

1. Salve a planilha como CSV UTF-8. Arquivos que usam `;` como separador também
   funcionam.
2. Em **Arquivo CSV**, escolha o arquivo.
3. **Como suas colunas foram lidas** mostra para qual coluna cada cabeçalho foi
   enviado. O Memory Card reconhece os próprios nomes e nomes comuns de outras
   planilhas e apps de coleção, como `Title`, `Name`, `System`, `Console`,
   `Serial`, `UPC`, `Price Paid`, `Purchase Price`, `Collection Status` ou
   `Comments`. Escolha outra coluna para um cabeçalho lido errado, ou **Não
   importada** para deixá-lo de fora.
4. Confira a prévia: jogos e cópias novos e atualizados, além das plataformas,
   dispositivos, contas, lojas e etiquetas que serão criados.
5. Linhas com erros bloqueiam a importação. Corrija-as na planilha e selecione
   **Escolher outro arquivo**. Linhas com avisos são importadas como estão.
6. Selecione **Importar**. Nada muda até você fazer isso.

Depois de importar, **Revisar correspondências do IGDB** passa pelos jogos que
ainda não estão vinculados ao IGDB. Veja [IGDB](igdb.md#close-matches).

## Como as linhas são lidas

- Uma linha por cópia. Uma linha só com `game_title`, e opcionalmente `igdb_id`
  e `play_status`, adiciona um jogo sem cópias.
- Os nomes das colunas não diferenciam maiúsculas e podem estar em qualquer
  ordem. Só `game_title` é obrigatória em toda linha; linhas que adicionam uma
  cópia também precisam de `platform`, `region` e `format`.
- Nomes de plataforma, dispositivo, conta e loja são comparados com os que você
  já tem, ignorando maiúsculas e minúsculas. Plataformas comuns e as lojas
  digitais (Nintendo eShop, PlayStation Store, Microsoft Store, Steam, Epic
  Games Store, GOG, itch.io) já existem desde o início. Nomes que não
  correspondem a nada são criados: um dispositivo novo entra como console, o
  que você pode alterar em [Hardware](hardware.md), e uma conta nova aparece em
  [Gerenciar > Contas](manage.md#accounts). Uma linha não pode colocar uma
  cópia em um controle, acessório ou brinquedo.
- A exportação preenche `copy_id`. Mantenha-o para atualizar essa cópia ao
  importar o arquivo de novo; deixe-o vazio para adicionar uma cópia nova.
  Colunas que o arquivo não tem mantêm os valores da cópia, enquanto uma célula
  vazia apaga o valor. Importar uma exportação sem alterações não muda nada.
- Uma importação nunca exclui uma cópia, um jogo ou uma etiqueta.

## Colunas

| Coluna | Valores aceitos |
|---|---|
| `copy_id` | Número da exportação, ou vazio |
| `status` | `owned` ou `wanted` (vazio significa `owned`). `In Collection` é lido como `owned`, `Wish List` e `On Order` como `wanted`. |
| `igdb_id` | ID do jogo no IGDB, opcional. Diferencia jogos com o mesmo título. |
| `game_title` | Obrigatória |
| `play_status` | O seu: `backlog`, `playing`, `paused`, `completed`, `dropped`, `mastered`, ou o nome ou a chave de um status adicionado por um gestor. Palavras de outros apps também funcionam, como `Unplayed`, `Unfinished`, `Beaten` ou `Retired`. |
| `labels` | Nomes das etiquetas, até 50, separados por vírgulas ou ponto e vírgula. A importação as adiciona ao jogo e cria em cinza as que ainda não existem. A exportação as separa com `; `. |
| `platform` | Nome, abreviação ou slug da plataforma |
| `region` | `na`, `eu`, `au`, `nz`, `jp`, `cn`, `asia`, `kr`, `br`, `ww` (mundial ou sem região); também `NTSC-U`, `PAL`, `NTSC-J` |
| `release_title` | Título neste lançamento, quando difere do título do jogo |
| `product_code` | Serial ou código do produto |
| `format` | `physical` ou `digital` |
| `edition` | Texto livre, como `Edição de Colecionador` |
| `device` | Console, portátil ou PC onde a cópia está |
| `account` | Só cópias digitais: a conta de loja dona da cópia |
| `store` | Nome da loja |
| `condition` | Só cópias físicas: `sealed`, `cib`, `no_manual`, `loose` |
| `purchased_on` | Data da compra, `AAAA-MM-DD` |
| `price` | Preço pago, ou esperado para uma cópia desejada. `59.99`, `59,99` e `1.299,90` funcionam; a exportação escreve `1299.90`. Precisa de `currency`. |
| `currency` | Código ISO 4217 de três letras, como `BRL`, `USD` ou `EUR` |
| `estimated_value` | Quanto uma cópia que você tem venderia hoje, lido como `price`. Usa `estimated_currency`, ou `currency` quando ela estiver vazia. |
| `estimated_currency` | Código de três letras de `estimated_value`, quando difere de `currency` |
| `barcode` | Números do UPC ou EAN da caixa |
| `notes` | Texto livre |
| `bundle_copy_id` | `copy_id` da cópia da coletânea em que esta cópia veio. Plataforma, região, formato, dispositivo, conta, loja, estado e status passam a seguir essa cópia, e preço, valor estimado e data de compra próprios ficam de fora. Vazio tira a cópia da coletânea. |

**Colunas e valores aceitos** no fim da página mostra a mesma lista no app.

A exportação escapa células que uma planilha interpretaria como fórmula, para
que abrir o arquivo seja seguro.

## Hardware

A aba **Hardware** leva seu [hardware](hardware.md) para um arquivo CSV próprio
e de volta. **Exportar hardware** baixa todos os itens, e **Baixar modelo em
branco** baixa só as colunas.

Nenhum dos dois arquivos guarda as contas em si. O arquivo de jogos informa a
conta de cada cópia digital, mas o dono, a loja, as plataformas e as
observações de uma conta ficam no banco de dados, incluído nos backups do Home
Assistant.

**Colunas e valores aceitos** no fim da aba lista as colunas dentro do app.

A importação mostra uma prévia antes. Cada linha atualiza o hardware com o mesmo
`device_id` ou, sem ele, o mesmo nome, e adiciona hardware novo caso contrário.
Colunas que o arquivo não tem mantêm os valores, enquanto uma célula vazia
apaga o valor. Lojas que ainda não existem são adicionadas. Linhas com erros
bloqueiam a importação.

| Coluna | Valores aceitos |
|---|---|
| `device_id` | Número da exportação, ou vazio |
| `status` | `owned` ou `wanted` (vazio significa `owned`) |
| `name` | Obrigatória |
| `kind` | `console`, `handheld`, `pc`, `controller`, `accessory` ou `toy`. Obrigatória para hardware novo. |
| `model` | Texto livre |
| `platform` | Para qual plataforma um controle, acessório ou brinquedo serve: nome, abreviação ou slug |
| `plays` | O que um console, portátil ou PC roda: plataformas separadas por `;` |
| `region` | Como nas cópias |
| `condition` | `sealed`, `cib`, `no_manual`, `loose` |
| `store` | Nome da loja |
| `purchased_on` | `AAAA-MM-DD` |
| `price`, `currency` | Preço pago, ou esperado para hardware desejado, lido como nas cópias |
| `estimated_value`, `estimated_currency` | Quanto o hardware que você tem venderia hoje. `estimated_value` usa `estimated_currency`, ou `currency` quando ela estiver vazia. |
| `serial_number` | Texto livre |
| `barcode` | Números do UPC ou EAN |
| `notes` | Texto livre |
| `active` | `yes` ou `no` |
