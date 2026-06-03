[EN](user-manual-en.md) · [UA](user-manual-ua.md) · [ES](user-manual-es.md) · [FR](user-manual-fr.md) · **IT** · [PT](user-manual-pt.md)

> **Nota.** Questo documento è una traduzione automatica per la v0.1.0. In attesa di revisione umana. Se notate un errore di traduzione, aprite per favore una segnalazione su <https://github.com/ratushnyi-labs/xdu>.

# xdu — manuale utente

Questo è il manuale utente v0.1.0 di `xdu`, un esploratore dell'uso del
disco in sola lettura con interfaccia a terminale. È volutamente breve.
Tutto ciò che vi è descritto funziona nei binari rilasciati; nulla è
proiettato sul futuro.

## Cosa fa xdu

`xdu` esegue la scansione di un albero di directory e ne presenta il
risultato come interfaccia a terminale navigabile. Ogni riga mostra un nome,
una barra di dimensione relativa, la dimensione assoluta di quella voce
(includendo tutti i discendenti per le directory) e la data dell'ultimo
accesso. Il bordo destro del piè di pagina mostra lo spazio libero sul
volume che contiene la radice della scansione. Ci si sposta con le frecce o
con i tasti vim, si entra in una directory con Enter e si risale con
Backspace. È disponibile un'azione di esportazione (`e`) che scrive il
sottoalbero attualmente in focus in un file JSON, una sovrapposizione di
errori (`!`) per i singoli fallimenti di scansione e un pannello di aiuto
integrato (`?` o F1).

## Cosa xdu NON fa

xdu è in sola lettura per impostazione predefinita. Non elimina, non
sposta, non rinomina, non esegue `chmod`, non tronca, non termina processi e
non modifica in alcun modo i vostri file a meno che non premiate un tasto
specifico e confermiate. Non è un servizio né un demone. Funziona soltanto
mentre interagite con esso. Non telefona a casa, non raccoglie telemetria,
non invia dati analitici e non apre alcuna connessione di rete. Non ha una
modalità di auto-eliminazione, né un pulitore in background, né un motore
di suggerimenti "intelligente", né una modalità grafica in v0.1.0.

## Installazione

I binari precompilati per la v0.1.0 sono pubblicati sulla pagina Releases
di GitHub: <https://github.com/ratushnyi-labs/xdu/releases>. Scaricate
l'archivio relativo alla vostra piattaforma, estraetelo e collocate il
binario `xdu` (o `xdu.exe` su Windows) in una directory inclusa nel vostro
`PATH`.

**Linux (musl).** Sono pubblicate due build statiche:
`x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`. Entrambe sono
completamente statiche — senza glibc, senza dipendenze da librerie
condivise — e funzionano su qualsiasi kernel Linux 3.2 o successivo.
Scegliete quella corrispondente a `uname -m` (`x86_64` o `aarch64`).

**macOS.** Sono pubblicate due build: `x86_64-apple-darwin` (Intel) e
`aarch64-apple-darwin` (Apple Silicon). Dopo il download potrebbe essere
necessario rimuovere l'attributo di quarantena che macOS appone ai binari
scaricati: `xattr -d com.apple.quarantine ./xdu`.

**Windows (MSVC).** Sono pubblicate due build console PE32+:
`x86_64-pc-windows-msvc` e `aarch64-pc-windows-msvc`. Sono file `.exe`
autonomi — senza installer, senza dipendenze da DLL oltre al runtime MSVC
che viene incluso con Windows. Eseguiteli da `cmd.exe`, PowerShell o
Windows Terminal.

## Primo avvio

Lanciare `xdu` senza argomenti scansiona la directory di lavoro corrente
su Unix. Su Windows, il lancio senza argomenti mostra un elenco numerato
delle unità montate (unità di sistema, supporti rimovibili, condivisioni
di rete con una dimensione totale nota); digitate il numero e premete
Enter, oppure premete semplicemente Enter per selezionare la prima voce.
Passate un percorso esplicito (`xdu /home/me`, `xdu C:\Users`) per saltare
il selettore. Mentre la scansione iniziale è in corso vedete una riga di
avanzamento; non appena la scansione termina, il TUI prende il controllo e
potete navigare immediatamente.

## Navigazione

Spostate la selezione con le frecce su/giù o con `k` / `j`. Entrate nella
directory selezionata con la freccia destra, `l` o Enter. Risalite al
genitore con la freccia sinistra, `h` o Backspace; in alternativa,
selezionate la riga `..` in cima all'elenco e premete Enter. Risalire
oltre la radice originaria della scansione attiva una nuova scansione a
livello di filesystem del nuovo genitore. Premete `q` o Ctrl-C per uscire.
Premete `?` o F1 in qualsiasi momento per un pannello che riepiloga tutte
le scorciatoie da tastiera.

## Ordinamento e unità

Premete `s` per scorrere ciclicamente la chiave di ordinamento. L'ordine
del ciclo è: dimensione decrescente (predefinito — i più grandi per
primi), dimensione crescente, nome crescente, nome decrescente, conteggio
file decrescente, conteggio file crescente. Premete `s` di nuovo per
continuare; la chiave di ordinamento attiva è mostrata nel piè di pagina.

