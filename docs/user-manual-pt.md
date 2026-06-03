[EN](user-manual-en.md) · [UA](user-manual-ua.md) · [ES](user-manual-es.md) · [FR](user-manual-fr.md) · [IT](user-manual-it.md) · **PT**

> **Nota.** Este documento é uma tradução automática para a v0.1.0. A aguardar revisão humana. Se detectar um erro de tradução, por favor abra um issue em <https://github.com/ratushnyi-labs/xdu>.

# xdu — manual do utilizador

Este é o manual do utilizador v0.1.0 do `xdu`, um explorador de utilização
de disco apenas de leitura com interface de terminal. É propositadamente
curto. Tudo o que aqui se descreve funciona nos binários publicados; nada é
prospectivo.

## O que o xdu faz

O `xdu` faz a varredura de uma árvore de directórios e apresenta o
resultado como uma interface de terminal navegável. Cada linha mostra um
nome, uma barra de tamanho relativo, o tamanho absoluto dessa entrada
(incluindo todos os descendentes no caso de directórios) e a data do último
acesso. A margem direita do rodapé mostra o espaço livre no volume que
contém a raiz da varredura. Move-se com as setas ou com as teclas vim,
entra-se num directório com Enter e sobe-se com Backspace. Existe uma acção
de exportação (`e`) que escreve a sub-árvore focada num ficheiro JSON, uma
sobreposição de erros (`!`) para falhas de varredura por caminho e um
painel de ajuda integrado (`?` ou F1).

## O que o xdu NÃO faz

O xdu é apenas de leitura por omissão. Não elimina, não move, não renomeia,
não executa `chmod`, não trunca, não termina processos e não modifica os
seus ficheiros de qualquer forma a menos que você prima uma tecla específica
e confirme. Não é um serviço nem um daemon. Apenas é executado enquanto
estiver a interagir com ele. Não comunica com qualquer servidor, não recolhe
telemetria, não envia analítica nem abre qualquer ligação de rede. Não tem
modo de auto-eliminação, nem limpador em segundo plano, nem motor de
sugestões "inteligente", nem modo gráfico em v0.1.0.

## Instalação

Os binários pré-compilados para a v0.1.0 são publicados na página de
Releases do GitHub: <https://github.com/ratushnyi-labs/xdu/releases>.
Descarregue o arquivo correspondente à sua plataforma, extraia-o e coloque
o binário `xdu` (ou `xdu.exe` no Windows) num directório incluído na sua
`PATH`.

**Linux (musl).** São publicadas duas builds estáticas:
`x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`. Ambas são
totalmente estáticas — sem glibc, sem dependências de bibliotecas
partilhadas — e correm em qualquer kernel Linux 3.2 ou superior. Escolha a
que corresponde a `uname -m` (`x86_64` ou `aarch64`).

**macOS.** São publicadas duas builds: `x86_64-apple-darwin` (Intel) e
`aarch64-apple-darwin` (Apple Silicon). Após o download, pode ser necessário
remover o atributo de quarentena que o macOS aplica aos binários
descarregados: `xattr -d com.apple.quarantine ./xdu`.

**Windows (MSVC).** São publicadas duas builds de consola PE32+:
`x86_64-pc-windows-msvc` e `aarch64-pc-windows-msvc`. São ficheiros `.exe`
independentes — sem instalador, sem dependências de DLL para além do runtime
MSVC que vem incluído no Windows. Execute a partir de `cmd.exe`, PowerShell
ou Windows Terminal.

## Primeira execução

Lançar `xdu` sem argumentos faz a varredura do directório de trabalho
actual no Unix. No Windows, o lançamento sem argumentos mostra uma lista
numerada de unidades montadas (unidade de sistema, dispositivos amovíveis,
partilhas de rede com um tamanho total conhecido); escreva o número e prima
Enter, ou prima simplesmente Enter para escolher a primeira entrada. Passe
um caminho explícito (`xdu /home/me`, `xdu C:\Users`) para saltar o
selector. Enquanto a varredura inicial decorre vê-se uma linha de
progresso; quando a varredura termina, o TUI assume o controlo e pode
navegar imediatamente.

## Navegação

Mova a selecção com as setas para cima/baixo ou com `k` / `j`. Entre no
directório seleccionado com a seta direita, `l` ou Enter. Suba ao pai com a
seta esquerda, `h` ou Backspace; em alternativa, seleccione a linha `..` no
topo da lista e prima Enter. Subir para além da raiz original da varredura
desencadeia uma nova varredura ao nível do sistema de ficheiros do novo
pai. Prima `q` ou Ctrl-C para sair. Prima `?` ou F1 a qualquer momento para
abrir um painel que resume todos os atalhos de teclado.

## Ordenação e unidades

Prima `s` para percorrer ciclicamente a chave de ordenação. A ordem do
ciclo é: tamanho descendente (a omissão — os maiores primeiro), tamanho
ascendente, nome ascendente, nome descendente, contagem de ficheiros
descendente, contagem de ficheiros ascendente. Prima `s` novamente para
continuar; a chave de ordenação activa é mostrada no rodapé.

