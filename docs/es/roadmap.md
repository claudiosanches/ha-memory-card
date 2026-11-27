# Hoja de ruta

Ideas para las próximas versiones de Memory Card. Ninguna tiene fecha todavía,
y el orden puede cambiar según lo que pida la gente.

## Ideas

### Importar la biblioteca de Steam

Traer tus juegos de Steam como copias digitales en tu cuenta de Steam, cada uno
emparejado con su juego de IGDB por su ID de Steam, sin adivinar por el título.
Usa la Web API oficial de Steam con tu propio ID de Steam y tu clave de API.

### Partidas desde las integraciones de juegos de Home Assistant

Las integraciones PlayStation Network, Steam y Xbox de Home Assistant ya
muestran a qué juega cada persona. Cada una de esas entidades se asignaría a una
persona, como `tag_readers` asigna un lector NFC, y Memory Card registraría una
partida cuando aparezca un juego de la biblioteca.

### Préstamos

Anotar quién se llevó prestada una copia y desde cuándo, y luego marcar la
devolución. Un sensor de Home Assistant podría listar los juegos prestados.

### Valoraciones y reseñas

Una valoración y una reseña breve de cada persona, una al lado de otra en la
página del juego.

### Resumen del año

El año del hogar en juegos: lo que terminó cada persona, las horas jugadas y los
juegos más jugados.

### Próximos lanzamientos

Las fechas de lanzamiento de los juegos de la lista de deseos, desde IGDB, como
un calendario de Home Assistant.

### Etiquetas QR para la estantería

Etiquetas QR para imprimir y pegar en la estantería, que abren la página del
juego. Las etiquetas NFC siguen siendo la forma de registrar partidas.

## Solo API oficiales

Memory Card usa solo API oficiales y las integraciones del propio Home
Assistant, para que un cambio en otro servicio no lo rompa sin aviso. Las
tiendas sin una forma oficial de leer la biblioteca, como Nintendo, PlayStation
y GOG, siguen con la [importación CSV](./docs/import-export.md), la
[lectura de códigos de barras](./docs/barcodes.md) o el alta a mano.

## Apoya el desarrollo

Memory Card es gratuito y de código abierto. Si quieres ver estas ideas antes,
puedes apoyar su desarrollo en
[GitHub Sponsors](https://github.com/sponsors/claudiosanches) o en
[Ko-fi](https://ko-fi.com/claudiosanches).

Las ideas y peticiones son bienvenidas en
[GitHub issues](https://github.com/claudiosanches/ha-memory-card/issues).
