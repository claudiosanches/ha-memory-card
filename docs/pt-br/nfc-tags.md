# Tags NFC

Coloque uma tag NFC na caixa de um jogo e, ao lê-la com o aplicativo
complementar do Home Assistant, você inicia essa cópia: o Memory Card registra
uma sessão de jogo para hoje nas suas partidas e muda o seu status do jogo para
jogando.

## Vincular uma tag

1. Na [página do jogo](game.md), abra o menu **⋯** da cópia e escolha
   **Vincular tag NFC**.
2. Em até um minuto, leia a tag com o aplicativo do Home Assistant.
3. O Memory Card exibe **Tag vinculada**.

Para usar o aplicativo do Home Assistant, grave uma tag em branco primeiro: no
Home Assistant, vá em **Configurações > Conectividade > Tags**, selecione
**Adicionar tag** e leia a tag com o aplicativo. Leitores ESPHome leem qualquer
tag como ela vem. Uma cópia pode ter várias tags. **Desvincular tags NFC**, no
mesmo menu, remove todas elas.

Um vínculo aguarda a leitura feita pelo gestor que o iniciou, ou por um leitor
sem usuário que `tag_readers` não atribua a outra pessoa.

## O que uma leitura faz

- Registra uma sessão para hoje e muda o seu status do jogo para jogando. Ler a
  mesma tag de novo no mesmo dia não cria outra sessão.
- Um jogo cujo status conta como terminado também volta para jogando, a menos
  que a opção `tag_scan_finished` esteja definida como `keep`.
- Páginas abertas no Memory Card mostram a leitura em até um segundo.

Uma leitura registra partidas para gestores e para visualizadores quando
`viewers_track_progress` está ligado. Nada é registrado enquanto não houver
gestores definidos.

## Leitores sem usuário

Um leitor sem usuário do Home Assistant, como um leitor NFC no ESPHome,
registra a partida para a pessoa indicada pela opção `tag_readers`, e não faz
nada caso contrário.

1. Encontre o ID do dispositivo do leitor: no Home Assistant, abra o leitor em
   **Configurações > Dispositivos e serviços > Dispositivos**. O ID é a última
   parte do endereço da página. Um leitor que não esteja na configuração também
   registra seu ID de dispositivo na aba **Log** do aplicativo quando lê uma
   tag vinculada.
2. Encontre o ID de usuário da pessoa em **Configurações > Status > Seu
   acesso** no Memory Card, aberto como essa pessoa.
3. Na aba **Configuração** do aplicativo, adicione o par em `tag_readers`,
   salve e reinicie o aplicativo:

   ```yaml
   tag_readers:
     - device: 4f1c9e0d8a7b6c5d4e3f2a1b0c9d8e7f
       person: your-user-id
   ```

## Conferir se está funcionando

O Memory Card recebe as leituras pela API do Home Assistant, e é por isso que o
aplicativo pede acesso a ela. **Configurações > Status > Leitura de tags NFC**
informa se ele está ouvindo.

Para testar uma leitura sem uma tag, dispare um evento `tag_scanned` com
`{"tag_id": "your-tag-id"}` em **Ferramentas de desenvolvedor > Eventos** no
Home Assistant. A leitura conta como feita pelo usuário que disparou o evento.
