# IGDB

[IGDB](https://www.igdb.com) aporta a Memory Card títulos, portadas, fechas de
lanzamiento, desarrolladoras, editoras, géneros, modos de juego, número de
jugadores, otros nombres y el tiempo para terminar cada juego. Memory Card
funciona sin él: en ese caso introduces los juegos a mano.
[Primeros pasos](getting-started.md#connect-igdb) explica cómo conectarlo.

## Vincular un juego

- **Añadir juego** en la [Biblioteca](library.md) busca en IGDB mientras añades
  un juego.
- En la [página de un juego](game.md), **Vincular a IGDB** o **Cambiar
  coincidencia de IGDB** vuelve a buscar. Los resultados muestran el año y las
  plataformas de cada coincidencia, e indican qué juegos ya están en tu
  colección.
- **Actualizar desde IGDB** en la página de un juego obtiene sus detalles de
  nuevo.

Los lanzamientos de Japón, Corea y Europa reciben su portada regional cuando
IGDB la tiene. Una portada subida a mano se conserva siempre.

Cuando IGDB indica que dos juegos vinculados de tu colección son versiones entre
sí, como un juego y su remasterización, la página de cada juego muestra el otro
en **Otras versiones**.

## Sincronización

**Mantenimiento > IGDB > Sincronizar con IGDB** vincula cada juego cuyo título
coincide con exactamente un juego de IGDB y después actualiza detalles y portadas
de todos los juegos vinculados. Muestra cuántos vinculó, actualizó, dejó sin
coincidencia y fallaron.

Debajo, **Revisar títulos de juegos** enumera los juegos vinculados cuyo título
difiere en IGDB. **Usar el de IGDB** toma el título de IGDB, **Conservar el
mío** mantiene el tuyo y **Usar el de IGDB en todos** aplica todos los títulos
de IGDB. Un título conservado solo vuelve a aparecer aquí si IGDB cambia el
nombre del juego.

## Coincidencias cercanas {#close-matches}

**Revisar coincidencias cercanas**, junto a **Sincronizar con IGDB**, lista los
juegos sin vínculo con IGDB junto con los nombres más parecidos que tiene IGDB y
su porcentaje de coincidencia.

![Coincidencias cercanas de IGDB](https://ha-memory-card.pages.dev/images/close-matches.webp)

_Los títulos de los juegos pertenecen a sus respectivos dueños._

Para cada juego:

- elige el juego correcto de IGDB y selecciona **Vincular**;
- **Más tarde** lo deja para otro momento;
- **No está en IGDB** evita que vuelva a aparecer, para un juego que IGDB no
  tiene.

**Mostrar más juegos** carga los siguientes. Un juego sin un nombre cercano
todavía se puede vincular a mano desde su página.

## Sin conexión a internet

Para evitar que Memory Card se conecte a internet, desactiva la opción
`external_services`. Entonces se ignoran las credenciales de IGDB, los juegos,
portadas y detalles se introducen a mano, y la lectura de [códigos de
barras](barcodes.md) solo comprueba tus propias copias.
