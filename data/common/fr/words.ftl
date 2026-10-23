### French machine translation of `../en/words.ftl` (2026-10-18), a baseline for a translator to
### correct. The English file says what each message is said of, and where; a message missing
### here is said in English.

## Ways and distances

way-north = nord
way-east = est
way-south = sud
way-west = ouest
way-north-east = nord-est
way-south-east = sud-est
way-south-west = sud-ouest
way-north-west = nord-ouest
way-ahead = devant
way-behind = derrière
way-left = gauche
way-right = droite
steps = { $count } { $way }
leg-with = { $how } { $steps } { $way }
here = ici
rail-forward = { $steps } en avant
rail-back = { $steps } en arrière
rail-hold-right = maintenir droite
rail-hold-left = maintenir gauche
rail-angled = caméra en biais, maintenir gauche, maintenir droite

## The list of what is on the map, and the navigation keys' answers

category-all = tout
category-exits = sorties
category-signs = objets interactifs
category-obstacles = obstacles
category-people = personnes
category-items = objets
category-berries = baies
category-ash = cendres
category-bike = vélo
category-riding = monture
category-currents = courants
category-pokemon = pokémon
key-category = { $category }, { $count }
position = { $number } sur { $count }
key-nothing-way = rien direction { $way }
key-none-way = pas de { $category } direction { $way }
key-nothing-here = rien ici
key-no-map = pas de carte
key-cant-walk-now = impossible de marcher maintenant
key-list-mode = mode liste
key-direction-mode = mode direction
key-touch-mode = mode tactile, { $first }
key-unreachable-hidden = inaccessibles masqués
key-unreachable-shown = inaccessibles affichés
key-path-off = chemin désactivé
key-no-path = pas de chemin
key-no-battle = pas de combat
key-one-way = un seul chemin
out-at = sortie à { $bearing }
leads-to = { $thing } vers { $place }
span = { $low } à { $high }
either = { $these } ou { $last }

## Big boulders

key-push = pousser { $way }
boulder-into-hole = { $way } dans un trou
boulder-moveable = déplaçable { $ways }
boulder-immoveable = immobile
boulder-moved = déplacé de { $bearing }

## Renaming

rename-title = Renommer
rename-label = { $thing }, { $bearing }. { $name }
rename-speaker-label = Le dernier interlocuteur. { $name }
rename-called = Nom : { $name }
rename-not-named = Pas encore nommé
rename-only-this-one = Celui-ci seulement
rename-now = { $thing } s'appelle maintenant { $name }
rename-no-name = { $thing } n'a pas de nom
rename-the-last-speaker = le dernier interlocuteur
rename-nothing = rien à renommer
rename-cannot = { $thing } ne peut pas être renommé
rename-already = renommage déjà en cours
rename-nobody-spoke = personne n'a parlé

## P: the way to the thing chosen, and guiding the player along it

how-water = eau
how-bush = buisson
how-boulder = rocher
how-rock = roche
how-waterfall = cascade
how-slope = pente
how-bumpy-slope = pente cahoteuse
how-push = pousser
how-ride-the-current = suivre le courant
how-get-off = descendre
how-smash = briser
p-current = suivre le courant { $bearing }
p-to-push = pour pousser { $way } :
p-nudge = un peu { $way }
p-nudge-then = { $nudge }, puis
p-lined-up = aligné
p-then-cant-continue = puis impossible de continuer
p-cant-continue = impossible de continuer
p-in-the-way = { $thing } bloque le passage
p-surface = remonter et aller au point de plongée { $bearing }
p-and-ice-cleared = et la glace est brisée
p-ice-cleared = glace brisée
p-no-path-to = pas de chemin vers { $thing }
p-no-route-in-game = pas encore d'itinéraire dans ce jeu
p-no-route-on-streets = pas d'itinéraire dans ces rues
p-gone = { $thing } a disparu

## Touch mode

touch-nothing = rien à toucher
touch-off = le tactile est désactivé
touch-not-listening = Azahar ne reçoit pas les touches de pa3ds

## What the field shows without words

dowsing = Cherch'Objet { $light }
dowsing-blue = bleu
dowsing-green = vert
dowsing-yellow = jaune
dowsing-red = rouge
dowsing-nothing = rien
decorating-placeable = plaçable
decorating-not-placeable = non plaçable

## The Pokédex's habitat map

habitat-found = Présent : { $place }
habitat-grass = herbes hautes
habitat-water = eau
habitat-fishing = pêche

## Super Training and Pokémon-Amie

training-goal = but
training-red-goal = but rouge
training-shot-coming = tir en approche
training-left = encore { $time }
amie-stroke = caresser { $pokemon }

## What pa3ds says of itself

app-waiting = En attente du jeu
app-ready = pa3ds prêt
app-restarted = pa3ds redémarré
app-reader-error = Erreur de lecture
dialog-ok = OK
dialog-cancel = Annuler

## As pa3ds starts: its keys, its settings and Azahar's

both = { $these } et { $last }
keys-taken = pa3ds utilise { $controls }
keys-applied = touches appliquées à Azahar
keys-waiting = fermez Azahar et relancez pa3ds pour appliquer vos touches
keys-commented-out = nouvelle touche mise en commentaire dans { $file } : { $key } pour { $action }, car { $key } sert déjà à { $taken }
keys-edit-to-choose = modifiez { $file } pour choisir
file-unreadable = { $file } n'a pas pu être lu : { $problem }. Corrigez-le, ou supprimez-le pour que pa3ds en écrive un nouveau
azahar-settings-applied = réglages d'Azahar appliqués
azahar-settings-waiting = fermez Azahar et relancez pa3ds pour appliquer ses réglages
azahar-settings-missing = réglages d'Azahar introuvables : lancez Azahar une fois, fermez-le, puis relancez pa3ds
azahar-folder-ask = Réglages d'Azahar introuvables. Choisissez le dossier qui contient azahar.exe, ou annulez si Azahar est installé
azahar-folder-again = { $problem }, choisissez un autre dossier ou annulez
azahar-folder-no-exe = { $folder } ne contient pas azahar.exe
azahar-folder-not-portable = { $folder } n'a pas de dossier user à côté de son azahar.exe, donc cet Azahar garde ses réglages dans AppData et non à côté

## What a thing on the map is, where neither the game nor a name says

kind-door = porte
kind-way-out = sortie
kind-warp = transfert
kind-path = chemin
kind-edge = bord
kind-stairs = escalier
kind-stairs-up = escalier montant
kind-stairs-down = escalier descendant
kind-passage = passage
kind-entrance = entrée
kind-teleporter = téléporteur
kind-elevator = ascenseur
kind-sign = panneau
kind-flag = drapeau
kind-object = objet
kind-person = personne
kind-item-ball = objet au sol
kind-barrier = barrière
kind-hole = trou
kind-dive-spot = point de plongée
kind-current = courant
kind-flat-ground = sol plat
kind-slope = pente
kind-bumpy-slope = pente cahoteuse
kind-rail = rail
kind-ice = glace
kind-blank = vide
kind-wall = mur
kind-crumbled-cliff = falaise effondrée

## Battle

battle-battler = { $name } N{ $level } { $hp }
battle-share = { $percent } %

## Scenes

narration = Narration
