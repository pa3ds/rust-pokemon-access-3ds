### German machine translation of `../en/words.ftl` (2026-10-18), a baseline for a translator to
### correct. The English file says what each message is said of, and where; a message missing
### here is said in English.

## Ways and distances

way-north = Norden
way-east = Osten
way-south = Süden
way-west = Westen
way-north-east = Nordosten
way-south-east = Südosten
way-south-west = Südwesten
way-north-west = Nordwesten
way-ahead = vorne
way-behind = hinten
way-left = links
way-right = rechts
steps = { $count } { $way }
leg-with = { $how } { $steps } { $way }
here = hier
rail-forward = { $steps } vorwärts
rail-back = { $steps } rückwärts
rail-hold-right = rechts halten
rail-hold-left = links halten
rail-angled = Kamera schräg, links halten, rechts halten
turn-right = rechts drehen
turn-left = links drehen
turn-around = umdrehen

## The list of what is on the map, and the navigation keys' answers

category-all = alles
category-exits = Ausgänge
category-signs = interaktive Objekte
category-obstacles = Hindernisse
category-people = Personen
category-items = Items
category-berries = Beeren
category-ash = Asche
category-bike = Rad
category-riding = Reiten
category-currents = Strömungen
category-pokemon = Pokémon
key-category = { $category }, { $count }
position = { $number } von { $count }
key-nothing-way = nichts Richtung { $way }
key-none-way = keine { $category } Richtung { $way }
key-nothing-here = nichts hier
key-no-map = keine Karte
key-cant-walk-now = kann jetzt nicht gehen
key-list-mode = Listenmodus
key-direction-mode = Richtungsmodus
key-touch-mode = Touch-Modus, { $first }
key-unreachable-hidden = Unerreichbares ausgeblendet
key-unreachable-shown = Unerreichbares eingeblendet
key-path-off = Weg aus
key-no-path = kein Weg
key-no-battle = kein Kampf
key-one-way = nur ein Weg dorthin
out-at = Ausgang bei { $bearing }
leads-to = { $thing } nach { $place }
span = { $low } bis { $high }
either = { $these } oder { $last }

## Big boulders

key-push = nach { $way } schieben
boulder-into-hole = nach { $way } in ein Loch
boulder-moveable = beweglich nach { $ways }
boulder-immoveable = unbeweglich
boulder-moved = verschoben um { $bearing }

## Renaming

rename-title = Umbenennen
rename-label = { $thing }, { $bearing }. { $name }
rename-speaker-label = Die letzte sprechende Person. { $name }
rename-called = Name: { $name }
rename-not-named = Noch kein Name
rename-only-this-one = Nur dieses eine
rename-now = { $thing } heißt jetzt { $name }
rename-no-name = { $thing } hat keinen Namen
rename-the-last-speaker = die letzte sprechende Person
rename-nothing = nichts zum Umbenennen
rename-cannot = { $thing } kann nicht umbenannt werden
rename-already = Umbenennen läuft schon
rename-nobody-spoke = niemand hat gesprochen

## P: the way to the thing chosen, and guiding the player along it

how-water = Wasser
how-bush = Busch
how-boulder = Felsblock
how-rock = Fels
how-waterfall = Wasserfall
how-slope = Hang
how-bumpy-slope = holpriger Hang
how-push = schieben
how-ride-the-current = mit der Strömung treiben
how-get-off = absteigen
how-smash = zertrümmern
p-current = mit der Strömung { $bearing }
p-to-push = um nach { $way } zu schieben:
p-nudge = ein wenig nach { $way }
p-nudge-then = { $nudge }, dann
p-lined-up = ausgerichtet
p-then-cant-continue = dann geht es nicht weiter
p-cant-continue = es geht nicht weiter
p-in-the-way = { $thing } im Weg
p-surface = auftauchen und zur Tauchstelle { $bearing }
p-and-ice-cleared = und das Eis ist gebrochen
p-ice-cleared = Eis gebrochen
p-no-path-to = kein Weg zu { $thing }
p-no-route-in-game = in diesem Spiel noch keine Route
p-no-route-on-streets = keine Route in diesen Straßen
p-gone = { $thing } ist weg

