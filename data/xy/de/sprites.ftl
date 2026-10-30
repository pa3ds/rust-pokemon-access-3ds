### What to call the people, Pokémon and things on X and Y's maps in German, and the use tiles,
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

## sprites
# People the ROM never lists as trainers, read off lines of its own text that name them —
# the same line in every language (spikes/108_x_people.py lines).
# Mom: "Mom's words echoed..." (system 140:43)
sprite-0x7 = Mama
# Looker: "It's me! Looker!" (system 102:16)
sprite-0x26 = LeBelle
# Alexa: "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = Alba
# Bonnie: "You can call me Bonnie." (story 404:0)
sprite-0x9b = Heureka
# the Mega Evolution guru: "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = Mr. Mega
# the Game Director: "Hello! I am the Game Director." (story 272:0)
sprite-0x98 = Spieldirektor
# Nurse: PCWOMAN1, at the counter in all 16 Pokémon Centers (spike 101)
sprite-0x3b = Krankenschwester
# en masked hero: MYSTERYW and MYSTERYM: Geosenge Town, the Kalos Power Plant and a room of Lysandre Labs, zone 0x61, where "We're the masked heroes!" (story 35:25)
sprite-0x8 = maskierter Held
sprite-0x9 = maskierter Held

# The game's own words for the story's people and for kinds of trainer, as `archives`
# reads them (findings, spikes 90 and 108): spikes/108_x_people.py table de.
sprite-0x3 = Sannah
sprite-0x4 = Tierno
sprite-0x5 = Trovato
sprite-0x6 = Platan
sprite-0xa = Diantha
sprite-0xb = Thymelot
sprite-0xc = Pachira
sprite-0xd = Dracena
sprite-0xe = Narcisse
sprite-0xf = Viola
sprite-0x10 = Lino
sprite-0x11 = Connie
sprite-0x12 = Amaro
sprite-0x13 = Citro
sprite-0x14 = Valerie
sprite-0x15 = Astrid
sprite-0x16 = Galantho
sprite-0x17 = Flordelis
sprite-0x21 = Azett
sprite-0x93 = Kalem
sprite-0x94 = Serena
sprite-0x9c = Flordelis
sprite-0x9f = Kalem
sprite-0xa0 = Serena
sprite-0x1a = Team Flare Rüpel
sprite-0x1b = Team Flare Rüpel
sprite-0x28 = Vorschülerin
sprite-0x2a = Vorschüler
sprite-0x2c = Schülerin
sprite-0x30 = Schüler
sprite-0x34 = Touristin
sprite-0x35 = Backpacker
sprite-0x43 = Teenager
sprite-0x44 = Pokéfan
sprite-0x47 = Göre
sprite-0x4a = Kämpferin
sprite-0x4b = Rollerskaterin
sprite-0x4c = Rollerskater
sprite-0x4d = Angler
sprite-0x4e = Trainerhoffnung
sprite-0x4f = Ass-Trainer
sprite-0x50 = Veteranin
sprite-0x52 = Schwimmerin
sprite-0x55 = Touristin
sprite-0x57 = Pokémon-Züchterin
sprite-0x58 = Psycho
sprite-0x59 = Trainerhoffnung
sprite-0x5b = Veteran
sprite-0x5d = Pokémon Ranger
sprite-0x5f = Forscher
sprite-0x61 = Arbeiter
sprite-0x64 = Träumerin
sprite-0x67 = Gärtner
sprite-0x70 = Wanderer
sprite-0xa3 = Schwimmer
sprite-0xa4 = Schwimmerin

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# Mr. Bonding: KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4)
sprite-0x77 = Mr. Miteinander
# Dexio: MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = Dexio
# Sina: MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar, "I came in place of Dexio" (256:1)
sprite-0x8c = Sina
# Aliana: SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = Aliana
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
sprite-0x18 = Team Flare Vorstand
# Team Flare Admin: FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = Team Flare Vorstand
# Team Flare Scientist: SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = Team Flare Wissenschaftler
# Team Flare Scientist: SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = Team Flare Wissenschaftler
# Team Flare Scientist: SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = Team Flare Wissenschaftler
# Team Flare Scientist: SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = Team Flare Wissenschaftler

