# Integración de Memory Card

La integración de Memory Card lleva los [sensores](sensors.md) de la aplicación
y un botón **Sortear un juego** a Home Assistant, además de una acción
`memory_card.roll_pick` para automatizaciones y scripts. No necesita ningún
broker MQTT.

La aplicación hace el trabajo: cuenta, sortea y avisa a la integración de lo
que cambió. La integración crea los mismos sensores, con los mismos ID, que el
modo MQTT de la aplicación, así que puedes cambiar de uno a otro sin romper tus
paneles. El botón **Sortear un juego** solo está disponible con la integración.

## Integración o MQTT

| | Integración | MQTT |
|---|---|---|
| Necesita | La integración desde el Marketplace | La aplicación Mosquitto broker y la integración MQTT |
| Sensores | Los mismos | Los mismos |
| Botón **Sortear un juego** | Sortea para quien lo pulsa | Ninguno |
| Sorteos desde automatizaciones | `memory_card.roll_pick`, con respuesta | `mqtt.publish` con el ID de usuario de la persona, sin respuesta |

Elige la integración, salvo que ya uses Mosquitto para otros dispositivos y
quieras tenerlo todo allí.

## Requisitos

Home Assistant 2026.11 o posterior, con la aplicación Memory Card 0.1.0 o
posterior.

## Instalación

1. En Home Assistant, ve a **Ajustes > Marketplace**, busca **Memory Card** y
   ábrelo. Selecciona **Instalar**, luego **Instalar** otra vez en el diálogo,
   y reinicia Home Assistant si te lo pide. Si la configuración de la
   integración se abre enseguida, indica que Memory Card se configura solo:
   ciérrala y continúa.
2. En Home Assistant, ve a **Ajustes > Aplicaciones > Memory Card** y abre la
   pestaña **Configuración**. Establece **Sensores de Home Assistant**
   (`sensors`) en `integration`, guarda y reinicia la aplicación desde la
   pestaña **Información**.
3. Ve a **Ajustes > Dispositivos y servicios**. Memory Card aparecerá en
   **Descubierto**. Selecciona **Añadir** y luego **Enviar**.

No hace falta escribir nada: la aplicación indica a Home Assistant dónde
encontrarla y qué token usar. **Ajustes > Estado** en Memory Card muestra
**Sensores de Home Assistant: Conectado** en cuanto funciona.

Añadir Memory Card a mano desde **Añadir integración** se detiene con un
mensaje que indica que se configura sola: usa en su lugar la opción de la
aplicación.

