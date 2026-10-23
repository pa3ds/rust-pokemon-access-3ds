### What to call the people, Pokémon and things on X and Y's maps in Italian, and the use tiles,
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
sprite-0x7 = mamma
# Looker: "It's me! Looker!" (system 102:16)
sprite-0x26 = Bellocchio
# Alexa: "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = Alexia
# Bonnie: "You can call me Bonnie." (story 404:0)
sprite-0x9b = Clem
# the Mega Evolution guru: "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = Sapiente della megaevoluzione
# the Game Director: "Hello! I am the Game Director." (story 272:0)
sprite-0x98 = Game Director
# Nurse: PCWOMAN1, at the counter in all 16 Pokémon Centers (spike 101)
sprite-0x3b = Infermiera

# The game's own words for the story's people and for kinds of trainer, as `archives`
# reads them (findings, spikes 90 and 108): spikes/108_x_people.py table it.
sprite-0x3 = Shana
sprite-0x4 = Tierno
sprite-0x5 = Trovato
sprite-0x6 = Platan
sprite-0xa = Diantha
sprite-0xb = Timeus
sprite-0xc = Malva
sprite-0xd = Lilia
sprite-0xe = Narciso
sprite-0xf = Violetta
sprite-0x10 = Lino
sprite-0x11 = Ornella
sprite-0x12 = Amur
sprite-0x13 = Lem
sprite-0x14 = Valérie
sprite-0x15 = Astra
sprite-0x16 = Edel
sprite-0x17 = Elisio
sprite-0x21 = AZ
sprite-0x93 = Calem
sprite-0x94 = Serena
sprite-0x9c = Elisio
sprite-0x9f = Calem
sprite-0xa0 = Serena
sprite-0x1a = Team Flare Recluta
sprite-0x1b = Team Flare Recluta
sprite-0x28 = Bimba
sprite-0x2a = Bimbo
sprite-0x2c = Studentessa
sprite-0x30 = Studente
sprite-0x34 = Turista
sprite-0x35 = Giramondo
sprite-0x43 = Bullo
sprite-0x44 = Pokéfan
sprite-0x47 = Pupa
sprite-0x4a = Combat Girl
sprite-0x4b = Pattinatrice
sprite-0x4c = Pattinatore
sprite-0x4d = Pescatore
sprite-0x4e = Giovane speranza
sprite-0x4f = Fantallenatrice
sprite-0x50 = Veterana
sprite-0x52 = Nuotatrice
sprite-0x55 = Turista
sprite-0x57 = Allevapokémon
sprite-0x58 = Sensitivo
sprite-0x59 = Giovane speranza
sprite-0x5b = Veterano
sprite-0x5d = Pokémon Ranger
sprite-0x5f = Scienziato
sprite-0x61 = Lavoratore
sprite-0x64 = Ragazza delle fiabe
sprite-0x67 = Giardiniere
sprite-0x70 = Montanaro
sprite-0xa3 = Nuotatore
sprite-0xa4 = Nuotatrice

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# Mr. Bonding: KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4)
sprite-0x77 = Mr. Fratellanza
# Dexio: MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = Dexio
# Sina: MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar, "I came in place of Dexio" (256:1)
sprite-0x8c = Sina
# Aliana: SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = Akebia
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
sprite-0x18 = Team Flare Ufficiale
# Team Flare Admin: FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = Team Flare Ufficiale
# Team Flare Scientist: SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = Scienziato del Team Flare
# Team Flare Scientist: SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = Scienziato del Team Flare
# Team Flare Scientist: SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = Scienziato del Team Flare
# Team Flare Scientist: SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = Scienziato del Team Flare

