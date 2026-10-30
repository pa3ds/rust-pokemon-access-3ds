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
above = { $count } più in alto
below = { $count } più in basso
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
category-climbing = arrampicata
category-currents = correnti
category-arrows = frecce
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
past-teleporter = oltre un teletrasportatore
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
how-climb = arrampicati
p-climb-up = arrampicati { $holds } su
p-climb-down = arrampicati { $holds } giù
how-slide = scivola
how-grind-the-rail = scivola sul binario
p-grind = scivola sul binario { $bearing }
how-ride-the-belt = segui il nastro
p-belt = segui il nastro { $bearing }
how-arrow-pushes = la freccia spinge
p-arrow = la freccia spinge { $bearing }
p-switch-first = prima { $thing }, { $way }
how-climb-the-vine-up = arrampicati sulla liana
how-climb-the-vine-down = scendi dalla liana
how-swing-on-the-vine = dondola sulla liana
p-vine = guarda verso { $way }, { $how }
p-more-vines = { $count ->
    [one] poi ancora 1 liana
   *[other] poi ancora { $count } liane
}
p-more-teleporters = { $count ->
    [one] poi ancora 1 teletrasportatore
   *[other] poi ancora { $count } teletrasportatori
}
how-up-the-ramp = sali la rampa
how-down-the-ramp = scendi la rampa
p-current = segui la corrente { $bearing }
p-to-push = per spingere verso { $way }:
p-nudge = un po' verso { $way }
p-nudge-then = { $nudge }, poi
p-lined-up = allineato
p-then-cant-continue = poi impossibile proseguire
p-cant-continue = impossibile proseguire
p-in-the-way = { $thing } blocca la strada
p-surface = riemergi e vai al punto di immersione { $bearing }
p-into = { $leg } verso { $place }
p-and-ice-cleared = e il ghiaccio è rotto
p-ice-cleared = ghiaccio rotto
p-no-path-to = nessun percorso verso { $thing }
p-no-route-in-game = ancora nessun percorso in questo gioco
p-no-route-on-streets = nessun percorso in queste strade
p-gone = { $thing } non c'è più

## Touch mode

touch-nothing = niente da toccare
touch-no-drag = niente da trascinare
touch-continue = continua
touch-off = il touch è disattivato
touch-not-listening = Azahar non riceve tocchi da pa3ds
touch-holding = tenendo { $target }
touch-dropped = lasciato su { $target }

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
puzzle-cell = { $column }{ $row }
puzzle-home = { $tile }, al suo posto
puzzle-goes = { $tile }, va in { $home }
puzzle-picture-left = { $pokemon } a sinistra
puzzle-picture-front = { $pokemon } davanti
puzzle-picture-right = { $pokemon } a destra
puzzle-picked = { $tile }, selezionata
berry-picker-wants = { $pokemon } vuole { $berry }
berry-picker-for = { $berry } per { $pokemon }
berry-picker-wanted = si chiede { $berry }
berry-picker-lineup = { $first }, { $second } e { $third }
berry-picker-not-up = { $pokemon } non è fuori
berry-picker-no-berry = nessuna bacca { $number }
berry-picker-no-pokemon = nessun Pokémon { $number }

## What pa3ds says of itself

