[EN](user-manual-en.md) · [UA](user-manual-ua.md) · [ES](user-manual-es.md) · **FR** · [IT](user-manual-it.md) · [PT](user-manual-pt.md)

> **Note.** Ce document est une traduction automatique pour la version v0.1.0. En attente de relecture humaine. Si vous repérez une erreur de traduction, veuillez ouvrir un ticket sur <https://github.com/ratushnyi-labs/xdu>.

# xdu — manuel utilisateur

Ceci est le manuel utilisateur v0.1.0 de `xdu`, un explorateur d'utilisation
disque en lecture seule avec une interface terminal. Il est volontairement
court. Tout ce qui y est décrit fonctionne dans les binaires publiés ; rien
n'est anticipé.

## Ce que fait xdu

`xdu` scanne une arborescence de répertoires et présente le résultat sous
forme d'interface terminal navigable. Chaque ligne affiche un nom, une barre
de taille relative, la taille absolue de cette entrée (incluant tous les
descendants pour les répertoires), ainsi que la date du dernier accès. Le
bord droit du pied de page affiche l'espace libre sur le volume contenant la
racine du scan. Vous vous déplacez avec les touches fléchées ou les touches
vim, vous entrez dans un répertoire avec Enter et vous remontez avec
Backspace. Il existe une action d'export (`e`) qui écrit le sous-arbre
focalisé dans un fichier JSON, une superposition d'erreurs (`!`) pour les
échecs de scan par chemin, et un panneau d'aide intégré (`?` ou F1).

## Ce que xdu NE fait PAS

xdu est en lecture seule par défaut. Il ne supprime pas, ne déplace pas, ne
renomme pas, ne fait pas de `chmod`, ne tronque pas, ne tue pas de processus
et ne modifie en aucune manière vos fichiers, sauf si vous appuyez sur une
touche spécifique et confirmez. Ce n'est pas un service ni un démon. Il ne
s'exécute que pendant que vous interagissez avec lui. Il ne contacte pas de
serveur distant, ne collecte aucune télémétrie, n'envoie pas de données
analytiques et n'ouvre aucune connexion réseau. Il n'a pas de mode
auto-suppression, pas de nettoyeur en arrière-plan, pas de moteur de
suggestions « intelligent » et pas de mode graphique en v0.1.0.

## Installation

Les binaires précompilés de v0.1.0 sont publiés sur la page Releases de
GitHub : <https://github.com/ratushnyi-labs/xdu/releases>. Téléchargez
l'archive correspondant à votre plateforme, extrayez-la et placez le binaire
`xdu` (ou `xdu.exe` sous Windows) dans un répertoire de votre `PATH`.

**Linux (musl).** Deux builds statiques sont publiés :
`x86_64-unknown-linux-musl` et `aarch64-unknown-linux-musl`. Les deux sont
entièrement statiques — pas de glibc, aucune dépendance à des bibliothèques
partagées — et s'exécutent sur n'importe quel noyau Linux 3.2 ou supérieur.
Choisissez celui qui correspond à `uname -m` (`x86_64` ou `aarch64`).

**macOS.** Deux builds sont publiés : `x86_64-apple-darwin` (Intel) et
`aarch64-apple-darwin` (Apple Silicon). Après téléchargement, il peut être
nécessaire de retirer l'attribut de quarantaine que macOS appose aux
binaires téléchargés : `xattr -d com.apple.quarantine ./xdu`.

**Windows (MSVC).** Deux builds console PE32+ sont publiés :
`x86_64-pc-windows-msvc` et `aarch64-pc-windows-msvc`. Ce sont des fichiers
`.exe` autonomes — pas d'installateur, pas de dépendance à des DLL au-delà
du runtime MSVC livré avec Windows. Lancez-les depuis `cmd.exe`, PowerShell
ou Windows Terminal.

## Premier lancement

Lancer `xdu` sans argument scanne le répertoire de travail courant sous
Unix. Sous Windows, le lancement sans argument affiche une liste numérotée
des lecteurs montés (lecteur système, supports amovibles, partages réseau
dont la taille totale est connue) ; tapez le numéro et appuyez sur Enter,
ou appuyez simplement sur Enter pour choisir la première entrée. Passez un
chemin explicite (`xdu /home/me`, `xdu C:\Users`) pour court-circuiter le
sélecteur. Pendant que le scan initial s'exécute, une ligne de progression
s'affiche ; dès que le scan se termine, le TUI prend le relais et vous
pouvez naviguer immédiatement.

## Navigation

Déplacez la sélection avec les flèches haut/bas ou `k` / `j`. Entrez dans
le répertoire sélectionné avec la flèche droite, `l` ou Enter. Remontez au
parent avec la flèche gauche, `h` ou Backspace ; alternativement,
sélectionnez la ligne `..` en haut de la liste et appuyez sur Enter.
Remonter au-delà de la racine initiale du scan déclenche un nouveau scan au
niveau du système de fichiers du nouveau parent. Appuyez sur `q` ou Ctrl-C
pour quitter. Appuyez sur `?` ou F1 à tout moment pour ouvrir un panneau
récapitulant tous les raccourcis clavier.

## Tri et unités

Appuyez sur `s` pour faire défiler cycliquement la clé de tri. L'ordre du
cycle est : taille décroissante (par défaut — les plus grands en premier),
taille croissante, nom croissant, nom décroissant, nombre de fichiers
décroissant, nombre de fichiers croissant. Appuyez à nouveau sur `s` pour
continuer ; la clé de tri active est indiquée dans le pied de page.

