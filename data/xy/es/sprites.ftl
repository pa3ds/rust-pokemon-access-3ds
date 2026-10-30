### What to call the people, Pokémon and things on X and Y's maps in Spanish, and the use tiles,
### furniture and slots they stand among, when nothing better is known.
###
### The same as data/en/sprites.ftl, in this language: a key is the sprite a move model is
### drawn with (findings, spikes 75, 89 and 100), and a name is looked for in order — what the
### player has called them, what the game calls them out of the player's own ROM, and only
### then this file. So it is heard only when pa3ds cannot read the ROM.
###
### Each table names what the English one does (Orion, 2026-10-14). Most words are the ROM's
### own in this language, out of the line the English word's comment names. Where the ROM has
### none, as for the Nurse, the people and looks the English table calls by what their sprite's
### id says, and the sparkles and cones, the word is pa3ds's own, as the English one is.
###
### Past [sprites], a key of [tiles] is a kind of use tile, a key of [furniture] the shared script a
### piece of furniture runs, and a key of [places] a slot, "zone:furniture:index" (spikes 83, 210,
### 211 and D).
###
### Nor has the Spanish text a word for Mom: it calls the player's mother only "tu madre", your
### mother (spike 108). "Mamá" is pa3ds's own, the word Omega Ruby's Spanish calls her by.

## sprites
# People the ROM never lists as trainers, read off lines of its own text that name them —
# the same line in every language (spikes/108_x_people.py lines).
# Mom: MAMA, alone in the player's house
sprite-0x7 = Mamá
# Looker: "It's me! Looker!" (system 102:16)
sprite-0x26 = Handsome
# Alexa: "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = Alexia
# Bonnie: "You can call me Bonnie." (story 404:0)
sprite-0x9b = Clem
# the Mega Evolution guru: "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = Megayayo
# the Game Director: "Hello! I am the Game Director." (story 272:0)
sprite-0x98 = director del juego
# Nurse: PCWOMAN1, at the counter in all 16 Pokémon Centers (spike 101)
sprite-0x3b = Enfermera

# The game's own words for the story's people and for kinds of trainer, as `archives`
# reads them (findings, spikes 90 and 108): spikes/108_x_people.py table es.
sprite-0x3 = Xana
sprite-0x4 = Beni
sprite-0x5 = Trovato
sprite-0x6 = Ciprés
sprite-0xa = Dianta
sprite-0xb = Tileo
sprite-0xc = Malva
sprite-0xd = Drácena
sprite-0xe = Narciso
sprite-0xf = Violeta
sprite-0x10 = Lino
sprite-0x11 = Corelia
sprite-0x12 = Amaro
sprite-0x13 = Lem
sprite-0x14 = Valeria
sprite-0x15 = Ástrid
sprite-0x16 = Édel
sprite-0x17 = Lysson
sprite-0x21 = A. Z.
sprite-0x93 = Kalm
sprite-0x94 = Serena
sprite-0x9c = Lysson
sprite-0x9f = Kalm
sprite-0xa0 = Serena
sprite-0x1a = Team Flare Recluta
sprite-0x1b = Team Flare Recluta
sprite-0x28 = Preescolar
sprite-0x2a = Preescolar
sprite-0x2c = Estudiante
sprite-0x30 = Estudiante
sprite-0x34 = Turista
sprite-0x35 = Mochilero
sprite-0x43 = Joven
sprite-0x44 = Pokéfan
sprite-0x47 = Chica
sprite-0x4a = Luchadora
sprite-0x4b = Patinadora
sprite-0x4c = Patinador
sprite-0x4d = Pescador
sprite-0x4e = Entrenadora Promesa
sprite-0x4f = Entrenadora Guay
sprite-0x50 = Veterana
sprite-0x52 = Nadadora
sprite-0x55 = Turista
sprite-0x57 = Criapokémon
sprite-0x58 = Médium
sprite-0x59 = Entrenador Promesa
sprite-0x5b = Veterano
sprite-0x5d = Pokémon Ranger
sprite-0x5f = Científico
sprite-0x61 = Operario
sprite-0x64 = Niña Soñadora
sprite-0x67 = Jardinero
sprite-0x70 = Montañero
sprite-0xa3 = Nadador
sprite-0xa4 = Nadadora

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# Mr. Bonding: KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4)
sprite-0x77 = Mr. Buen Rollo
# Dexio: MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = Dexio
# Sina: MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar, "I came in place of Dexio" (256:1)
sprite-0x8c = Sina
# Aliana: SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = Amapola
# Essentia: C02GIRL_EX: Lysandre Labs and Lumiose, Looker's story; trainer 511 (inferred from the id)
sprite-0x9e = Esprit
# Emma: C02GIRL: Lumiose, Looker's story; "Looker, you, Emma, Mimi..." (story 347:9) (inferred)
sprite-0x9d = Matière
# Sina: MYSTERYW: Geosenge's masked heroes, "Who are these Sina and Dexio you speak of?" (story 35:25) (inferred)
sprite-0x8 = Sina
# Dexio: MYSTERYM: likewise (inferred)
sprite-0x9 = Dexio