app-waiting = In attesa del gioco
app-ready = pa3ds pronto
app-restarted = pa3ds riavviato
app-reader-error = Errore di lettura
dialog-ok = OK
dialog-cancel = Annulla
window-title = Pokémon Access 3DS
window-menu-mod = &Mod
window-updates = Cerca &aggiornamenti
window-exit = &Esci
window-menu-azahar = &Azahar
window-azahar-portable = Scarica Azahar (&portatile)
window-azahar-installer = Scarica Azahar (&installer)
window-menu-help = &Guida
window-documentation = &Documentazione...
window-keys = &Comandi da tastiera...
window-known-issues = &Problemi noti...
window-translating = &Traduzione...
window-changelog = Registro delle &modifiche...
window-settings-guide = Guida alle &impostazioni...
window-issues = &Segnala un problema su GitHub
setting-pa3ds-keys = Rispondi ai tasti della mod
setting-pa3ds-keys-about = Risponde ai tasti di pa3ds e collega i pulsanti di Azahar ai tasti, come indicato in keys.toml.
setting-pa3ds-poll_rate = Letture al secondo
setting-pa3ds-poll_rate-about = Quante volte al secondo leggere il gioco: 60, la frequenza dei fotogrammi del gioco; 30 per un computer più lento.
setting-pa3ds-azahar_folder = Cartella di Azahar portatile
setting-pa3ds-azahar_folder-about = La cartella in cui si trova un Azahar portatile, quella con azahar.exe e una cartella user; vuota per un Azahar installato nel modo consueto, che tiene le impostazioni in AppData. pa3ds la compila quando scegli la cartella che ti chiede.
setting-pa3ds-hide_window = Nascondi la finestra della mod mentre Azahar è aperto
setting-pa3ds-hide_window-about = Nasconde la finestra di pa3ds, quella che si apre avviando pa3ds con un doppio clic, mentre Azahar è aperto, così in Alt+Tab non c'è niente di pa3ds tra te e il gioco; false la lascia visibile, perché OBS possa registrare i suoni di pa3ds attraverso di essa.
setting-pa3ds-ui-pauses = Pause nel testo a schermo
setting-pa3ds-ui-pauses-about = Inserisce pause tra le parti di testo a schermo lette insieme: i due punti dopo un'etichetta, come in "AI: Raiden", e una virgola tra le altre.
setting-pa3ds-ui-interrupt = I menu interrompono la voce
setting-pa3ds-ui-interrupt-about = Interrompe la voce quando il cursore di un menu si sposta, invece di metterla in coda; i tasti di pa3ds la interrompono sempre.
setting-pa3ds-ui-unfocused = Parla mentre Azahar è in secondo piano
setting-pa3ds-ui-unfocused-about = Continua a dire ciò che il gioco mostra mentre un'altra finestra è in primo piano; altrimenti pa3ds resta in silenzio, tranne per i suoi tasti e i suoi messaggi.
setting-pa3ds-ui-descriptions = Descrizioni delle scene
setting-pa3ds-ui-descriptions-about = Descrive le scene che il gioco mostra senza parole, come il risveglio di Groudon. Ogni descrizione viene detta dopo la riga di testo che precede la scena. Finora sono descritte solo alcune scene.
setting-pa3ds-ui-language = Lingua
setting-pa3ds-ui-language-about = La lingua in cui pa3ds dice le sue parole, come nome di una cartella dei suoi dati: en, fr, de, es, it, ja, ko, zh-CN o zh-TW. Se vuota, segue la lingua del gioco, che pa3ds legge dal salvataggio; impostala quando questa è sbagliata, come per una traduzione amatoriale sopra un'altra lingua, e in Sole e Luna, il cui salvataggio pa3ds non sa leggere, dove è anche la lingua del gioco.
setting-pa3ds-audio-tones = Volume relativo dei toni
setting-pa3ds-audio-tones-about = Quanto è forte il breve tono che suona quando si sposta il cursore di una mappa o della modalità touch, più acuto quanto più è in alto e spostato a destra quanto più è a destra, per dire dove si trova il cursore: 1 forte quanto il gioco, 0.5 a metà volume, 1.5 più forte, 0 nessun tono; nella finestra delle impostazioni, in percentuale, 100 forte quanto il gioco.
setting-pa3ds-audio-footsteps = Volume relativo dei passi
setting-pa3ds-audio-footsteps-about = Quanto sono forti i passi di pa3ds sul terreno dove il gioco non ne riproduce, come strade, pavimenti e grotte: 1 forti quanto li riprodurrebbe il gioco, 0.5 a metà volume, 1.5 più forti, 0 nessun passo; nella finestra delle impostazioni, in percentuale, 100 forti quanto il gioco.
setting-pa3ds-audio-ticks = Volume relativo del ticchettio dell'orologio
setting-pa3ds-audio-ticks-about = Quanto è forte un breve tono a ogni secondo dell'orologio di un gioco, come nei giochi di Poké io&te; l'orologio stesso viene detto ogni mezzo minuto e negli ultimi secondi: 1 forte quanto il gioco, 0.5 a metà volume, 1.5 più forte, 0 nessun tono, come di serie; nella finestra delle impostazioni, in percentuale, 100 forte quanto il gioco.
setting-pa3ds-audio-feedback_sounds = Volume relativo dei suoni di risposta
setting-pa3ds-audio-feedback_sounds-about = Quanto sono forti i brevi suoni del gioco per ciò che un tasto ha fatto: i suoni del Box del PC quando si prende e si posa un Pokémon, quando la modalità touch prende qualcosa da trascinare e lo lascia, e il suono di errore del gioco per un tasto che non ha niente da fare, come una bacca che il turno di Cogli la Bacca non ha. A 0 pa3ds li dice invece, "tenendo", "lasciato su" e il motivo: 1 forte quanto il gioco, 0.5 a metà volume, 1.5 più forte; nella finestra delle impostazioni, in percentuale, 100 forte quanto il gioco.
setting-pa3ds-navigation-positions = Annuncia la posizione nell'elenco
setting-pa3ds-navigation-positions-about = Dice dove si trova una cosa nell'elenco, come in "porta, 1 di 3".
setting-pa3ds-navigation-position_first = Posizione nell'elenco prima delle direzioni
setting-pa3ds-navigation-position_first-about = Dice il posto nell'elenco prima della strada per raggiungerlo, "porta, 1 di 3, 1 nord"; false dice "porta, 1 nord, 1 di 3".
setting-pa3ds-navigation-turn_by_turn = Indicazioni passo passo
setting-pa3ds-navigation-turn_by_turn-about = Quando P ha trovato la strada, dice ogni svolta man mano che la raggiungi; false dice la strada una volta sola, e P la ripete da dove ti trovi.
setting-pa3ds-navigation-guide_to_hidden = Guida verso i ritrovamenti del NaviDex
setting-pa3ds-navigation-guide_to_hidden-about = Quando una ricerca del NaviDex trova il Pokémon cercato, o il NaviDex ne trova uno da solo mentre è aperto e sei in modalità touch, trova la strada per raggiungerlo come fa P e tiene P su di esso, qualunque cosa fosse selezionata nell'elenco; uno trovato mentre il NaviMappa è aperto viene ignorato.
setting-pa3ds-navigation-track_boulders = Segui lo spostamento dei massi
setting-pa3ds-navigation-track_boulders-about = Dice di quanto si è spostato un masso dal punto in cui parte nella sua stanza, come in "masso, 2 est, spostato di 1 nord", così sai quali hai spinto; il gioco mostra dove si trova ogni masso, mai da dove è partito.
setting-pa3ds-navigation-hide_door_signs = Nascondi i cartelli delle porte
setting-pa3ds-navigation-hide_door_signs-about = Esclude dall'elenco un cartello quando dice solo il nome di una porta accanto, come "Interno 13, Walter" al Residence Ciclamipoli; un cartello che dice di più, come quello di una Palestra, resta.
setting-pa3ds-navigation-show_y = Annuncia l'altezza
setting-pa3ds-navigation-show_y-about = Dice anche l'altezza, come y dopo x e z, quando C dice dove ti trovi e Maiusc e C dove si trova la cosa scelta: "x 71, z 35, y 20"; nelle unità del gioco, 18 per la larghezza di un riquadro, e omessa dove pa3ds non la conosce, come in Sole e Luna.
setting-pa3ds-navigation-sort_by_distance = Ordina per distanza
setting-pa3ds-navigation-sort_by_distance-about = Ordina l'elenco della modalità elenco dal più vicino, e l'ordine cambia mentre cammini; false tiene ogni cosa sempre allo stesso posto, nell'ordine delle tabelle del gioco, così puoi capire in quale casa sei già entrato da dove si trova; la modalità direzione ordina sempre dal più vicino.
setting-pa3ds-touch-wrap = Ricomincia dall'inizio
setting-pa3ds-touch-wrap-about = In modalità touch, passa dall'ultimo pulsante al primo, e dalla riga in basso a quella in alto; false si ferma agli estremi, così premendo O qualche volta si finisce sempre sulla riga in basso.
setting-pa3ds-touch-next_row = Passa alla riga successiva
setting-pa3ds-touch-next_row-about = In modalità touch, J e L passano alla riga sopra o sotto alla fine di una riga; false li tiene nella loro riga, così premendo L qualche volta si finisce sempre a destra.
setting-pa3ds-touch-drag = Come trascinare
setting-pa3ds-touch-drag-about = Trascina tenendo premuto Maiusc, o un tuo tasto assegnato al trascinamento in keys.toml, e lascia il tocco quando lo rilasci ("hold"), oppure premilo per prendere e premilo di nuovo per lasciare ("toggle"). Si trascina solo in Poké io&te.
setting-pa3ds-touch-drag-hold = Tieni premuto Maiusc
setting-pa3ds-touch-drag-toggle = Premi per prendere, premi per lasciare
setting-pa3ds-amie-berry_first = Cogli la Bacca dice prima la bacca
setting-pa3ds-amie-berry_first-about = Dice ogni Pokémon che vuole una bacca nell'ordine in cui la dai, la bacca e poi il Pokémon, "Baccananas per Pikachu"; false dice prima il Pokémon, "Pikachu vuole Baccananas".
setting-pa3ds-amie-one_at_a_time = Cogli la Bacca dice un Pokémon alla volta
setting-pa3ds-amie-one_at_a_time-about = Dice un solo Pokémon che vuole una bacca alla volta, e il successivo quando hai dato una bacca o quello detto se n'è andato; false li dice man mano che arrivano.
setting-pa3ds-extras-conditions = Caratteristiche da gara in numeri
setting-pa3ds-extras-conditions-about = Dice le caratteristiche da gara di un Pokémon in numeri dove il gioco le disegna come una stella, nella schermata del Pokémon e nel Portapokémelle: la stella è visibile a tutti ma pa3ds non la sa leggere, quindi i numeri sono tutto ciò che può dare, e sono più precisi della stella.
setting-pa3ds-extras-hidden_items = Strumenti nascosti
setting-pa3ds-extras-hidden_items-about = Elenca tra gli strumenti quelli nascosti ancora da trovare, come "strumento nascosto": solo sott'acqua ("underwater"), dove il Ricerca Strumenti non si può usare e un giocatore vedente può notare un luccichio tra le rocce e le alghe, o ovunque ("all"), il che rivela ciò che il gioco nasconde a tutti; "off" non ne elenca nessuno.
setting-pa3ds-extras-hidden_items-off = Disattivato
setting-pa3ds-extras-hidden_items-underwater = Sott'acqua
setting-pa3ds-extras-hidden_items-all = Ovunque
settings-title = Impostazioni della mod
settings-page-general = Generale
settings-page-speech = Voce
settings-page-audio = Audio
settings-page-navigation = Navigazione
settings-page-touch = Modalità touch
settings-page-amie = Poké io&&te
settings-page-extras = Extra
settings-browse = &Sfoglia...
settings-language-game = La lingua del gioco
settings-restart = Riavviare pa3ds ora per applicare queste impostazioni? Se scegli No, verranno applicate al prossimo avvio di pa3ds.
settings-unwritten = pa3ds non ha potuto scrivere { $path }, quindi non è cambiato nulla.
dialog-apply = &Applica
window-settings = &Impostazioni...

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
kind-way-down = discesa
kind-conveyor-belt = nastro trasportatore
kind-switch = interruttore
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
kind-hidden-item = strumento nascosto
kind-barrier = barriera
kind-hole = buca
kind-dive-spot = punto di immersione
kind-current = corrente
kind-arrow-east = freccia verso est
kind-arrow-west = freccia verso ovest
kind-arrow-north = freccia verso nord
kind-arrow-south = freccia verso sud
kind-flat-ground = terreno piano
kind-open-ground = spazio aperto
kind-quiz-panel = pannello del quiz
kind-blue-spot = punto blu
kind-wall-ledge = sporgenza della parete
kind-vine-up = liana in su
kind-vine-down = liana in giù
kind-swinging-vine = liana oscillante
kind-gym-leader-spot = posto del capopalestra
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
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = Italiano
