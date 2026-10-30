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
above = { $count } plus haut
below = { $count } plus bas
rail-forward = { $steps } en avant
rail-back = { $steps } en arrière
rail-hold-right = maintenir droite
rail-hold-left = maintenir gauche
rail-angled = caméra en biais, maintenir gauche, maintenir droite
turn-right = tourner à droite
turn-left = tourner à gauche
turn-around = faire demi-tour

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
category-climbing = escalade
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
how-climb = grimper
p-climb-up = grimper { $holds } vers le haut
p-climb-down = grimper { $holds } vers le bas
how-slide = glisser
how-grind-the-rail = glisser sur le rail
p-grind = glisser sur le rail { $bearing }
how-ride-the-belt = suivre le tapis roulant
p-belt = suivre le tapis roulant { $bearing }
how-climb-the-vine-up = grimper à la liane
how-climb-the-vine-down = descendre la liane
how-swing-on-the-vine = se balancer à la liane
p-vine = face au { $way }, { $how }
p-more-vines = { $count ->
    [one] puis encore 1 liane
   *[other] puis encore { $count } lianes
}
p-more-teleporters = { $count ->
    [one] puis encore 1 téléporteur
   *[other] puis encore { $count } téléporteurs
}
how-up-the-ramp = monter la rampe
how-down-the-ramp = descendre la rampe
p-current = suivre le courant { $bearing }
p-to-push = pour pousser { $way } :
p-nudge = un peu { $way }
p-nudge-then = { $nudge }, puis
p-lined-up = aligné
p-then-cant-continue = puis impossible de continuer
p-cant-continue = impossible de continuer
p-in-the-way = { $thing } bloque le passage
p-surface = remonter et aller au point de plongée { $bearing }
p-into = { $leg } vers { $place }
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
window-title = Pokémon Access 3DS
window-menu-mod = &Mod
window-updates = Rechercher des &mises à jour
window-exit = &Quitter
window-menu-azahar = &Azahar
window-azahar-portable = Télécharger Azahar (&portable)
window-azahar-installer = Télécharger Azahar (programme d'&installation)
window-menu-help = A&ide
window-documentation = &Documentation...
window-keys = &Commandes clavier...
window-known-issues = Problèmes co&nnus...
window-translating = &Traduction...
window-changelog = &Journal des modifications...
window-settings-guide = &Guide des paramètres...
window-issues = &Signaler un problème sur GitHub
setting-pa3ds-keys = Réagir aux touches du mod
setting-pa3ds-keys-about = Réagir aux touches de pa3ds et assigner les boutons d'Azahar, comme le dit keys.toml.
setting-pa3ds-poll_rate = Lectures par seconde
setting-pa3ds-poll_rate-about = Combien de fois par seconde lire le jeu : 60, la fréquence d'images du jeu ; 30 pour une machine plus lente.
setting-pa3ds-azahar_folder = Dossier d'Azahar portable
setting-pa3ds-azahar_folder-about = Le dossier où se trouve un Azahar portable, celui qui contient azahar.exe et un dossier user ; vide pour un Azahar installé de la façon habituelle, dont les réglages sont dans AppData. pa3ds le remplit quand vous choisissez le dossier qu'il demande.
setting-pa3ds-hide_window = Masquer la fenêtre du mod quand Azahar est ouvert
setting-pa3ds-hide_window-about = Masquer la fenêtre de pa3ds, celle qu'ouvre un double-clic sur pa3ds, quand Azahar est ouvert, pour que rien de pa3ds ne se place entre vous et le jeu dans Alt+Tab ; false la laisse visible, pour qu'OBS enregistre les sons de pa3ds à travers elle.
setting-pa3ds-ui-pauses = Pauses dans le texte à l'écran
setting-pa3ds-ui-pauses-about = Mettre des pauses entre les éléments de texte à l'écran dits ensemble : deux-points après une étiquette, comme dans « DO : Raiden », et une virgule entre les autres.
setting-pa3ds-ui-interrupt = Les menus interrompent la parole
setting-pa3ds-ui-interrupt-about = Couper la parole quand le curseur d'un menu bouge, au lieu de mettre en file d'attente ; les touches de pa3ds le font toujours.
setting-pa3ds-ui-unfocused = Parler quand Azahar est en arrière-plan
setting-pa3ds-ui-unfocused-about = Continuer à dire ce que montre le jeu quand une autre fenêtre est au premier plan ; sinon pa3ds se tait alors, sauf pour ses propres touches et messages.
setting-pa3ds-ui-descriptions = Descriptions des scènes
setting-pa3ds-ui-descriptions-about = Décrire les scènes que le jeu montre sans texte, comme le réveil de Groudon. Chaque description est dite après la ligne de texte que la scène suit. Seules certaines scènes sont décrites pour l'instant.
setting-pa3ds-ui-language = Langue
setting-pa3ds-ui-language-about = La langue dans laquelle pa3ds dit ses propres mots, sous la forme du nom d'un dossier de ses données : en, fr, de, es, it, ja, ko, zh-CN ou zh-TW. Vide suit la langue du jeu, que pa3ds lit dans la sauvegarde ; réglez-la quand celle-ci est fausse, comme pour une traduction de fans par-dessus une autre langue, et dans Soleil et Lune, dont pa3ds ne peut pas lire la sauvegarde, où elle sert aussi de langue du jeu.
setting-pa3ds-audio-tones = Volume relatif des tonalités
setting-pa3ds-audio-tones-about = Le volume de la courte tonalité quand le curseur d'une carte ou du mode tactile bouge, plus aiguë plus il est haut et plus à droite plus il est à droite, pour dire où est le curseur : 1 aussi fort que le jeu, 0.5 moitié moins fort, 1.5 plus fort, 0 aucune ; dans la boîte de dialogue des paramètres, en pourcentage, 100 aussi fort que le jeu.
setting-pa3ds-audio-footsteps = Volume relatif des pas
setting-pa3ds-audio-footsteps-about = Le volume des bruits de pas de pa3ds sur les sols où le jeu n'en joue pas, comme les rues, les planchers et les grottes : 1 aussi fort que le jeu les jouerait, 0.5 moitié moins fort, 1.5 plus fort, 0 aucun ; dans la boîte de dialogue des paramètres, en pourcentage, 100 aussi fort que le jeu.
setting-pa3ds-navigation-positions = Annoncer la position dans la liste
setting-pa3ds-navigation-positions-about = Dire où se trouve un élément dans la liste, comme dans « porte, 1 sur 3 ».
setting-pa3ds-navigation-position_first = Position dans la liste avant les directions
setting-pa3ds-navigation-position_first-about = Dire la place dans la liste avant le chemin pour y aller, « porte, 1 sur 3, 1 nord » ; false dit « porte, 1 nord, 1 sur 3 ».
setting-pa3ds-navigation-turn_by_turn = Guidage virage par virage
setting-pa3ds-navigation-turn_by_turn-about = Une fois que P a trouvé le chemin, annoncer chaque virage quand vous l'atteignez ; false dit le chemin une seule fois, et P le redit depuis l'endroit où vous êtes.
setting-pa3ds-navigation-guide_to_hidden = Guider vers les trouvailles du Navi-Dex
setting-pa3ds-navigation-guide_to_hidden-about = Quand une recherche du Navi-Dex trouve le Pokémon recherché, ou que le Navi-Dex en trouve un de lui-même pendant qu'il est ouvert et que vous êtes en mode tactile, trouver le chemin jusqu'à lui comme le fait P et garder P dessus, quelle que soit la liste choisie ; un Pokémon trouvé pendant que la Navi-Map est ouverte est ignoré.
setting-pa3ds-navigation-track_boulders = Suivre le déplacement des rochers
setting-pa3ds-navigation-track_boulders-about = Dire de combien un gros rocher s'est déplacé depuis sa position de départ dans la salle, comme dans « gros rocher, 2 est, déplacé de 1 nord », pour savoir lesquels vous avez poussés ; le jeu montre où est chaque rocher, jamais d'où il est parti.
setting-pa3ds-navigation-hide_door_signs = Masquer les panneaux de porte
setting-pa3ds-navigation-hide_door_signs-about = Retirer un panneau de la liste quand il ne dit que le nom d'une porte à côté de lui, comme celui d'un appartement de Lavandia qui annonce la maison de Voltère ; un panneau qui en dit plus, comme celui d'une Arène, reste.
setting-pa3ds-navigation-show_y = Annoncer la hauteur
setting-pa3ds-navigation-show_y-about = Dire aussi la hauteur, en y après x et z, quand C dit où vous êtes et Maj et C où se trouve l'élément choisi : « x 71, z 35, y 20 » ; dans les unités du jeu, 18 pour la largeur d'une case, et omise là où pa3ds ne la connaît pas, comme dans Soleil et Lune.
setting-pa3ds-navigation-sort_by_distance = Trier par distance
setting-pa3ds-navigation-sort_by_distance-about = Trier la liste du mode liste du plus proche au plus loin, ce qui change à mesure que vous marchez ; false garde chaque élément à la même place, dans l'ordre des tables du jeu, pour que vous sachiez à sa place dans quelle maison vous êtes déjà entré ; le mode direction trie toujours du plus proche au plus loin.
setting-pa3ds-extras-conditions = Conditions de concours en chiffres
setting-pa3ds-extras-conditions-about = Dire les conditions de concours d'un Pokémon en chiffres là où le jeu les dessine sous forme d'étoile, dans le résumé et dans la Boîte Pokéblocs : l'étoile est visible de tous mais pa3ds ne peut pas la lire, donc ses chiffres sont tout ce qu'il peut donner, et ils sont plus précis que l'étoile.
setting-pa3ds-extras-hidden_items = Objets cachés
setting-pa3ds-extras-hidden_items-about = Lister parmi les objets ceux qui sont cachés et restent à trouver, comme « objet caché » : sous la mer seulement ("underwater"), où le Cherch'Objet ne peut pas servir et où un joueur voyant peut apercevoir un reflet parmi les rochers et les algues, ou partout ("all"), ce qui révèle ce que le jeu cache à tout le monde ; "off" n'en liste aucun.
setting-pa3ds-extras-hidden_items-off = Désactivé
setting-pa3ds-extras-hidden_items-underwater = Sous la mer
setting-pa3ds-extras-hidden_items-all = Partout
settings-title = Paramètres du mod
settings-page-general = Général
settings-page-speech = Parole
settings-page-audio = Audio
settings-page-navigation = Navigation
settings-page-extras = Extras
settings-browse = &Parcourir...
settings-language-game = La langue du jeu
settings-restart = Redémarrer pa3ds maintenant pour appliquer ces paramètres ? Si vous choisissez Non, ils seront appliqués au prochain démarrage de pa3ds.
settings-unwritten = pa3ds n'a pas pu écrire { $path }, donc rien n'a changé.
dialog-apply = &Appliquer
window-settings = &Paramètres...

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
kind-way-down = descente
kind-conveyor-belt = tapis roulant
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
kind-hidden-item = objet caché
kind-barrier = barrière
kind-hole = trou
kind-dive-spot = point de plongée
kind-current = courant
kind-flat-ground = sol plat
kind-open-ground = terrain dégagé
kind-quiz-panel = dalle du quiz
kind-blue-spot = point bleu
kind-wall-ledge = rebord du mur
kind-vine-up = liane vers le haut
kind-vine-down = liane vers le bas
kind-swinging-vine = liane à balancer
kind-gym-leader-spot = place du champion d'arène
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
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = Français
