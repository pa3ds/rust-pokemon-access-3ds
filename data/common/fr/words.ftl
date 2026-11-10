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
way-up = haut
way-down = bas
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
category-arrows = flèches
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
key-nothing-said = rien n'a encore été dit
key-one-way = un seul chemin
out-at = sortie à { $bearing }
past-teleporter = après un téléporteur
leads-to = { $thing } vers { $place }
sign-saying = panneau { $text }
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
how-rail = rail
how-hop = sauter
how-bunny-hop = bunny hop
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
how-arrow-pushes = la flèche pousse
p-arrow = la flèche pousse { $bearing }
p-switch-first = d'abord { $thing }, { $way }
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
p-requires = nécessite { $move }
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
touch-no-drag = rien à faire glisser
touch-continue = continuer
touch-off = le tactile est désactivé
touch-not-listening = Azahar ne reçoit pas les touches de pa3ds
touch-holding = maintenu sur { $target }
touch-dropped = relâché sur { $target }

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
training-barrier = barrière
training-shot-coming = tir en approche
training-left = encore { $time }
training-empty-bag = sac vide
training-bag-finished = { $pokemon } a terminé sa Préparation
training-resting = { $pokemon } se repose
training-total = total
training-total-points = total { $points } sur { $full }
training-charged = chargé
training-fizzled = éteint
training-blocked = bloqué
training-at-start = au départ
training-block-now = maintenant
training-energy = énergie
training-no-numbers = pas encore de valeurs
training-medals = { $count ->
    [one] 1 médaille
   *[other] { $count } médailles
}
amie-beckon = appeler { $pokemon }
amie-visitor = { $pokemon } en visite
amie-stylus = stylet
amie-stylus-on = stylet, { $part }
amie-stylus-edge = bord
amie-view-down = faire descendre { $pokemon }
amie-view-up = renvoyer { $pokemon } en haut
amie-view-came-down = { $pokemon } arrive en bas
amie-view-went-up = { $pokemon } repart en haut
amie-mood-coming = { $pokemon } vient vers vous
amie-mood-back-turned = { $pokemon } vous tourne le dos
amie-mood-turned-round = { $pokemon } se retourne
amie-mood-dozing = { $pokemon } s'assoupit
amie-mood-asleep = { $pokemon } dort
amie-mood-awake = { $pokemon } se réveille
amie-mood-too-full = { $pokemon } n'a plus faim
amie-mood-puff-seen = { $pokemon } lorgne la { $puff }
amie-mood-reaching = { $pokemon } tend la patte vers la { $puff }
amie-mood-eating = { $pokemon } mange
amie-mood-eaten = { $pokemon } a fini la { $puff }
amie-mood-puff-fell = la { $puff } est tombée
amie-mood-picky = { $pokemon } fronce les sourcils et se détourne
amie-mood-watching = { $pokemon } regarde votre main
amie-mood-looking-around = { $pokemon } regarde partout
amie-mood-hand-raised = { $pokemon } lève la main
amie-mood-your-face = { $pokemon } se penche vers votre visage
amie-high-five = Tope là !
amie-mood-disliked = { $pokemon } n'a pas aimé ça...
amie-mood-closer = { $pokemon } déborde de joie, et des cœurs jaillissent
amie-mood-best-friends = { $pokemon } exulte sous une pluie de cœurs
amie-meter = { $label } { $count } sur 5
amie-hearts = { $count ->
    [one] 1 cœur
   *[other] { $count } cœurs
}
amie-part-off = à côté de { $pokemon }
amie-part-head = tête
amie-part-body = corps
amie-part-right-eye = œil droit
amie-part-left-eye = œil gauche
amie-part-eye = œil
amie-part-right-ear = oreille droite
amie-part-left-ear = oreille gauche
amie-part-hand = main
amie-part-spikes = piquants
amie-part-ghostly = spectral
amie-part-flame = flamme
amie-part-sparks = étincelles
amie-part-slime = bave
amie-part-sludge = boue
amie-part-ice = glace
amie-part = partie
puzzle-cell = { $column }{ $row }
puzzle-home = { $tile }, à sa place
puzzle-goes = { $tile }, va en { $home }
puzzle-picture-left = { $pokemon } à gauche
puzzle-picture-front = { $pokemon } devant
puzzle-picture-right = { $pokemon } à droite
puzzle-picked = { $tile }, choisie
berry-picker-wants = { $pokemon } veut { $berry }
berry-picker-for = { $berry } pour { $pokemon }
berry-picker-wanted = { $berry } demandée
berry-picker-lineup = { $first }, { $second } et { $third }
berry-picker-not-up = { $pokemon } n'est pas là
berry-picker-no-berry = pas de baie { $number }
berry-picker-no-pokemon = pas de Pokémon { $number }
head-it-ball = { $pokemon }
head-it-ball-nobody = pelote de laine
head-it-no-pokemon = pas de Pokémon { $number }
head-it-no-score = pas de score
head-it-lineup = { $first }, { $second } et { $third }
head-it-estimate = pa3ds estime votre avance à { $seconds } secondes, sur { $taps } appuis

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
window-amie = &Poké Récré...
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
setting-pa3ds-ui-trainer_party = Combien de Pokémon a un Dresseur
setting-pa3ds-ui-trainer_party-about = Quand un combat contre un Dresseur commence, après la ligne où il vous défie, dire combien de Pokémon il a (« text »), jouer un bip pour chacun (« beeps ») ou ni l'un ni l'autre (« off »). Les bips sont aussi forts que les tonalités de [pa3ds.audio].
setting-pa3ds-ui-trainer_party-text = Le dire
setting-pa3ds-ui-trainer_party-beeps = Un bip pour chacun
setting-pa3ds-ui-trainer_party-off = Ni l'un ni l'autre
setting-pa3ds-ui-language = Langue
setting-pa3ds-ui-language-about = La langue dans laquelle pa3ds dit ses propres mots, sous la forme du nom d'un dossier de ses données : en, fr, de, es, it, ja, ko, zh-CN ou zh-TW. Vide suit la langue du jeu, que pa3ds lit dans la sauvegarde ; réglez-la quand celle-ci est fausse, comme pour une traduction de fans par-dessus une autre langue, et dans Soleil et Lune, dont pa3ds ne peut pas lire la sauvegarde, où elle sert aussi de langue du jeu.
setting-pa3ds-audio-tones = Volume relatif des tonalités
setting-pa3ds-audio-tones-about = Le volume de la courte tonalité quand le curseur d'une carte ou du mode tactile bouge, plus aiguë plus il est haut et plus à droite plus il est à droite, pour dire où est le curseur : 1 aussi fort que le jeu, 0.5 moitié moins fort, 1.5 plus fort, 0 aucune ; dans la boîte de dialogue des paramètres, en pourcentage, 100 aussi fort que le jeu.
setting-pa3ds-audio-footsteps = Volume relatif des pas
setting-pa3ds-audio-footsteps-about = Le volume des bruits de pas de pa3ds sur les sols où le jeu n'en joue pas, comme les rues, les planchers et les grottes : 1 aussi fort que le jeu les jouerait, 0.5 moitié moins fort, 1.5 plus fort, 0 aucun ; dans la boîte de dialogue des paramètres, en pourcentage, 100 aussi fort que le jeu.
setting-pa3ds-audio-ticks = Volume relatif du tic-tac de l'horloge
setting-pa3ds-audio-ticks-about = Le volume d'une courte tonalité à chaque seconde de l'horloge d'un jeu, comme dans les jeux de Poké Récré ; l'horloge elle-même est annoncée toutes les demi-minutes et dans ses dernières secondes : 1 aussi fort que le jeu, 0.5 moitié moins fort, 1.5 plus fort, 0 aucune, la valeur par défaut ; dans la boîte de dialogue des paramètres, en pourcentage, 100 aussi fort que le jeu.
setting-pa3ds-audio-feedback_sounds = Volume relatif des sons de retour
setting-pa3ds-audio-feedback_sounds-about = Le volume des courts sons du jeu pour ce qu'une touche a fait : les sons des Boîtes du PC quand on prend et pose un Pokémon, lorsque le mode tactile prend un élément à faire glisser et le relâche, et le son d'erreur du jeu pour une touche qui n'a rien à faire, comme pour une baie qu'une manche de Cueille-Baies n'a pas ; et le signal de Jongle-Tête. À 0, pa3ds les annonce à la place et ne joue pas le signal, « maintenu sur », « relâché sur » et pourquoi : 1 aussi fort que le jeu, 0.5 moitié moins fort, 1.5 plus fort ; dans la boîte de dialogue des paramètres, en pourcentage, 100 aussi fort que le jeu.
setting-pa3ds-navigation-positions =Annoncer la position dans la liste
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
setting-pa3ds-touch-wrap = Boucler
setting-pa3ds-touch-wrap-about = En mode tactile, passer du dernier bouton au premier, et de la rangée du bas à celle du haut ; false s'arrête aux extrémités, pour qu'appuyer quelques fois sur O finisse toujours sur la rangée du bas.
setting-pa3ds-touch-next_row = Passer à la rangée suivante
setting-pa3ds-touch-next_row-about = En mode tactile, J et L passent à la rangée du dessus ou du dessous au bout d'une rangée ; false les garde dans leur rangée, pour qu'appuyer quelques fois sur L finisse toujours à son extrémité droite.
setting-pa3ds-touch-drag = Comment faire glisser
setting-pa3ds-touch-drag-about = Faire glisser en maintenant Maj, ou une touche de votre choix assignée au glissement dans keys.toml, et relâcher le toucher quand la touche est relâchée (« hold »), ou appuyer une fois pour prendre et une autre fois pour relâcher (« toggle »). Seul Poké Récré fait glisser.
setting-pa3ds-touch-drag-hold = Maintenir Maj
setting-pa3ds-touch-drag-toggle = Appuyer pour prendre, appuyer pour relâcher
setting-pa3ds-amie-descriptions = Décrire ce que fait le Pokémon
setting-pa3ds-amie-descriptions-about = Dans Poké Récré, dire ce que votre Pokémon est vu faire quand il commence et s'arrête : regarder partout, regarder votre main, manger, lever la main pour taper dedans. Chacune de ses animations est décrite pendant qu'elle se joue, par une phrase propre à ce Pokémon. Dit aussi les cœurs qu'apporte une caresse, et quand vous tapez bien dans sa main.
setting-pa3ds-amie-fidgets = Décrire les mouvements spontanés du Pokémon
setting-pa3ds-amie-fidgets-about = Avec les descriptions de Poké Récré activées, dit aussi les petits mouvements que votre Pokémon fait tout seul toutes les quelques secondes. Désactivé, seule sa réaction à vous est dite.
setting-pa3ds-amie-berry_first = Cueille-Baies annonce la baie d'abord
setting-pa3ds-amie-berry_first-about = Annoncer chaque Pokémon qui veut une baie dans l'ordre où vous la donnez, la baie puis le Pokémon, « Baie Nanana pour Pikachu » ; false annonce le Pokémon d'abord, « Pikachu veut Baie Nanana ».
setting-pa3ds-amie-one_at_a_time = Cueille-Baies annonce un Pokémon à la fois
setting-pa3ds-amie-one_at_a_time-about = Annoncer un seul Pokémon qui veut une baie à la fois, et le suivant une fois que vous avez donné une baie ou que celui annoncé est parti ; false annonce chacun dès qu'il arrive.
setting-pa3ds-amie-head_it_cue = Jongle-Tête signale quand toucher
setting-pa3ds-amie-head_it_cue-about = Dans Jongle-Tête, jouer le son « ! » du jeu quand vient le moment de toucher une pelote, autant avant que la pelote atteigne son Pokémon que le dit l'avance ci-dessous, dans l'oreille du côté de ce Pokémon et dans les deux pour celui du milieu ; il faut les sons de retour activés.
setting-pa3ds-amie-head_it_lead = Avance du signal de Jongle-Tête
setting-pa3ds-amie-head_it_lead-about = Combien de secondes avant qu'une pelote atteigne son Pokémon le signal retentit : à peu près le temps qu'il vous faut entre l'entendre et appuyer sur la touche du Pokémon. À la fin de chaque manche, pa3ds annonce son estimation de la vôtre, à mettre ici. Augmentez-la si vous touchez trop tard, baissez-la si vous touchez trop tôt.
setting-pa3ds-amie-berry_picker_style = Comment Cueille-Baies distingue les Pokémon
setting-pa3ds-amie-berry_picker_style-about = Dans Cueille-Baies, annoncer un Pokémon par son nom (« name »), par son propre cri (« cry ») ou par les deux (« both »). Le cri vient de sa place dans la composition de la manche, où qu'il apparaisse : le premier dans l'oreille gauche, le deuxième dans les deux, le troisième dans la droite. Le cri demande les sons de retour activés ; là où il ne peut pas jouer, le nom est annoncé.
setting-pa3ds-amie-berry_picker_style-name = Nom
setting-pa3ds-amie-berry_picker_style-cry = Cri
setting-pa3ds-amie-berry_picker_style-both = Nom et cri
setting-pa3ds-amie-head_it_style = Comment Jongle-Tête distingue les Pokémon
setting-pa3ds-amie-head_it_style-about = Dans Jongle-Tête, annoncer le Pokémon sur lequel tombe une pelote par son nom (« name »), par son propre cri (« cry »), par les deux (« both ») ou par aucun (« none »), pour le signal seul. Le cri vient de sa place dans la composition de la manche : le premier dans l'oreille gauche, le deuxième dans les deux, le troisième dans la droite. Le cri demande les sons de retour activés ; là où il ne peut pas jouer, le nom est annoncé.
setting-pa3ds-amie-head_it_style-name = Nom
setting-pa3ds-amie-head_it_style-cry = Cri
setting-pa3ds-amie-head_it_style-both = Nom et cri
setting-pa3ds-amie-head_it_style-none = Aucun
setting-pa3ds-amie-head_it_score = Jongle-Tête annonce le score quand il change
setting-pa3ds-amie-head_it_score-about = Dans Jongle-Tête, annoncer le score, le combo et chaque raté à mesure qu'ils arrivent, le score et le combo une fois tant qu'ils continuent de monter ; false les laisse à la touche de score de Jongle-Tête (R), qui les annonce avec le temps ou les chances restantes chaque fois que vous appuyez dessus.
setting-pa3ds-extras-conditions = Conditions de concours en chiffres
setting-pa3ds-extras-conditions-about = Dire les conditions de concours d'un Pokémon en chiffres là où le jeu les dessine sous forme d'étoile, dans le résumé et dans la Boîte Pokéblocs : l'étoile est visible de tous mais pa3ds ne peut pas la lire, donc ses chiffres sont tout ce qu'il peut donner, et ils sont plus précis que l'étoile.
setting-pa3ds-training-shot_cue = Quand un Ballon-Bot tire
setting-pa3ds-training-shot_cue-about = Dans un Super Entraînement, comment vous entendez qu'un Ballon-Bot tire : l'alerte du jeu, les mots « tir en approche » ou rien. Sans sons de retour, le son est dit en mots.
setting-pa3ds-training-shot_cue-sound = Son
setting-pa3ds-training-shot_cue-speech = Voix
setting-pa3ds-training-shot_cue-off = Non
setting-pa3ds-training-block_cue = Quand bloquer ou esquiver
setting-pa3ds-training-block_cue-about = Dans un Super Entraînement, comment vous entendez qu'un tir va arriver : le « ! » du jeu, le mot « maintenant » ou rien. Sans sons de retour, le son est dit en mots.
setting-pa3ds-training-block_cue-sound = Son
setting-pa3ds-training-block_cue-speech = Voix
setting-pa3ds-training-block_cue-off = Non
setting-pa3ds-training-block_lead = Avance du signal de blocage
setting-pa3ds-training-block_lead-about = Combien de secondes avant l'arrivée d'un tir le signal retentit. Une barrière dure moins d'une seconde après avoir appuyé sur L.
setting-pa3ds-extras-base_stats = Stats du Super Entraînement en chiffres
setting-pa3ds-extras-base_stats-about = R dit sur la page du Super Entraînement les points de chaque stat, que le jeu dessine en graphique, plutôt qu'une part approximative.
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
settings-page-touch = Mode tactile
settings-page-amie = Poké Récré
settings-page-training = Super Entraînement
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
kind-switch = interrupteur
kind-pink-switch = interrupteur rose
kind-blue-switch = interrupteur bleu
kind-yellow-switch = interrupteur jaune
kind-green-switch = interrupteur vert
kind-warp = transfert
kind-path = chemin
kind-edge = chemin
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
kind-arrow-east = flèche vers l'est
kind-arrow-west = flèche vers l'ouest
kind-arrow-north = flèche vers le nord
kind-arrow-south = flèche vers le sud
kind-flat-ground = sol plat
kind-open-ground = terrain dégagé
kind-quiz-panel = dalle du quiz
kind-blue-spot = point bleu
kind-wall-ledge = rebord du mur
kind-vine-up = liane vers le haut
kind-vine-down = liane vers le bas
kind-swinging-vine = liane à balancer
kind-gym-leader-spot = place du champion d'arène
kind-machine = machine
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
battle-party = { $count } Pokémon

## Scenes

narration = Narration
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = Français

registered-slot = { $way } { $item }
registered-up = haut
registered-right = droite
registered-down = bas
registered-left = gauche

bag-moving = déplacement de { $item }
bag-between = entre { $above } et { $below }
bag-above = au-dessus de { $below }
bag-below = en dessous de { $above }