# Team Flare's admins and scientists: the admins by their trainers' class and name, as
# the grunts are; the scientists as story 378:5 calls them, not yet told apart.
# Team Flare Admin: FLAREKANBU1: the Power Plant, the Secret HQ; trainers "Admin" of class Team Flare
sprite-0x18 = Team Flare Comandante
# Team Flare Admin: FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = Team Flare Comandante
# Team Flare Scientist: SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = Científico del Team Flare
# Team Flare Scientist: SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = Científico del Team Flare
# Team Flare Scientist: SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = Científico del Team Flare
# Team Flare Scientist: SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = Científico del Team Flare

# Shared looks the game never names, called what the sprite's id says they are (text
# file 76), in Omega Ruby's words for the same look where it has one. Left out:
# NONDRAW, STOPPER, the work objects WKOBJCODE00 to 04, and ENEMY3D and GIANTMAN02,
# which no line names.
# Little girl: CHILDGIRL1: Vaniville, Aquacorde
sprite-0x27 = Niña
# Little boy: CHILDBOY1: 7 zones
sprite-0x29 = Niño
# Girl: GIRL1: 10 zones
sprite-0x2b = Muchacha
# Girl: GIRL3: 17 zones
sprite-0x2e = Muchacha
# Boy: BOY1: 3 zones
sprite-0x2d = Chico
# Boy: BOY2: 13 zones
sprite-0x32 = Chico
# Woman: WOMAN1: 33 zones
sprite-0x2f = Mujer
# Woman: WOMAN2: 34 zones, one Beauty among them
sprite-0x38 = Mujer
# Man: MAN1: 36 zones
sprite-0x31 = Hombre
# Middle-aged man: MIDDLEMAN2: 9 zones
sprite-0xa6 = Hombre de mediana edad
# Gentleman: GENTLEMAN: 45 zones; X has no such class, Omega Ruby's class
sprite-0x37 = Aristócrata
# Clerk: SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = Dependiente
# Clerk: SHOPW1: 11 zones
sprite-0x3f = Dependiente
# Gym guide: DANPEI: in every Gym
sprite-0x3d = Guía del Gimnasio
# Receptionist: INFORMATIONW: 37 zones, the gates
sprite-0x3e = Recepcionista
# Delivery person: DELIVER: in all 17 Pokémon Centers
sprite-0x40 = Repartidor
# Delivery person: DELIVER2: likewise
sprite-0xa1 = Repartidor
# Cameraman: CAMERAMAN: 14 zones
sprite-0x42 = Cámara
# Twin: TWIN: two pairs of Twins among 10 zones
sprite-0x46 = Gemela
# Twin: TWIN2: 4 zones
sprite-0xa2 = Gemela
# Hairdresser: HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = Peluquero
# Café owner: MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = Dueño del café
# Cyclist: CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = Ciclista
# Mannequin: HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = Maniquí
# Mannequin: HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = Maniquí
# Model: MODEL01: Route 7 (id only)
sprite-0xa7 = Modelo
# Model: MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = Modelo
# Model: MODEL03: likewise (id only)
sprite-0xa9 = Modelo

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# Punk Girl: BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = Chica Mala
# Poké Fan: MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = Pokéfan
# Artist: ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = Artista
# Artist: ARTISTW: one Artist
sprite-0x54 = Artista
# Pokémon Breeder: BREEDERM: one Pokémon Breeder
sprite-0x56 = Criapokémon
# Scientist: ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = Científica
# Worker: WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = Operario
# Furisode Girl: KIMONO1: one Furisode Girl
sprite-0x68 = Jovencita con Kimono
# Furisode Girl: KIMONO2: one Furisode Girl
sprite-0x69 = Jovencita con Kimono
# Furisode Girl: KIMONO3: one Furisode Girl
sprite-0x6a = Jovencita con Kimono
# Furisode Girl: KIMONO4: one Furisode Girl
sprite-0x6b = Jovencita con Kimono
# Lady: LADY: 19 zones, no trainer
sprite-0x36 = Damisela
# Waitress: WAITRESS: 24 zones, no trainer
sprite-0x49 = Camarera
# Garçon: GARCON: 25 zones, no trainer
sprite-0x72 = Garçon
# Chef: COOK: 12 zones, no trainer
sprite-0x5e = Cocinero
# Butler: BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = Mayordomo Leal
# Maid: MAID: 32 zones, the Pokémon Centers
sprite-0x66 = Criada
# Sky Trainer: SKYM: 8 zones, no trainer
sprite-0x6c = Entrenador Aéreo
# Sky Trainer: SKYW: 8 zones, no trainer
sprite-0x6d = Entrenadora Aérea
# Fisherman: FISHING2: 4 zones, as FISHING is
sprite-0xa5 = Pescador

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = Turista
# BADMAN
sprite-0x39 = Chico Malo
# TRAINERM
sprite-0x5a = Entrenador Guay
# RANGERM
sprite-0x5c = Pokémon Ranger
# OCCULTW
sprite-0x63 = Bruja
# FIGHTERM
sprite-0x71 = Karateka