# Shared looks the game never names, called what the sprite's id says they are (text
# file 76), in Omega Ruby's words for the same look where it has one. Left out:
# NONDRAW, STOPPER, the work objects WKOBJCODE00 to 04, and ENEMY3D and GIANTMAN02,
# which no line names.
# Little girl: CHILDGIRL1: Vaniville, Aquacorde
sprite-0x27 = Bambina
# Little boy: CHILDBOY1: 7 zones
sprite-0x29 = Bambino
# Girl: GIRL1: 10 zones
sprite-0x2b = Ragazza
# Girl: GIRL3: 17 zones
sprite-0x2e = Ragazza
# Boy: BOY1: 3 zones
sprite-0x2d = Ragazzo
# Boy: BOY2: 13 zones
sprite-0x32 = Ragazzo
# Woman: WOMAN1: 33 zones
sprite-0x2f = Donna
# Woman: WOMAN2: 34 zones, one Beauty among them
sprite-0x38 = Donna
# Man: MAN1: 36 zones
sprite-0x31 = Uomo
# Middle-aged man: MIDDLEMAN2: 9 zones
sprite-0xa6 = Uomo di mezza età
# Gentleman: GENTLEMAN: 45 zones; X has no such class, Omega Ruby's class
sprite-0x37 = Gentiluomo
# Clerk: SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = Commesso
# Clerk: SHOPW1: 11 zones
sprite-0x3f = Commesso
# Gym guide: DANPEI: in every Gym
sprite-0x3d = Guida della Palestra
# Receptionist: INFORMATIONW: 37 zones, the gates
sprite-0x3e = Receptionist
# Delivery person: DELIVER: in all 17 Pokémon Centers
sprite-0x40 = Fattorino
# Delivery person: DELIVER2: likewise
sprite-0xa1 = Fattorino
# Cameraman: CAMERAMAN: 14 zones
sprite-0x42 = Cameraman
# Twin: TWIN: two pairs of Twins among 10 zones
sprite-0x46 = Gemella
# Twin: TWIN2: 4 zones
sprite-0xa2 = Gemella
# Hairdresser: HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = Parrucchiere
# Café owner: MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = Proprietario del caffè
# Cyclist: CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = Ciclista
# Mannequin: HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = Manichino
# Mannequin: HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = Manichino
# Model: MODEL01: Route 7 (id only)
sprite-0xa7 = Modello
# Model: MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = Modello
# Model: MODEL03: likewise (id only)
sprite-0xa9 = Modello

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# Punk Girl: BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = Ribelle
# Poké Fan: MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = Pokéfan
# Artist: ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = Artista
# Artist: ARTISTW: one Artist
sprite-0x54 = Artista
# Pokémon Breeder: BREEDERM: one Pokémon Breeder
sprite-0x56 = Allevapokémon
# Scientist: ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = Scienziata
# Worker: WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = Lavoratore
# Furisode Girl: KIMONO1: one Furisode Girl
sprite-0x68 = Ragazza in kimono
# Furisode Girl: KIMONO2: one Furisode Girl
sprite-0x69 = Ragazza in kimono
# Furisode Girl: KIMONO3: one Furisode Girl
sprite-0x6a = Ragazza in kimono
# Furisode Girl: KIMONO4: one Furisode Girl
sprite-0x6b = Ragazza in kimono
# Lady: LADY: 19 zones, no trainer
sprite-0x36 = Lady
# Waitress: WAITRESS: 24 zones, no trainer
sprite-0x49 = Cameriera
# Garçon: GARCON: 25 zones, no trainer
sprite-0x72 = Garçon
# Chef: COOK: 12 zones, no trainer
sprite-0x5e = Chef
# Butler: BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = Maître
# Maid: MAID: 32 zones, the Pokémon Centers
sprite-0x66 = Domestica
# Sky Trainer: SKYM: 8 zones, no trainer
sprite-0x6c = Aeroallenatore
# Sky Trainer: SKYW: 8 zones, no trainer
sprite-0x6d = Aeroallenatrice
# Fisherman: FISHING2: 4 zones, as FISHING is
sprite-0xa5 = Pescatore

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = Turista
# BADMAN
sprite-0x39 = Ribelle
# TRAINERM
sprite-0x5a = Fantallenatore
# RANGERM
sprite-0x5c = Pokémon Ranger
# OCCULTW
sprite-0x63 = Streghetta
# FIGHTERM
sprite-0x71 = Cinturanera

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
sprite-0x200b = albero spinoso
# en cracked boulder: IROCK01, runs 0x2710: story 130:1
sprite-0x200c = masso incrinato
# en big boulder: KROCK01, runs 0x2714: story 130:5
sprite-0x2009 = masso enorme
# en cracked wall: IWALL01, runs 0x2716: story 130:9
sprite-0x2034 = muro con una crepa
# en rock: ISNOW01: ten on Route 17, where "all the rocks along the path are nothing against Mamoswine's strength!" (story 70:5)
sprite-0x2035 = masso
# en rock: IROCK02: ten on Route 9, crossed on Rhyhorn; no line, its model is it0014_rock (inferred), the word of story 70:5
sprite-0x200d = masso

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# en Central Kalos Pokédex award: AWARD04, case 0xd: story 55:4
sprite-0x203b = attestato del Pokédex di Kalos Centrale
# en Coastal Kalos Pokédex award: AWARD03, case 0xe: story 55:5
sprite-0x203a = attestato del Pokédex di Kalos Costiera
# en Mountain Kalos Pokédex award: AWARD05, case 0xf: story 55:6
sprite-0x203c = attestato del Pokédex di Kalos Montana
# en Kalos region award: AWARD01, case 0x10: story 55:7
sprite-0x2036 = attestato dei Pokémon di Kalos
# en National Pokédex award: AWARD02, case 0x11: story 55:8
sprite-0x2037 = attestato del Pokédex Nazionale
# en Berry tree: KINOMI, runs 0x2939: story 142:15, "There are small Berries growing high up in the tree!"; Route 15
sprite-0x200e = albero di bacche
# en Berry tree: KINOMI2, Routes 5 and 19
sprite-0x2011 = albero di bacche
# en Berry tree: KINOMI3, Routes 11 and 12
sprite-0x2012 = albero di bacche
# en Berry tree: KINOMI4, Routes 16 and 22
sprite-0x2013 = albero di bacche
# en Berry tree: KINOMI11, Route 7
sprite-0x201a = albero di bacche
# en Berry tree: KINOMI15, Route 18
sprite-0x201e = albero di bacche
# en Berry tree: KINOMI31, Route 21
sprite-0x202e = albero di bacche
# en Berry tree: KINOMI33, Route 10
sprite-0x2030 = albero di bacche
# en Berry tree: KINOMI34, Route 6
sprite-0x2031 = albero di bacche
# en Berry tree: KINOMI35, Routes 8 and 14
sprite-0x2032 = albero di bacche
# en Berry tree: KINOMI36, Route 20
sprite-0x2033 = albero di bacche