# Shared looks the game never names, called what the sprite's id says they are (text
# file 76), in Omega Ruby's words for the same look where it has one. Left out:
# NONDRAW, STOPPER, the work objects WKOBJCODE00 to 04, and ENEMY3D and GIANTMAN02,
# which no line names.
# Little girl: CHILDGIRL1: Vaniville, Aquacorde
sprite-0x27 = Kleines Mädchen
# Little boy: CHILDBOY1: 7 zones
sprite-0x29 = Kleiner Junge
# Girl: GIRL1: 10 zones
sprite-0x2b = Mädchen
# Girl: GIRL3: 17 zones
sprite-0x2e = Mädchen
# Boy: BOY1: 3 zones
sprite-0x2d = Junge
# Boy: BOY2: 13 zones
sprite-0x32 = Junge
# Woman: WOMAN1: 33 zones
sprite-0x2f = Frau
# Woman: WOMAN2: 34 zones, one Beauty among them
sprite-0x38 = Frau
# Man: MAN1: 36 zones
sprite-0x31 = Mann
# Middle-aged man: MIDDLEMAN2: 9 zones
sprite-0xa6 = Mann mittleren Alters
# Gentleman: GENTLEMAN: 45 zones; X has no such class, Omega Ruby's class
sprite-0x37 = Gentleman
# Clerk: SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = Verkäufer
# Clerk: SHOPW1: 11 zones
sprite-0x3f = Verkäufer
# Gym guide: DANPEI: in every Gym
sprite-0x3d = Arenaberater
# Receptionist: INFORMATIONW: 37 zones, the gates
sprite-0x3e = Empfangsdame
# Delivery person: DELIVER: in all 17 Pokémon Centers
sprite-0x40 = Lieferbote
# Delivery person: DELIVER2: likewise
sprite-0xa1 = Lieferbote
# Cameraman: CAMERAMAN: 14 zones
sprite-0x42 = Kameramann
# Twin: TWIN: two pairs of Twins among 10 zones
sprite-0x46 = Zwilling
# Twin: TWIN2: 4 zones
sprite-0xa2 = Zwilling
# Hairdresser: HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = Friseur
# Café owner: MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = Cafébesitzer
# Cyclist: CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = Radfahrer
# Mannequin: HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = Schaufensterpuppe
# Mannequin: HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = Schaufensterpuppe
# Model: MODEL01: Route 7 (id only)
sprite-0xa7 = Model
# Model: MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = Model
# Model: MODEL03: likewise (id only)
sprite-0xa9 = Model

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# Punk Girl: BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = Punkerin
# Poké Fan: MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = Pokéfan
# Artist: ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = Künstler
# Artist: ARTISTW: one Artist
sprite-0x54 = Künstlerin
# Pokémon Breeder: BREEDERM: one Pokémon Breeder
sprite-0x56 = Pokémon-Züchter
# Scientist: ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = Forscherin
# Worker: WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = Arbeiter
# Furisode Girl: KIMONO1: one Furisode Girl
sprite-0x68 = Kimono-Trägerin
# Furisode Girl: KIMONO2: one Furisode Girl
sprite-0x69 = Kimono-Trägerin
# Furisode Girl: KIMONO3: one Furisode Girl
sprite-0x6a = Kimono-Trägerin
# Furisode Girl: KIMONO4: one Furisode Girl
sprite-0x6b = Kimono-Trägerin
# Lady: LADY: 19 zones, no trainer
sprite-0x36 = Lady
# Waitress: WAITRESS: 24 zones, no trainer
sprite-0x49 = Serviererin
# Garçon: GARCON: 25 zones, no trainer
sprite-0x72 = Garçon
# Chef: COOK: 12 zones, no trainer
sprite-0x5e = Koch
# Butler: BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = Butler
# Maid: MAID: 32 zones, the Pokémon Centers
sprite-0x66 = Zofe
# Sky Trainer: SKYM: 8 zones, no trainer
sprite-0x6c = Himmelstrainer
# Sky Trainer: SKYW: 8 zones, no trainer
sprite-0x6d = Himmelstrainer
# Fisherman: FISHING2: 4 zones, as FISHING is
sprite-0xa5 = Angler

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = Tourist
# BADMAN
sprite-0x39 = Punker
# TRAINERM
sprite-0x5a = Ass-Trainer
# RANGERM
sprite-0x5c = Pokémon Ranger
# OCCULTW
sprite-0x63 = Hexe
# FIGHTERM
sprite-0x71 = Schwarzgurt