# The Pokémon on the map, by the species each one's model draws, its tag read through
# the battle models (spike 218); with the ROM, its own species names. Mewtwo's model
# names none.
# Mewtwo: P_MODEL01: the Unknown Dungeon, "Mewtwo blended into the darkness of the cave" (story 185:1); its model names no species
sprite-0x1017 = Mewtwo
# EV_POKE01
sprite-0x1000 = Pumpkaboo
# EV_POKE02
sprite-0x1001 = Diggersby
# EV_POKE03
sprite-0x1002 = Zygarde
# EV_POKE04
sprite-0x1003 = Litleo
# EV_POKE05
sprite-0x1004 = Gogoat
# RIDE_POKE02
sprite-0x1007 = Mamoswine
# RIDE_POKE04
sprite-0x1008 = Skiddo
# EV_POKE07
sprite-0x1009 = Lucario
# EV_POKE08
sprite-0x100a = Furfrou
# EV_POKE10
sprite-0x100b = Fletchling
# EV_POKE11
sprite-0x100c = Snorlax
# EV_POKE12
sprite-0x100d = Espurr
# RIDE_POKE05
sprite-0x100f = Rhyhorn
# EV_POKE09
sprite-0x1010 = Abomasnow
# EV_POKE14
sprite-0x1011 = Skiddo
# EV_POKE15
sprite-0x1013 = Marill

