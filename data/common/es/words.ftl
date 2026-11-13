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
way-up = arriba
way-down = abajo
steps = { $count } { $way }
leg-with = { $how } { $steps } { $way }
here = aquí
above = { $count } más arriba
below = { $count } más abajo
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
category-climbing = escalada
category-currents = corrientes
category-arrows = flechas
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
key-battle-not-shown = aún no se muestra nada
key-nothing-said = aún no se ha dicho nada
key-one-way = solo un camino
out-at = salida a { $bearing }
past-teleporter = tras un teletransportador
leads-to = { $thing } a { $place }
sign-saying = cartel { $text }
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
how-rail = raíl
how-hop = saltar
how-bunny-hop = bunny hop
how-push = empujar
how-ride-the-current = dejarse llevar por la corriente
how-get-off = bajarse
how-smash = romper
how-climb = escalar
p-climb-up = escalar { $holds } arriba
p-climb-down = escalar { $holds } abajo
how-slide = deslizarse
how-grind-the-rail = deslizarse por el raíl
p-grind = deslizarse por el raíl { $bearing }
how-ride-the-belt = dejarse llevar por la cinta
p-belt = dejarse llevar por la cinta { $bearing }
how-arrow-pushes = la flecha empuja
p-arrow = la flecha empuja { $bearing }
p-switch-first = primero { $thing }, { $way }
how-climb-the-vine-up = trepar por la liana
how-climb-the-vine-down = bajar por la liana
how-swing-on-the-vine = balancearse en la liana
p-vine = mirar al { $way }, { $how }
p-more-vines = { $count ->
    [one] luego 1 liana más
   *[other] luego { $count } lianas más
}
p-more-teleporters = { $count ->
    [one] luego 1 teletransportador más
   *[other] luego { $count } teletransportadores más
}
how-up-the-ramp = sube la rampa
how-down-the-ramp = baja la rampa
p-current = dejarse llevar por la corriente { $bearing }
p-to-push = para empujar al { $way }:
p-nudge = un poco al { $way }
p-nudge-then = { $nudge }, luego
p-lined-up = alineado
p-then-cant-continue = luego no se puede seguir
p-cant-continue = no se puede seguir
p-in-the-way = { $thing } bloquea el paso
p-requires = requiere { $move }
p-surface = sal a la superficie y ve al punto de buceo { $bearing }
p-into = { $leg } hacia { $place }
p-and-ice-cleared = y el hielo queda roto
p-ice-cleared = hielo roto
p-no-path-to = no hay ruta a { $thing }
p-no-route-in-game = aún no hay rutas en este juego
p-no-route-on-streets = no hay ruta en estas calles
p-gone = { $thing } ya no está

## Touch mode

