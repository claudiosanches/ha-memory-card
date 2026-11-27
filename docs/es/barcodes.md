# Códigos de barras

**Leer código de barras** en la [Biblioteca](library.md) y la [Lista de
deseos](wishlist.md) lee el código de barras de una caja, para que en una tienda
puedas ver si ya tienes ese juego.

1. Selecciona **Leer código de barras**.
2. Apunta la cámara al código de barras. Cuando abres Home Assistant por HTTPS,
   Memory Card lo lee en directo desde la cámara. Por HTTP sin cifrar, los
   navegadores bloquean el acceso de las páginas web a la cámara, así que
   selecciona **Hacer una foto**.
3. Si no se puede leer el código, selecciona **Escribir el código** e introduce
   los dígitos que están bajo las barras.

Memory Card lee el código en el navegador y comprueba primero tus copias:

- Si alguna copia tiene ese código, muestra **Ya tienes esto** y esa copia.
- Si ninguna lo tiene, consulta a [UPCitemdb](https://www.upcitemdb.com) por el
  nombre del producto y luego permite que un gestor elija el juego y añada la
  copia con el código de barras, la plataforma y la región ya rellenados. Desde
  la lista de deseos, la copia se añade a la lista de deseos.

UPCitemdb permite 100 consultas al día sin cuenta. Los lectores solo comprueban
las copias de la colección, así que no consumen las consultas diarias. Con la opción
`external_services` desactivada, las lecturas solo comprueban tus copias.

Para guardar un código de barras en una copia que ya tienes, edita la copia en
su [página del juego](game.md) y rellena **Código de barras**.