Appuyez sur `u` pour basculer l'unité de taille entre les unités binaires
IEC (`KiB`, `MiB`, `GiB`, `TiB` — puissances de 1024) et les unités
décimales SI (`KB`, `MB`, `GB`, `TB` — puissances de 1000). Vous pouvez
aussi démarrer en mode SI en passant `--si` en ligne de commande. Le mode
s'applique de manière cohérente à toutes les tailles à l'écran — la ligne
sélectionnée, le pied de page avec l'espace libre, et tout export écrit
pendant que ce mode est actif.

## Suppression et corbeille

xdu est en lecture seule par défaut. Pour supprimer l'entrée sélectionnée,
appuyez sur `d`. Une fenêtre modale de confirmation apparaît avec `[ No ]`
focalisé ; appuyer sur Enter ou Esc annule. Pour continuer, naviguez
jusqu'à `[ Yes ]` (Tab ou flèche droite) et appuyez sur Enter. Ce
comportement avec `[ No ]` par défaut est un choix de sécurité délibéré et
il est identique dans tous les modes.

Exécutez `xdu --trash` pour passer en mode corbeille. En mode corbeille, la
modale de confirmation affiche « Move to trash » au lieu de « Delete », et
l'action cible la corbeille native du système d'exploitation :

- **Linux :** la spécification freedesktop.org Trash v1.0. Les fichiers
  sont déplacés dans `$XDG_DATA_HOME/Trash/files/` (avec repli sur
  `$HOME/.local/share/Trash/files/`), accompagnés d'un fichier de
  métadonnées `.trashinfo`. Les actions standard « Restaurer depuis la
  corbeille » des gestionnaires de fichiers fonctionnent ensuite.
- **Windows :** la Corbeille système, via le même chemin kernel
  qu'utilise Explorer.
- **macOS :** non pris en charge en v0.1.0. Lancer `xdu --trash` sous
  macOS se termine avec un code non nul et un message clair. La
  suppression ordinaire avec la touche `d` (sans corbeille) continue de
  fonctionner normalement sur macOS.

Le mode corbeille reste protégé par la modale de confirmation avec `[ No ]`
par défaut — il n'existe nulle part dans xdu de chemin « supprimer sans
demander ».

## Détenteur de verrou et fermeture forcée

Lorsqu'une suppression sous Windows échoue parce qu'un autre processus a le
fichier ouvert, xdu énumère les détenteurs via Windows Restart Manager et
affiche une modale informative listant les noms de processus et les PID.
Depuis cette modale, vous pouvez appuyer sur `f` pour tenter une fermeture
forcée (ceci appelle `DuplicateHandle` avec `DUPLICATE_CLOSE_SOURCE` sur
chaque handle étranger) ; la fermeture forcée elle-même est protégée par
une modale de confirmation séparée avec `[ No ]` par défaut. C'est
agressif : ne l'utilisez que lorsque vous savez ce qu'est le détenteur du
verrou.

Sous Linux, avant qu'une suppression ne soit tentée, xdu parcourt de
manière opportuniste `/proc/*/fd/` et liste tout processus avec un
descripteur de fichier ouvert pointant vers le chemin sélectionné. La liste
est uniquement informative — l'`unlink` POSIX de Linux réussit même
lorsqu'il y a des détenteurs ouverts, la suppression se poursuit donc après
confirmation. L'intention est de vous laisser décider s'il faut supprimer
un fichier actuellement utilisé.

## Superposition d'erreurs

Un scan sur un vrai système de fichiers rencontre occasionnellement une
erreur sur un chemin : un répertoire en accès refusé, un handle NFS
obsolète, un lien symbolique cassé. xdu enregistre chacune de ces erreurs
sans arrêter le scan ; appuyez sur `!` pour ouvrir la superposition
d'erreurs et les lire. La superposition se fait défiler avec les flèches
et se ferme avec Esc.

## Export

Appuyez sur `e` pour exporter le sous-arbre actuellement focalisé vers un
fichier JSON. xdu demande le chemin de sortie avec `xdu-export.json` dans
le répertoire courant comme valeur par défaut ; appuyez sur Enter pour
accepter la valeur par défaut ou saisissez un nouveau chemin. Le format du
fichier est documenté dans `docs/spec.md` (étiquette de schéma
`xdu-export-1`). L'export est purement une écriture du fichier choisi — il
ne touche pas au système de fichiers scanné.

## Note de sécurité

xdu est en lecture seule par défaut. Les seules écritures qu'il effectue
sont :

1. Une suppression explicite via la touche `d`, protégée par une modale de
   confirmation avec `[ No ]` par défaut.
2. Une suppression optionnelle en mode corbeille via `--trash` + `d`,
   toujours protégée par la modale avec `[ No ]` par défaut, ciblant la
   corbeille native du système d'exploitation là où elle est supportée.
3. Un export JSON via la touche `e`, qui écrit un fichier au chemin que
   vous confirmez.

Il n'y a pas d'auto-suppression, pas de mutation en arrière-plan, pas de
renommage, pas de `chmod`, pas de `truncate`, pas de `kill` /
`TerminateProcess`, et aucune activité réseau d'aucune sorte. Si quelque
chose change dans votre système de fichiers pendant que xdu est en cours
d'exécution, c'est parce que vous avez appuyé sur une touche et confirmé.