# The Pokémon on the map, by the species each one's model draws, its tag read through
# the battle models (spike 218); with the ROM, its own species names. Mewtwo's model
# names none.
# Mewtwo: P_MODEL01: the Unknown Dungeon, "Mewtwo blended into the darkness of the cave" (story 185:1); its model names no species
sprite-0x1017 = Mewtu
# EV_POKE01
sprite-0x1000 = Irrbis
# EV_POKE02
sprite-0x1001 = Grebbit
# EV_POKE03
sprite-0x1002 = Zygarde
# EV_POKE04
sprite-0x1003 = Leufeo
# EV_POKE05
sprite-0x1004 = Chevrumm
# RIDE_POKE02
sprite-0x1007 = Mamutel
# RIDE_POKE04
sprite-0x1008 = Mähikel
# EV_POKE07
sprite-0x1009 = Lucario
# EV_POKE08
sprite-0x100a = Coiffwaff
# EV_POKE10
sprite-0x100b = Dartiri
# EV_POKE11
sprite-0x100c = Relaxo
# EV_POKE12
sprite-0x100d = Psiau
# RIDE_POKE05
sprite-0x100f = Rihorn
# EV_POKE09
sprite-0x1010 = Rexblisar
# EV_POKE14
sprite-0x1011 = Mähikel
# EV_POKE15
sprite-0x1013 = Marill

# Things standing on the map that the game never names, called by a word of the line the shared
# script they run shows when the player faces one (a/0/3/1 file 27, story 130; spike 219), as
# ORAS's are (spike 137).
# en bush: ITREE01, runs 0x2712: story 130:3
sprite-0x200b = Dornen
# en cracked boulder: IROCK01, runs 0x2710: story 130:1
sprite-0x200c = rissiger Felsen
# en big boulder: KROCK01, runs 0x2714: story 130:5
sprite-0x2009 = großer Felsen
# en cracked wall: IWALL01, runs 0x2716: story 130:9
sprite-0x2034 = rissige Wand
# en rock: ISNOW01: ten on Route 17, where "all the rocks along the path are nothing against Mamoswine's strength!" (story 70:5)
sprite-0x2035 = Felsbrocken
# en rock: IROCK02: ten on Route 9, crossed on Rhyhorn; no line, its model is it0014_rock (inferred), the word of story 70:5
sprite-0x200d = Felsbrocken

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# en Central Kalos Pokédex award: AWARD04, case 0xd: story 55:4
sprite-0x203b = Urkunde des Kalos-Zentral-Pokédex
# en Coastal Kalos Pokédex award: AWARD03, case 0xe: story 55:5
sprite-0x203a = Urkunde des Kalos-Küsten-Pokédex
# en Mountain Kalos Pokédex award: AWARD05, case 0xf: story 55:6
sprite-0x203c = Urkunde des Kalos-Gebirgs-Pokédex
# en Kalos region award: AWARD01, case 0x10: story 55:7
sprite-0x2036 = Urkunde des Kalos-Pokédex
# en National Pokédex award: AWARD02, case 0x11: story 55:8
sprite-0x2037 = Urkunde des Nationalen Pokédex
# en Berry tree: KINOMI, runs 0x2939: story 142:15, "There are small Berries growing high up in the tree!"; Route 15
sprite-0x200e = Beerenbaum
# en Berry tree: KINOMI2, Routes 5 and 19
sprite-0x2011 = Beerenbaum
# en Berry tree: KINOMI3, Routes 11 and 12
sprite-0x2012 = Beerenbaum
# en Berry tree: KINOMI4, Routes 16 and 22
sprite-0x2013 = Beerenbaum
# en Berry tree: KINOMI11, Route 7
sprite-0x201a = Beerenbaum
# en Berry tree: KINOMI15, Route 18
sprite-0x201e = Beerenbaum
# en Berry tree: KINOMI31, Route 21
sprite-0x202e = Beerenbaum
# en Berry tree: KINOMI33, Route 10
sprite-0x2030 = Beerenbaum
# en Berry tree: KINOMI34, Route 6
sprite-0x2031 = Beerenbaum
# en Berry tree: KINOMI35, Routes 8 and 14
sprite-0x2032 = Beerenbaum
# en Berry tree: KINOMI36, Route 20
sprite-0x2033 = Beerenbaum

