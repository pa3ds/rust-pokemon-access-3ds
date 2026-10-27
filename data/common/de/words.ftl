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
above = { $count } höher
below = { $count } tiefer
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
category-climbing = Klettern
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
how-climb = klettern
p-climb-up = { $holds } hochklettern
p-climb-down = { $holds } runterklettern
how-slide = rutschen
how-grind-the-rail = auf der Schiene grinden
p-grind = auf der Schiene grinden { $bearing }
how-climb-the-vine-up = die Ranke hochklettern
how-climb-the-vine-down = die Ranke runterklettern
how-swing-on-the-vine = an der Ranke schwingen
p-vine = nach { $way } drehen, { $how }
p-more-vines = { $count ->
    [one] dann noch 1 Ranke
   *[other] dann noch { $count } Ranken
}
how-up-the-ramp = die Rampe hinauf
how-down-the-ramp = die Rampe hinunter
p-current = mit der Strömung { $bearing }
p-to-push = um nach { $way } zu schieben:
p-nudge = ein wenig nach { $way }
p-nudge-then = { $nudge }, dann
p-lined-up = ausgerichtet
p-then-cant-continue = dann geht es nicht weiter
p-cant-continue = es geht nicht weiter
p-in-the-way = { $thing } im Weg
p-surface = auftauchen und zur Tauchstelle { $bearing }
p-into = { $leg } nach { $place }
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
window-title = Pokémon Access 3DS
window-menu-mod = &Mod
window-updates = Nach &Updates suchen
window-exit = &Beenden
window-menu-azahar = &Azahar
window-azahar-portable = Azahar herunterladen (&portabel)
window-azahar-installer = Azahar herunterladen (&Installationsprogramm)
window-menu-help = &Hilfe
window-documentation = &Dokumentation...
window-keys = &Tastenbefehle...
window-known-issues = Bekannte &Probleme...
window-translating = Über&setzen...
window-changelog = Ä&nderungsprotokoll...
window-settings-guide = &Einstellungshilfe...
window-issues = Problem auf &GitHub melden
setting-pa3ds-keys = Auf Mod-Tasten reagieren
setting-pa3ds-keys-about = Auf die Tasten von pa3ds reagieren und die Tasten von Azahar belegen, wie es in keys.toml steht.
setting-pa3ds-poll_rate = Lesevorgänge pro Sekunde
setting-pa3ds-poll_rate-about = Wie oft pro Sekunde das Spiel gelesen wird: 60, die Bildrate des Spiels selbst; 30 für einen langsameren Rechner.
setting-pa3ds-azahar_folder = Ordner des portablen Azahar
setting-pa3ds-azahar_folder-about = Der Ordner, in dem ein portables Azahar liegt, der mit azahar.exe und einem user-Ordner; leer für ein auf dem üblichen Weg installiertes Azahar, dessen Einstellungen in AppData liegen. pa3ds trägt ihn ein, wenn du den Ordner wählst, nach dem es fragt.
setting-pa3ds-hide_window = Mod-Fenster ausblenden, solange Azahar offen ist
setting-pa3ds-hide_window-about = Das Fenster von pa3ds, das ein doppelt angeklicktes pa3ds öffnet, ausblenden, solange Azahar offen ist, damit bei Alt+Tab nichts von pa3ds zwischen dir und dem Spiel liegt; false lässt es sichtbar, damit OBS darüber die Töne von pa3ds aufnehmen kann.
setting-pa3ds-ui-pauses = Pausen in Bildschirmtext
setting-pa3ds-ui-pauses-about = Pausen zwischen Teile von Bildschirmtext setzen, die zusammen gesagt werden: einen Doppelpunkt nach einer Beschriftung, wie in „OT: Raiden“, und ein Komma zwischen den übrigen.
setting-pa3ds-ui-interrupt = Menüs unterbrechen die Sprachausgabe
setting-pa3ds-ui-interrupt-about = Die Sprachausgabe abbrechen, wenn sich der Cursor eines Menüs bewegt, statt sie einzureihen; die eigenen Tasten von pa3ds tun das immer.
setting-pa3ds-ui-unfocused = Sprechen, während Azahar im Hintergrund ist
setting-pa3ds-ui-unfocused-about = Weiter sagen, was das Spiel zeigt, während ein anderes Fenster im Vordergrund ist; sonst schweigt pa3ds dann, außer bei seinen eigenen Tasten und Meldungen.
setting-pa3ds-ui-descriptions = Szenenbeschreibungen
setting-pa3ds-ui-descriptions-about = Die Szenen beschreiben, die das Spiel ohne Worte zeigt, etwa wie Groudon erwacht. Jede Beschreibung wird nach der Textzeile gesagt, auf die die Szene folgt. Bisher sind nur einige Szenen beschrieben.
setting-pa3ds-ui-language = Sprache
setting-pa3ds-ui-language-about = Die Sprache, in der pa3ds seine eigenen Worte sagt, als Name eines Ordners seiner Daten: en, fr, de, es, it, ja, ko, zh-CN oder zh-TW. Leer folgt der Sprache des Spiels, die pa3ds aus dem Spielstand liest; stelle sie ein, wo diese falsch ist, etwa bei einer Fan-Übersetzung über einer anderen Sprache, und in Sonne und Mond, deren Spielstand pa3ds nicht lesen kann, wo sie auch die Sprache des Spiels ist.
setting-pa3ds-audio-tones = Relative Lautstärke der Töne
setting-pa3ds-audio-tones-about = Wie laut der kurze Ton ist, wenn sich der Cursor einer Karte oder der Touch-Modus bewegt, höher, je weiter oben, und weiter rechts, je weiter rechts, damit du hörst, wo der Cursor ist: 1 so laut wie das Spiel, 0.5 halb so laut, 1.5 lauter, 0 gar nicht; im Einstellungsdialog in Prozent, 100 so laut wie das Spiel.
setting-pa3ds-audio-footsteps = Relative Lautstärke der Schritte
setting-pa3ds-audio-footsteps-about = Wie laut die eigenen Schritte von pa3ds auf Boden sind, auf dem das Spiel keine abspielt, etwa auf Straßen, Fußböden und in Höhlen: 1 so laut, wie das Spiel sie abspielen würde, 0.5 halb so laut, 1.5 lauter, 0 gar nicht; im Einstellungsdialog in Prozent, 100 so laut wie das Spiel.
setting-pa3ds-navigation-positions = Listenposition ansagen
setting-pa3ds-navigation-positions-about = Sagen, wo ein Ding in der Liste steht, wie in „Tür, 1 von 3“.
setting-pa3ds-navigation-position_first = Listenposition vor der Richtung
setting-pa3ds-navigation-position_first-about = Die Stelle in der Liste vor dem Weg dorthin sagen, „Tür, 1 von 3, 1 Norden“; false sagt „Tür, 1 Norden, 1 von 3“.
setting-pa3ds-navigation-turn_by_turn = Wegbeschreibung Schritt für Schritt
setting-pa3ds-navigation-turn_by_turn-about = Sobald P den Weg gefunden hat, jede Abzweigung ansagen, wenn du sie erreichst; false sagt den Weg einmal, und P sagt ihn erneut von dort, wo du stehst.
setting-pa3ds-navigation-guide_to_hidden = Zu Funden des Pokédex-Navi führen
setting-pa3ds-navigation-guide_to_hidden-about = Wenn eine Suche mit dem Pokédex-Navi das gesuchte Pokémon findet, oder das Pokédex-Navi von selbst eines findet, während es offen ist und du im Touch-Modus bist, den Weg dorthin finden wie mit P und P darauf halten, egal worauf die Liste stand; eines, das gefunden wird, während das Karten-Navi offen ist, bleibt unberührt.
setting-pa3ds-navigation-track_boulders = Bewegung der Felsblöcke verfolgen
setting-pa3ds-navigation-track_boulders-about = Sagen, wie weit ein großer Felsblock von dort bewegt wurde, wo sein Raum ihn beginnen lässt, wie in „großer Felsblock, 2 Osten, verschoben um 1 Norden“, damit du weißt, welche du geschoben hast; das Spiel zeigt, wo jeder Felsblock ist, nie, wo er anfing.
setting-pa3ds-navigation-hide_door_signs = Türschilder ausblenden
setting-pa3ds-navigation-hide_door_signs-about = Ein Schild aus der Liste weglassen, wenn es nur den Namen einer Tür daneben nennt, wie die Wohnungsschilder in Malvenfroh City; ein Schild, das mehr sagt, wie das einer Arena, bleibt.
setting-pa3ds-navigation-show_y = Höhe ansagen
setting-pa3ds-navigation-show_y-about = Auch sagen, wie hoch, als y nach x und z, wenn C sagt, wo du stehst, und Umschalt und C, wo das gewählte Ding ist: „x 71, z 35, y 20“; in den Einheiten des Spiels, 18 auf die Breite eines Feldes, und weggelassen, wo pa3ds sie nicht kennt, wie in Sonne und Mond.
setting-pa3ds-navigation-sort_by_distance = Nach Entfernung sortieren
setting-pa3ds-navigation-sort_by_distance-about = Die Liste des Listenmodus mit dem Nächsten zuerst, was sich beim Gehen ändert; false hält jedes Ding an einem festen Platz darin, in der Reihenfolge der Tabellen des Spiels, damit du an seinem Platz erkennst, in welchem Haus du schon warst; der Richtungsmodus zeigt so oder so das Nächste zuerst.
setting-pa3ds-extras-conditions = Wettbewerbswerte als Zahlen
setting-pa3ds-extras-conditions-about = Die Wettbewerbswerte eines Pokémon als Zahlen sagen, wo das Spiel sie als Stern zeichnet, auf der Statusseite und in der Knurspbox: Der Stern ist für alle sichtbar, aber pa3ds kann ihn nicht lesen, also sind seine Zahlen alles, was es geben kann, und sie sind genauer als der Stern.
setting-pa3ds-extras-hidden_items = Versteckte Items
setting-pa3ds-extras-hidden_items-about = Die noch nicht gefundenen versteckten Items als „verstecktes Item“ unter den Items auflisten: nur unter dem Meer („underwater“), wo das Itemradar nicht eingesetzt werden kann und sehende Spieler zwischen Felsen und Seetang ein Glitzern sehen können, oder überall („all“), was verrät, was das Spiel vor allen verbirgt; „off“ listet keine.
setting-pa3ds-extras-hidden_items-off = Aus
setting-pa3ds-extras-hidden_items-underwater = Unter dem Meer
setting-pa3ds-extras-hidden_items-all = Überall
settings-title = Mod-Einstellungen
settings-page-general = Allgemein
settings-page-speech = Sprachausgabe
settings-page-audio = Audio
settings-page-navigation = Navigation
settings-page-extras = Extras
settings-browse = &Durchsuchen...
settings-language-game = Die Sprache des Spiels
settings-restart = pa3ds jetzt neu starten, damit diese Einstellungen wirksam werden? Wenn du Nein wählst, werden sie beim nächsten Start von pa3ds wirksam.
settings-unwritten = pa3ds konnte { $path } nicht schreiben, also hat sich nichts geändert.
dialog-apply = Ü&bernehmen
window-settings = &Einstellungen...

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
kind-way-down = Weg nach unten
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
kind-hidden-item = verstecktes Item
kind-barrier = Barriere
kind-hole = Loch
kind-dive-spot = Tauchstelle
kind-current = Strömung
kind-flat-ground = flacher Boden
kind-open-ground = offenes Gelände
kind-quiz-panel = Quizfeld
kind-blue-spot = blauer Punkt
kind-wall-ledge = Felsvorsprung
kind-vine-up = Ranke hoch
kind-vine-down = Ranke runter
kind-swinging-vine = Schwingranke
kind-gym-leader-spot = Platz des Arenaleiters
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
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = Deutsch
