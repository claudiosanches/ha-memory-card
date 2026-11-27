# Primeros pasos

Memory Card necesita Home Assistant 2026.11 o posterior.

## Instala la aplicación

[![Abre tu instancia de Home Assistant y muestra el diálogo para añadir un repositorio de aplicaciones con una URL predefinida.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card)

1. Selecciona el botón de arriba para añadir el repositorio de la aplicación a
   Home Assistant. O ve a **Ajustes > Aplicaciones > Instalar aplicación**,
   abre el menú ⋮, elige **Repositorios** y añade
   `https://github.com/claudiosanches/ha-memory-card`.
2. Busca **Memory Card** en la lista, selecciónalo y elige **Instalar**.
3. Inicia la aplicación. **Memory Card** aparecerá en la barra lateral.

## Cómo lo usa una casa

Memory Card mantiene una colección para toda la casa y el registro de juego de
cada persona por separado.

- La colección es compartida: juegos, copias, hardware, cuentas, tiendas,
  plataformas, categorías, la lista de estados de juego, etiquetas NFC, fotos y
  portadas. Todos pueden explorarla y los gestores pueden cambiarla.
- El juego es de cada persona: su estado de juego para cada título, que forma
  su lista de pendientes, sus sesiones de juego y sus sorteos. Solo cada
  persona cambia los suyos, incluidos los gestores.
- Cada persona es un usuario de Home Assistant. Home Assistant inicia sesión
  por ti, así que Memory Card no tiene un inicio de sesión propio.

## Nombra a los gestores {#name-the-managers}

Una instalación nueva es de solo lectura para todos hasta que tiene un gestor.

1. Abre Memory Card como la persona que va a gestionar la colección.
2. Ve a **Ajustes > Estado**. En **Tu acceso**, selecciona **Copiar ID de usuario**.
3. En Home Assistant, ve a **Ajustes > Aplicaciones > Memory Card** y abre la
   pestaña **Configuración**.
4. Pega el ID en **Managers** y guarda. Añade una entrada por cada gestor. Si
   el formulario no muestra ningún campo para ello, elige **Editar en YAML** en
   el menú ⋮ y escribe:

   ```yaml
   managers:
     - tu-id-de-usuario
   ```

5. Reinicia la aplicación desde la pestaña **Información**.

De vuelta en Memory Card, **Tu acceso** indicará ahora que puedes gestionar la
colección.

El resto de personas son lectores: exploran la colección, pero no la modifican.
Para que los lectores lleven su propio estado de juego, sesiones y sorteos,
activa **viewers_track_progress** en la misma pestaña **Configuración** y
reinicia la aplicación.

## Conecta IGDB {#connect-igdb}

[IGDB](https://www.igdb.com) proporciona a Memory Card títulos, portadas,
fechas de lanzamiento, géneros y otros detalles. Es opcional: sin él, introduces
los juegos a mano. IGDB usa cuentas de Twitch:

1. Activa la autenticación en dos pasos en tu cuenta de Twitch. Twitch la exige
   para registrar aplicaciones.
2. Abre la [consola de desarrolladores de Twitch](https://dev.twitch.tv/console/apps)
   y registra una aplicación:
   - **OAuth Redirect URLs:** `http://localhost`
   - **Category:** Application Integration
   - **Client Type:** Confidential
3. Abre la aplicación con **Manage**, copia el **Client ID** y selecciona
   **New Secret** para crear un **Client Secret**.
4. En Home Assistant, abre la pestaña **Configuración** de la aplicación,
   rellena **igdb_client_id** e **igdb_client_secret** y guarda. Para
   guardarlos en el archivo `secrets.yaml` de Home Assistant, introduce
   `!secret` seguido de la clave, como `!secret igdb_client_secret`.
5. Reinicia la aplicación.

**Ajustes > Estado** en Memory Card muestra **IGDB: Conectado** en cuanto
funciona. Consulta [IGDB](igdb.md) para ver qué rellena.

## Añade tus primeros juegos

Hay tres formas de empezar:

- **Uno a uno:** selecciona **Añadir juego** en la [Biblioteca](library.md),
  escribe el título y elige la coincidencia de IGDB. Luego añade tus copias en
  la [página del juego](game.md).
- **Desde una hoja de cálculo:** ve a [Importar / exportar](import-export.md),
  empieza con **Descargar plantilla vacía** o usa un archivo que ya tengas, e
  impórtalo.
- **Desde la caja:** selecciona **Leer código de barras** y apunta con el móvil
  a la caja. Consulta [Códigos de barras](barcodes.md).

Antes de añadir muchas copias, añade tus consolas en [Hardware](hardware.md) y
tus cuentas de tienda en [Gestionar > Cuentas](manage.md#accounts), para poder
indicar dónde está cada copia.

## Siguiente paso

- Define tu estado de juego para los juegos que tienes y Memory Card creará tu
  lista de [Pendientes](backlog.md).
- Añade los [sensores](sensors.md) de Memory Card a los paneles de Home Assistant.
- Elige un [tema](themes.md).