# Things the game never names, called what the ROM's own id for the sprite says they are.
# en sparkle: KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower; its model's joints are kira00 to kira03
sprite-0x203d = scintillio
# en sparkle: KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone, items 665 to 683 (text 96)
sprite-0x2042 = scintillio
# en cone: CONE, model com_bm_skatecone: two across a Lumiose street, "There is a fence blocking the road." (story 317:18), and seven in Lysandre Labs
sprite-0x203e = cono
# en shaking grass: TOBI_KUSA, no script: three on Route 6 and three on Route 18, rustling till a wild Pokémon "leaped out of the tall grass" (text 124:261)
sprite-0x2002 = erba che si muove
# en lurking Pokémon: ENEMY3D, shared script 2000: nine in the Glittering Cave, drawn as nothing, a wild battle as the player comes up to one (spike 239)
sprite-0x95 = Pokémon in agguato

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 132, which says what is there; the word here is taken from
# that line (spike 220). The PC's word is read from the ROM ahead of this one, and the TV's is story
# 390:19's, since X has no line naming a TV alone.
# PC: the PC's own lines (archives.PC_LINES)
tile-0xd5 = PC
# TV: story 390:19, "a TV and a washing machine": X has no line naming a TV alone
tile-0xd7 = TV
# Pokémon knickknacks: text 132:0
tile-0xd8 = oggetti per i Pokémon
# Pokémon books: text 132:1, books and photo albums
tile-0xd9 = libri sui Pokémon
# cabinet: text 132:2, ORAS's line and words
tile-0xda = mobile
# useful items: text 132:3
tile-0xdb = oggetti utilissimi
# Pokémon novelties: text 132:4
tile-0xdc = oggetti per i Pokémon
# trash can: text 132:6
tile-0xdd = cestino dei rifiuti
# books for Pokémon: text 132:5, "Are these books for Pokémon to read?"
tile-0xde = libri per i Pokémon

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# elevator: story 137:21 and 145:11, the elevators' own file: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = ascensore
furniture-0x27db = ascensore
furniture-0x27dc = ascensore
furniture-0x27dd = ascensore
furniture-0x27de = ascensore
furniture-0x27df = ascensore
furniture-0x27e0 = ascensore
furniture-0x27e1 = ascensore
furniture-0x27e2 = ascensore
furniture-0x27e3 = ascensore
furniture-0x27e4 = ascensore
furniture-0x27e5 = ascensore
furniture-0x27e6 = ascensore
furniture-0x27e7 = ascensore
furniture-0x27e8 = ascensore
furniture-0x27e9 = ascensore
furniture-0x27ea = ascensore
furniture-0x27eb = ascensore
furniture-0x27ec = ascensore
furniture-0x27ed = ascensore
furniture-0x27ee = ascensore
furniture-0x27ef = ascensore
furniture-0x27f0 = ascensore
furniture-0x27f1 = ascensore
furniture-0x27f2 = ascensore
furniture-0x27f3 = ascensore
furniture-0x27f4 = ascensore
furniture-0x27f5 = ascensore
furniture-0x27f6 = ascensore
furniture-0x27f7 = ascensore
furniture-0x27f8 = ascensore
furniture-0x27f9 = ascensore
furniture-0x27fa = ascensore
furniture-0x27fb = ascensore
furniture-0x27fc = ascensore
furniture-0x27fd = ascensore
furniture-0x27fe = ascensore
furniture-0x27ff = ascensore
furniture-0x2800 = ascensore
furniture-0x2801 = ascensore
furniture-0x2802 = ascensore
furniture-0x2803 = ascensore
furniture-0x2804 = ascensore
furniture-0x2805 = ascensore
furniture-0x2806 = ascensore
furniture-0x2807 = ascensore
furniture-0x2808 = ascensore
furniture-0x2809 = ascensore
# meteor: story 221:8, Ambrette Town
furniture-0x281e = meteorite
# cardboard boxes: story 220:0, Lumiose City, zone 0x54
furniture-0x284b = scatoloni
# photo spot: story 222:0, one in each of 12 places
furniture-0x28c8 = insegna per le foto
furniture-0x28c9 = insegna per le foto
furniture-0x28ca = insegna per le foto
furniture-0x28cb = insegna per le foto
furniture-0x28cc = insegna per le foto
furniture-0x28cd = insegna per le foto
furniture-0x28ce = insegna per le foto
furniture-0x28cf = insegna per le foto
furniture-0x28d0 = insegna per le foto
furniture-0x28d1 = insegna per le foto
furniture-0x28d2 = insegna per le foto
furniture-0x28d3 = insegna per le foto
# trash can: story 142:14, Pokémon Village and the Lost Hotel
furniture-0x28f0 = cestino dei rifiuti
furniture-0x28f1 = cestino dei rifiuti
furniture-0x28f2 = cestino dei rifiuti
furniture-0x28f3 = cestino dei rifiuti
furniture-0x28f4 = cestino dei rifiuti
furniture-0x28f5 = cestino dei rifiuti
furniture-0x28f6 = cestino dei rifiuti
furniture-0x28f7 = cestino dei rifiuti
furniture-0x28f8 = cestino dei rifiuti
furniture-0x28f9 = cestino dei rifiuti
# Berry tree: story 142:15, its Berries and its tree, Routes 5 to 22
furniture-0x2922 = albero di bacche
furniture-0x2923 = albero di bacche
furniture-0x2924 = albero di bacche
furniture-0x2925 = albero di bacche
furniture-0x2926 = albero di bacche
furniture-0x2927 = albero di bacche
furniture-0x2928 = albero di bacche
furniture-0x2929 = albero di bacche
furniture-0x292a = albero di bacche
furniture-0x292b = albero di bacche
furniture-0x292c = albero di bacche
furniture-0x292d = albero di bacche
furniture-0x292e = albero di bacche
furniture-0x292f = albero di bacche
furniture-0x2930 = albero di bacche
# fitting room: text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = camerino
# clothes: text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = abiti
# bed: story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = letto
# taxi stand: story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = fermata del taxi
# Gogoat Shuttle: story 399:218, Lumiose's 12 stops
furniture-0x2a3b = Gogo Shuttle
furniture-0x2a3c = Gogo Shuttle
furniture-0x2a3d = Gogo Shuttle
furniture-0x2a3e = Gogo Shuttle
furniture-0x2a3f = Gogo Shuttle
furniture-0x2a40 = Gogo Shuttle
furniture-0x2a41 = Gogo Shuttle
furniture-0x2a42 = Gogo Shuttle
furniture-0x2a43 = Gogo Shuttle
furniture-0x2a44 = Gogo Shuttle
furniture-0x2a45 = Gogo Shuttle
furniture-0x2a46 = Gogo Shuttle

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# soft soil: story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = terreno soffice
place-0x160-furniture-3 = terreno soffice
place-0x160-furniture-4 = terreno soffice
place-0x160-furniture-5 = terreno soffice
place-0x160-furniture-6 = terreno soffice
place-0x160-furniture-7 = terreno soffice
place-0x160-furniture-8 = terreno soffice
place-0x160-furniture-9 = terreno soffice
place-0x160-furniture-10 = terreno soffice
place-0x160-furniture-11 = terreno soffice
place-0x160-furniture-12 = terreno soffice
place-0x160-furniture-13 = terreno soffice
place-0x160-furniture-14 = terreno soffice
place-0x160-furniture-15 = terreno soffice
place-0x160-furniture-16 = terreno soffice
place-0x160-furniture-17 = terreno soffice
place-0x160-furniture-18 = terreno soffice
place-0x160-furniture-19 = terreno soffice
place-0x160-furniture-20 = terreno soffice
place-0x160-furniture-21 = terreno soffice
place-0x160-furniture-22 = terreno soffice
place-0x160-furniture-23 = terreno soffice
place-0x160-furniture-24 = terreno soffice
place-0x160-furniture-25 = terreno soffice
place-0x160-furniture-26 = terreno soffice
place-0x160-furniture-27 = terreno soffice
place-0x160-furniture-28 = terreno soffice
place-0x160-furniture-29 = terreno soffice
place-0x160-furniture-30 = terreno soffice
place-0x160-furniture-31 = terreno soffice
place-0x160-furniture-32 = terreno soffice
place-0x160-furniture-33 = terreno soffice
place-0x160-furniture-34 = terreno soffice
place-0x160-furniture-35 = terreno soffice
place-0x160-furniture-36 = terreno soffice
place-0x160-furniture-37 = terreno soffice
# audio guide: story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = audioguida
place-0x68-furniture-1 = audioguida
place-0x68-furniture-2 = audioguida
place-0x68-furniture-3 = audioguida
place-0x68-furniture-4 = audioguida
place-0x68-furniture-5 = audioguida
place-0x68-furniture-6 = audioguida
place-0x69-furniture-0 = audioguida
place-0x69-furniture-1 = audioguida
place-0x69-furniture-2 = audioguida
place-0x69-furniture-3 = audioguida
place-0x69-furniture-4 = audioguida
place-0x69-furniture-5 = audioguida
place-0x69-furniture-6 = audioguida
place-0x6a-furniture-0 = audioguida
place-0x6a-furniture-1 = audioguida
place-0x6a-furniture-2 = audioguida
place-0x6a-furniture-3 = audioguida
place-0x6a-furniture-4 = audioguida
place-0x6a-furniture-5 = audioguida
place-0x6a-furniture-6 = audioguida
place-0x6a-furniture-7 = audioguida
place-0x6a-furniture-8 = audioguida
place-0x6a-furniture-9 = audioguida
# statue of a Bisharp: story 215:21, Parfum Palace
place-0x120-furniture-7 = statua di un Bisharp
place-0x120-furniture-8 = statua di un Bisharp
place-0x120-furniture-9 = statua di un Bisharp
place-0x120-furniture-10 = statua di un Bisharp
place-0x120-furniture-11 = statua di un Bisharp
place-0x120-furniture-12 = statua di un Bisharp
place-0x120-furniture-13 = statua di un Bisharp
place-0x120-furniture-14 = statua di un Bisharp
place-0x125-furniture-2 = statua di un Bisharp
place-0x125-furniture-3 = statua di un Bisharp
place-0x127-furniture-2 = statua di un Bisharp
place-0x127-furniture-3 = statua di un Bisharp
place-0x12c-furniture-3 = statua di un Bisharp
place-0x12c-furniture-4 = statua di un Bisharp
# golden statue of a knight: story 215:22, Parfum Palace
place-0x120-furniture-15 = statua d’oro di un cavaliere
place-0x120-furniture-16 = statua d’oro di un cavaliere
place-0x120-furniture-17 = statua d’oro di un cavaliere
place-0x120-furniture-18 = statua d’oro di un cavaliere
place-0x120-furniture-19 = statua d’oro di un cavaliere
place-0x120-furniture-20 = statua d’oro di un cavaliere
place-0x120-furniture-21 = statua d’oro di un cavaliere
place-0x120-furniture-22 = statua d’oro di un cavaliere
place-0x120-furniture-23 = statua d’oro di un cavaliere
place-0x120-furniture-24 = statua d’oro di un cavaliere
place-0x125-furniture-4 = statua d’oro di un cavaliere
place-0x125-furniture-5 = statua d’oro di un cavaliere
place-0x127-furniture-4 = statua d’oro di un cavaliere
place-0x127-furniture-5 = statua d’oro di un cavaliere
# whiteboard: story 412:7, Santalune City, zone 0x3e
place-0x3e-furniture-0 = lavagna
place-0x3e-furniture-1 = lavagna
place-0x3e-furniture-2 = lavagna
place-0x3e-furniture-3 = lavagna
place-0x3e-furniture-4 = lavagna
place-0x3e-furniture-5 = lavagna
place-0x3e-furniture-6 = lavagna
place-0x3e-furniture-7 = lavagna
place-0x3e-furniture-8 = lavagna
# statue of a Poké Ball: story 216:47, Parfum Palace's garden
place-0x12e-furniture-2 = scultura di una Poké Ball
place-0x12e-furniture-3 = scultura di una Poké Ball
place-0x12e-furniture-4 = scultura di una Poké Ball
place-0x12e-furniture-5 = scultura di una Poké Ball
place-0x12e-furniture-6 = scultura di una Poké Ball
place-0x12e-furniture-7 = scultura di una Poké Ball
# monument: story 226:7, the Battle Maison
place-0xf1-furniture-0 = monumento
place-0xf1-furniture-1 = monumento
place-0xf1-furniture-2 = monumento
place-0xf1-furniture-3 = monumento
place-0xf1-furniture-4 = monumento
# portrait: story 203:0, Parfum Palace
place-0x12c-furniture-0 = ritratto
place-0x12c-furniture-1 = ritratto
place-0x12c-furniture-2 = ritratto
# record machine: story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = archivio elettronico
place-0x54-furniture-3 = archivio elettronico
place-0x54-furniture-4 = archivio elettronico
place-0x54-furniture-5 = archivio elettronico
# PC: story 55:1, the bedroom's PC
place-0x2-furniture-0 = PC
# mirror: story 55:2
place-0x2-furniture-1 = specchio
# TV: story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = TV
# TV: story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = TV
# Wii U: story 55:0
place-0x2-furniture-4 = console Wii U
# statue: story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = scultura
place-0x37-furniture-1 = scultura
# pole: story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = palo
place-0x38-furniture-0 = palo
# photographs: story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = foto
place-0x37-furniture-4 = foto
place-0x37-furniture-5 = foto
# photograph: story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = foto
place-0x37-furniture-7 = foto
