# Configurações

**Configurações**, na parte inferior da barra lateral, tem duas abas:
**Aparência** e **Status**. Qualquer pessoa pode abrir. **Aparência** abre
primeiro depois que um gestor é configurado; em uma instalação nova, **Status**
abre primeiro para você copiar seu ID de usuário.

![Configurações, na aba Aparência](https://ha-memory-card.pages.dev/images/settings.webp)

## Aparência

As escolhas feitas aqui ficam salvas nesse navegador, então cada pessoa e cada
dispositivo mantém as próprias.

- **Tema:** um dos 12 temas. Veja [Temas](themes.md).
- **Esquema de cores:** **Sistema**, **Claro** ou **Escuro**. O botão no canto
  superior direito de cada página também alterna isso.
- **Idioma:** inglês, espanhol ou português do Brasil. **Automático** segue o
  idioma do Home Assistant, ou o do navegador quando o Memory Card não tem o do
  Home Assistant.

As datas seguem o idioma, então em português do Brasil aparecem como
23/11/2026. Para escrevê-las de outro jeito em todos os idiomas, defina a opção
`date_format` do aplicativo.

## Status

**Integrações** mostra se cada conexão está funcionando:

- **IGDB:** conectado, não configurado ou desligado porque os serviços externos
  estão desligados. Veja [IGDB](igdb.md).
- **Sensores do Home Assistant:** conectado, aguardando a integração ou o que
  impede a conexão. Veja [sensores](sensors.md).
- **Leitura de tags NFC:** se o Memory Card está ouvindo as leituras do Home
  Assistant. Veja [tags NFC](nfc-tags.md).

**Seu acesso** mostra com qual usuário você está conectado, seu papel e seu ID
de usuário do Home Assistant. **Copiar ID de usuário** copia o ID, para nomear
um gestor na configuração do aplicativo ou associar uma pessoa a um leitor NFC.

**Sobre o Memory Card** mostra a versão, a licença, links para a documentação e
o suporte, e onde apoiar o desenvolvimento.