## Touch mode

touch-nothing = nichts zum Antippen
touch-off = Touch ist aus
touch-not-listening = Azahar nimmt keine Berührungen von pa3ds an

## What the field shows without words

dowsing = Itemradar { $light }
dowsing-blue = blau
dowsing-green = grün
dowsing-yellow = gelb
dowsing-red = rot
dowsing-nothing = nichts
decorating-placeable = platzierbar
decorating-not-placeable = nicht platzierbar

## The Pokédex's habitat map

habitat-found = Vorkommen: { $place }
habitat-grass = Gras
habitat-water = Wasser
habitat-fishing = Angeln

## Super Training and Pokémon-Amie

training-goal = Tor
training-red-goal = rotes Tor
training-shot-coming = Schuss kommt
training-left = noch { $time }
amie-stroke = { $pokemon } streicheln

## What pa3ds says of itself

app-waiting = Warte auf das Spiel
app-ready = pa3ds bereit
app-restarted = pa3ds neu gestartet
app-reader-error = Lesefehler
dialog-ok = OK
dialog-cancel = Abbrechen

## As pa3ds starts: its keys, its settings and Azahar's

both = { $these } und { $last }
keys-taken = pa3ds belegt { $controls }
keys-applied = Tasten in Azahar übernommen
keys-waiting = schließe Azahar und starte pa3ds neu, um deine Tasten zu übernehmen
keys-commented-out = neue Taste in { $file } auskommentiert: { $key } für { $action }, weil { $key } schon { $taken } ist
keys-edit-to-choose = bearbeite { $file }, um zu wählen
file-unreadable = { $file } konnte nicht gelesen werden: { $problem }. Korrigiere die Datei oder lösche sie, damit pa3ds eine neue schreibt
azahar-settings-applied = Azahar-Einstellungen übernommen
azahar-settings-waiting = schließe Azahar und starte pa3ds neu, um seine Einstellungen zu übernehmen
azahar-settings-missing = Azahars Einstellungen wurden nicht gefunden: starte Azahar einmal, schließe es und starte pa3ds neu
azahar-folder-ask = Azahars Einstellungen wurden nicht gefunden. Wähle den Ordner, in dem azahar.exe liegt, oder brich ab, wenn Azahar installiert ist
azahar-folder-again = { $problem }, wähle einen anderen Ordner oder brich ab
azahar-folder-no-exe = in { $folder } liegt keine azahar.exe
azahar-folder-not-portable = { $folder } hat keinen user-Ordner neben seiner azahar.exe, also speichert dieses Azahar seine Einstellungen in AppData und nicht daneben

## What a thing on the map is, where neither the game nor a name says

kind-door = Tür
kind-way-out = Ausgang
kind-warp = Warp
kind-path = Pfad
kind-edge = Rand
kind-stairs = Treppe
kind-stairs-up = Treppe hoch
kind-stairs-down = Treppe runter
kind-passage = Durchgang
kind-entrance = Eingang
kind-teleporter = Teleporter
kind-elevator = Aufzug
kind-sign = Schild
kind-flag = Flagge
kind-object = Objekt
kind-person = Person
kind-item-ball = Item am Boden
kind-barrier = Barriere
kind-hole = Loch
kind-dive-spot = Tauchstelle
kind-current = Strömung
kind-flat-ground = flacher Boden
kind-slope = Hang
kind-bumpy-slope = holpriger Hang
kind-rail = Schiene
kind-ice = Eis
kind-blank = leer
kind-wall = Wand
kind-crumbled-cliff = eingestürzte Klippe

## Battle

battle-battler = { $name } Lv{ $level } { $hp }
battle-share = { $percent } %

## Scenes

narration = Erzählung
