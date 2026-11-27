# Pendientes

**Pendientes** en la barra lateral muestra tus juegos según tu estado de juego:
lo que estás jugando, lo que está en pausa y lo que te espera. Es tuyo; cada
persona de la casa tiene el suyo. La barra superior cuenta cuántos has
terminado.

![La lista de pendientes con un sorteo aleatorio](https://ha-memory-card.pages.dev/images/backlog.webp)

_Los títulos de juegos y las portadas pertenecen a sus respectivos propietarios._

## Clasifica tus juegos

Los juegos a los que no les has dado un estado de juego esperan en **Aún sin
clasificar**. **Clasificarlos** los recorre uno a uno: elige un estado o
**Saltar** para dejarlo para después, y luego **Listo**.

Registrar una sesión para un juego en tus pendientes, o sin estado, lo pasa a
jugando. **Jugado hoy** en la [página del juego](game.md) hace lo mismo.

## Elige por mí

**¿No te decides?** sortea un juego al azar de tus pendientes.

1. En **Jugar en**, deja **Cualquier dispositivo** o elige el dispositivo en el
   que quieres jugar.
2. En **Duración**, deja **Cualquier duración** o elige cuántas horas puede
   durar como máximo la historia principal. En ese caso, los juegos sin tiempo
   en IGDB quedan fuera.
3. Selecciona **Elige por mí**. Un juego sorteado en el último día solo vuelve a
   salir cuando no queda otro.
4. **Empezar a jugar** cambia el juego a jugando. **Elegir otro** vuelve a
   sortear.

El último sorteo también aparece en `sensor.memory_card_pick` en Home Assistant,
y Home Assistant puede hacer un sorteo por ti. Consulta [sensores](sensors.md).

## Secciones

- **Jugando ahora:** **Completado**, **100%**, **Pausar** y **Abandonar** hacen
  avanzar un juego.
- **En pausa:** **Reanudar** devuelve un juego a jugando.
- **En pendientes:** **Empezar** inicia uno. Cada tarjeta muestra las horas de
  IGDB para la historia principal, cuando están disponibles.

En qué sección aparece cada estado depende del grupo como el que cuenta, que los
gestores configuran en **Gestionar > Estados de juego**:

| Cuenta como | En la página Pendientes |
| --- | --- |
| Pendiente | En pendientes, y elegible en **Elige por mí** |
| Jugando | Jugando ahora |
| En pausa | En pausa |
| Terminado | Contado como terminado |
| No se cuenta | Omitido |

## Lectores

Los gestores siempre tienen su propia lista de pendientes. Un lector ve la suya
cuando un gestor activa `viewers_track_progress` en las opciones de la
aplicación.
