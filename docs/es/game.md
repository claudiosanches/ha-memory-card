# Página del juego

Selecciona un juego en la [Biblioteca](library.md) para abrir su página. Tiene
cuatro partes: el encabezado, tu progreso, los detalles del juego y sus
lanzamientos y copias.

![Una página de juego con tu progreso y el de la casa](https://ha-memory-card.pages.dev/images/game.webp)

_Los títulos de juegos y las portadas pertenecen a sus respectivos propietarios._

## Encabezado

El encabezado muestra la portada, el título y, para un juego vinculado a IGDB,
su fecha de primer lanzamiento, desarrolladoras y editoras. También indica quién
añadió el juego y quién hizo el último cambio.

Los gestores tienen estos botones:

- **Actualizar desde IGDB** vuelve a obtener los detalles y la portada del juego.
- **Cambiar coincidencia de IGDB** vincula el juego con otro de IGDB; **Vincular
  a IGDB** cuando no tiene ninguno. Consulta [IGDB](igdb.md).
- **Editar** cambia el título, los otros nombres en **También conocido como**,
  el resumen, los modos de juego, el número de jugadores y las categorías.
- El botón de papelera elimina el juego con todas sus copias y el progreso de
  todas las personas para ese juego.

Para usar tu propia portada, un gestor selecciona la portada y sube una imagen
JPEG, PNG o WebP de hasta 10 MB y 100 megapíxeles. IGDB nunca sustituye una
portada subida, y quitarla recupera la de IGDB. Cada lanzamiento puede tener su
propia portada de la misma forma.

## Tu progreso

Esta parte es tuya. Cada persona de la casa tiene la suya.

- **Estado de juego** coloca el juego en tus [Pendientes](backlog.md): pendiente,
  jugando, en pausa, completado, abandonado, 100%, o un estado añadido por los
  gestores.
- **Jugado hoy** registra una sesión para hoy. Cambia a **Registrado hoy** en
  cuanto tienes una.
- **Última partida** muestra tu sesión más reciente.
- **Historial de juego** lista tus sesiones, de la más reciente a la más
  antigua. **Registrar una sesión** añade una con el día, la copia jugada y los
  minutos. Los botones de lápiz y papelera editan o eliminan una sesión.
- Registrar una sesión para un juego en tus pendientes, o sin estado, lo pasa a
  jugando. Los juegos en pausa y terminados conservan su estado.
- **Otros en casa** muestra el estado de cada persona para el juego, cuántas
  sesiones lleva y cuándo lo jugó por última vez. Es de solo lectura.

Los gestores siempre llevan su propio registro de juego. Los lectores llevan el
suyo cuando la opción `viewers_track_progress` de la aplicación está activada;
de lo contrario ven el progreso de la casa, pero no pueden definir el suyo.

## Detalles del juego

Los modos de juego, los géneros, los jugadores en la misma pantalla y en línea,
los otros nombres y el resumen vienen de IGDB o de **Editar**.

**Tiempo para terminarlo** viene de IGDB: las horas para la historia principal,
con extras y al 100%, según informan los jugadores de IGDB.

**Categorías** aparece después del resumen. Seleccionar una abre la biblioteca
filtrada por ella. Los gestores crean categorías en [Gestionar](manage.md).

**Otras versiones** lista juegos de la colección que IGDB considera versiones de
este, como su remasterización, remake, port u otra edición. Siguen siendo juegos
independientes con sus propias copias y progreso.

## Lanzamientos y copias

Un lanzamiento es el juego en una plataforma y en una región, como el
lanzamiento norteamericano de Switch. Una copia es una unidad que tienes o
quieres: física o digital, en un dispositivo y comprada en una tienda.

Los gestores los añaden y modifican:

1. **Añadir copia** pide la plataforma, la región y el formato, y si la tienes
   (**Lo tengo**) o la quieres (**En mi lista de deseos**).
2. Rellena lo que sepas: código de producto, edición, el dispositivo donde está,
   para una copia digital la cuenta dueña de la copia, dónde la compraste,
   conservación, fecha de compra, precio pagado, valor estimado actual, código
   de barras y notas.
3. **Guardar y añadir otra** deja el formulario abierto para la siguiente copia.

Cada lanzamiento tiene **Gestionar lanzamiento**, para editar su título, código
de producto y fecha de lanzamiento, o para añadir otra copia del mismo. Cada
copia tiene un botón de lápiz para editarla y un menú **⋯**:

- **Añadir fotos** guarda fotos de la caja, el cartucho o el manual. En un móvil
  ofrece la cámara. Una copia admite hasta 20 fotos JPEG, PNG o WebP de hasta
  10 MB cada una. Memory Card elimina la ubicación, los datos de la cámara y
  otros metadatos que el teléfono guarda en las fotos.
- **Añadir juegos de esta copia** convierte la copia en un recopilatorio, como
  una colección en un solo cartucho. Cada juego incluido recibe una copia que
  sigue a esta: si cambias la plataforma, la región, el formato, el dispositivo,
  la cuenta, la tienda, la conservación o el estado de lista de deseos del
  recopilatorio, también cambian las de dentro. Cuando el recopilatorio está
  vinculado a IGDB, Memory Card ofrece los juegos que IGDB incluye en él.
- **Vincular etiqueta NFC** hace que leer una etiqueta registre una partida para
  esta copia. Consulta [Etiquetas NFC](nfc-tags.md).
- **Eliminar copia** la elimina. Al eliminar la última copia pregunta si quieres
  eliminar también el juego.

Un lanzamiento desaparece con su última copia, también cuando esa copia se mueve
a otra plataforma, región o código de producto.
