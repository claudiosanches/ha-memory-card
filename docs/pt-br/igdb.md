# IGDB

O [IGDB](https://www.igdb.com) fornece ao Memory Card títulos, capas, datas de
lançamento, desenvolvedoras, publicadoras, gêneros, modos de jogo, contagem de
jogadores, outros nomes e o tempo para zerar cada jogo. O Memory Card funciona
sem ele: nesse caso você preenche os jogos à mão.
[Primeiros passos](getting-started.md#connect-igdb) explica como conectá-lo.

## Vincular um jogo

- **Adicionar jogo** na [Biblioteca](library.md) busca no IGDB enquanto você
  adiciona um jogo.
- Na [página de um jogo](game.md), **Vincular ao IGDB** ou **Trocar
  correspondência do IGDB** faz uma nova busca. Os resultados mostram o ano e
  as plataformas de cada correspondência e indicam quais jogos já estão na sua
  coleção.
- **Atualizar pelo IGDB** na página de um jogo busca os detalhes dele de novo.

Lançamentos do Japão, da Coreia e da Europa recebem a capa regional quando o
IGDB tem uma. Uma capa enviada manualmente sempre permanece.

Quando o IGDB indica que dois jogos vinculados na sua coleção são versões um do
outro, como um jogo e seu remaster, a página de cada um lista o outro em
**Outras versões**.

## Sincronização

**Manutenção > IGDB > Sincronizar com o IGDB** vincula cada jogo cujo título
corresponde a exatamente um jogo do IGDB e depois atualiza os detalhes e as
capas de todos os jogos vinculados. O resultado mostra quantos foram
vinculados, atualizados, ficaram sem correspondência e falharam.

Logo abaixo, **Revisar títulos dos jogos** lista jogos vinculados cujo título
difere no IGDB. **Usar o do IGDB** adota o título do IGDB, **Manter o meu**
mantém o seu e **Usar o do IGDB em todos** adota todos de uma vez. Um título
mantido só volta a aparecer aqui se o IGDB renomear o jogo.

## Correspondências próximas {#close-matches}

**Revisar correspondências próximas**, ao lado de **Sincronizar com o IGDB**,
lista os jogos sem vínculo com o IGDB junto com os nomes mais próximos que o
IGDB tem e o grau de semelhança de cada um.

![Correspondências próximas do IGDB](https://ha-memory-card.pages.dev/images/close-matches.webp)

_Títulos de jogos pertencem aos seus respectivos proprietários._

Para cada jogo:

- escolha o jogo certo do IGDB e selecione **Vincular**;
- **Depois** deixa para outra hora;
- **Não está no IGDB** impede que ele volte, para jogos que o IGDB não tem.

**Mostrar mais jogos** carrega os próximos. Um jogo sem nenhum nome parecido
ainda pode ser vinculado à mão pela página dele.

## Ficar offline

Para impedir que o Memory Card acesse a internet, desligue a opção
`external_services`. As credenciais do IGDB são ignoradas, jogos, capas e
detalhes são informados à mão, e as leituras de [código de barras](barcodes.md)
só verificam as suas próprias cópias.
