# Datos y copias de seguridad

Todo lo que guarda la aplicación vive en su carpeta de datos, que Home
Assistant incluye en sus copias de seguridad:

- `db/memory-card.sqlite`: la colección y el registro de juego de cada persona.
- `db/pre-migration/`: una copia de la base de datos hecha antes de cada
  actualización que cambia su estructura. Se conservan las últimas cinco.
- `covers/`: portadas descargadas y subidas, y fotos de copias y hardware.

Las copias de seguridad detienen la aplicación un momento para copiar la base
de datos en un estado coherente. La aplicación se inicia de nuevo sola al
terminar la copia de seguridad.

Para guardar una copia fuera de Home Assistant, **Exportar colección** en
[Importar / exportar](import-export.md) descarga todas las copias en un archivo
CSV, y su pestaña **Hardware** descarga todos los artículos de hardware. Quedan
fuera las sesiones de juego, el historial de estados, las fotos y las portadas.
