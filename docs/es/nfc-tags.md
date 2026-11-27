# Etiquetas NFC

Coloca una etiqueta NFC en la caja de un juego y, al leerla con la aplicación
complementaria de Home Assistant, empezará esa copia para ti: Memory Card
registra una sesión de juego para hoy en tu historial y cambia tu estado para
ese juego a Jugando.

## Vincular una etiqueta

1. En la [página del juego](game.md), abre el menú **⋯** de la copia y elige
   **Vincular etiqueta NFC**.
2. En el plazo de un minuto, lee la etiqueta con la aplicación de Home
   Assistant.
3. Memory Card muestra **Etiqueta vinculada**.

Con la aplicación de Home Assistant, graba primero una etiqueta vacía: en Home
Assistant, ve a **Ajustes > Conectividad > Etiquetas**, selecciona **Añadir
etiqueta** y lee la etiqueta con la aplicación. Los lectores ESPHome leen
cualquier etiqueta tal cual. Una copia puede tener varias etiquetas.
**Desvincular etiquetas NFC**, en el mismo menú, las quita.

Una vinculación espera una lectura del gestor que la inició, o de un lector sin
usuario que `tag_readers` no asigne a otra persona.

## Qué hace una lectura

- Registra una sesión para hoy y cambia tu estado para el juego a Jugando. Leer
  la misma etiqueta de nuevo el mismo día no añade otra sesión.
- Un juego cuyo estado cuenta como terminado también vuelve a Jugando, salvo
  que la opción `tag_scan_finished` esté establecida en `keep`.
- Las páginas abiertas de Memory Card muestran la lectura en cuestión de un
  segundo.

Una lectura registra la partida para gestores y para lectores cuando
`viewers_track_progress` está activada. No se registra nada mientras no haya
gestores configurados.

## Lectores sin usuario

Un lector sin usuario de Home Assistant, como un lector NFC de ESPHome,
registra la partida para la persona que indique la opción `tag_readers`, y no
hace nada en caso contrario.

1. Busca el ID de dispositivo del lector: en Home Assistant, abre el lector en
   **Ajustes > Dispositivos y servicios > Dispositivos**. El ID es la última
   parte de la dirección de la página. Un lector que no figura en ninguna
   opción también registra su ID de dispositivo en la pestaña **Registro** de
   la aplicación cuando lee una etiqueta vinculada.
2. Busca el ID de usuario de la persona en **Ajustes > Estado > Tu acceso** en
   Memory Card, abierto con esa cuenta.
3. En la pestaña **Configuración** de la aplicación, añade el par a
   `tag_readers`, guarda y reinicia la aplicación:

   ```yaml
   tag_readers:
     - device: 4f1c9e0d8a7b6c5d4e3f2a1b0c9d8e7f
       person: your-user-id
   ```

## Comprobar que funciona

Memory Card recibe las lecturas mediante la API de Home Assistant, motivo por
el cual la aplicación pide acceso a ella. **Ajustes > Estado > Lectura de
etiquetas NFC** indica si está escuchando.

Para probar una lectura sin etiqueta, dispara un evento `tag_scanned` con
`{"tag_id": "your-tag-id"}` en **Herramientas para desarrolladores > Eventos**
en Home Assistant. La lectura cuenta como del usuario que lo dispara.
