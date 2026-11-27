# Barcodes

**Scan barcode** on the [Library](library.md) and [Wishlist](wishlist.md)
reads the barcode on a box, so in a shop you can see whether you
already own a game.

1. Select **Scan barcode**.
2. Point the camera at the barcode. When you open Home Assistant over HTTPS,
   Memory Card reads it live from the camera. Over plain HTTP, browsers keep
   the camera from web pages, so select **Take a photo** instead.
3. If the code cannot be read, select **Type the code** and enter the digits
   under the bars.

Memory Card reads the code in the browser and checks your copies first:

- When a copy has the code, it shows **You already have this** and the copy.
- When none has it, it asks [UPCitemdb](https://www.upcitemdb.com) for the
  product name, then lets a manager pick the game and add the copy with the
  barcode, platform and region filled in. From the wishlist, the copy goes on
  the wishlist.

UPCitemdb allows 100 lookups a day without an account. Viewers only check
copies in the collection, so they cannot use up the daily lookups. With the
`external_services` option off, scans only check your copies.

To save a barcode on a copy you already have, edit the copy on its
[game page](game.md) and fill in **Barcode**.
