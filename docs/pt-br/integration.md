# Integração Memory Card

A integração Memory Card leva os [sensores](sensors.md) do aplicativo e um
botão **Sortear um jogo** para o Home Assistant, além da ação
`memory_card.roll_pick` para automações e scripts. Ela não precisa de broker
MQTT.

O aplicativo faz o trabalho: conta, sorteia e avisa a integração sobre o que
mudou. A integração cria os mesmos sensores, com os mesmos IDs, que o modo MQTT
do aplicativo, então você pode alternar entre os dois sem quebrar painéis. O
botão **Sortear um jogo** só existe com a integração.

## Integração ou MQTT

| | Integração | MQTT |
|---|---|---|
| Precisa de | A integração pelo Marketplace | O aplicativo Mosquitto broker e a integração MQTT |
| Sensores | Os mesmos | Os mesmos |
| Botão **Sortear um jogo** | Sorteia para quem pressiona | Não tem |
| Sorteios por automações | `memory_card.roll_pick`, com resposta | `mqtt.publish` com o ID de usuário da pessoa, sem resposta |

Escolha a integração, a menos que você já rode o Mosquitto para outros
dispositivos e queira manter tudo nele.

## Requisitos

Home Assistant 2026.11 ou mais recente, com o aplicativo Memory Card 0.1.0 ou
mais recente.

## Instalar

1. No Home Assistant, vá em **Configurações > Marketplace**, pesquise
   **Memory Card** e abra-o. Selecione **Instalar**, depois **Instalar** de novo
   na janela, e reinicie o Home Assistant se ele pedir. Se a configuração da
   integração abrir logo em seguida, ela avisa que o Memory Card se configura
   sozinho: feche-a e siga em frente.
2. No Home Assistant, vá em **Configurações > Aplicativos > Memory Card** e
   abra a aba **Configuração**. Defina **Sensores do Home Assistant**
   (`sensors`) como `integration`, salve e reinicie o aplicativo na aba
   **Informações**.
3. Vá em **Configurações > Dispositivos e serviços**. O Memory Card aparece em
   **Descoberto**. Selecione **Adicionar** e depois **Enviar**.

Não é preciso digitar nada: o aplicativo informa ao Home Assistant onde
encontrá-lo e qual token usar. **Configurações > Status** no Memory Card mostra
**Sensores do Home Assistant: Conectado** assim que funciona.

Tentar adicionar o Memory Card à mão por **Adicionar integração** para com uma
mensagem avisando que ele se configura sozinho: use a opção do aplicativo em
vez disso.

