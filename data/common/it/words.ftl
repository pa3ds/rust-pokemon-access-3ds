### Italian machine translation of `../en/words.ftl` (2026-10-18), a baseline for a translator to
### correct. The English file says what each message is said of, and where; a message missing
### here is said in English.

## Ways and distances

way-north = nord
way-east = est
way-south = sud
way-west = ovest
way-north-east = nord-est
way-south-east = sud-est
way-south-west = sud-ovest
way-north-west = nord-ovest
way-ahead = avanti
way-behind = dietro
way-left = sinistra
way-right = destra
steps = { $count } { $way }
leg-with = { $how } { $steps } { $way }
here = qui
rail-forward = { $steps } avanti
rail-back = { $steps } indietro
rail-hold-right = tieni destra
rail-hold-left = tieni sinistra
rail-angled = telecamera inclinata, tieni sinistra, tieni destra
turn-right = gira a destra
turn-left = gira a sinistra
turn-around = girati

## The list of what is on the map, and the navigation keys' answers

category-all = tutto
category-exits = uscite
category-signs = oggetti interattivi
category-obstacles = ostacoli
category-people = persone
category-items = strumenti
category-berries = bacche
category-ash = cenere
category-bike = bici
category-riding = cavalcatura
category-currents = correnti
category-pokemon = pokémon
key-category = { $category }, { $count }
position = { $number } di { $count }
key-nothing-way = niente verso { $way }
key-none-way = niente { $category } verso { $way }
key-nothing-here = niente qui
key-no-map = nessuna mappa
key-cant-walk-now = impossibile camminare ora
key-list-mode = modalità elenco
key-direction-mode = modalità direzione
key-touch-mode = modalità touch, { $first }
key-unreachable-hidden = irraggiungibili nascosti
key-unreachable-shown = irraggiungibili mostrati
key-path-off = percorso disattivato
key-no-path = nessun percorso
key-no-battle = nessuna lotta
key-one-way = una sola strada
out-at = uscita a { $bearing }
leads-to = { $thing } verso { $place }
span = da { $low } a { $high }
either = { $these } o { $last }

## Big boulders

key-push = spingi verso { $way }
boulder-into-hole = verso { $way } in una buca
boulder-moveable = spostabile verso { $ways }
boulder-immoveable = inamovibile
boulder-moved = spostato di { $bearing }

## Renaming

rename-title = Rinomina
rename-label = { $thing }, { $bearing }. { $name }
rename-speaker-label = L'ultimo a parlare. { $name }
rename-called = Nome: { $name }
rename-not-named = Non ancora nominato
rename-only-this-one = Solo questo
rename-now = { $thing } ora si chiama { $name }
rename-no-name = { $thing } non ha nome
rename-the-last-speaker = l'ultimo a parlare
rename-nothing = niente da rinominare
rename-cannot = { $thing } non può essere rinominato
rename-already = rinomina già in corso
rename-nobody-spoke = nessuno ha parlato

## P: the way to the thing chosen, and guiding the player along it

how-water = acqua
how-bush = cespuglio
how-boulder = masso
how-rock = roccia
how-waterfall = cascata
how-slope = pendio
how-bumpy-slope = pendio accidentato
how-push = spingi
how-ride-the-current = segui la corrente
how-get-off = scendi
how-smash = spacca
p-current = segui la corrente { $bearing }
p-to-push = per spingere verso { $way }:
p-nudge = un po' verso { $way }
p-nudge-then = { $nudge }, poi
p-lined-up = allineato
p-then-cant-continue = poi impossibile proseguire
p-cant-continue = impossibile proseguire
p-in-the-way = { $thing } blocca la strada
p-surface = riemergi e vai al punto di immersione { $bearing }
p-and-ice-cleared = e il ghiaccio è rotto
p-ice-cleared = ghiaccio rotto
p-no-path-to = nessun percorso verso { $thing }
p-no-route-in-game = ancora nessun percorso in questo gioco
p-no-route-on-streets = nessun percorso in queste strade
p-gone = { $thing } non c'è più

## Touch mode

touch-nothing = niente da toccare
touch-off = il touch è disattivato
touch-not-listening = Azahar non riceve tocchi da pa3ds

## What the field shows without words

dowsing = rilevatore { $light }
dowsing-blue = blu
dowsing-green = verde
dowsing-yellow = giallo
dowsing-red = rosso
dowsing-nothing = niente
decorating-placeable = posizionabile
decorating-not-placeable = non posizionabile

## The Pokédex's habitat map

habitat-found = Si trova in { $place }
habitat-grass = erba alta
habitat-water = acqua
habitat-fishing = pesca

## Super Training and Pokémon-Amie

training-goal = porta
training-red-goal = porta rossa
training-shot-coming = tiro in arrivo
training-left = mancano { $time }
amie-stroke = accarezza { $pokemon }

## What pa3ds says of itself

app-waiting = In attesa del gioco
app-ready = pa3ds pronto
app-restarted = pa3ds riavviato
app-reader-error = Errore di lettura
dialog-ok = OK
dialog-cancel = Annulla

## As pa3ds starts: its keys, its settings and Azahar's

both = { $these } e { $last }
keys-taken = pa3ds usa { $controls }
keys-applied = tasti applicati ad Azahar
keys-waiting = chiudi Azahar e riavvia pa3ds per applicare i tuoi tasti
keys-commented-out = nuovo tasto commentato in { $file }: { $key } per { $action }, perché { $key } è già { $taken }
keys-edit-to-choose = modifica { $file } per scegliere
file-unreadable = impossibile leggere { $file }: { $problem }. Correggilo, o eliminalo perché pa3ds ne scriva uno nuovo
azahar-settings-applied = impostazioni di Azahar applicate
azahar-settings-waiting = chiudi Azahar e riavvia pa3ds per applicare le sue impostazioni
azahar-settings-missing = impostazioni di Azahar non trovate: avvia Azahar una volta, chiudilo e riavvia pa3ds
azahar-folder-ask = Impostazioni di Azahar non trovate. Scegli la cartella in cui si trova azahar.exe, o annulla se Azahar è installato
azahar-folder-again = { $problem }, scegli un'altra cartella o annulla
azahar-folder-no-exe = { $folder } non contiene azahar.exe
azahar-folder-not-portable = { $folder } non ha una cartella user accanto al suo azahar.exe, quindi questo Azahar tiene le impostazioni in AppData e non accanto

## What a thing on the map is, where neither the game nor a name says

kind-door = porta
kind-way-out = uscita
kind-warp = teletrasporto
kind-path = sentiero
kind-edge = bordo
kind-stairs = scale
kind-stairs-up = scale in su
kind-stairs-down = scale in giù
kind-passage = passaggio
kind-entrance = ingresso
kind-teleporter = teletrasportatore
kind-elevator = ascensore
kind-sign = cartello
kind-flag = bandiera
kind-object = oggetto
kind-person = persona
kind-item-ball = strumento a terra
kind-barrier = barriera
kind-hole = buca
kind-dive-spot = punto di immersione
kind-current = corrente
kind-flat-ground = terreno piano
kind-slope = pendio
kind-bumpy-slope = pendio accidentato
kind-rail = binario
kind-ice = ghiaccio
kind-blank = vuoto
kind-wall = muro
kind-crumbled-cliff = scogliera franata

## Battle

battle-battler = { $name } Lv{ $level } { $hp }
battle-share = { $percent }%

## Scenes

narration = Narrazione
