# Gestionar

**Gestionar** en la barra lateral reúne las listas sobre las que se construye la
colección. Solo los gestores la ven.

![Gestionar, en la pestaña Plataformas](https://ha-memory-card.pages.dev/images/manage.webp)

Las consolas, las portátiles y los PC están en la página [Hardware](hardware.md),
junto con el resto de tu hardware.

## Cuentas {#accounts}

Son las cuentas de tienda dueñas de tus juegos digitales, como una Cuenta
Nintendo, PlayStation Network o Steam. Una copia digital indica su cuenta por
separado del dispositivo en el que está instalada, de modo que un juego comprado
en una cuenta de PlayStation Network y jugado en la PS5 conserva ambos datos.
**Añadir cuenta** pide:

- un nombre, con el que eliges esta cuenta para una copia;
- **Nombre de usuario:** identificador o nombre de perfil opcional, como un
  ID online de PSN, gamertag de Xbox, Cuenta Nintendo o nombre de perfil de
  Steam. Nunca guardes una contraseña aquí;
- **Tienda:** dónde compra juegos. Una copia nueva en esta cuenta la toma como
  su tienda;
- **Persona:** de quién es la cuenta en la casa;
- **Plataformas:** las plataformas cuyos juegos guarda esta cuenta. No se ofrece
  para copias de otras plataformas.

Las cuentas inactivas no se ofrecen al elegir la cuenta de una copia. Al
eliminar una cuenta, sus copias se conservan sin cuenta.

## Tiendas

Son las tiendas físicas y en línea donde compras juegos. **Añadir tienda** pide
un nombre. Las tiendas digitales ya existen desde el principio. Al eliminar una
tienda, sus copias se conservan sin tienda.

## Plataformas

**Tus plataformas** son las que se muestran al añadir juegos, dispositivos y
cuentas, en el orden que aparece. Arrastra para reordenarlas.

**Todas las plataformas** muestra todas las plataformas que conoce Memory Card,
por tipo: consolas, portátiles, ordenadores, móviles, arcade y otras. **Añadir**
incorpora una a tus plataformas. Quitar una la oculta al añadir juegos,
dispositivos y cuentas; todo lo que ya la usa se conserva. **Añadir plataforma
personalizada** crea una que no está en la lista. Una plataforma personalizada
que usan juegos, dispositivos o cuentas no se puede eliminar; quítala de tus
plataformas en su lugar.

## Estados de juego

Son los estados entre los que elige cada persona para llevar su propio registro
de juego, en el orden que aparece. Arrastra para reordenar.

Los seis estados predefinidos (`backlog`, `playing`, `paused`, `completed`,
`dropped` y `mastered`) se pueden renombrar, cambiar de color y reordenar, pero
no eliminar. Sus claves nunca cambian, para que los archivos CSV y Home
Assistant puedan contar con ellos.

**Añadir estado** pide un nombre, un color y como qué cuenta, lo cual decide
dónde aparecen sus juegos en la página [Pendientes](backlog.md):

- **Pendiente:** en espera, y elegible en **Elige por mí**.
- **Jugando** y **En pausa:** en esas secciones.
- **Terminado:** cuenta como completado en pendientes.
- **No se cuenta:** queda fuera de la página Pendientes.

Al eliminar un estado, se pregunta adónde mover los juegos de todas las personas
que lo tienen, o si dejarlos sin estado.

## Categorías

Son tus propios grupos para ordenar juegos, como "Cooperativo en el sofá" o
"Volver a jugar". **Añadir categoría** pide un nombre y un color, y la lista
muestra cuántos juegos tiene cada una. Asigna las categorías de un juego con
**Editar** en su [página](game.md). Los nombres no pueden contener comas ni
punto y coma, que separan categorías en los archivos CSV. Al eliminar una
categoría, sus juegos se conservan.