# Things standing on the map that the game never names, called by a word of the line the shared
# script they run shows when the player faces one (a/0/3/1 file 27, story 130; spike 219), as
# ORAS's are (spike 137).
# en bush: ITREE01, runs 0x2712: story 130:3
sprite-0x200b = espinas
# en cracked boulder: IROCK01, runs 0x2710: story 130:1; ORAS's es word; X's story 130:1 is "Esta roca presenta algunas grietas"
sprite-0x200c = roca agrietada
# en big boulder: KROCK01, runs 0x2714: story 130:5
sprite-0x2009 = roca enorme
# en cracked wall: IWALL01, runs 0x2716: story 130:9
sprite-0x2034 = pared agrietada
# en rock: ISNOW01: ten on Route 17, where "all the rocks along the path are nothing against Mamoswine's strength!" (story 70:5)
sprite-0x2035 = roca
# en rock: IROCK02: ten on Route 9, crossed on Rhyhorn; no line, its model is it0014_rock (inferred), the word of story 70:5
sprite-0x200d = roca

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# en Central Kalos Pokédex award: AWARD04, case 0xd: story 55:4
sprite-0x203b = diploma de la Pokédex Kalos Centro
# en Coastal Kalos Pokédex award: AWARD03, case 0xe: story 55:5
sprite-0x203a = diploma de la Pokédex Kalos Costa
# en Mountain Kalos Pokédex award: AWARD05, case 0xf: story 55:6
sprite-0x203c = diploma de la Pokédex Kalos Montaña
# en Kalos region award: AWARD01, case 0x10: story 55:7
sprite-0x2036 = diploma de la región de Kalos
# en National Pokédex award: AWARD02, case 0x11: story 55:8
sprite-0x2037 = diploma de la Pokédex Nacional
# en Berry tree: KINOMI, runs 0x2939: story 142:15, "There are small Berries growing high up in the tree!"; Route 15
sprite-0x200e = árbol de bayas
# en Berry tree: KINOMI2, Routes 5 and 19
sprite-0x2011 = árbol de bayas
# en Berry tree: KINOMI3, Routes 11 and 12
sprite-0x2012 = árbol de bayas
# en Berry tree: KINOMI4, Routes 16 and 22
sprite-0x2013 = árbol de bayas
# en Berry tree: KINOMI11, Route 7
sprite-0x201a = árbol de bayas
# en Berry tree: KINOMI15, Route 18
sprite-0x201e = árbol de bayas
# en Berry tree: KINOMI31, Route 21
sprite-0x202e = árbol de bayas
# en Berry tree: KINOMI33, Route 10
sprite-0x2030 = árbol de bayas
# en Berry tree: KINOMI34, Route 6
sprite-0x2031 = árbol de bayas
# en Berry tree: KINOMI35, Routes 8 and 14
sprite-0x2032 = árbol de bayas
# en Berry tree: KINOMI36, Route 20
sprite-0x2033 = árbol de bayas

