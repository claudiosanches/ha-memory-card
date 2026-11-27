# Ajustes

**Ajustes**, en la parte inferior de la barra lateral, tiene dos pestañas:
**Apariencia** y **Estado**. Todas las personas pueden abrirlo. **Apariencia**
se abre primero una vez que hay un gestor configurado; en una instalación nueva,
**Estado** se abre primero para que puedas copiar tu ID de usuario.

![Ajustes, en la pestaña Apariencia](https://ha-memory-card.pages.dev/images/settings.webp)

## Apariencia

Las elecciones de esta pestaña se guardan en ese navegador, así que cada persona
y cada dispositivo conservan las suyas.

- **Tema:** uno de los 12 temas. Consulta [Temas](themes.md).
- **Esquema de colores:** **Sistema**, **Claro** u **Oscuro**. El botón en la
  parte superior derecha de cada página también lo cambia.
- **Idioma:** inglés, español o portugués de Brasil. **Automático** sigue el
  idioma de Home Assistant, o el del navegador si Memory Card no tiene el de
  Home Assistant.

Las fechas siguen el idioma; por ejemplo, en español se escribe 23/11/2026 y en
inglés 11/23/2026. Para mostrarlas con otro formato en todos los idiomas,
configura la opción `date_format` de la aplicación.

## Estado

**Integraciones** indica si cada conexión funciona:

- **IGDB:** conectado, sin configurar o desactivado porque los servicios
  externos están desactivados. Consulta [IGDB](igdb.md).
- **Sensores de Home Assistant:** conectado, esperando la integración o qué lo
  impide. Consulta [sensores](sensors.md).
- **Lectura de etiquetas NFC:** indica si Memory Card recibe lecturas de
  etiquetas desde Home Assistant. Consulta [etiquetas NFC](nfc-tags.md).

**Tu acceso** muestra con qué usuario estás conectado, tu rol y tu ID de usuario
de Home Assistant. **Copiar ID de usuario** lo copia, para nombrar un gestor en
las opciones de la aplicación o una persona para un lector NFC.

**Acerca de Memory Card** muestra la versión, la licencia, enlaces a la
documentación y al soporte, y dónde apoyar el desarrollo.