Este botón abre Memory Card en tu Marketplace. Un Marketplace que aún no lo
incluye ofrece añadir
[el repositorio de la integración](https://github.com/claudiosanches/ha-memory-card-integration)
como repositorio personalizado, lo que requiere el Marketplace conectado a
GitHub.

[![Abre tu instancia de Home Assistant y abre un repositorio en el Marketplace.](https://my.home-assistant.io/badges/marketplace_repository.svg)](https://my.home-assistant.io/redirect/marketplace_repository/?owner=claudiosanches&repository=ha-memory-card-integration&category=integration)

## Qué añade

Un dispositivo **Memory Card** con:

- los sensores del hogar: juegos, copias, lista de deseos, jugando, último
  añadido y la sugerencia actual;
- tres sensores por cada persona que lleva registro de juego en Memory Card,
  hasta 16: sus pendientes, a qué está jugando y lo último que jugó;
- el botón **Sortear un juego**.

[Sensores de Home Assistant](sensors.md) enumera cada entidad con su estado y
atributos. Los valores se actualizan alrededor de un segundo después de cada
cambio en Memory Card, y las entidades aparecen como no disponibles mientras la
aplicación está detenida. Los sensores de una persona aparecen y desaparecen a
medida que las personas empiezan o dejan de llevar registro de juego.

Los nombres de entidad vienen en inglés, español y portugués de Brasil, según
el idioma de Home Assistant.

## Botón Sortear un juego

**Sortear un juego** (`button.memory_card_roll_pick`) elige al azar un juego
propio de los pendientes de quien lo pulsa, igual que **Elige por mí** en
[Pendientes](backlog.md). El resultado aparece en `sensor.memory_card_pick`,
con el nombre de la persona en su atributo `person`.

Una tarjeta del panel con ambos:

```yaml
type: entities
title: Qué jugar
entities:
  - entity: sensor.memory_card_pick
  - entity: button.memory_card_roll_pick
```

Una tarjeta **Botón** también puede pulsarlo: configura su acción al pulsar
como **Realizar acción**, elige **Pulsar botón** y selecciona **Sortear un
juego** como objetivo.

![Tarjeta Botón que pulsa Sortear un juego](https://ha-memory-card.pages.dev/images/integration-roll-button.webp)

La misma tarjeta en YAML:

```yaml
type: button
name: Sortear un juego
icon: mdi:dice-5
show_state: false
tap_action:
  action: perform-action
  perform_action: button.press
  target:
    entity_id: button.memory_card_roll_pick
```

Una automatización que pulsa el botón no tiene persona asociada, por lo que se
rechaza. Usa en su lugar la acción de abajo.

## La acción roll_pick {#the-roll_pick-action}

`memory_card.roll_pick` elige al azar un juego propio de los pendientes de una
persona y responde con su `title` y `platforms`.

| Campo | Qué hace |
|---|---|
| `person` | De quién son los pendientes, como entidad `person`. Déjalo fuera para sortear para ti. Una automatización debe nombrar a alguien. |
| `play_status` | Claves de estados de juego entre las que elegir. Si está vacío, usa todos los estados que cuentan como pendiente. |
| `platform` | Slugs de plataformas. |
| `device` | Slugs de dispositivos: copias en estos dispositivos, o sin dispositivo en una plataforma que ejecute alguno de ellos. |
| `genre` | Slugs de géneros. |
| `game_mode` | Slugs de modos de juego. |
| `min_local_players` | Al menos esta cantidad de jugadores en la misma pantalla. |
| `max_hours_to_beat` | Como máximo esta cantidad de horas para la historia principal, según IGDB. Los juegos sin duración quedan fuera. |

Los campos de slugs aceptan un solo slug o una lista de hasta 10. Cada
plataforma muestra su slug en su formulario de [Gestionar](manage.md) y cada
dispositivo en su formulario de [Hardware](hardware.md).

Un sorteo de noche de juegos para dos en la Switch, de los pendientes de Ana:

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
      title: Noche de juegos
      message: "{{ pick.title }} en {{ pick.platforms | join(', ') }}"
```

Quién puede sortear:

- La aplicación necesita gestores, y la persona debe ser un gestor o un lector
  al que la aplicación permita llevar su progreso (`viewers_track_progress`).
- Cada persona solo puede sortear para sí misma. Un script que alguien ejecuta
  desde un panel cuenta como esa persona, así que no puede nombrar a otra.
- Una automatización no tiene persona propia, así que debe nombrar a una, y
  Memory Card debe conocerla: un gestor o alguien que ya haya cambiado algo en
  ella, como su estado de juego.
- La entidad `person` debe estar vinculada a un usuario de Home Assistant.

## Errores

| Mensaje | Qué hacer |
|---|---|
| Indica una persona para sortear un juego. | Añade `person` o sortea desde un panel con tu propio usuario. |
| Solo puedes sortear un juego para ti. | Quita `person` o nómbrate a ti mismo. |
| Memory Card no lleva el registro de juego de esta persona. | Hazla gestor o activa `viewers_track_progress`. Una persona que Memory Card nunca ha visto debe abrirla y definir primero un estado de juego. |
| Esta persona no tiene usuario de Home Assistant. | Vincula la persona a un usuario en **Ajustes > Personas**. |
| Ningún juego coincide con estos filtros. | Haz los filtros más amplios o añade juegos a tus pendientes. |
| Memory Card no pudo leer estos filtros. | Comprueba los slugs en los formularios de Gestionar y Hardware. |
| No se puede conectar con la aplicación Memory Card. | Inicia la aplicación y revisa su pestaña **Registro**. |
| Memory Card no está configurada. | Añade Memory Card desde **Descubierto** en **Ajustes > Dispositivos y servicios**. |

## Solución de problemas

- **Ajustes > Estado** en Memory Card indica qué están haciendo los sensores.
  **Esperando la integración Memory Card** significa que la aplicación está
  lista pero la integración aún no se ha añadido: añádela desde **Descubierto**,
  o instálala si no aparece ahí.
- La pestaña **Registro** de la aplicación indica por qué no pudo iniciarse la
  conexión.
- **Descargar diagnóstico**, en la página de la integración en **Ajustes >
  Dispositivos y servicios**, entrega lo que la integración recibe de la
  aplicación, sin el token, ni el registro de juego ni los ID de usuario de
  nadie. Adjúntalo en un informe de error.

## Actualizar y eliminar

Las actualizaciones llegan a través del Marketplace. Reinicia Home Assistant
después de actualizar.

Para eliminarla, establece la opción `sensors` de la aplicación en `off` o
`mqtt` y reinicia la aplicación: Home Assistant elimina la entrada y las
entidades de la integración. Después elimina **Memory Card** del Marketplace.
Con `mqtt`, los mismos ID de entidad vuelven mediante MQTT.

Los errores de la integración se notifican en su sección de
[incidencias](https://github.com/claudiosanches/ha-memory-card-integration/issues).
