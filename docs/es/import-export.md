# Importar / exportar

**Importar / exportar** en la barra lateral mueve la colección entre Memory Card
y una hoja de cálculo mediante un archivo CSV. Solo los gestores ven esta
página.

![Importación de un archivo desde otra hoja de cálculo](https://ha-memory-card.pages.dev/images/import.webp)

## Exportar

- **Exportar colección** descarga todas las copias, una por fila.
- **Descargar plantilla vacía** descarga solo las columnas, sin filas, para
  rellenar en una hoja de cálculo.
- **Exportar CSV** en la [Biblioteca](library.md) descarga únicamente las copias
  que coinciden con los filtros actuales.

La columna `play_status` guarda tu propio estado. El progreso de las demás
personas queda fuera del archivo.

## Importar

1. Guarda la hoja de cálculo como CSV UTF-8. También sirven los archivos que
   usan `;` como separador.
2. En **Archivo CSV**, elige el archivo.
3. **Cómo se leen tus columnas** muestra a qué columna corresponde cada
   encabezado. Memory Card reconoce sus propios nombres y los habituales de
   otras hojas de cálculo y aplicaciones de colección, como `Title`, `Name`,
   `System`, `Console`, `Serial`, `UPC`, `Price Paid`, `Purchase Price`,
   `Collection Status` o `Comments`. Elige otra columna para cualquier
   encabezado mal leído, o **No se importa** para dejarlo fuera.
4. Revisa la vista previa: los juegos y copias nuevos o actualizados, y las
   plataformas, dispositivos, cuentas, tiendas y categorías que se crearán.
5. Las filas con errores bloquean la importación. Corrígelas en tu hoja de
   cálculo y selecciona **Elegir otro archivo**. Las filas con avisos se
   importan tal cual.
6. Selecciona **Importar**. Nada cambia hasta que lo haces.

Después de importar, **Revisar coincidencias de IGDB** repasa los juegos que aún
no están vinculados a IGDB. Consulta [IGDB](igdb.md#close-matches).

## Cómo se leen las filas

- Una fila por copia. Una fila solo con `game_title`, y opcionalmente `igdb_id`
  y `play_status`, añade un juego sin copias.
- Los nombres de columna no distinguen mayúsculas y pueden ir en cualquier orden.
  Solo `game_title` es obligatoria en cada fila; las filas que añaden una copia
  también necesitan `platform`, `region` y `format`.
- Los nombres de plataforma, dispositivo, cuenta y tienda se emparejan con los
  que ya tienes sin distinguir mayúsculas. Las plataformas habituales y las
  tiendas digitales (Nintendo eShop, PlayStation Store, Microsoft Store, Steam,
  Epic Games Store, GOG, itch.io) existen desde el principio. Los nombres que no
  coinciden con nada se crean: un dispositivo nuevo se crea como consola, lo que
  puedes cambiar en [Hardware](hardware.md), y una cuenta nueva aparece en
  [Gestionar > Cuentas](manage.md#accounts). Una fila no puede asignar una copia
  a un mando, accesorio o juguete.
- La exportación rellena `copy_id`. Consérvalo para actualizar esa copia al
  volver a importar el archivo; déjalo vacío para añadir una copia nueva. Las
  columnas que el archivo no tiene conservan sus valores en esa copia, mientras
  que una celda vacía borra el valor. Importar una exportación sin cambios no
  modifica nada.
- Una importación nunca elimina una copia, un juego ni una categoría.

## Columnas

| Columna | Valores aceptados |
|---|---|
| `copy_id` | Número de la exportación, o vacío |
| `status` | `owned` o `wanted` (vacío significa `owned`). `In Collection` se lee como `owned`; `Wish List` y `On Order`, como `wanted`. |
| `igdb_id` | ID opcional del juego en IGDB. Distingue juegos con el mismo título. |
| `game_title` | Obligatoria |
| `play_status` | El tuyo: `backlog`, `playing`, `paused`, `completed`, `dropped`, `mastered`, o el nombre o clave de un estado añadido por un gestor. También sirven palabras de otras aplicaciones, como `Unplayed`, `Unfinished`, `Beaten` o `Retired`. |
| `labels` | Nombres de categorías, hasta 50, separados por comas o punto y coma. La importación las añade al juego y crea en gris las que aún no existen. La exportación las escribe separadas por `; `. |
| `platform` | Nombre, abreviatura o slug de la plataforma |
| `region` | `na`, `eu`, `au`, `nz`, `jp`, `cn`, `asia`, `kr`, `br`, `ww` (mundial o sin región); también `NTSC-U`, `PAL`, `NTSC-J` |
| `release_title` | Título en este lanzamiento, cuando difiere |
| `product_code` | Número de serie o código de producto |
| `format` | `physical` o `digital` |
| `edition` | Texto libre, como `Edición de coleccionista` |
| `device` | Consola, portátil o PC donde está la copia |
| `account` | Solo copias digitales: la cuenta de tienda dueña de la copia |
| `store` | Nombre de la tienda |
| `condition` | Solo copias físicas: `sealed`, `cib`, `no_manual`, `loose` |
| `purchased_on` | Fecha de compra, `AAAA-MM-DD` |
| `price` | Precio pagado, o esperado para una copia deseada. Sirven `59.99`, `59,99` y `1.299,90`; la exportación escribe `1299.90`. Necesita `currency`. |
| `currency` | Código ISO 4217 de tres letras, como `USD`, `EUR` o `BRL` |
| `estimated_value` | Por cuánto se vendería hoy una copia que tienes, leído igual que `price`. Usa `estimated_currency`, o `currency` cuando esa está vacía. |
| `estimated_currency` | Código de tres letras de `estimated_value`, cuando difiere de `currency` |
| `barcode` | Dígitos UPC o EAN de la caja |
| `notes` | Texto libre |
| `bundle_copy_id` | `copy_id` de la copia del recopilatorio en el que vino esta copia. Plataforma, región, formato, dispositivo, cuenta, tienda, conservación y estado pasan a seguir esa copia, y se omiten su propio precio, valor estimado y fecha de compra. Si está vacío, saca la copia del recopilatorio. |

**Columnas y valores aceptados**, al final de la página, muestra lo mismo dentro
de la aplicación.

La exportación escapa las celdas que una hoja de cálculo interpretaría como
fórmulas, por lo que abrir el archivo es seguro.

## Hardware

La pestaña **Hardware** mueve tu [hardware](hardware.md) desde y hacia su propio
archivo CSV. **Exportar hardware** descarga todos los artículos y **Descargar
plantilla vacía** descarga solo las columnas.

Ninguno de los dos archivos guarda las cuentas en sí. El archivo de juegos
indica la cuenta de cada copia digital, pero la persona dueña de la cuenta, la
tienda, las plataformas y las notas permanecen en la base de datos, que incluyen
las copias de seguridad de Home Assistant.

**Columnas y valores aceptados**, al final de la pestaña, enumera las columnas
dentro de la aplicación.

Una importación muestra primero una vista previa. Cada fila actualiza el
hardware con el mismo `device_id`, o en su defecto el mismo nombre, y si no,
añade hardware nuevo. Las columnas que el archivo no tiene conservan sus
valores, mientras que una celda vacía borra un valor. Las tiendas que aún no
existen se añaden. Las filas con errores bloquean la importación.

| Columna | Valores aceptados |
|---|---|
| `device_id` | Número de la exportación, o vacío |
| `status` | `owned` o `wanted` (vacío significa `owned`) |
| `name` | Obligatoria |
| `kind` | `console`, `handheld`, `pc`, `controller`, `accessory` o `toy`. Obligatoria para el hardware nuevo. |
| `model` | Texto libre |
| `platform` | Para qué plataforma es un mando, accesorio o juguete: nombre, abreviatura o slug |
| `plays` | Qué ejecuta una consola, portátil o PC: plataformas separadas por `;` |
| `region` | Igual que para las copias |
| `condition` | `sealed`, `cib`, `no_manual`, `loose` |
| `store` | Nombre de la tienda |
| `purchased_on` | `AAAA-MM-DD` |
| `price`, `currency` | Precio pagado, o esperado para hardware deseado, leído igual que en las copias |
| `estimated_value`, `estimated_currency` | Por cuánto se vendería hoy el hardware que tienes. `estimated_value` usa `estimated_currency`, o `currency` cuando esa está vacía. |
| `serial_number` | Texto libre |
| `barcode` | Dígitos UPC o EAN |
| `notes` | Texto libre |
| `active` | `yes` o `no` |
