[EN](user-manual-en.md) · [UA](user-manual-ua.md) · **ES** · [FR](user-manual-fr.md) · [IT](user-manual-it.md) · [PT](user-manual-pt.md)

> **Nota.** Este documento es una traducción automática para v0.1.0. Pendiente de revisión humana. Si detecta un error de traducción, abra una incidencia en <https://github.com/ratushnyi-labs/xdu>.

# xdu — manual de usuario

Este es el manual de usuario v0.1.0 de `xdu`, un explorador de uso de disco de
sólo lectura con interfaz de terminal. Es intencionadamente breve. Todo lo
descrito aquí funciona en los binarios publicados; nada de lo aquí escrito es
una promesa de futuro.

## Qué hace xdu

`xdu` escanea un árbol de directorios y presenta el resultado como una
interfaz de terminal navegable. Cada fila muestra un nombre, una barra de
tamaño relativo, el tamaño absoluto de esa entrada (incluyendo todos los
descendientes en el caso de directorios) y la fecha del último acceso. El
borde derecho del pie de página muestra el espacio libre en el volumen que
contiene la raíz del escaneo. Se navega con las teclas de flecha o con las
teclas vim; se entra en un directorio con Enter y se sube al directorio
superior con Backspace. Hay una acción de exportación (`e`) que escribe el
subárbol enfocado en un archivo JSON, una superposición de errores (`!`) para
los fallos de escaneo por ruta y un panel de ayuda integrado (`?` o F1).

## Qué NO hace xdu

xdu es de sólo lectura por defecto. No elimina, mueve, renombra, hace `chmod`,
trunca, mata procesos ni modifica sus archivos de ninguna manera salvo que
pulse una tecla específica y confirme. No es un servicio ni un demonio. Sólo
se ejecuta mientras interactúa con él. No llama a casa, no recopila
telemetría, no envía analíticas ni abre ninguna conexión de red. No tiene
modo de auto-eliminación, ni limpiador en segundo plano, ni motor de
sugerencias "inteligente", ni modo gráfico en v0.1.0.

## Instalación

Los binarios precompilados para v0.1.0 se publican en la página de Releases
de GitHub: <https://github.com/ratushnyi-labs/xdu/releases>. Descargue el
archivo correspondiente a su plataforma, extráigalo y coloque el binario
`xdu` (o `xdu.exe` en Windows) en algún directorio incluido en su `PATH`.

**Linux (musl).** Se publican dos compilaciones estáticas:
`x86_64-unknown-linux-musl` y `aarch64-unknown-linux-musl`. Ambas son
completamente estáticas — sin glibc y sin dependencias de bibliotecas
compartidas — y funcionan en cualquier kernel Linux 3.2 o superior. Elija la
que corresponda a `uname -m` (`x86_64` o `aarch64`).

**macOS.** Se publican dos compilaciones: `x86_64-apple-darwin` (Intel) y
`aarch64-apple-darwin` (Apple Silicon). Tras la descarga, es posible que deba
eliminar el atributo de cuarentena que macOS añade a los binarios
descargados: `xattr -d com.apple.quarantine ./xdu`.

**Windows (MSVC).** Se publican dos compilaciones de consola PE32+:
`x86_64-pc-windows-msvc` y `aarch64-pc-windows-msvc`. Son archivos `.exe`
autónomos — sin instalador y sin dependencias de DLL más allá del runtime de
MSVC que viene incluido con Windows. Ejecútelos desde `cmd.exe`, PowerShell
o Windows Terminal.

## Primer arranque

Lanzar `xdu` sin argumentos escanea el directorio de trabajo actual en Unix.
En Windows, lanzarlo sin argumentos muestra una lista numerada de unidades
montadas (unidad de sistema, medios extraíbles, recursos de red con un
tamaño total conocido); escriba el número y pulse Enter, o simplemente pulse
Enter para elegir la primera entrada. Pase una ruta explícita
(`xdu /home/me`, `xdu C:\Users`) para evitar el selector. Mientras se
ejecuta el escaneo inicial verá una línea de progreso; una vez que finalice
el escaneo, el TUI toma el control y puede navegar de inmediato.

## Navegación

Mueva la selección con las flechas arriba/abajo o con `k` / `j`. Entre en el
directorio seleccionado con la flecha derecha, `l` o Enter. Suba al
directorio padre con la flecha izquierda, `h` o Backspace; alternativamente,
seleccione la fila `..` en la parte superior del listado y pulse Enter.
Subir más allá de la raíz original del escaneo dispara un nuevo escaneo a
nivel de sistema de archivos del nuevo padre. Pulse `q` o Ctrl-C para salir.
Pulse `?` o F1 en cualquier momento para abrir un panel que resume todos los
atajos de teclado.

## Ordenación y unidades

Pulse `s` para alternar cíclicamente la clave de ordenación. El orden del
ciclo es: tamaño descendente (el predeterminado — los más grandes primero),
tamaño ascendente, nombre ascendente, nombre descendente, conteo de archivos
descendente, conteo de archivos ascendente. Pulse `s` de nuevo para seguir
avanzando; la clave de ordenación activa se muestra en el pie de página.