# Things the game never names, called what the ROM's own id for the sprite says they are.
# en sparkle: KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower; its model's joints are kira00 to kira03
sprite-0x203d = Funkeln
# en sparkle: KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone, items 665 to 683 (text 96)
sprite-0x2042 = Funkeln
# en cone: CONE, model com_bm_skatecone: two across a Lumiose street, "There is a fence blocking the road." (story 317:18), and seven in Lysandre Labs
sprite-0x203e = Leitkegel
# en shaking grass: TOBI_KUSA, no script: three on Route 6 and three on Route 18, rustling till a wild Pokémon "leaped out of the tall grass" (text 124:261)
sprite-0x2002 = raschelndes Gras
# en lurking Pokémon: ENEMY3D, shared script 2000: nine in the Glittering Cave, drawn as nothing, a wild battle as the player comes up to one (spike 239)
sprite-0x95 = lauerndes Pokémon

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 132, which says what is there; the word here is taken from
# that line (spike 220). The PC's word is read from the ROM ahead of this one, and the TV's is story
# 390:19's, since X has no line naming a TV alone.
# PC: the PC's own lines (archives.PC_LINES)
tile-0xd5 = PC
# TV: story 390:19, "a TV and a washing machine": X has no line naming a TV alone
tile-0xd7 = Fernseher
# Pokémon knickknacks: text 132:0
tile-0xd8 = Pokémon-Objekte
# Pokémon books: text 132:1, books and photo albums
tile-0xd9 = Pokémon-Bücher
# cabinet: text 132:2, ORAS's line and words
tile-0xda = Gegenstände
# useful items: text 132:3
tile-0xdb = praktische Items
# Pokémon novelties: text 132:4
tile-0xdc = Schubladen
# trash can: text 132:6
tile-0xdd = Abfalleimer
# books for Pokémon: text 132:5, "Are these books for Pokémon to read?"
tile-0xde = Bücher für Pokémon

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# elevator: story 137:21 and 145:11, the elevators' own file: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = Fahrstuhl
furniture-0x27db = Fahrstuhl
furniture-0x27dc = Fahrstuhl
furniture-0x27dd = Fahrstuhl
furniture-0x27de = Fahrstuhl
furniture-0x27df = Fahrstuhl
furniture-0x27e0 = Fahrstuhl
furniture-0x27e1 = Fahrstuhl
furniture-0x27e2 = Fahrstuhl
furniture-0x27e3 = Fahrstuhl
furniture-0x27e4 = Fahrstuhl
furniture-0x27e5 = Fahrstuhl
furniture-0x27e6 = Fahrstuhl
furniture-0x27e7 = Fahrstuhl
furniture-0x27e8 = Fahrstuhl
furniture-0x27e9 = Fahrstuhl
furniture-0x27ea = Fahrstuhl
furniture-0x27eb = Fahrstuhl
furniture-0x27ec = Fahrstuhl
furniture-0x27ed = Fahrstuhl
furniture-0x27ee = Fahrstuhl
furniture-0x27ef = Fahrstuhl
furniture-0x27f0 = Fahrstuhl
furniture-0x27f1 = Fahrstuhl
furniture-0x27f2 = Fahrstuhl
furniture-0x27f3 = Fahrstuhl
furniture-0x27f4 = Fahrstuhl
furniture-0x27f5 = Fahrstuhl
furniture-0x27f6 = Fahrstuhl
furniture-0x27f7 = Fahrstuhl
furniture-0x27f8 = Fahrstuhl
furniture-0x27f9 = Fahrstuhl
furniture-0x27fa = Fahrstuhl
furniture-0x27fb = Fahrstuhl
furniture-0x27fc = Fahrstuhl
furniture-0x27fd = Fahrstuhl
furniture-0x27fe = Fahrstuhl
furniture-0x27ff = Fahrstuhl
furniture-0x2800 = Fahrstuhl
furniture-0x2801 = Fahrstuhl
furniture-0x2802 = Fahrstuhl
furniture-0x2803 = Fahrstuhl
furniture-0x2804 = Fahrstuhl
furniture-0x2805 = Fahrstuhl
furniture-0x2806 = Fahrstuhl
furniture-0x2807 = Fahrstuhl
furniture-0x2808 = Fahrstuhl
furniture-0x2809 = Fahrstuhl
# meteor: story 221:8, Ambrette Town
furniture-0x281e = Meteorit
# cardboard boxes: story 220:0, Lumiose City, zone 0x54
furniture-0x284b = Kartons
# photo spot: story 222:0, one in each of 12 places
furniture-0x28c8 = Fotoschild
furniture-0x28c9 = Fotoschild
furniture-0x28ca = Fotoschild
furniture-0x28cb = Fotoschild
furniture-0x28cc = Fotoschild
furniture-0x28cd = Fotoschild
furniture-0x28ce = Fotoschild
furniture-0x28cf = Fotoschild
furniture-0x28d0 = Fotoschild
furniture-0x28d1 = Fotoschild
furniture-0x28d2 = Fotoschild
furniture-0x28d3 = Fotoschild
# trash can: story 142:14, Pokémon Village and the Lost Hotel
furniture-0x28f0 = Abfalleimer
furniture-0x28f1 = Abfalleimer
furniture-0x28f2 = Abfalleimer
furniture-0x28f3 = Abfalleimer
furniture-0x28f4 = Abfalleimer
furniture-0x28f5 = Abfalleimer
furniture-0x28f6 = Abfalleimer
furniture-0x28f7 = Abfalleimer
furniture-0x28f8 = Abfalleimer
furniture-0x28f9 = Abfalleimer
# Berry tree: story 142:15, its Berries and its tree, Routes 5 to 22
furniture-0x2922 = Beerenbaum
furniture-0x2923 = Beerenbaum
furniture-0x2924 = Beerenbaum
furniture-0x2925 = Beerenbaum
furniture-0x2926 = Beerenbaum
furniture-0x2927 = Beerenbaum
furniture-0x2928 = Beerenbaum
furniture-0x2929 = Beerenbaum
furniture-0x292a = Beerenbaum
furniture-0x292b = Beerenbaum
furniture-0x292c = Beerenbaum
furniture-0x292d = Beerenbaum
furniture-0x292e = Beerenbaum
furniture-0x292f = Beerenbaum
furniture-0x2930 = Beerenbaum
# fitting room: text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = Umkleidekabine
# fitting room: text 142:85, the 7 boutiques' fitting room, called as the Pokémon Centers' is
furniture-0x29d3 = Umkleidekabine
# clothes: text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = Kleidungsstücke
# bed: story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = Bett
# taxi stand: story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = Taxistand
# Gogoat Shuttle: story 399:218, Lumiose's 12 stops
furniture-0x2a3b = Chevrummobil
furniture-0x2a3c = Chevrummobil
furniture-0x2a3d = Chevrummobil
furniture-0x2a3e = Chevrummobil
furniture-0x2a3f = Chevrummobil
furniture-0x2a40 = Chevrummobil
furniture-0x2a41 = Chevrummobil
furniture-0x2a42 = Chevrummobil
furniture-0x2a43 = Chevrummobil
furniture-0x2a44 = Chevrummobil
furniture-0x2a45 = Chevrummobil
furniture-0x2a46 = Chevrummobil

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# soft soil: story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = Lehmboden
place-0x160-furniture-3 = Lehmboden
place-0x160-furniture-4 = Lehmboden
place-0x160-furniture-5 = Lehmboden
place-0x160-furniture-6 = Lehmboden
place-0x160-furniture-7 = Lehmboden
place-0x160-furniture-8 = Lehmboden
place-0x160-furniture-9 = Lehmboden
place-0x160-furniture-10 = Lehmboden
place-0x160-furniture-11 = Lehmboden
place-0x160-furniture-12 = Lehmboden
place-0x160-furniture-13 = Lehmboden
place-0x160-furniture-14 = Lehmboden
place-0x160-furniture-15 = Lehmboden
place-0x160-furniture-16 = Lehmboden
place-0x160-furniture-17 = Lehmboden
place-0x160-furniture-18 = Lehmboden
place-0x160-furniture-19 = Lehmboden
place-0x160-furniture-20 = Lehmboden
place-0x160-furniture-21 = Lehmboden
place-0x160-furniture-22 = Lehmboden
place-0x160-furniture-23 = Lehmboden
place-0x160-furniture-24 = Lehmboden
place-0x160-furniture-25 = Lehmboden
place-0x160-furniture-26 = Lehmboden
place-0x160-furniture-27 = Lehmboden
place-0x160-furniture-28 = Lehmboden
place-0x160-furniture-29 = Lehmboden
place-0x160-furniture-30 = Lehmboden
place-0x160-furniture-31 = Lehmboden
place-0x160-furniture-32 = Lehmboden
place-0x160-furniture-33 = Lehmboden
place-0x160-furniture-34 = Lehmboden
place-0x160-furniture-35 = Lehmboden
place-0x160-furniture-36 = Lehmboden
place-0x160-furniture-37 = Lehmboden
# audio guide: story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = Audioguide
place-0x68-furniture-1 = Audioguide
place-0x68-furniture-2 = Audioguide
place-0x68-furniture-3 = Audioguide
place-0x68-furniture-4 = Audioguide
place-0x68-furniture-5 = Audioguide
place-0x68-furniture-6 = Audioguide
place-0x69-furniture-0 = Audioguide
place-0x69-furniture-1 = Audioguide
place-0x69-furniture-2 = Audioguide
place-0x69-furniture-3 = Audioguide
place-0x69-furniture-4 = Audioguide
place-0x69-furniture-5 = Audioguide
place-0x69-furniture-6 = Audioguide
place-0x6a-furniture-0 = Audioguide
place-0x6a-furniture-1 = Audioguide
place-0x6a-furniture-2 = Audioguide
place-0x6a-furniture-3 = Audioguide
place-0x6a-furniture-4 = Audioguide
place-0x6a-furniture-5 = Audioguide
place-0x6a-furniture-6 = Audioguide
place-0x6a-furniture-7 = Audioguide
place-0x6a-furniture-8 = Audioguide
place-0x6a-furniture-9 = Audioguide
# statue of a Bisharp: story 215:21, Parfum Palace
place-0x120-furniture-7 = Statue eines Caesurio
place-0x120-furniture-8 = Statue eines Caesurio
place-0x120-furniture-9 = Statue eines Caesurio
place-0x120-furniture-10 = Statue eines Caesurio
place-0x120-furniture-11 = Statue eines Caesurio
place-0x120-furniture-12 = Statue eines Caesurio
place-0x120-furniture-13 = Statue eines Caesurio
place-0x120-furniture-14 = Statue eines Caesurio
place-0x125-furniture-2 = Statue eines Caesurio
place-0x125-furniture-3 = Statue eines Caesurio
place-0x127-furniture-2 = Statue eines Caesurio
place-0x127-furniture-3 = Statue eines Caesurio
place-0x12c-furniture-3 = Statue eines Caesurio
place-0x12c-furniture-4 = Statue eines Caesurio
# golden statue of a knight: story 215:22, Parfum Palace
place-0x120-furniture-15 = goldene Statue eines Ritters
place-0x120-furniture-16 = goldene Statue eines Ritters
place-0x120-furniture-17 = goldene Statue eines Ritters
place-0x120-furniture-18 = goldene Statue eines Ritters
place-0x120-furniture-19 = goldene Statue eines Ritters
place-0x120-furniture-20 = goldene Statue eines Ritters
place-0x120-furniture-21 = goldene Statue eines Ritters
place-0x120-furniture-22 = goldene Statue eines Ritters
place-0x120-furniture-23 = goldene Statue eines Ritters
place-0x120-furniture-24 = goldene Statue eines Ritters
place-0x125-furniture-4 = goldene Statue eines Ritters
place-0x125-furniture-5 = goldene Statue eines Ritters
place-0x127-furniture-4 = goldene Statue eines Ritters
place-0x127-furniture-5 = goldene Statue eines Ritters
# whiteboard: story 412:7, Santalune City, zone 0x3e
place-0x3e-furniture-0 = Tafel
place-0x3e-furniture-1 = Tafel
place-0x3e-furniture-2 = Tafel
place-0x3e-furniture-3 = Tafel
place-0x3e-furniture-4 = Tafel
place-0x3e-furniture-5 = Tafel
place-0x3e-furniture-6 = Tafel
place-0x3e-furniture-7 = Tafel
place-0x3e-furniture-8 = Tafel
# statue of a Poké Ball: story 216:47, Parfum Palace's garden
place-0x12e-furniture-2 = Skulptur eines Pokéballs
place-0x12e-furniture-3 = Skulptur eines Pokéballs
place-0x12e-furniture-4 = Skulptur eines Pokéballs
place-0x12e-furniture-5 = Skulptur eines Pokéballs
place-0x12e-furniture-6 = Skulptur eines Pokéballs
place-0x12e-furniture-7 = Skulptur eines Pokéballs
# monument: story 226:7, the Battle Maison
place-0xf1-furniture-0 = Monument
place-0xf1-furniture-1 = Monument
place-0xf1-furniture-2 = Monument
place-0xf1-furniture-3 = Monument
place-0xf1-furniture-4 = Monument
# portrait: story 203:0, Parfum Palace
place-0x12c-furniture-0 = Porträt
place-0x12c-furniture-1 = Porträt
place-0x12c-furniture-2 = Porträt
# record machine: story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = Großrechner
place-0x54-furniture-3 = Großrechner
place-0x54-furniture-4 = Großrechner
place-0x54-furniture-5 = Großrechner
# PC: story 55:1, the bedroom's PC
place-0x2-furniture-0 = Laptop
# mirror: story 55:2
place-0x2-furniture-1 = Spiegel
# TV: story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = Fernseher
# TV: story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = Fernseher
# Wii U: story 55:0
place-0x2-furniture-4 = Wii U-Konsole
# statue: story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = Arenastatue
place-0x37-furniture-1 = Arenastatue
# pole: story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = Stange
place-0x38-furniture-0 = Stange
# photographs: story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = Fotos
place-0x37-furniture-4 = Fotos
place-0x37-furniture-5 = Fotos
# photograph: story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = Foto
place-0x37-furniture-7 = Foto
# Lysandre Café: story 313:4, Lumiose City, zone 0x43: the plate of script 22, which no record shows, so no door is paired with it
place-0x43-warp-1 = Bistro Flordelis
# bed: story 374:0, Lysandre Labs, zone 0x64: "The bed looks nice and comfortable."
place-0x64-furniture-0 = Bett
place-0x64-furniture-1 = Bett
place-0x64-furniture-2 = Bett
place-0x64-furniture-3 = Bett
place-0x64-furniture-4 = Bett
place-0x64-furniture-5 = Bett