Este botão abre o Memory Card no seu Marketplace. Um Marketplace que ainda não o
lista oferece adicionar
[o repositório da integração](https://github.com/claudiosanches/ha-memory-card-integration)
como repositório personalizado, o que exige o Marketplace conectado ao GitHub.

[![Abrir sua instância do Home Assistant e abrir um repositório no Marketplace.](https://my.home-assistant.io/badges/marketplace_repository.svg)](https://my.home-assistant.io/redirect/marketplace_repository/?owner=claudiosanches&repository=ha-memory-card-integration&category=integration)

## O que ela adiciona

Um dispositivo **Memory Card** com:

- os sensores da casa: jogos, cópias, lista de desejos, jogando, último
  adicionado e o sorteio atual;
- três sensores para cada pessoa que registra partidas no Memory Card, até 16
  pessoas: o backlog dela, o que está jogando e o que jogou por último;
- o botão **Sortear um jogo**.

[Sensores do Home Assistant](sensors.md) lista cada entidade com seu estado e
seus atributos. Os valores atualizam cerca de um segundo após cada alteração no
Memory Card, e as entidades aparecem como indisponíveis enquanto o aplicativo
está parado. Os sensores de uma pessoa aparecem e somem conforme pessoas
começam ou deixam de registrar partidas.

Os nomes das entidades vêm em inglês, espanhol e português do Brasil, seguindo
o idioma do Home Assistant.

## Botão Sortear um jogo

**Sortear um jogo** (`button.memory_card_roll_pick`) escolhe um jogo aleatório
da coleção no backlog de quem pressiona o botão, como faz **Escolha por mim**
no [Backlog](backlog.md). O sorteio aparece em `sensor.memory_card_pick`, com a
indicação de quem sorteou no atributo `person`.

Um cartão de painel com ambos:

```yaml
type: entities
title: O que jogar
entities:
  - entity: sensor.memory_card_pick
  - entity: button.memory_card_roll_pick
```

Um cartão **Botão** também pode acioná-lo: defina a ação ao tocar como
**Executar ação**, escolha **Pressionar botão** e selecione **Sortear um jogo**
como alvo.

![Um cartão Botão que pressiona Sortear um jogo](https://ha-memory-card.pages.dev/images/integration-roll-button.webp)

O mesmo cartão em YAML:

```yaml
type: button
name: Sortear um jogo
icon: mdi:dice-5
show_state: false
tap_action:
  action: perform-action
  perform_action: button.press
  target:
    entity_id: button.memory_card_roll_pick
```

Uma automação que tenta pressionar o botão não tem uma pessoa associada, então
o pedido é recusado. Use a ação abaixo nesse caso.

## A ação roll_pick {#the-roll_pick-action}

`memory_card.roll_pick` sorteia um jogo aleatório da coleção a partir do
backlog de uma pessoa e responde com `title` e `platforms`.

| Campo | O que faz |
|---|---|
| `person` | De quem é o backlog, como uma entidade `person`. Deixe vazio para sortear para você mesmo. Uma automação precisa informar alguém. |
| `play_status` | Chaves de status de jogo para sortear. Vazio usa todo status que conta como backlog. |
| `platform` | Slugs de plataforma. |
| `device` | Slugs de dispositivo: cópias nesses dispositivos, ou sem dispositivo em uma plataforma que um deles roda. |
| `genre` | Slugs de gênero. |
| `game_mode` | Slugs de modo de jogo. |
| `min_local_players` | Pelo menos essa quantidade de jogadores na mesma tela. |
| `max_hours_to_beat` | No máximo essa quantidade de horas na história principal, segundo o IGDB. Jogos sem tempo ficam de fora. |

Os campos de slug aceitam um único slug ou uma lista de até 10. Um slug é o
nome em minúsculas separado por hifens, como `switch` ou `living-room-switch`.
Ligue `show_slugs` na aba Configuração do aplicativo para vê-los ou editá-los
nos formulários de [Gerenciar](manage.md) e [Hardware](hardware.md).

Um sorteio de noite de jogos para duas pessoas no Switch, a partir do backlog
de Ana:

```yaml
actions:
  - action: memory_card.roll_pick
    data:
      person: person.ana
      device: switch
      min_local_players: 2
    response_variable: pick
  - action: persistent_notification.create
    data:
      title: Noite de jogos
      message: "{{ pick.title }} em {{ pick.platforms | join(', ') }}"
```

Quem pode sortear:

- O aplicativo precisa ter gestores, e a pessoa precisa ser gestora ou
  visualizadora autorizada a registrar partidas (`viewers_track_progress`).
- Cada pessoa sorteia apenas para si mesma. Um script executado por alguém a
  partir de um painel conta como essa pessoa, então não pode informar outra.
- Uma automação não tem pessoa própria, então precisa indicar uma, e o Memory
  Card precisa conhecê-la: um gestor, ou alguém que já tenha alterado algo
  nele, como o próprio status de jogo.
- A entidade `person` precisa estar vinculada a um usuário do Home Assistant.

## Erros

| Mensagem | O que fazer |
|---|---|
| Informe uma pessoa para sortear um jogo. | Adicione `person`, ou faça o sorteio de um painel como você mesmo. |
| Você só pode sortear um jogo para si mesmo. | Deixe `person` de fora, ou informe você mesmo. |
| O Memory Card não registra partidas para esta pessoa. | Torne-a gestora ou ligue `viewers_track_progress`. Uma pessoa que o Memory Card nunca viu precisa abri-lo e definir um status de jogo primeiro. |
| Esta pessoa não tem um usuário do Home Assistant. | Vincule a pessoa a um usuário em **Configurações > Pessoas**. |
| Nenhum jogo corresponde a estes filtros. | Afrouxe os filtros ou adicione jogos ao backlog. |
| O Memory Card não conseguiu ler estes filtros. | Confira se cada filtro usa slugs e chaves válidos em minúsculas. |
| Não foi possível alcançar o aplicativo Memory Card. | Inicie o aplicativo e confira a aba **Log**. |
| O Memory Card não está configurado. | Adicione o Memory Card em **Descoberto**, em **Configurações > Dispositivos e serviços**. |

## Solução de problemas

- **Configurações > Status** no Memory Card mostra o estado dos sensores.
  **Aguardando a integração Memory Card** significa que o aplicativo está
  pronto, mas a integração ainda não foi adicionada: adicione-a em
  **Descoberto**, ou instale-a se ela ainda não estiver lá.
- A aba **Log** do aplicativo mostra o motivo caso a conexão não tenha iniciado.
- **Baixar diagnóstico**, na página da integração em **Configurações >
  Dispositivos e serviços**, traz o que a integração recebe do aplicativo, sem
  o token, as partidas de ninguém nem os IDs de usuário. Anexe esse arquivo ao
  relatar um problema.

## Atualizar e remover

As atualizações chegam pelo Marketplace. Reinicie o Home Assistant depois de
atualizar.

Para remover, defina a opção `sensors` do aplicativo como `off` ou `mqtt` e
reinicie o aplicativo: o Home Assistant remove a entrada e as entidades da
integração. Depois, remova **Memory Card** do Marketplace. Com `mqtt`, os
mesmos IDs de entidade voltam por MQTT.

Bugs na integração devem ser enviados na página de
[issues dela](https://github.com/claudiosanches/ha-memory-card-integration/issues).
