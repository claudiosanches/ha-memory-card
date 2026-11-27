# Sensores de Home Assistant

Memory Card puede añadir un dispositivo **Memory Card** a Home Assistant con
sensores para el hogar y para cada persona, pensados para paneles y
automatizaciones.

## Configuración

Elige una de dos formas con la opción `sensors`.

### Con la integración de Memory Card

No hace falta un broker MQTT, y añade un botón **Sortear un juego** y una
acción con respuesta. [Integración de Memory Card](integration.md) explica la
instalación paso a paso.

### Con MQTT

1. Instala la aplicación **Mosquitto broker** e iníciala. Home Assistant ofrece
   configurar la integración **MQTT** en cuanto el broker está en marcha.
2. En la pestaña **Configuración** de Memory Card, establece `sensors` en
   `mqtt`, guarda y reinicia Memory Card. Iníciala después del broker para que
   lo encuentre.

Ambas formas crean los mismos sensores con los mismos ID. Cambiar de una a otra
elimina los antiguos en el siguiente inicio, de modo que las gráficas de
historial continúan. Volver a poner `sensors` en `off` elimina el dispositivo.
**Ajustes > Estado** en Memory Card indica si los sensores están conectados.

## Sensores del hogar

| Entidad | Estado | Atributos |
|---|---|---|
| `sensor.memory_card_games` | Juegos con al menos una copia propia | |
| `sensor.memory_card_copies` | Copias propias | |
| `sensor.memory_card_wishlist` | Copias en la lista de deseos | |
| `sensor.memory_card_playing` | Juegos propios que alguien está jugando | `games`: hasta 50 títulos, los jugados más recientemente primero |
| `sensor.memory_card_last_added` | Título del juego cuya copia propia se añadió más recientemente. Una copia de la lista de deseos cuenta desde que la marcas como Ya lo tengo. | `added_at`, `platform` |
| `sensor.memory_card_pick` | Título de la última sugerencia aleatoria, desde Memory Card o Home Assistant | `suggested_at`, `platforms` de sus copias propias, `person` de quien es el sorteo |

## Sensores de cada persona

Cada persona con un estado de juego o una sesión obtiene sus propios sensores,
hasta 16, con su nombre, como `sensor.memory_card_ana_backlog`. Una persona de
la que Memory Card no tiene nombre, como alguien conocido solo por un lector
NFC, recibe ID de entidad a partir de su ID de usuario y aparece como Alguien:

| Entidad | Estado | Atributos |
|---|---|---|
| `sensor.memory_card_<name>_backlog` | Sus juegos propios en un estado que cuenta como pendiente | |
| `sensor.memory_card_<name>_playing` | Sus juegos propios en un estado que cuenta como jugando | `games`: hasta 10 títulos, los jugados más recientemente primero |
| `sensor.memory_card_<name>_last_played` | Título del último juego que jugó | `date`, `platform` de la copia jugada |

Los sensores de una persona conservan sus ID de entidad aunque cambie su nombre
en Home Assistant.

Los recuentos guardan estadísticas a largo plazo, así que las gráficas de
historial muestran cómo evolucionan la colección y cada lista de pendientes.
Los sensores se actualizan un segundo después de cada cambio y aparecen como no
disponibles mientras Memory Card está detenida.

## Sortear un juego desde Home Assistant

Cada sorteo sale de los pendientes de una persona, así que debe indicar de
quién.

Con la integración, el botón **Sortear un juego** sortea para quien lo pulsa, y
la acción `memory_card.roll_pick` sortea para quien la llama o para la persona
que indique una automatización. [Integración de Memory
Card](integration.md#the-roll_pick-action) detalla sus campos y ejemplos.

Con MQTT, que no sabe quién envió un mensaje, publica en
`memory_card/pick/roll` con el ID de usuario de la persona. Un mensaje sin
persona se ignora, y los sorteos llegan como máximo una vez por segundo.

```yaml
action: mqtt.publish
data:
  topic: memory_card/pick/roll
  payload: '{"person": "<user ID>", "device": ["switch"], "min_local_players": 2}'
```

Un botón del panel puede sortear para una persona. Añade una tarjeta **Botón**,
configura su acción al pulsar como **Realizar acción**, elige **MQTT:
Publicar** y rellena el tema y el payload. Cada persona copia su ID de usuario
en **Ajustes** de Memory Card, como en [Nombrar a los
gestores](getting-started.md#name-the-managers).

![Tarjeta Botón que sortea un juego para una persona por MQTT](https://ha-memory-card.pages.dev/images/mqtt-roll-button.webp)

La misma tarjeta en YAML:

```yaml
type: button
name: Sortear un juego para Ana
icon: mdi:dice-5
show_state: false
tap_action:
  action: perform-action
  perform_action: mqtt.publish
  data:
    topic: memory_card/pick/roll
    payload: '{"person": "<user ID>"}'
```

Los filtros coinciden con el selector de la página Pendientes: `play_status`
(una lista de claves de estados de juego), `platform`, `device`, `genre`,
`game_mode` (listas de slugs), `min_local_players` y `max_hours_to_beat` (horas
de la historia principal). Un slug es la forma en minúsculas y separada por
guiones del nombre, como `switch` o `living-room-switch`.

Los sorteos requieren que haya gestores en la configuración, y la persona debe
ser un gestor o un lector con `viewers_track_progress` activada. Cada persona
solo puede sortear para sí misma. Un sorteo que nombra a una persona, desde una
automatización o por MQTT, requiere a alguien que Memory Card conozca: un
gestor o alguien que ya haya cambiado algo en la aplicación, como su estado de
juego.
