# Gerenciar

**Gerenciar** na barra lateral reúne as listas usadas para montar a coleção. Só
gestores veem esta página.

![Gerenciar, na aba Plataformas](https://ha-memory-card.pages.dev/images/manage.webp)

Ao editar uma conta, loja, plataforma ou etiqueta, ou um dispositivo em
[Hardware](hardware.md), o formulário mostra o slug: o nome curto gerado a
partir do nome, usado pelos links da biblioteca e pelos sorteios no Home
Assistant.

Consoles, portáteis e PCs ficam na página [Hardware](hardware.md), junto com o
restante do seu equipamento.

## Contas {#accounts}

As contas de loja onde ficam seus jogos digitais, como uma Conta Nintendo,
PlayStation Network ou Steam. Uma cópia digital registra sua conta
separadamente do dispositivo em que está instalada, então um jogo comprado na
PlayStation Network e jogado no PS5 tem ambos. **Adicionar conta** pede:

- um nome, usado para escolher a conta de uma cópia;
- **Loja:** onde compra jogos. Uma cópia nova nesta conta a usa como loja;
- **Pessoa:** de quem é a conta na casa;
- **Plataformas:** as plataformas cujos jogos esta conta guarda. Cópias de
  outras plataformas não recebem esta conta como opção.

Contas inativas ficam de fora ao escolher a conta de uma cópia. Excluir uma
conta mantém as cópias dela, sem conta.

## Lojas

As lojas físicas e online onde você compra jogos. **Adicionar loja** pede um
nome. As lojas digitais já existem desde o início. Excluir uma loja mantém as
cópias dela, sem loja.

## Plataformas

**Suas plataformas** são as mostradas ao adicionar jogos, dispositivos e
contas, na ordem exibida. Arraste para reordenar.

**Todas as plataformas** lista todas as plataformas que o Memory Card conhece,
por tipo: consoles, portáteis, computadores, celulares, fliperama e outras.
**Adicionar** coloca uma plataforma nas suas plataformas. Remover uma
plataforma a oculta ao adicionar jogos, dispositivos e contas; o que já a usa
continua. **Adicionar plataforma personalizada** cria uma que não está na
lista. Uma plataforma personalizada usada por jogos, dispositivos ou contas não
pode ser excluída; remova-a das suas plataformas em vez disso.

## Status de jogo

Os status que cada pessoa escolhe para as próprias partidas, na ordem exibida.
Arraste para reordenar.

Os seis status nativos (`backlog`, `playing`, `paused`, `completed`, `dropped`
e `mastered`) podem ser renomeados, recoloridos e reordenados, mas não
excluídos. As chaves deles nunca mudam, para que arquivos CSV e o Home
Assistant possam contar com eles.

**Adicionar status** pede um nome, uma cor e como ele conta, o que define onde
os jogos dele aparecem na página [Backlog](backlog.md):

- **Backlog:** esperando para jogar, e sorteado por **Escolha por mim**.
- **Jogando** e **Em pausa:** nessas seções.
- **Terminado:** contado como concluído.
- **Não conta:** fica de fora da página Backlog.

Excluir um status pergunta para onde mover os jogos de cada pessoa que estão
nele, ou os deixa sem status.

## Etiquetas

Seus próprios grupos para organizar os jogos, como "Cooperativo no sofá" ou
"Jogar de novo". **Adicionar etiqueta** pede um nome e uma cor, e a lista
mostra quantos jogos têm cada etiqueta. Defina as etiquetas de um jogo com
**Editar** na [página dele](game.md). Os nomes não podem ter vírgulas nem ponto
e vírgula, pois esses caracteres separam etiquetas em arquivos CSV. Excluir uma
etiqueta mantém os jogos dela.