touch-nothing = nada que tocar
touch-no-drag = nada que arrastrar
touch-continue = continuar
touch-off = el táctil está desactivado
touch-not-listening = Azahar no recibe toques de pa3ds
touch-holding = sujetando { $target }
touch-dropped = soltado en { $target }

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
training-barrier = barrera
training-shot-coming = disparo en camino
training-left = quedan { $time }
training-empty-bag = saco vacío
training-bag-finished = { $pokemon } ha terminado el Entrenamiento Base
training-resting = { $pokemon } está descansando
training-total = total
training-total-points = total { $points } de { $full }
training-charged = cargado
training-fizzled = se apagó
training-blocked = bloqueado
training-at-start = en la salida
training-block-now = ahora
training-energy = energía
training-no-numbers = aún sin datos
training-medals = { $count ->
    [one] 1 medalla
   *[other] { $count } medallas
}
amie-beckon = llamar a { $pokemon }
amie-visitor = { $pokemon } de visita
amie-stylus = lápiz
amie-stylus-on = lápiz, { $part }
amie-stylus-edge = borde
amie-view-down = hacer bajar a { $pokemon }
amie-view-up = devolver arriba a { $pokemon }
amie-view-came-down = { $pokemon } ha bajado
amie-view-went-up = { $pokemon } ha vuelto arriba
amie-mood-coming = { $pokemon } se acerca a ti
amie-mood-back-turned = { $pokemon } está de espaldas
amie-mood-turned-round = { $pokemon } se ha dado la vuelta
amie-mood-dozing = { $pokemon } está dando cabezadas
amie-mood-asleep = { $pokemon } se ha dormido
amie-mood-awake = { $pokemon } se ha despertado
amie-mood-too-full = { $pokemon } no puede comer más
amie-mood-puff-seen = { $pokemon } no le quita ojo al { $puff }
amie-mood-reaching = { $pokemon } intenta alcanzar el { $puff }
amie-mood-eating = { $pokemon } está comiendo
amie-mood-eaten = { $pokemon } se ha terminado el { $puff }
amie-mood-puff-fell = el { $puff } se ha caído
amie-mood-picky = { $pokemon } frunce el ceño y se aparta
amie-mood-watching = { $pokemon } está mirando tu mano
amie-mood-looking-around = { $pokemon } está mirando a su alrededor
amie-mood-hand-raised = { $pokemon } está levantando la mano
amie-mood-your-face = { $pokemon } se acerca a tu cara
amie-high-five = ¡Chócala!
amie-mood-disliked = A { $pokemon } no le ha gustado...
amie-mood-closer = { $pokemon } está loco de alegría, y brotan corazones
amie-mood-best-friends = { $pokemon } salta como loco en una lluvia de corazones
amie-meter = { $label } { $count } de 5
amie-hearts = { $count ->
    [one] 1 corazón
   *[other] { $count } corazones
}
amie-part-off = fuera de { $pokemon }
amie-part-head = cabeza
amie-part-body = cuerpo
amie-part-right-eye = ojo derecho
amie-part-left-eye = ojo izquierdo
amie-part-eye = ojo
amie-part-right-ear = oreja derecha
amie-part-left-ear = oreja izquierda
amie-part-hand = mano
amie-part-spikes = pinchos
amie-part-ghostly = fantasmal
amie-part-flame = llama
amie-part-sparks = chispas
amie-part-slime = baba
amie-part-sludge = lodo
amie-part-ice = hielo
amie-part = parte
puzzle-cell = { $column }{ $row }
puzzle-home = { $tile }, en su sitio
puzzle-goes = { $tile }, va a { $home }
puzzle-picture-left = { $pokemon } a la izquierda
puzzle-picture-front = { $pokemon } delante
puzzle-picture-right = { $pokemon } a la derecha
puzzle-picked = { $tile }, elegida
berry-picker-wants = { $pokemon } quiere { $berry }
berry-picker-for = { $berry } para { $pokemon }
berry-picker-wanted = se pide { $berry }
berry-picker-lineup = { $first }, { $second } y { $third }
berry-picker-not-up = { $pokemon } no ha salido
berry-picker-no-berry = sin baya { $number }
berry-picker-no-pokemon = sin Pokémon { $number }
head-it-ball = { $pokemon }
head-it-ball-nobody = ovillo
head-it-no-pokemon = sin Pokémon { $number }
head-it-no-score = sin puntuación
head-it-lineup = { $first }, { $second } y { $third }
head-it-estimate = pa3ds calcula que tu anticipación es de { $seconds } segundos, con { $taps } toques

## What pa3ds says of itself