Pulse `u` para alternar la unidad de tamaño entre unidades binarias IEC
(`KiB`, `MiB`, `GiB`, `TiB` — potencias de 1024) y unidades decimales SI
(`KB`, `MB`, `GB`, `TB` — potencias de 1000). También puede arrancar en
modo SI pasando `--si` en la línea de comandos. El modo se aplica de manera
consistente a todos los tamaños en pantalla — la fila seleccionada, el pie
de página con espacio libre y cualquier exportación escrita mientras el
modo está activo.

## Eliminación y papelera

xdu es de sólo lectura por defecto. Para eliminar la entrada seleccionada,
pulse `d`. Aparece un modal de confirmación con `[ No ]` enfocado; pulsar
Enter o Esc cancela la acción. Para continuar, navegue hasta `[ Yes ]` (Tab
o flecha derecha) y pulse Enter. Este comportamiento con `[ No ]` como
predeterminado es una elección deliberada de seguridad y es el mismo en
todos los modos.

Ejecute `xdu --trash` para activar el modo papelera. En modo papelera el
modal de confirmación muestra "Move to trash" en lugar de "Delete", y la
acción apunta a la papelera nativa del sistema operativo:

- **Linux:** la especificación freedesktop.org Trash v1.0. Los archivos se
  mueven a `$XDG_DATA_HOME/Trash/files/` (con respaldo
  `$HOME/.local/share/Trash/files/`), con un archivo hermano de metadatos
  `.trashinfo`. Las acciones estándar "Restaurar desde la papelera" de los
  gestores de archivos funcionan después.
- **Windows:** la Papelera de reciclaje del sistema, a través de la misma
  ruta del kernel que utiliza Explorer.
- **macOS:** no soportado en v0.1.0. Ejecutar `xdu --trash` en macOS
  termina con código distinto de cero y un mensaje claro. La eliminación
  ordinaria con la tecla `d` (sin papelera) sigue funcionando con
  normalidad en macOS.

El modo papelera sigue estando protegido por el modal de confirmación con
`[ No ]` por defecto — en xdu no hay ningún camino de "eliminar sin
preguntar".

## Bloqueador y cierre forzado

Cuando una eliminación en Windows falla porque otro proceso tiene el archivo
abierto, xdu enumera los titulares del bloqueo mediante Windows Restart
Manager y muestra un modal informativo con la lista de nombres de proceso y
PIDs. Desde ese modal puede pulsar `f` para intentar un cierre forzado (esto
invoca `DuplicateHandle` con `DUPLICATE_CLOSE_SOURCE` sobre cada handle
externo); el propio cierre forzado está protegido por un modal de
confirmación independiente con `[ No ]` por defecto. Es agresivo: úselo
únicamente cuando sepa qué es el proceso bloqueante.

En Linux, antes de intentar una eliminación, xdu recorre oportunistamente
`/proc/*/fd/` y enumera cualquier proceso con un descriptor de archivo
abierto apuntando a la ruta seleccionada. La lista es meramente informativa
— el `unlink` POSIX de Linux tiene éxito incluso cuando hay titulares
abiertos, por lo que la eliminación procede tras la confirmación. La
intención es permitirle decidir si eliminar un archivo que está actualmente
en uso.

## Superposición de errores

Un escaneo sobre un sistema de archivos real ocasionalmente encontrará un
error por ruta: un directorio con permiso denegado, un handle NFS obsoleto,
un enlace simbólico roto. xdu registra cada uno de esos errores sin detener
el escaneo; pulse `!` para abrir la superposición de errores y leerlos. La
superposición se desplaza con las flechas y se cierra con Esc.

## Exportación

Pulse `e` para exportar el subárbol actualmente enfocado a un archivo JSON.
xdu solicita la ruta de salida con `xdu-export.json` en el directorio
actual como valor por defecto; pulse Enter para aceptar el valor por
defecto o escriba una ruta nueva. El formato del archivo está documentado
en `docs/spec.md` (etiqueta de esquema `xdu-export-1`). La exportación es
puramente una escritura del archivo elegido — no toca el sistema de
archivos escaneado.

## Nota de seguridad

xdu es de sólo lectura por defecto. Las únicas escrituras que llega a
realizar son:

1. Una eliminación explícita mediante la tecla `d`, protegida por un modal
   de confirmación con `[ No ]` por defecto.
2. Una eliminación opcional en modo papelera mediante `--trash` + `d`,
   también protegida por el modal con `[ No ]` por defecto, dirigida a la
   papelera nativa del sistema operativo donde esté soportada.
3. Una exportación JSON mediante la tecla `e`, que escribe un archivo en la
   ruta que usted confirme.

No hay auto-eliminación, ni mutación en segundo plano, ni renombrado, ni
`chmod`, ni `truncate`, ni `kill` / `TerminateProcess`, ni actividad de red
de ningún tipo. Si algo en su sistema de archivos cambia mientras xdu se
está ejecutando, es porque usted pulsó una tecla y confirmó.