Premete `u` per alternare l'unità di dimensione tra unità binarie IEC
(`KiB`, `MiB`, `GiB`, `TiB` — potenze di 1024) e unità decimali SI
(`KB`, `MB`, `GB`, `TB` — potenze di 1000). Potete inoltre avviare in
modalità SI passando `--si` da riga di comando. La modalità si applica in
modo coerente a tutte le dimensioni a schermo — la riga selezionata, il
piè di pagina con lo spazio libero e qualsiasi esportazione scritta
mentre la modalità è attiva.

## Eliminazione e cestino

xdu è in sola lettura per impostazione predefinita. Per eliminare la voce
selezionata, premete `d`. Appare una finestra modale di conferma con
`[ No ]` in focus; premere Enter o Esc annulla. Per procedere, spostatevi
su `[ Yes ]` (Tab o freccia destra) e premete Enter. Questo comportamento
con `[ No ]` predefinito è una scelta deliberata di sicurezza ed è
identico in tutte le modalità.

Eseguite `xdu --trash` per passare in modalità cestino. In modalità
cestino la modale di conferma mostra "Move to trash" anziché "Delete" e
l'azione punta al cestino nativo del sistema operativo:

- **Linux:** la specifica freedesktop.org Trash v1.0. I file vengono
  spostati in `$XDG_DATA_HOME/Trash/files/` (con ripiego su
  `$HOME/.local/share/Trash/files/`), insieme a un file di metadati
  affine `.trashinfo`. Le azioni standard "Ripristina dal cestino" dei
  file manager funzionano successivamente.
- **Windows:** il Cestino di sistema, attraverso lo stesso percorso
  kernel utilizzato da Explorer.
- **macOS:** non supportato in v0.1.0. Eseguire `xdu --trash` su macOS
  termina con codice diverso da zero e con un messaggio chiaro.
  L'eliminazione ordinaria con il tasto `d` (senza cestino) continua a
  funzionare normalmente su macOS.

La modalità cestino resta protetta dalla modale di conferma con `[ No ]`
predefinito — in xdu non esiste alcun percorso "elimina senza chiedere".

## Detentore del lock e chiusura forzata

Quando un'eliminazione su Windows fallisce perché un altro processo ha il
file aperto, xdu enumera i detentori tramite Windows Restart Manager e
mostra una modale informativa con l'elenco dei nomi di processo e dei PID.
Da quella modale potete premere `f` per tentare una chiusura forzata
(questo invoca `DuplicateHandle` con `DUPLICATE_CLOSE_SOURCE` su ciascun
handle esterno); la chiusura forzata stessa è protetta da una modale di
conferma separata con `[ No ]` predefinito. È aggressiva: usatela soltanto
quando sapete cosa sia il detentore del lock.

Su Linux, prima che venga tentata un'eliminazione, xdu percorre
opportunisticamente `/proc/*/fd/` ed elenca qualsiasi processo con un
descrittore di file aperto puntato al percorso selezionato. L'elenco è
puramente informativo — l'`unlink` POSIX di Linux riesce anche in presenza
di detentori aperti, quindi l'eliminazione procede dopo la conferma.
L'intento è lasciarvi decidere se eliminare un file attualmente in uso.

## Sovrapposizione di errori

Una scansione su un vero filesystem occasionalmente incontra un errore
per-percorso: una directory con permesso negato, un handle NFS obsoleto,
un collegamento simbolico interrotto. xdu registra ognuno di tali errori
senza fermare la scansione; premete `!` per aprire la sovrapposizione
degli errori e leggerli. La sovrapposizione si scorre con le frecce e si
chiude con Esc.

## Esportazione

Premete `e` per esportare il sottoalbero attualmente in focus in un file
JSON. xdu richiede il percorso di output con `xdu-export.json` nella
directory corrente come valore predefinito; premete Enter per accettare
il valore predefinito oppure digitate un nuovo percorso. Il formato del
file è documentato in `docs/spec.md` (tag di schema `xdu-export-1`).
L'esportazione è puramente una scrittura del file scelto — non tocca il
filesystem scansionato.

## Nota di sicurezza

xdu è in sola lettura per impostazione predefinita. Le uniche scritture
che esegue sono:

1. Un'eliminazione esplicita tramite il tasto `d`, protetta da una modale
   di conferma con `[ No ]` predefinito.
2. Un'eliminazione opzionale in modalità cestino tramite `--trash` + `d`,
   anch'essa protetta dalla modale con `[ No ]` predefinito, diretta al
   cestino nativo del sistema operativo dove supportata.
3. Un'esportazione JSON tramite il tasto `e`, che scrive un file nel
   percorso che confermate.

Non c'è auto-eliminazione, né mutazione in background, né rinomina, né
`chmod`, né `truncate`, né `kill` / `TerminateProcess`, e nessuna attività
di rete di alcun tipo. Se qualcosa cambia nel vostro filesystem mentre
xdu è in esecuzione, è perché avete premuto un tasto e confermato.
