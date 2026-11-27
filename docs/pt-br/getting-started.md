# Primeiros passos

O Memory Card precisa do Home Assistant 2026.11 ou mais recente.

## Instalar o aplicativo

[![Abra sua instância do Home Assistant e exiba a janela para adicionar repositório de aplicativos com a URL do repositório já preenchida.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card)

1. Selecione o botão acima para adicionar o repositório do aplicativo ao Home
   Assistant. Ou vá em **Configurações > Aplicativos > Instalar aplicativo**,
   abra o menu ⋮, escolha **Repositórios** e adicione
   `https://github.com/claudiosanches/ha-memory-card`.
2. Encontre **Memory Card** na lista, selecione-o e escolha **Instalar**.
3. Inicie o aplicativo. **Memory Card** aparece na barra lateral.

## Como a casa usa

O Memory Card mantém uma coleção única para toda a casa e separa as partidas de
cada pessoa.

- A coleção é compartilhada: jogos, cópias, hardware, contas, lojas,
  plataformas, etiquetas, a lista de status de jogo, tags NFC, fotos e capas.
  Todo mundo pode navegar por ela, e os gestores podem alterá-la.
- As partidas são individuais: o status de jogo de cada pessoa para cada jogo,
  que forma seu backlog, suas sessões de jogo e seus sorteios. Só a própria
  pessoa altera isso, incluindo gestores.
- Cada pessoa é um usuário do Home Assistant. O Home Assistant faz o login por
  você, então o Memory Card não tem login próprio.

## Definir os gestores {#name-the-managers}

Uma instalação nova fica somente leitura para todo mundo até ter um gestor.

1. Abra o Memory Card conectado como a pessoa que vai gerenciar a coleção.
2. Vá em **Configurações > Status**. Em **Seu acesso**, selecione
   **Copiar ID de usuário**.
3. No Home Assistant, vá em **Configurações > Aplicativos > Memory Card** e
   abra a aba **Configuração**.
4. Cole o ID em **Managers** e salve. Adicione uma entrada para cada gestor. Se
   o formulário não mostrar um campo para isso, escolha **Editar em YAML** no
   menu ⋮ e escreva:

   ```yaml
   managers:
     - seu-id-de-usuario
   ```

5. Reinicie o aplicativo pela aba **Informações**.

De volta ao Memory Card, **Seu acesso** agora informa que você pode gerenciar a
coleção.

Todas as outras pessoas são visualizadores: podem navegar pela coleção, mas não
alterá-la. Para permitir que visualizadores mantenham o próprio status de jogo,
sessões e sorteios, ative **viewers_track_progress** na mesma aba
**Configuração** e reinicie o aplicativo.

## Conectar o IGDB {#connect-igdb}

O [IGDB](https://www.igdb.com) fornece ao Memory Card títulos, capas, datas de
lançamento, gêneros e outros detalhes. É opcional: sem ele, você cadastra os
jogos manualmente. O IGDB usa contas da Twitch:

1. Ative a autenticação em dois fatores na sua conta da Twitch. A Twitch exige
   isso para registrar aplicativos.
2. Abra o [console de desenvolvedor da Twitch](https://dev.twitch.tv/console/apps)
   e registre um aplicativo:
   - **OAuth Redirect URLs:** `http://localhost`
   - **Category:** Application Integration
   - **Client Type:** Confidential
3. Abra o aplicativo em **Manage**, copie o **Client ID** e selecione
   **New Secret** para gerar um **Client Secret**.
4. No Home Assistant, abra a aba **Configuração** do aplicativo, preencha
   **igdb_client_id** e **igdb_client_secret** e salve. Para guardar esses
   valores no `secrets.yaml` do Home Assistant, informe `!secret` seguido da
   chave, como `!secret igdb_client_secret`.
5. Reinicie o aplicativo.

**Configurações > Status** no Memory Card mostra **IGDB: Conectado** assim que
estiver funcionando. Veja [IGDB](igdb.md) para saber o que ele preenche.

## Adicionar seus primeiros jogos

Há três formas de começar:

- **Um de cada vez:** selecione **Adicionar jogo** na [Biblioteca](library.md),
  digite o título e escolha a correspondência do IGDB. Depois adicione suas
  cópias na [página do jogo](game.md).
- **Por uma planilha:** vá em [Importar / exportar](import-export.md), comece
  por **Baixar modelo em branco** ou use um arquivo que você já mantém, e
  importe-o.
- **Pela caixa:** selecione **Ler código de barras** e aponte o celular para a
  caixa. Veja [Códigos de barras](barcodes.md).

Antes de adicionar muitas cópias, cadastre seus consoles em
[Hardware](hardware.md) e suas contas de loja em
[Gerenciar > Contas](manage.md#accounts), para que cada cópia indique onde fica.

## Próximos passos

- Defina seu status de jogo para os jogos que você tem, e o Memory Card monta o
  seu [Backlog](backlog.md).
- Adicione os [sensores](sensors.md) do Memory Card aos painéis do Home
  Assistant.
- Escolha um [tema](themes.md).
