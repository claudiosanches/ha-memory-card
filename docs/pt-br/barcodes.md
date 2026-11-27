# Códigos de barras

**Ler código de barras**, na [Biblioteca](library.md) e na
[Lista de desejos](wishlist.md), lê o código de barras da caixa para você
conferir numa loja se a casa já tem o jogo.

1. Selecione **Ler código de barras**.
2. Aponte a câmera para o código de barras. Quando você abre o Home Assistant
   por HTTPS, o Memory Card lê ao vivo pela câmera. Por HTTP simples, os
   navegadores bloqueiam o acesso de páginas web à câmera, então selecione
   **Tirar uma foto** em vez disso.
3. Se não for possível ler o código, selecione **Digitar o código** e informe
   os números abaixo das barras.

O Memory Card lê o código no navegador e verifica suas cópias primeiro:

- Quando uma cópia tem o código, aparece **Você já tem isto** com a cópia.
- Quando nenhuma tem, ele consulta o nome do produto no
  [UPCitemdb](https://www.upcitemdb.com) e permite que um gestor escolha o jogo
  e adicione a cópia já com código de barras, plataforma e região preenchidos.
  A partir da lista de desejos, a cópia vai para a lista de desejos.

O UPCitemdb permite 100 consultas por dia sem conta. Visualizadores só
verificam as cópias da casa, então não gastam o limite diário de consultas. Com
a opção `external_services` desligada, a leitura só verifica as suas cópias.

Para salvar um código de barras em uma cópia que você já tem, edite a cópia na
[página do jogo](game.md) e preencha **Código de barras**.