app-waiting = Esperando al juego
app-ready = pa3ds listo
app-restarted = pa3ds reiniciado
app-reader-error = Error de lectura
dialog-ok = Aceptar
dialog-cancel = Cancelar
window-title = Pokémon Access 3DS
window-menu-mod = &Mod
window-updates = Buscar act&ualizaciones
window-exit = &Salir
window-menu-azahar = &Azahar
window-azahar-portable = Descargar Azahar (&portable)
window-azahar-installer = Descargar Azahar (&instalador)
window-menu-help = Ay&uda
window-documentation = &Documentación...
window-keys = Comandos de &teclado...
window-amie = &Poké Recreo...
window-known-issues = Problemas c&onocidos...
window-translating = T&raducción...
window-changelog = Registro de ca&mbios...
window-settings-guide = Guía de &ajustes...
window-issues = Informar de un problema en &GitHub
setting-pa3ds-keys = Responder a las teclas del mod
setting-pa3ds-keys-about = Responder a las teclas de pa3ds y asignar los botones de Azahar a teclas, como indica keys.toml.
setting-pa3ds-poll_rate = Lecturas por segundo
setting-pa3ds-poll_rate-about = Cuántas veces por segundo leer el juego: 60, la velocidad de fotogramas del propio juego; 30 para un equipo más lento.
setting-pa3ds-azahar_folder = Carpeta de Azahar portable
setting-pa3ds-azahar_folder-about = La carpeta donde está un Azahar portable, la que contiene azahar.exe y una carpeta user; vacía para un Azahar instalado de la forma habitual, cuyos ajustes están en AppData. pa3ds la rellena cuando eliges la carpeta que te pide.
setting-pa3ds-hide_window = Ocultar la ventana del mod mientras Azahar está abierto
setting-pa3ds-hide_window-about = Ocultar la ventana de pa3ds, la que se abre al hacer doble clic en pa3ds, mientras Azahar está abierto, para que nada de pa3ds quede en Alt+Tab entre tú y el juego; con false sigue a la vista, para que OBS grabe a través de ella los sonidos de pa3ds.
setting-pa3ds-ui-pauses = Pausas en el texto de pantalla
setting-pa3ds-ui-pauses-about = Poner pausas entre los fragmentos de texto de pantalla que se dicen juntos: dos puntos tras una etiqueta, como en "EO: Raiden", y una coma entre los demás.
setting-pa3ds-ui-interrupt = Los menús interrumpen la voz
setting-pa3ds-ui-interrupt-about = Cortar la voz cuando se mueve el cursor de un menú, en lugar de ponerla en cola; las teclas propias de pa3ds siempre lo hacen.
setting-pa3ds-ui-unfocused = Hablar con Azahar en segundo plano
setting-pa3ds-ui-unfocused-about = Seguir diciendo lo que muestra el juego mientras otra ventana está delante; si no, pa3ds se queda en silencio entonces, salvo por sus propias teclas y mensajes.
setting-pa3ds-ui-descriptions = Descripciones de escenas
setting-pa3ds-ui-descriptions-about = Describir las escenas que el juego muestra sin palabras, como el despertar de Groudon. Cada descripción se dice después de la línea de texto a la que sigue la escena. Por ahora solo se describen algunas escenas.
setting-pa3ds-ui-trainer_party = Cuántos Pokémon tiene un entrenador
setting-pa3ds-ui-trainer_party-about = Al empezar un combate contra un entrenador, tras la línea en que te desafía, decir cuántos Pokémon tiene ("text"), sonar un pitido por cada uno ("beeps") o ninguna de las dos cosas ("off"). Los pitidos suenan tan fuerte como los tonos de [pa3ds.audio].
setting-pa3ds-ui-trainer_party-text = Decirlo
setting-pa3ds-ui-trainer_party-beeps = Un pitido por cada uno
setting-pa3ds-ui-trainer_party-off = Ninguna de las dos
setting-pa3ds-ui-language = Idioma
setting-pa3ds-ui-language-about = El idioma en que pa3ds dice sus propias palabras, como el nombre de una carpeta de sus datos: en, fr, de, es, it, ja, ko, zh-CN o zh-TW. Vacío sigue el idioma del propio juego, que pa3ds lee de la partida guardada; cámbialo donde sea incorrecto, como en una traducción de fans sobre otro idioma, y en Sol y Luna, cuya partida pa3ds no puede leer, donde también es el idioma del juego.
setting-pa3ds-audio-tones = Volumen relativo de los tonos
setting-pa3ds-audio-tones-about = Lo fuerte que suena el tono corto al moverse el cursor de un mapa o del modo táctil, más agudo cuanto más arriba y más hacia la derecha cuanto más a la derecha, para indicar dónde está el cursor: 1 tan fuerte como el juego, 0.5 la mitad, 1.5 más fuerte, 0 nada; en el diálogo de ajustes, en porcentaje, 100 tan fuerte como el juego.
setting-pa3ds-audio-footsteps = Volumen relativo de los pasos
setting-pa3ds-audio-footsteps-about = Lo fuertes que suenan los pasos propios de pa3ds en el suelo donde el juego no reproduce ninguno, como calles, suelos interiores y cuevas: 1 tan fuerte como los reproduciría el juego, 0.5 la mitad, 1.5 más fuerte, 0 nada; en el diálogo de ajustes, en porcentaje, 100 tan fuerte como el juego.
setting-pa3ds-audio-ticks = Volumen relativo del tictac del reloj
setting-pa3ds-audio-ticks-about = Lo fuerte que suena un tono corto en cada segundo del reloj de un juego, como en los juegos del Poké Recreo; el propio reloj se dice cada medio minuto y en sus últimos segundos: 1 tan fuerte como el juego, 0.5 la mitad, 1.5 más fuerte, 0 nada, como viene de serie; en el diálogo de ajustes, en porcentaje, 100 tan fuerte como el juego.
setting-pa3ds-audio-feedback_sounds = Volumen relativo de los sonidos de respuesta
setting-pa3ds-audio-feedback_sounds-about = Lo fuertes que suenan los sonidos cortos del propio juego para lo que ha hecho una tecla: los de las cajas del PC al coger y dejar un Pokémon, cuando el modo táctil coge algo para arrastrarlo y lo suelta, y el sonido de error del juego para una tecla que no tiene nada que hacer, como una baya que la ronda de Recogebayas no tiene; y el aviso de Usa la cabeza. A 0, pa3ds lo dice en su lugar y no da el aviso, "sujetando", "soltado en" y el motivo: 1 tan fuerte como el juego, 0.5 la mitad, 1.5 más fuerte; en el diálogo de ajustes, en porcentaje, 100 tan fuerte como el juego.
setting-pa3ds-navigation-positions = Anunciar la posición en la lista
setting-pa3ds-navigation-positions-about = Decir en qué lugar de la lista está algo, como en "puerta, 1 de 3".
setting-pa3ds-navigation-position_first = Posición en la lista antes de las direcciones
setting-pa3ds-navigation-position_first-about = Decir el lugar en la lista antes del camino hasta él, "puerta, 1 de 3, 1 norte"; con false dice "puerta, 1 norte, 1 de 3".
setting-pa3ds-navigation-turn_by_turn = Indicaciones giro a giro
setting-pa3ds-navigation-turn_by_turn-about = Una vez que P ha encontrado el camino, decir cada giro al llegar a él; con false dice el camino una vez, y P lo vuelve a decir desde donde estás.
setting-pa3ds-navigation-guide_to_hidden = Guiar hacia los hallazgos del DexNav
setting-pa3ds-navigation-guide_to_hidden-about = Cuando una búsqueda del DexNav encuentra el Pokémon buscado, o el DexNav encuentra uno por sí solo mientras está abierto y estás en modo táctil, buscar el camino hasta él como hace P y mantener P en él, estuviera donde estuviera la lista; uno encontrado mientras el CartoNav está abierto se deja como está.
setting-pa3ds-navigation-track_boulders = Seguir el movimiento de las rocas
setting-pa3ds-navigation-track_boulders-about = Decir cuánto se ha movido una roca grande desde donde la coloca su sala al principio, como en "roca grande, 2 este, movida 1 norte", para que sepas cuáles has empujado; el juego muestra dónde está cada roca, nunca dónde empezó.
setting-pa3ds-navigation-hide_door_signs = Ocultar los carteles de puertas
setting-pa3ds-navigation-hide_door_signs-about = Dejar fuera de la lista un cartel cuando lo único que dice es el nombre de una puerta que tiene al lado, como el de la casa de Erico en el apartamento 13 de los pisos de Malvalona; un cartel que dice más, como el de un gimnasio, se queda.
setting-pa3ds-navigation-show_y = Anunciar la altura
setting-pa3ds-navigation-show_y-about = Decir también la altura, como y después de x y z, cuando C dice dónde estás y Mayús+C dónde está lo elegido: "x 71, z 35, y 20"; en las unidades del propio juego, 18 por el ancho de una casilla, y se omite donde pa3ds no la conoce, como en Sol y Luna.
setting-pa3ds-navigation-sort_by_distance = Ordenar por distancia
setting-pa3ds-navigation-sort_by_distance-about = Ordenar la lista del modo lista de lo más cercano a lo más lejano, lo que cambia mientras caminas; con false cada cosa tiene un sitio fijo en ella, el orden de las tablas del propio juego, para que sepas en qué casa has estado por el lugar que ocupa; el modo dirección va de lo más cercano a lo más lejano en ambos casos.
setting-pa3ds-touch-wrap = Volver al principio
setting-pa3ds-touch-wrap-about = En modo táctil, pasar del último botón al primero, y de la fila de abajo a la de arriba; con false se detiene en los extremos, así que pulsar O unas cuantas veces siempre acaba en la fila de abajo.
setting-pa3ds-touch-next_row = Pasar a la fila siguiente
setting-pa3ds-touch-next_row-about = En modo táctil, J y L pasan a la fila de arriba o de abajo al final de una fila; con false se quedan en su fila, así que pulsar L unas cuantas veces siempre acaba a su derecha.
setting-pa3ds-touch-drag = Cómo arrastrar
setting-pa3ds-touch-drag-about = Arrastrar con Mayús pulsada, o con una tecla propia asignada a arrastrar en keys.toml, soltando el toque al levantarla ("hold"), o pulsarla para coger y pulsarla otra vez para soltar ("toggle"). Solo se arrastra en el Poké Recreo.
setting-pa3ds-touch-drag-hold = Mantener Mayús
setting-pa3ds-touch-drag-toggle = Pulsar para coger, pulsar para soltar
setting-pa3ds-amie-descriptions = Describir lo que hace el Pokémon
setting-pa3ds-amie-descriptions-about = En el Poké Recreo, decir lo que se ve hacer a tu Pokémon cuando empieza y cuando termina: mirar a su alrededor, mirar tu mano, comer, levantar la mano para chocarla. Cada una de sus animaciones se describe mientras se reproduce, con una frase propia de ese Pokémon. También dice los corazones que trae una caricia, y cuándo aciertas al chocarla.
setting-pa3ds-amie-fidgets = Describir los movimientos espontáneos del Pokémon
setting-pa3ds-amie-fidgets-about = Con las descripciones del Poké Recreo activadas, dice también los pequeños movimientos que tu Pokémon hace por sí solo cada pocos segundos. Desactivado, solo se dice cómo reacciona a ti.
setting-pa3ds-amie-berry_first = Recogebayas dice primero la baya
setting-pa3ds-amie-berry_first-about = Decir cada Pokémon que quiere una baya en el orden en que se la das, la baya y luego el Pokémon, "Baya Pinia para Pikachu"; con false dice primero el Pokémon, "Pikachu quiere Baya Pinia".
setting-pa3ds-amie-one_at_a_time = Recogebayas dice un Pokémon cada vez
setting-pa3ds-amie-one_at_a_time-about = Decir un solo Pokémon que quiere una baya cada vez, y el siguiente cuando le hayas dado una baya o el que se ha dicho se haya ido; con false dice cada uno según llega.
setting-pa3ds-amie-head_it_cue = Usa la cabeza avisa de cuándo tocar
setting-pa3ds-amie-head_it_cue-about = En Usa la cabeza, reproducir el propio sonido "!" del juego cuando llega el momento de tocar un ovillo, tanto antes de que el ovillo llegue a su Pokémon como diga la anticipación de abajo, en el oído del lado de ese Pokémon y en los dos para el del centro; necesita los sonidos de respuesta activados.
setting-pa3ds-amie-head_it_lead = Anticipación del aviso de Usa la cabeza
setting-pa3ds-amie-head_it_lead-about = Cuántos segundos antes de que un ovillo llegue a su Pokémon suena el aviso: más o menos lo que tardas desde oírlo hasta pulsar la tecla del Pokémon. Al final de cada ronda pa3ds dice su cálculo de la tuya, para ponerla aquí. Súbela si tocas demasiado tarde, bájala si tocas demasiado pronto.
setting-pa3ds-amie-berry_picker_style = Cómo distingue Recogebayas a los Pokémon
setting-pa3ds-amie-berry_picker_style-about = En Recogebayas, decir un Pokémon por su nombre ("name"), por su propio grito ("cry") o por ambos ("both"). El grito llega desde su lugar en la alineación de la ronda, aparezca donde aparezca: el primero en el oído izquierdo, el segundo en los dos, el tercero en el derecho. El grito necesita los sonidos de respuesta activados; donde no puede sonar, se dice el nombre.
setting-pa3ds-amie-berry_picker_style-name = Nombre
setting-pa3ds-amie-berry_picker_style-cry = Grito
setting-pa3ds-amie-berry_picker_style-both = Nombre y grito
setting-pa3ds-amie-head_it_style = Cómo distingue Usa la cabeza a los Pokémon
setting-pa3ds-amie-head_it_style-about = En Usa la cabeza, decir el Pokémon al que cae un ovillo por su nombre ("name"), por su propio grito ("cry"), por ambos ("both") o por ninguno ("none"), para quedarse solo con el aviso. El grito llega desde su lugar en la alineación de la ronda: el primero en el oído izquierdo, el segundo en los dos, el tercero en el derecho. El grito necesita los sonidos de respuesta activados; donde no puede sonar, se dice el nombre.
setting-pa3ds-amie-head_it_style-name = Nombre
setting-pa3ds-amie-head_it_style-cry = Grito
setting-pa3ds-amie-head_it_style-both = Nombre y grito
setting-pa3ds-amie-head_it_style-none = Ninguno
setting-pa3ds-amie-head_it_score = Usa la cabeza dice la puntuación al cambiar
setting-pa3ds-amie-head_it_score-about = En Usa la cabeza, decir la puntuación, el combo y cada fallo cuando llegan, la puntuación y el combo una vez mientras siguen subiendo; false los deja a la tecla de puntuación de Usa la cabeza (R), que los dice con el tiempo o las oportunidades que quedan cada vez que la pulses.
setting-pa3ds-extras-conditions = Condiciones de concurso como números
setting-pa3ds-extras-conditions-about = Decir las condiciones de concurso de un Pokémon como números donde el juego las dibuja como una estrella, en la ficha del Pokémon y en el Kit de Pokécubos: la estrella se muestra a todos pero pa3ds no puede leerla, así que sus números son todo lo que puede dar, y son más exactos que la estrella.
setting-pa3ds-training-shot_cue = Cuando un Globot dispara
setting-pa3ds-training-shot_cue-about = En un Superentrenamiento, cómo oyes que un Globot dispara: el sonido de alerta del juego, las palabras «disparo en camino» o nada. Sin sonidos de respuesta, se dice con palabras.
setting-pa3ds-training-shot_cue-sound = Sonido
setting-pa3ds-training-shot_cue-speech = Voz
setting-pa3ds-training-shot_cue-off = No
setting-pa3ds-training-block_cue = Cuándo bloquear o esquivar
setting-pa3ds-training-block_cue-about = En un Superentrenamiento, cómo oyes que un disparo está a punto de llegar: el «!» del juego, la palabra «ahora» o nada. Sin sonidos de respuesta, se dice con palabras.
setting-pa3ds-training-block_cue-sound = Sonido
setting-pa3ds-training-block_cue-speech = Voz
setting-pa3ds-training-block_cue-off = No
setting-pa3ds-training-block_lead = Antelación del aviso de bloqueo
setting-pa3ds-training-block_lead-about = Cuántos segundos antes de que llegue un disparo se da el aviso. Una barrera dura menos de un segundo desde que pulsas L.
setting-pa3ds-extras-base_stats = Estadísticas del Superentrenamiento en números
setting-pa3ds-extras-base_stats-about = R dice en la página de Superentrenamiento los puntos de cada estadística, que el juego dibuja como gráfico, en vez de una proporción aproximada.
setting-pa3ds-extras-hidden_items = Objetos ocultos
setting-pa3ds-extras-hidden_items-about = Incluir entre los objetos los objetos ocultos que quedan por encontrar, como "objeto oculto": solo bajo el mar ("underwater"), donde no se puede usar el Zahorí y un jugador que ve puede distinguir un destello entre las rocas y las algas, o en todas partes ("all"), lo que revela lo que el juego oculta a todos; con "off" no se incluye ninguno.
setting-pa3ds-extras-hidden_items-off = Desactivado
setting-pa3ds-extras-hidden_items-underwater = Bajo el mar
setting-pa3ds-extras-hidden_items-all = En todas partes
settings-title = Ajustes del mod
settings-page-general = General
settings-page-speech = Voz
settings-page-audio = Audio
settings-page-navigation = Navegación
settings-page-touch = Modo táctil
settings-page-amie = Poké Recreo
settings-page-training = Superentrenamiento
settings-page-extras = Extras
settings-browse = &Examinar...
settings-language-game = El idioma del propio juego
settings-restart = ¿Reiniciar pa3ds ahora para que se apliquen estos ajustes? Si eliges No, se aplicarán la próxima vez que se inicie pa3ds.
settings-unwritten = pa3ds no pudo escribir { $path }, así que no ha cambiado nada.
dialog-apply = &Aplicar
window-settings = &Ajustes...

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
kind-way-down = bajada
kind-conveyor-belt = cinta transportadora
kind-switch = interruptor
kind-pink-switch = interruptor rosa
kind-blue-switch = interruptor azul
kind-yellow-switch = interruptor amarillo
kind-green-switch = interruptor verde
kind-warp = transporte
kind-path = camino
kind-edge = camino
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
kind-hidden-item = objeto oculto
kind-barrier = barrera
kind-hole = agujero
kind-dive-spot = punto de buceo
kind-current = corriente
kind-water = agua
kind-waterfall = cascada
kind-arrow-east = flecha al este
kind-arrow-west = flecha al oeste
kind-arrow-north = flecha al norte
kind-arrow-south = flecha al sur
kind-flat-ground = suelo llano
kind-open-ground = terreno abierto
kind-quiz-panel = panel del concurso
kind-blue-spot = punto azul
kind-wall-ledge = saliente del muro
kind-vine-up = liana hacia arriba
kind-vine-down = liana hacia abajo
kind-swinging-vine = liana para balancearse
kind-gym-leader-spot = lugar del líder de gimnasio
kind-north-waiting-spot = punto de espera norte, junto al callejón sin salida
kind-south-waiting-spot = punto de espera sur, junto a la entrada del laberinto
kind-east-waiting-spot = punto de espera este, extremo este del camino central
kind-west-waiting-spot = punto de espera oeste, extremo oeste del camino central
furfrou-wait = siguiente paso, { $shauna } espera aquí
furfrou-fetch = siguiente paso, habla con { $shauna } para que te siga
furfrou-chase = siguiente paso, camina hasta aquí y { $furfrou } huye
furfrou-keep-off = no pises aquí, { $furfrou } se escapa desde aquí
furfrou-leave = siguiente paso, sal del laberinto por aquí
furfrou-enter = siguiente paso, vuelve a entrar en el laberinto
kind-machine = máquina
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
battle-party = { $count } Pokémon

## Scenes

narration = Narración
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = Español

registered-slot = { $way } { $item }
registered-up = arriba
registered-right = derecha
registered-down = abajo
registered-left = izquierda

bag-moving = moviendo { $item }
bag-between = entre { $above } y { $below }
bag-above = encima de { $below }
bag-below = debajo de { $above }
