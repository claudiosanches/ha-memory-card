# Sensores do Home Assistant

O Memory Card pode adicionar um dispositivo **Memory Card** ao Home Assistant
com sensores para a casa e para cada pessoa, para usar em painéis e automações.

## Configurar os sensores

Escolha um de dois modos com a opção `sensors`.

### Com a integração Memory Card

Não precisa de broker MQTT e adiciona o botão **Sortear um jogo** e uma ação
com resposta. [Integração Memory Card](integration.md) mostra passo a passo
como instalar.

### Com MQTT

1. Instale o aplicativo **Mosquitto broker** e inicie-o. O Home Assistant
   oferece configurar a integração **MQTT** assim que o broker estiver rodando.
2. Na aba **Configuração** do Memory Card, defina `sensors` como `mqtt`, salve
   e reinicie o Memory Card. Inicie-o depois do broker para que ele o encontre.

Os dois modos criam os mesmos sensores com os mesmos IDs. Trocar de um modo
para o outro remove os antigos na próxima inicialização, então os gráficos de
histórico continuam sem interrupção. Voltar `sensors` para `off` remove o
dispositivo. **Configurações > Status** no Memory Card informa se os sensores
estão conectados.

## Sensores da casa

| Entidade | Estado | Atributos |
|---|---|---|
| `sensor.memory_card_games` | Jogos com pelo menos uma cópia que você tem | |
| `sensor.memory_card_copies` | Cópias que você tem | |
| `sensor.memory_card_wishlist` | Cópias na lista de desejos | |
| `sensor.memory_card_playing` | Jogos que você tem e alguém está jogando | `games`: até 50 títulos, o jogado mais recentemente primeiro |
| `sensor.memory_card_last_added` | Título do jogo cuja cópia adicionada à coleção foi a mais recente. Uma cópia da lista de desejos conta a partir do momento em que você marca Comprei. | `added_at`, `platform` |
| `sensor.memory_card_pick` | Título do último jogo sorteado, pelo Memory Card ou pelo Home Assistant | `suggested_at`, `platforms` das cópias na coleção, `person` de quem é o sorteio |

## Sensores de cada pessoa

Cada pessoa com um status de jogo ou uma sessão ganha seus próprios sensores,
até 16 pessoas, com o nome dela, como `sensor.memory_card_ana_backlog`. Uma
pessoa para a qual o Memory Card não conhece o nome, como alguém conhecido só
por um leitor NFC, recebe IDs de entidade a partir do ID de usuário e aparece
como Alguém:

| Entidade | Estado | Atributos |
|---|---|---|
| `sensor.memory_card_<name>_backlog` | Jogos da coleção em um status que conta como backlog para a pessoa | |
| `sensor.memory_card_<name>_playing` | Jogos da coleção em um status que conta como jogando para a pessoa | `games`: até 10 títulos, o jogado mais recentemente primeiro |
| `sensor.memory_card_<name>_last_played` | Título do último jogo jogado pela pessoa | `date`, `platform` da cópia jogada |

Os sensores de uma pessoa mantêm os mesmos IDs de entidade quando ela muda de
nome no Home Assistant.

As contagens mantêm estatísticas de longo prazo, então gráficos de histórico
mostram como a coleção e cada backlog mudam. Os sensores atualizam um segundo
após cada alteração e aparecem como indisponíveis enquanto o Memory Card está
parado.

## Sortear um jogo pelo Home Assistant

Um sorteio sai do backlog de uma pessoa, então o pedido precisa informar de
quem.

Com a integração, o botão **Sortear um jogo** sorteia para quem o pressiona, e
a ação `memory_card.roll_pick` sorteia para quem a chama ou para a pessoa que
uma automação informar. [Integração Memory Card](integration.md#the-roll_pick-action)
lista seus campos e exemplos.

Com MQTT, que não sabe quem enviou a mensagem, publique em
`memory_card/pick/roll` com o ID de usuário da pessoa. Uma mensagem sem pessoa
é ignorada, e os sorteios acontecem no máximo uma vez por segundo.

```yaml
action: mqtt.publish
data:
  topic: memory_card/pick/roll
  payload: '{"person": "<user ID>", "device": ["switch"], "min_local_players": 2}'
```

Um botão no painel pode sortear para uma pessoa. Adicione um cartão **Botão**,
defina a ação ao tocar como **Executar ação**, escolha **MQTT: Publicar** e
preencha o tópico e o payload. Cada pessoa copia seu ID de usuário em
**Configurações** no Memory Card, como em
[Nomear os gestores](getting-started.md#name-the-managers).

![Um cartão Botão que sorteia um jogo para uma pessoa por MQTT](https://ha-memory-card.pages.dev/images/mqtt-roll-button.webp)

O mesmo cartão em YAML:

```yaml
type: button
name: Sortear um jogo para Ana
icon: mdi:dice-5
show_state: false
tap_action:
  action: perform-action
  perform_action: mqtt.publish
  data:
    topic: memory_card/pick/roll
    payload: '{"person": "<user ID>"}'
```

Os filtros correspondem ao sorteador da página Backlog: `play_status` (uma
lista de chaves de status de jogo), `platform`, `device`, `genre`, `game_mode`
(listas de slugs), `min_local_players` e `max_hours_to_beat` (horas da história
principal). Um slug é o nome em minúsculas separado por hifens, como `switch`
ou `living-room-switch`. Ligue `show_slugs` na aba **Configuração** do aplicativo
para vê-los ou editá-los nos formulários de [Gerenciar](manage.md) e
[Hardware](hardware.md).

Os sorteios exigem gestores definidos na configuração, e a pessoa precisa ser
gestora ou visualizadora com `viewers_track_progress` ligado. Cada pessoa
sorteia apenas para si mesma. Um sorteio que nomeia uma pessoa, vindo de uma
automação ou do MQTT, precisa de alguém que o Memory Card já conheça: um
gestor, ou alguém que já tenha alterado algo nele, como o próprio status de
jogo.