Prima `u` para alternar a unidade de tamanho entre unidades binárias IEC
(`KiB`, `MiB`, `GiB`, `TiB` — potências de 1024) e unidades decimais SI
(`KB`, `MB`, `GB`, `TB` — potências de 1000). Pode também iniciar em modo
SI passando `--si` na linha de comandos. O modo aplica-se de forma
consistente a todos os tamanhos no ecrã — a linha seleccionada, o rodapé
com espaço livre e qualquer exportação escrita enquanto o modo está activo.

## Eliminação e reciclagem

O xdu é apenas de leitura por omissão. Para eliminar a entrada
seleccionada, prima `d`. Surge uma janela modal de confirmação com `[ No ]`
focado; premir Enter ou Esc cancela. Para prosseguir, navegue até `[ Yes ]`
(Tab ou seta direita) e prima Enter. Este comportamento com `[ No ]` por
omissão é uma escolha deliberada de segurança e é idêntico em todos os
modos.

Execute `xdu --trash` para mudar para modo de reciclagem. Em modo de
reciclagem o modal de confirmação mostra "Move to trash" em vez de
"Delete", e a acção tem como alvo a reciclagem nativa do sistema operativo:

- **Linux:** a especificação freedesktop.org Trash v1.0. Os ficheiros
  movem-se para `$XDG_DATA_HOME/Trash/files/` (com recurso a
  `$HOME/.local/share/Trash/files/`), com um ficheiro de metadados irmão
  `.trashinfo`. As acções padrão "Restaurar da reciclagem" dos gestores
  de ficheiros funcionam depois.
- **Windows:** a Reciclagem do sistema, através do mesmo caminho do
  kernel que o Explorer utiliza.
- **macOS:** não suportado em v0.1.0. Executar `xdu --trash` em macOS
  termina com código diferente de zero e com uma mensagem clara. A
  eliminação normal com a tecla `d` (sem reciclagem) continua a funcionar
  normalmente em macOS.

O modo de reciclagem permanece protegido pelo modal de confirmação com
`[ No ]` por omissão — não existe em lado nenhum no xdu um caminho de
"eliminar sem perguntar".

## Detentor do bloqueio e fecho forçado

Quando uma eliminação no Windows falha porque outro processo tem o
ficheiro aberto, o xdu enumera os detentores via Windows Restart Manager e
mostra uma modal informativa com a lista de nomes de processo e PIDs. A
partir dessa modal pode premir `f` para tentar um fecho forçado (isto
invoca `DuplicateHandle` com `DUPLICATE_CLOSE_SOURCE` em cada handle
externo); o próprio fecho forçado é protegido por uma modal de confirmação
separada com `[ No ]` por omissão. É agressivo: utilize apenas quando
souber o que é o detentor do bloqueio.

No Linux, antes de se tentar uma eliminação, o xdu percorre
oportunisticamente `/proc/*/fd/` e lista qualquer processo com um
descritor de ficheiro aberto a apontar para o caminho seleccionado. A
lista é meramente informativa — o `unlink` POSIX do Linux tem sucesso
mesmo na presença de detentores abertos, pelo que a eliminação prossegue
após confirmação. A intenção é deixá-lo decidir se elimina um ficheiro
actualmente em uso.

## Sobreposição de erros

Uma varredura sobre um sistema de ficheiros real depara-se ocasionalmente
com um erro por caminho: um directório com permissão negada, um handle NFS
obsoleto, um link simbólico partido. O xdu regista cada um desses erros
sem parar a varredura; prima `!` para abrir a sobreposição de erros e
lê-los. A sobreposição percorre-se com as setas e fecha-se com Esc.

## Exportação

Prima `e` para exportar a sub-árvore actualmente focada para um ficheiro
JSON. O xdu solicita o caminho de saída com `xdu-export.json` no directório
actual como valor por omissão; prima Enter para aceitar o valor por omissão
ou escreva um novo caminho. O formato do ficheiro está documentado em
`docs/spec.md` (etiqueta de esquema `xdu-export-1`). A exportação é
puramente a escrita do ficheiro escolhido — não toca no sistema de ficheiros
varrido.

## Nota de segurança

O xdu é apenas de leitura por omissão. As únicas escritas que alguma vez
realiza são:

1. Uma eliminação explícita através da tecla `d`, protegida por um modal
   de confirmação com `[ No ]` por omissão.
2. Uma eliminação opcional em modo de reciclagem através de `--trash` +
   `d`, também protegida pelo modal com `[ No ]` por omissão, dirigida à
   reciclagem nativa do sistema operativo onde for suportada.
3. Uma exportação JSON através da tecla `e`, que escreve um ficheiro no
   caminho que você confirmar.

Não há auto-eliminação, nem mutação em segundo plano, nem renomeação, nem
`chmod`, nem `truncate`, nem `kill` / `TerminateProcess`, nem qualquer
actividade de rede. Se algo no seu sistema de ficheiros mudar enquanto o
xdu estiver em execução, é porque você premiu uma tecla e confirmou.