# Things the game never names, called what the ROM's own id for the sprite says they are.
# en sparkle: KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower; its model's joints are kira00 to kira03
sprite-0x203d = destello
# en sparkle: KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone, items 665 to 683 (text 96)
sprite-0x2042 = destello
# en cone: CONE, model com_bm_skatecone: two across a Lumiose street, "There is a fence blocking the road." (story 317:18), and seven in Lysandre Labs
sprite-0x203e = cono
# en shaking grass: TOBI_KUSA, no script: three on Route 6 and three on Route 18, rustling till a wild Pokémon "leaped out of the tall grass" (text 124:261)
sprite-0x2002 = hierba que se agita
# en lurking Pokémon: ENEMY3D, shared script 2000: nine in the Glittering Cave, drawn as nothing, a wild battle as the player comes up to one (spike 239)
sprite-0x95 = Pokémon al acecho

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 132, which says what is there; the word here is taken from
# that line (spike 220). The PC's word is read from the ROM ahead of this one, and the TV's is story
# 390:19's, since X has no line naming a TV alone.
# PC: the PC's own lines (archives.PC_LINES)
tile-0xd5 = PC
# TV: story 390:19, "a TV and a washing machine": X has no line naming a TV alone
tile-0xd7 = televisor
# Pokémon knickknacks: text 132:0
tile-0xd8 = artículos Pokémon
# Pokémon books: text 132:1, books and photo albums
tile-0xd9 = libros de Pokémon
# cabinet: text 132:2, ORAS's line and words
tile-0xda = mueble
# useful items: text 132:3
tile-0xdb = cosas útiles
# Pokémon novelties: text 132:4
tile-0xdc = mueble práctico
# trash can: text 132:6
tile-0xdd = papelera
# books for Pokémon: text 132:5, "Are these books for Pokémon to read?"
tile-0xde = libros para Pokémon

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# elevator: story 137:21 and 145:11, the elevators' own file: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = ascensor
furniture-0x27db = ascensor
furniture-0x27dc = ascensor
furniture-0x27dd = ascensor
furniture-0x27de = ascensor
furniture-0x27df = ascensor
furniture-0x27e0 = ascensor
furniture-0x27e1 = ascensor
furniture-0x27e2 = ascensor
furniture-0x27e3 = ascensor
furniture-0x27e4 = ascensor
furniture-0x27e5 = ascensor
furniture-0x27e6 = ascensor
furniture-0x27e7 = ascensor
furniture-0x27e8 = ascensor
furniture-0x27e9 = ascensor
furniture-0x27ea = ascensor
furniture-0x27eb = ascensor
furniture-0x27ec = ascensor
furniture-0x27ed = ascensor
furniture-0x27ee = ascensor
furniture-0x27ef = ascensor
furniture-0x27f0 = ascensor
furniture-0x27f1 = ascensor
furniture-0x27f2 = ascensor
furniture-0x27f3 = ascensor
furniture-0x27f4 = ascensor
furniture-0x27f5 = ascensor
furniture-0x27f6 = ascensor
furniture-0x27f7 = ascensor
furniture-0x27f8 = ascensor
furniture-0x27f9 = ascensor
furniture-0x27fa = ascensor
furniture-0x27fb = ascensor
furniture-0x27fc = ascensor
furniture-0x27fd = ascensor
furniture-0x27fe = ascensor
furniture-0x27ff = ascensor
furniture-0x2800 = ascensor
furniture-0x2801 = ascensor
furniture-0x2802 = ascensor
furniture-0x2803 = ascensor
furniture-0x2804 = ascensor
furniture-0x2805 = ascensor
furniture-0x2806 = ascensor
furniture-0x2807 = ascensor
furniture-0x2808 = ascensor
furniture-0x2809 = ascensor
# meteor: story 221:8, Ambrette Town
furniture-0x281e = meteorito
# cardboard boxes: story 220:0, Lumiose City, zone 0x54
furniture-0x284b = cajas de cartón
# photo spot: story 222:0, one in each of 12 places
furniture-0x28c8 = señal de vista panorámica
furniture-0x28c9 = señal de vista panorámica
furniture-0x28ca = señal de vista panorámica
furniture-0x28cb = señal de vista panorámica
furniture-0x28cc = señal de vista panorámica
furniture-0x28cd = señal de vista panorámica
furniture-0x28ce = señal de vista panorámica
furniture-0x28cf = señal de vista panorámica
furniture-0x28d0 = señal de vista panorámica
furniture-0x28d1 = señal de vista panorámica
furniture-0x28d2 = señal de vista panorámica
furniture-0x28d3 = señal de vista panorámica
# trash can: story 142:14, Pokémon Village and the Lost Hotel
furniture-0x28f0 = papelera
furniture-0x28f1 = papelera
furniture-0x28f2 = papelera
furniture-0x28f3 = papelera
furniture-0x28f4 = papelera
furniture-0x28f5 = papelera
furniture-0x28f6 = papelera
furniture-0x28f7 = papelera
furniture-0x28f8 = papelera
furniture-0x28f9 = papelera
# Berry tree: story 142:15, its Berries and its tree, Routes 5 to 22
furniture-0x2922 = árbol de bayas
furniture-0x2923 = árbol de bayas
furniture-0x2924 = árbol de bayas
furniture-0x2925 = árbol de bayas
furniture-0x2926 = árbol de bayas
furniture-0x2927 = árbol de bayas
furniture-0x2928 = árbol de bayas
furniture-0x2929 = árbol de bayas
furniture-0x292a = árbol de bayas
furniture-0x292b = árbol de bayas
furniture-0x292c = árbol de bayas
furniture-0x292d = árbol de bayas
furniture-0x292e = árbol de bayas
furniture-0x292f = árbol de bayas
furniture-0x2930 = árbol de bayas
# fitting room: text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = probador
# fitting room: text 142:85, the 7 boutiques' fitting room, called as the Pokémon Centers' is
furniture-0x29d3 = probador
# clothes: text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = ropa
# bed: story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = cama
# taxi stand: story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = parada de taxis
# Gogoat Shuttle: story 399:218, Lumiose's 12 stops
furniture-0x2a3b = Gogonetas
furniture-0x2a3c = Gogonetas
furniture-0x2a3d = Gogonetas
furniture-0x2a3e = Gogonetas
furniture-0x2a3f = Gogonetas
furniture-0x2a40 = Gogonetas
furniture-0x2a41 = Gogonetas
furniture-0x2a42 = Gogonetas
furniture-0x2a43 = Gogonetas
furniture-0x2a44 = Gogonetas
furniture-0x2a45 = Gogonetas
furniture-0x2a46 = Gogonetas

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# soft soil: story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = tierra blanda
place-0x160-furniture-3 = tierra blanda
place-0x160-furniture-4 = tierra blanda
place-0x160-furniture-5 = tierra blanda
place-0x160-furniture-6 = tierra blanda
place-0x160-furniture-7 = tierra blanda
place-0x160-furniture-8 = tierra blanda
place-0x160-furniture-9 = tierra blanda
place-0x160-furniture-10 = tierra blanda
place-0x160-furniture-11 = tierra blanda
place-0x160-furniture-12 = tierra blanda
place-0x160-furniture-13 = tierra blanda
place-0x160-furniture-14 = tierra blanda
place-0x160-furniture-15 = tierra blanda
place-0x160-furniture-16 = tierra blanda
place-0x160-furniture-17 = tierra blanda
place-0x160-furniture-18 = tierra blanda
place-0x160-furniture-19 = tierra blanda
place-0x160-furniture-20 = tierra blanda
place-0x160-furniture-21 = tierra blanda
place-0x160-furniture-22 = tierra blanda
place-0x160-furniture-23 = tierra blanda
place-0x160-furniture-24 = tierra blanda
place-0x160-furniture-25 = tierra blanda
place-0x160-furniture-26 = tierra blanda
place-0x160-furniture-27 = tierra blanda
place-0x160-furniture-28 = tierra blanda
place-0x160-furniture-29 = tierra blanda
place-0x160-furniture-30 = tierra blanda
place-0x160-furniture-31 = tierra blanda
place-0x160-furniture-32 = tierra blanda
place-0x160-furniture-33 = tierra blanda
place-0x160-furniture-34 = tierra blanda
place-0x160-furniture-35 = tierra blanda
place-0x160-furniture-36 = tierra blanda
place-0x160-furniture-37 = tierra blanda
# audio guide: story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = audioguía
place-0x68-furniture-1 = audioguía
place-0x68-furniture-2 = audioguía
place-0x68-furniture-3 = audioguía
place-0x68-furniture-4 = audioguía
place-0x68-furniture-5 = audioguía
place-0x68-furniture-6 = audioguía
place-0x69-furniture-0 = audioguía
place-0x69-furniture-1 = audioguía
place-0x69-furniture-2 = audioguía
place-0x69-furniture-3 = audioguía
place-0x69-furniture-4 = audioguía
place-0x69-furniture-5 = audioguía
place-0x69-furniture-6 = audioguía
place-0x6a-furniture-0 = audioguía
place-0x6a-furniture-1 = audioguía
place-0x6a-furniture-2 = audioguía
place-0x6a-furniture-3 = audioguía
place-0x6a-furniture-4 = audioguía
place-0x6a-furniture-5 = audioguía
place-0x6a-furniture-6 = audioguía
place-0x6a-furniture-7 = audioguía
place-0x6a-furniture-8 = audioguía
place-0x6a-furniture-9 = audioguía
# statue of a Bisharp: story 215:21, Parfum Palace
place-0x120-furniture-7 = estatua de un Bisharp
place-0x120-furniture-8 = estatua de un Bisharp
place-0x120-furniture-9 = estatua de un Bisharp
place-0x120-furniture-10 = estatua de un Bisharp
place-0x120-furniture-11 = estatua de un Bisharp
place-0x120-furniture-12 = estatua de un Bisharp
place-0x120-furniture-13 = estatua de un Bisharp
place-0x120-furniture-14 = estatua de un Bisharp
place-0x125-furniture-2 = estatua de un Bisharp
place-0x125-furniture-3 = estatua de un Bisharp
place-0x127-furniture-2 = estatua de un Bisharp
place-0x127-furniture-3 = estatua de un Bisharp
place-0x12c-furniture-3 = estatua de un Bisharp
place-0x12c-furniture-4 = estatua de un Bisharp
# golden statue of a knight: story 215:22, Parfum Palace
place-0x120-furniture-15 = estatua de un caballero dorado
place-0x120-furniture-16 = estatua de un caballero dorado
place-0x120-furniture-17 = estatua de un caballero dorado
place-0x120-furniture-18 = estatua de un caballero dorado
place-0x120-furniture-19 = estatua de un caballero dorado
place-0x120-furniture-20 = estatua de un caballero dorado
place-0x120-furniture-21 = estatua de un caballero dorado
place-0x120-furniture-22 = estatua de un caballero dorado
place-0x120-furniture-23 = estatua de un caballero dorado
place-0x120-furniture-24 = estatua de un caballero dorado
place-0x125-furniture-4 = estatua de un caballero dorado
place-0x125-furniture-5 = estatua de un caballero dorado
place-0x127-furniture-4 = estatua de un caballero dorado
place-0x127-furniture-5 = estatua de un caballero dorado
# whiteboard: story 412:7, Santalune City, zone 0x3e
place-0x3e-furniture-0 = pizarra
place-0x3e-furniture-1 = pizarra
place-0x3e-furniture-2 = pizarra
place-0x3e-furniture-3 = pizarra
place-0x3e-furniture-4 = pizarra
place-0x3e-furniture-5 = pizarra
place-0x3e-furniture-6 = pizarra
place-0x3e-furniture-7 = pizarra
place-0x3e-furniture-8 = pizarra
# statue of a Poké Ball: story 216:47, Parfum Palace's garden
place-0x12e-furniture-2 = escultura de una Poké Ball
place-0x12e-furniture-3 = escultura de una Poké Ball
place-0x12e-furniture-4 = escultura de una Poké Ball
place-0x12e-furniture-5 = escultura de una Poké Ball
place-0x12e-furniture-6 = escultura de una Poké Ball
place-0x12e-furniture-7 = escultura de una Poké Ball
# monument: story 226:7, the Battle Maison
place-0xf1-furniture-0 = estatua
place-0xf1-furniture-1 = estatua
place-0xf1-furniture-2 = estatua
place-0xf1-furniture-3 = estatua
place-0xf1-furniture-4 = estatua
# portrait: story 203:0, Parfum Palace
place-0x12c-furniture-0 = retrato
place-0x12c-furniture-1 = retrato
place-0x12c-furniture-2 = retrato
# record machine: story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = procesador de datos
place-0x54-furniture-3 = procesador de datos
place-0x54-furniture-4 = procesador de datos
place-0x54-furniture-5 = procesador de datos
# PC: story 55:1, the bedroom's PC
place-0x2-furniture-0 = PC
# mirror: story 55:2
place-0x2-furniture-1 = espejo
# TV: story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = televisor
# TV: story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = televisor
# Wii U: story 55:0
place-0x2-furniture-4 = Wii U
# statue: story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = escultura
place-0x37-furniture-1 = escultura
# pole: story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = barra
place-0x38-furniture-0 = barra
# photographs: story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = fotografías
place-0x37-furniture-4 = fotografías
place-0x37-furniture-5 = fotografías
# photograph: story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = fotografía
place-0x37-furniture-7 = fotografía
# Lysandre Café: story 313:4, Lumiose City, zone 0x43: the plate of script 22, which no record shows, so no door is paired with it
place-0x43-warp-1 = Café Lysson
