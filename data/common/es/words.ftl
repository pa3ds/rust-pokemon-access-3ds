### Spanish machine translation of `../en/words.ftl` (2026-10-18), a baseline for a translator to
### correct. The English file says what each message is said of, and where; a message missing
### here is said in English.

## Ways and distances

way-north = norte
way-east = este
way-south = sur
way-west = oeste
way-north-east = noreste
way-south-east = sureste
way-south-west = suroeste
way-north-west = noroeste
way-ahead = delante
way-behind = detrás
way-left = izquierda
way-right = derecha
steps = { $count } { $way }
leg-with = { $how } { $steps } { $way }
here = aquí
rail-forward = { $steps } adelante
rail-back = { $steps } atrás
rail-hold-right = mantener derecha
rail-hold-left = mantener izquierda
rail-angled = cámara en ángulo, mantener izquierda, mantener derecha
turn-right = girar a la derecha
turn-left = girar a la izquierda
turn-around = darse la vuelta

## The list of what is on the map, and the navigation keys' answers

category-all = todo
category-exits = salidas
category-signs = objetos interactivos
category-obstacles = obstáculos
category-people = personas
category-items = objetos
category-berries = bayas
category-ash = ceniza
category-bike = bici
category-riding = montura
category-currents = corrientes
category-pokemon = pokémon
key-category = { $category }, { $count }
position = { $number } de { $count }
key-nothing-way = nada en dirección { $way }
key-none-way = sin { $category } en dirección { $way }
key-nothing-here = nada aquí
key-no-map = sin mapa
key-cant-walk-now = no se puede caminar ahora
key-list-mode = modo lista
key-direction-mode = modo dirección
key-touch-mode = modo táctil, { $first }
key-unreachable-hidden = inalcanzables ocultos
key-unreachable-shown = inalcanzables visibles
key-path-off = ruta desactivada
key-no-path = sin ruta
key-no-battle = sin combate
key-one-way = solo un camino
out-at = salida a { $bearing }
leads-to = { $thing } a { $place }
span = { $low } a { $high }
either = { $these } o { $last }

## Big boulders

key-push = empujar al { $way }
boulder-into-hole = al { $way } a un agujero
boulder-moveable = movible al { $ways }
boulder-immoveable = inamovible
boulder-moved = movida { $bearing }

## Renaming

rename-title = Renombrar
rename-label = { $thing }, { $bearing }. { $name }
rename-speaker-label = El último que habló. { $name }
rename-called = Nombre: { $name }
rename-not-named = Sin nombre todavía
rename-only-this-one = Solo este
rename-now = { $thing } ahora se llama { $name }
rename-no-name = { $thing } no tiene nombre
rename-the-last-speaker = el último que habló
rename-nothing = nada que renombrar
rename-cannot = { $thing } no se puede renombrar
rename-already = ya se está renombrando
rename-nobody-spoke = nadie ha hablado

## P: the way to the thing chosen, and guiding the player along it

how-water = agua
how-bush = arbusto
how-boulder = peñasco
how-rock = roca
how-waterfall = cascada
how-slope = pendiente
how-bumpy-slope = pendiente escarpada
how-push = empujar
how-ride-the-current = dejarse llevar por la corriente
how-get-off = bajarse
how-smash = romper
p-current = dejarse llevar por la corriente { $bearing }
p-to-push = para empujar al { $way }:
p-nudge = un poco al { $way }
p-nudge-then = { $nudge }, luego
p-lined-up = alineado
p-then-cant-continue = luego no se puede seguir
p-cant-continue = no se puede seguir
p-in-the-way = { $thing } bloquea el paso
p-surface = sal a la superficie y ve al punto de buceo { $bearing }
p-and-ice-cleared = y el hielo queda roto
p-ice-cleared = hielo roto
p-no-path-to = no hay ruta a { $thing }
p-no-route-in-game = aún no hay rutas en este juego
p-no-route-on-streets = no hay ruta en estas calles
p-gone = { $thing } ya no está

## Touch mode

touch-nothing = nada que tocar
touch-off = el táctil está desactivado
touch-not-listening = Azahar no recibe toques de pa3ds

## What the field shows without words

dowsing = Zahorí { $light }
dowsing-blue = azul
dowsing-green = verde
dowsing-yellow = amarillo
dowsing-red = rojo
dowsing-nothing = nada
decorating-placeable = colocable
decorating-not-placeable = no colocable

## The Pokédex's habitat map

habitat-found = Se encuentra en { $place }
habitat-grass = hierba
habitat-water = agua
habitat-fishing = pesca

## Super Training and Pokémon-Amie

training-goal = portería
training-red-goal = portería roja
training-shot-coming = disparo en camino
training-left = quedan { $time }
amie-stroke = acariciar a { $pokemon }

## What pa3ds says of itself

app-waiting = Esperando al juego
app-ready = pa3ds listo
app-restarted = pa3ds reiniciado
app-reader-error = Error de lectura
dialog-ok = Aceptar
dialog-cancel = Cancelar

## As pa3ds starts: its keys, its settings and Azahar's

both = { $these } y { $last }
keys-taken = pa3ds usa { $controls }
keys-applied = teclas aplicadas a Azahar
keys-waiting = cierra Azahar y vuelve a iniciar pa3ds para aplicar tus teclas
keys-commented-out = nueva tecla comentada en { $file }: { $key } para { $action }, porque { $key } ya es { $taken }
keys-edit-to-choose = edita { $file } para elegir
file-unreadable = no se pudo leer { $file }: { $problem }. Corrígelo, o bórralo para que pa3ds escriba uno nuevo
azahar-settings-applied = ajustes de Azahar aplicados
azahar-settings-waiting = cierra Azahar y vuelve a iniciar pa3ds para aplicar sus ajustes
azahar-settings-missing = no se encontraron los ajustes de Azahar: inicia Azahar una vez, ciérralo y vuelve a iniciar pa3ds
azahar-folder-ask = No se encontraron los ajustes de Azahar. Elige la carpeta donde está azahar.exe, o cancela si Azahar está instalado
azahar-folder-again = { $problem }, elige otra carpeta o cancela
azahar-folder-no-exe = { $folder } no contiene azahar.exe
azahar-folder-not-portable = { $folder } no tiene una carpeta user junto a su azahar.exe, así que este Azahar guarda sus ajustes en AppData y no a su lado

## What a thing on the map is, where neither the game nor a name says

kind-door = puerta
kind-way-out = salida
kind-warp = transporte
kind-path = camino
kind-edge = borde
kind-stairs = escaleras
kind-stairs-up = escaleras arriba
kind-stairs-down = escaleras abajo
kind-passage = pasaje
kind-entrance = entrada
kind-teleporter = teletransportador
kind-elevator = ascensor
kind-sign = cartel
kind-flag = bandera
kind-object = objeto
kind-person = persona
kind-item-ball = objeto en el suelo
kind-barrier = barrera
kind-hole = agujero
kind-dive-spot = punto de buceo
kind-current = corriente
kind-flat-ground = suelo llano
kind-slope = pendiente
kind-bumpy-slope = pendiente escarpada
kind-rail = raíl
kind-ice = hielo
kind-blank = vacío
kind-wall = pared
kind-crumbled-cliff = acantilado derrumbado

## Battle

battle-battler = { $name } Nv{ $level } { $hp }
battle-share = { $percent } %

## Scenes

narration = Narración
