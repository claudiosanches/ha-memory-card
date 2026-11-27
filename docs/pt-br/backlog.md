# Backlog

**Backlog** na barra lateral mostra seus jogos por status de jogo: o que você
está jogando, o que está em pausa e o que espera por você. É pessoal; cada
pessoa da casa tem o próprio. A barra no topo conta quantos você concluiu.

![O backlog com um jogo sorteado](https://ha-memory-card.pages.dev/images/backlog.webp)

_Títulos de jogos e capas pertencem aos seus respectivos proprietários._

## Classifique seus jogos

Jogos sem status de jogo ficam esperando em **Ainda sem status**.
**Classificar agora** passa por eles um de cada vez: escolha um status, ou
**Pular** para deixar para depois, e por fim **Pronto**.

Registrar uma sessão para um jogo no backlog, ou sem status, muda o status para
jogando. **Joguei hoje** na [página do jogo](game.md) faz o mesmo.

## Escolha por mim

**Não consegue decidir?** sorteia um jogo aleatório do seu backlog.

1. Em **Jogar em**, mantenha **Qualquer dispositivo** ou escolha o dispositivo
   em que quer jogar.
2. Em **Duração**, mantenha **Qualquer duração** ou escolha quantas horas a
   história principal pode levar no máximo. Jogos sem essa informação no IGDB
   ficam de fora.
3. Selecione **Escolha por mim**. Um jogo sorteado no último dia só volta a sair
   quando não sobrar outro.
4. **Começar a jogar** muda o status do jogo para jogando. **Sortear outro** faz
   um novo sorteio.

O último sorteio também aparece no sensor `sensor.memory_card_pick` do Home
Assistant, e o Home Assistant pode fazer o sorteio para você. Veja
[sensores](sensors.md).

## Seções

- **Jogando agora:** **Zerei**, **100%**, **Pausar** e **Abandonar** movem o
  jogo adiante.
- **Em pausa:** **Retomar** coloca o jogo de volta em jogando.
- **No backlog:** **Começar** inicia um jogo. Cada cartão mostra as horas da
  história principal segundo o IGDB, quando disponíveis.

A seção em que um status aparece depende do grupo como o qual ele conta,
definido pelos gestores em **Gerenciar > Status de jogo**:

| Conta como | Na página Backlog |
| --- | --- |
| Backlog | Em No backlog, e sorteado por **Escolha por mim** |
| Jogando | Em Jogando agora |
| Em pausa | Em Em pausa |
| Terminado | Contado como concluído |
| Não conta | Fica de fora |

## Visualizadores

Gestores sempre mantêm o próprio backlog. Um visualizador vê o próprio quando um
gestor ativa `viewers_track_progress` nas opções do aplicativo.
