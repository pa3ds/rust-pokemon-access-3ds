### What to call the people, Pokémon and things on X and Y's maps in French, and the use tiles,
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
sprite-0x7 = Maman
# Looker: "It's me! Looker!" (system 102:16)
sprite-0x26 = Beladonis
# Alexa: "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = Alexia
# Bonnie: "You can call me Bonnie." (story 404:0)
sprite-0x9b = Clem
# the Mega Evolution guru: "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = Pépé Volution
# the Game Director: "Hello! I am the Game Director." (story 272:0)
sprite-0x98 = réalisateur du jeu
# Nurse: PCWOMAN1, at the counter in all 16 Pokémon Centers (spike 101)
sprite-0x3b = Infirmière

# The game's own words for the story's people and for kinds of trainer, as `archives`
# reads them (findings, spikes 90 and 108): spikes/108_x_people.py table fr.
sprite-0x3 = Sannah
sprite-0x4 = Tierno
sprite-0x5 = Trovato
sprite-0x6 = Platane
sprite-0xa = Dianthéa
sprite-0xb = Thyméo
sprite-0xc = Malva
sprite-0xd = Dracéna
sprite-0xe = Narcisse
sprite-0xf = Violette
sprite-0x10 = Lino
sprite-0x11 = Cornélia
sprite-0x12 = Amaro
sprite-0x13 = Lem
sprite-0x14 = Valériane
sprite-0x15 = Astera
sprite-0x16 = Urup
sprite-0x17 = Lysandre
sprite-0x21 = A.Z.
sprite-0x93 = Kalem
sprite-0x94 = Serena
sprite-0x9c = Lysandre
sprite-0x9f = Kalem
sprite-0xa0 = Serena
sprite-0x1a = Team Flare Sbire
sprite-0x1b = Team Flare Sbire
sprite-0x28 = Petite
sprite-0x2a = Petit
sprite-0x2c = Écolière
sprite-0x30 = Écolier
sprite-0x34 = Touriste
sprite-0x35 = Randonneur
sprite-0x43 = Gamin
sprite-0x44 = Pokéfan
sprite-0x47 = Fillette
sprite-0x4a = Combattante
sprite-0x4b = Roller Skateuse
sprite-0x4c = Roller Skateur
sprite-0x4d = Pêcheur
sprite-0x4e = Dresseur Prometteur
sprite-0x4f = Topdresseur
sprite-0x50 = Vénérable
sprite-0x52 = Nageuse
sprite-0x55 = Touriste
sprite-0x57 = Éleveuse
sprite-0x58 = Kinésiste
sprite-0x59 = Dresseur Prometteur
sprite-0x5b = Vénérable
sprite-0x5d = Pokémon Ranger
sprite-0x5f = Scientifique
sprite-0x61 = Ouvrier
sprite-0x64 = Fillette de Conte
sprite-0x67 = Jardinier
sprite-0x70 = Montagnard
sprite-0xa3 = Nageur
sprite-0xa4 = Nageuse

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# Mr. Bonding: KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4)
sprite-0x77 = Pierce Hanlov
# Dexio: MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = Dexio
# Sina: MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar, "I came in place of Dexio" (256:1)
sprite-0x8c = Sina
# Aliana: SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = Ancolie
# Essentia: C02GIRL_EX: Lysandre Labs and Lumiose, Looker's story; trainer 511 (inferred from the id)
sprite-0x9e = Elili
# Emma: C02GIRL: Lumiose, Looker's story; "Looker, you, Emma, Mimi..." (story 347:9) (inferred)
sprite-0x9d = Millie
# Sina: MYSTERYW: Geosenge's masked heroes, "Who are these Sina and Dexio you speak of?" (story 35:25) (inferred)
sprite-0x8 = Sina
# Dexio: MYSTERYM: likewise (inferred)
sprite-0x9 = Dexio

# Team Flare's admins and scientists: the admins by their trainers' class and name, as
# the grunts are; the scientists as story 378:5 calls them, not yet told apart.
# Team Flare Admin: FLAREKANBU1: the Power Plant, the Secret HQ; trainers "Admin" of class Team Flare
sprite-0x18 = Team Flare Manager
# Team Flare Admin: FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = Team Flare Manager
# Team Flare Scientist: SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = Scientifique de la Team Flare
# Team Flare Scientist: SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = Scientifique de la Team Flare
# Team Flare Scientist: SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = Scientifique de la Team Flare
# Team Flare Scientist: SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = Scientifique de la Team Flare

# Shared looks the game never names, called what the sprite's id says they are (text
# file 76), in Omega Ruby's words for the same look where it has one. Left out:
# NONDRAW, STOPPER, the work objects WKOBJCODE00 to 04, and ENEMY3D and GIANTMAN02,
# which no line names.
# Little girl: CHILDGIRL1: Vaniville, Aquacorde
sprite-0x27 = Petite fille
# Little boy: CHILDBOY1: 7 zones
sprite-0x29 = Petit garçon
# Girl: GIRL1: 10 zones
sprite-0x2b = Jeune fille
# Girl: GIRL3: 17 zones
sprite-0x2e = Jeune fille
# Boy: BOY1: 3 zones
sprite-0x2d = Garçon
# Boy: BOY2: 13 zones
sprite-0x32 = Garçon
# Woman: WOMAN1: 33 zones
sprite-0x2f = Femme
# Woman: WOMAN2: 34 zones, one Beauty among them
sprite-0x38 = Femme
# Man: MAN1: 36 zones
sprite-0x31 = Homme
# Middle-aged man: MIDDLEMAN2: 9 zones
sprite-0xa6 = Homme d’âge mûr
# Gentleman: GENTLEMAN: 45 zones; X has no such class, Omega Ruby's class
sprite-0x37 = Gentleman
# Clerk: SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = Vendeur
# Clerk: SHOPW1: 11 zones
sprite-0x3f = Vendeur
# Gym guide: DANPEI: in every Gym
sprite-0x3d = Guide de l’Arène
# Receptionist: INFORMATIONW: 37 zones, the gates
sprite-0x3e = Réceptionniste
# Delivery person: DELIVER: in all 17 Pokémon Centers
sprite-0x40 = Livreur
# Delivery person: DELIVER2: likewise
sprite-0xa1 = Livreur
# Cameraman: CAMERAMAN: 14 zones
sprite-0x42 = Caméraman
# Twin: TWIN: two pairs of Twins among 10 zones
sprite-0x46 = Jumelle
# Twin: TWIN2: 4 zones
sprite-0xa2 = Jumelle
# Hairdresser: HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = Coiffeur
# Café owner: MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = Patron du café
# Cyclist: CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = Cycliste
# Mannequin: HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = Mannequin
# Mannequin: HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = Mannequin
# Model: MODEL01: Route 7 (id only)
sprite-0xa7 = Modèle
# Model: MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = Modèle
# Model: MODEL03: likewise (id only)
sprite-0xa9 = Modèle

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# Punk Girl: BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = Vaurienne
# Poké Fan: MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = Pokéfan
# Artist: ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = Peintre
# Artist: ARTISTW: one Artist
sprite-0x54 = Peintre
# Pokémon Breeder: BREEDERM: one Pokémon Breeder
sprite-0x56 = Éleveur
# Scientist: ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = Scientifique
# Worker: WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = Ouvrier
# Furisode Girl: KIMONO1: one Furisode Girl
sprite-0x68 = Fille en Kimono
# Furisode Girl: KIMONO2: one Furisode Girl
sprite-0x69 = Fille en Kimono
# Furisode Girl: KIMONO3: one Furisode Girl
sprite-0x6a = Fille en Kimono
# Furisode Girl: KIMONO4: one Furisode Girl
sprite-0x6b = Fille en Kimono
# Lady: LADY: 19 zones, no trainer
sprite-0x36 = Mademoiselle
# Waitress: WAITRESS: 24 zones, no trainer
sprite-0x49 = Serveuse
# Garçon: GARCON: 25 zones, no trainer
sprite-0x72 = Garçon de Café
# Chef: COOK: 12 zones, no trainer
sprite-0x5e = Cuisinier
# Butler: BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = Majordome
# Maid: MAID: 32 zones, the Pokémon Centers
sprite-0x66 = Gouvernante
# Sky Trainer: SKYM: 8 zones, no trainer
sprite-0x6c = Dresseur Aérien
# Sky Trainer: SKYW: 8 zones, no trainer
sprite-0x6d = Dresseur Aérien
# Fisherman: FISHING2: 4 zones, as FISHING is
sprite-0xa5 = Pêcheur

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = Touriste
# BADMAN
sprite-0x39 = Vaurien
# TRAINERM
sprite-0x5a = Topdresseur
# RANGERM
sprite-0x5c = Pokémon Ranger
# OCCULTW
sprite-0x63 = Mystimaniac
# FIGHTERM
sprite-0x71 = Karatéka

# The Pokémon on the map, by the species each one's model draws, its tag read through
# the battle models (spike 218); with the ROM, its own species names. Mewtwo's model
# names none.
# Mewtwo: P_MODEL01: the Unknown Dungeon, "Mewtwo blended into the darkness of the cave" (story 185:1); its model names no species
sprite-0x1017 = Mewtwo
# EV_POKE01
sprite-0x1000 = Pitrouille
# EV_POKE02
sprite-0x1001 = Excavarenne
# EV_POKE03
sprite-0x1002 = Zygarde
# EV_POKE04
sprite-0x1003 = Hélionceau
# EV_POKE05
sprite-0x1004 = Chevroum
# RIDE_POKE02
sprite-0x1007 = Mammochon
# RIDE_POKE04
sprite-0x1008 = Cabriolaine
# EV_POKE07
sprite-0x1009 = Lucario
# EV_POKE08
sprite-0x100a = Couafarel
# EV_POKE10
sprite-0x100b = Passerouge
# EV_POKE11
sprite-0x100c = Ronflex
# EV_POKE12
sprite-0x100d = Psystigri
# RIDE_POKE05
sprite-0x100f = Rhinocorne
# EV_POKE09
sprite-0x1010 = Blizzaroi
# EV_POKE14
sprite-0x1011 = Cabriolaine
# EV_POKE15
sprite-0x1013 = Marill

# Things standing on the map that the game never names, called by a word of the line the shared
# script they run shows when the player faces one (a/0/3/1 file 27, story 130; spike 219), as
# ORAS's are (spike 137).
# en bush: ITREE01, runs 0x2712: story 130:3
sprite-0x200b = arbuste
# en cracked boulder: IROCK01, runs 0x2710: story 130:1
sprite-0x200c = rocher fissuré
# en big boulder: KROCK01, runs 0x2714: story 130:5
sprite-0x2009 = gros rocher
# en cracked wall: IWALL01, runs 0x2716: story 130:9
sprite-0x2034 = mur fissuré
# en rock: ISNOW01: ten on Route 17, where "all the rocks along the path are nothing against Mamoswine's strength!" (story 70:5)
sprite-0x2035 = rocher
# en rock: IROCK02: ten on Route 9, crossed on Rhyhorn; no line, its model is it0014_rock (inferred), the word of story 70:5
sprite-0x200d = rocher

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# en Central Kalos Pokédex award: AWARD04, case 0xd: story 55:4
sprite-0x203b = certificat du Pokédex du Centre de Kalos
# en Coastal Kalos Pokédex award: AWARD03, case 0xe: story 55:5
sprite-0x203a = certificat du Pokédex des Côtes de Kalos
# en Mountain Kalos Pokédex award: AWARD05, case 0xf: story 55:6
sprite-0x203c = certificat du Pokédex des Monts de Kalos
# en Kalos region award: AWARD01, case 0x10: story 55:7
sprite-0x2036 = certificat du Pokédex de Kalos
# en National Pokédex award: AWARD02, case 0x11: story 55:8
sprite-0x2037 = certificat du Pokédex National
# en Berry tree: KINOMI, runs 0x2939: story 142:15, "There are small Berries growing high up in the tree!"; Route 15; story 142:15 and 127:29
sprite-0x200e = arbuste à Baies
# en Berry tree: KINOMI2, Routes 5 and 19
sprite-0x2011 = arbuste à Baies
# en Berry tree: KINOMI3, Routes 11 and 12
sprite-0x2012 = arbuste à Baies
# en Berry tree: KINOMI4, Routes 16 and 22
sprite-0x2013 = arbuste à Baies
# en Berry tree: KINOMI11, Route 7
sprite-0x201a = arbuste à Baies
# en Berry tree: KINOMI15, Route 18
sprite-0x201e = arbuste à Baies
# en Berry tree: KINOMI31, Route 21
sprite-0x202e = arbuste à Baies
# en Berry tree: KINOMI33, Route 10
sprite-0x2030 = arbuste à Baies
# en Berry tree: KINOMI34, Route 6
sprite-0x2031 = arbuste à Baies
# en Berry tree: KINOMI35, Routes 8 and 14
sprite-0x2032 = arbuste à Baies
# en Berry tree: KINOMI36, Route 20
sprite-0x2033 = arbuste à Baies

# Things the game never names, called what the ROM's own id for the sprite says they are.
# en sparkle: KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower; its model's joints are kira00 to kira03
sprite-0x203d = scintillement
# en sparkle: KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone, items 665 to 683 (text 96)
sprite-0x2042 = scintillement
# en cone: CONE, model com_bm_skatecone: two across a Lumiose street, "There is a fence blocking the road." (story 317:18), and seven in Lysandre Labs
sprite-0x203e = plot
# en shaking grass: TOBI_KUSA, no script: three on Route 6 and three on Route 18, rustling till a wild Pokémon "leaped out of the tall grass" (text 124:261)
sprite-0x2002 = herbe qui bouge
# en lurking Pokémon: ENEMY3D, shared script 2000: nine in the Glittering Cave, drawn as nothing, a wild battle as the player comes up to one (spike 239)
sprite-0x95 = Pokémon embusqué

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 132, which says what is there; the word here is taken from
# that line (spike 220). The PC's word is read from the ROM ahead of this one, and the TV's is story
# 390:19's, since X has no line naming a TV alone.
# PC: the PC's own lines (archives.PC_LINES)
tile-0xd5 = PC
# TV: story 390:19, "a TV and a washing machine": X has no line naming a TV alone
tile-0xd7 = télévision
# Pokémon knickknacks: text 132:0
tile-0xd8 = objets Pokémon colorés
# Pokémon books: text 132:1, books and photo albums
tile-0xd9 = livres sur les Pokémon
# cabinet: text 132:2, ORAS's line and words
tile-0xda = objets bien rangés
# useful items: text 132:3
tile-0xdb = objets utiles
# Pokémon novelties: text 132:4
tile-0xdc = objets Pokémon
# trash can: text 132:6
tile-0xdd = poubelle
# books for Pokémon: text 132:5, "Are these books for Pokémon to read?"
tile-0xde = livres pour Pokémon

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# elevator: story 137:21 and 145:11, the elevators' own file: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = ascenseur
furniture-0x27db = ascenseur
furniture-0x27dc = ascenseur
furniture-0x27dd = ascenseur
furniture-0x27de = ascenseur
furniture-0x27df = ascenseur
furniture-0x27e0 = ascenseur
furniture-0x27e1 = ascenseur
furniture-0x27e2 = ascenseur
furniture-0x27e3 = ascenseur
furniture-0x27e4 = ascenseur
furniture-0x27e5 = ascenseur
furniture-0x27e6 = ascenseur
furniture-0x27e7 = ascenseur
furniture-0x27e8 = ascenseur
furniture-0x27e9 = ascenseur
furniture-0x27ea = ascenseur
furniture-0x27eb = ascenseur
furniture-0x27ec = ascenseur
furniture-0x27ed = ascenseur
furniture-0x27ee = ascenseur
furniture-0x27ef = ascenseur
furniture-0x27f0 = ascenseur
furniture-0x27f1 = ascenseur
furniture-0x27f2 = ascenseur
furniture-0x27f3 = ascenseur
furniture-0x27f4 = ascenseur
furniture-0x27f5 = ascenseur
furniture-0x27f6 = ascenseur
furniture-0x27f7 = ascenseur
furniture-0x27f8 = ascenseur
furniture-0x27f9 = ascenseur
furniture-0x27fa = ascenseur
furniture-0x27fb = ascenseur
furniture-0x27fc = ascenseur
furniture-0x27fd = ascenseur
furniture-0x27fe = ascenseur
furniture-0x27ff = ascenseur
furniture-0x2800 = ascenseur
furniture-0x2801 = ascenseur
furniture-0x2802 = ascenseur
furniture-0x2803 = ascenseur
furniture-0x2804 = ascenseur
furniture-0x2805 = ascenseur
furniture-0x2806 = ascenseur
furniture-0x2807 = ascenseur
furniture-0x2808 = ascenseur
furniture-0x2809 = ascenseur
# meteor: story 221:8, Ambrette Town
furniture-0x281e = météorite
# cardboard boxes: story 220:0, Lumiose City, zone 0x54
furniture-0x284b = cartons
# photo spot: story 222:0, one in each of 12 places
furniture-0x28c8 = panneau photo
furniture-0x28c9 = panneau photo
furniture-0x28ca = panneau photo
furniture-0x28cb = panneau photo
furniture-0x28cc = panneau photo
furniture-0x28cd = panneau photo
furniture-0x28ce = panneau photo
furniture-0x28cf = panneau photo
furniture-0x28d0 = panneau photo
furniture-0x28d1 = panneau photo
furniture-0x28d2 = panneau photo
furniture-0x28d3 = panneau photo
# trash can: story 142:14, Pokémon Village and the Lost Hotel
furniture-0x28f0 = poubelle
furniture-0x28f1 = poubelle
furniture-0x28f2 = poubelle
furniture-0x28f3 = poubelle
furniture-0x28f4 = poubelle
furniture-0x28f5 = poubelle
furniture-0x28f6 = poubelle
furniture-0x28f7 = poubelle
furniture-0x28f8 = poubelle
furniture-0x28f9 = poubelle
# Berry tree: story 142:15, its Berries and its tree, Routes 5 to 22
furniture-0x2922 = arbuste à Baies
furniture-0x2923 = arbuste à Baies
furniture-0x2924 = arbuste à Baies
furniture-0x2925 = arbuste à Baies
furniture-0x2926 = arbuste à Baies
furniture-0x2927 = arbuste à Baies
furniture-0x2928 = arbuste à Baies
furniture-0x2929 = arbuste à Baies
furniture-0x292a = arbuste à Baies
furniture-0x292b = arbuste à Baies
furniture-0x292c = arbuste à Baies
furniture-0x292d = arbuste à Baies
furniture-0x292e = arbuste à Baies
furniture-0x292f = arbuste à Baies
furniture-0x2930 = arbuste à Baies
# fitting room: text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = cabine d’essayage
# fitting room: text 142:85, the 7 boutiques' fitting room, called as the Pokémon Centers' is
furniture-0x29d3 = cabine d’essayage
# clothes: text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = vêtements
# bed: story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = lit
# taxi stand: story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = station de taxis
# Gogoat Shuttle: story 399:218, Lumiose's 12 stops
furniture-0x2a3b = Navette Chevroum
furniture-0x2a3c = Navette Chevroum
furniture-0x2a3d = Navette Chevroum
furniture-0x2a3e = Navette Chevroum
furniture-0x2a3f = Navette Chevroum
furniture-0x2a40 = Navette Chevroum
furniture-0x2a41 = Navette Chevroum
furniture-0x2a42 = Navette Chevroum
furniture-0x2a43 = Navette Chevroum
furniture-0x2a44 = Navette Chevroum
furniture-0x2a45 = Navette Chevroum
furniture-0x2a46 = Navette Chevroum

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# soft soil: story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = sol meuble
place-0x160-furniture-3 = sol meuble
place-0x160-furniture-4 = sol meuble
place-0x160-furniture-5 = sol meuble
place-0x160-furniture-6 = sol meuble
place-0x160-furniture-7 = sol meuble
place-0x160-furniture-8 = sol meuble
place-0x160-furniture-9 = sol meuble
place-0x160-furniture-10 = sol meuble
place-0x160-furniture-11 = sol meuble
place-0x160-furniture-12 = sol meuble
place-0x160-furniture-13 = sol meuble
place-0x160-furniture-14 = sol meuble
place-0x160-furniture-15 = sol meuble
place-0x160-furniture-16 = sol meuble
place-0x160-furniture-17 = sol meuble
place-0x160-furniture-18 = sol meuble
place-0x160-furniture-19 = sol meuble
place-0x160-furniture-20 = sol meuble
place-0x160-furniture-21 = sol meuble
place-0x160-furniture-22 = sol meuble
place-0x160-furniture-23 = sol meuble
place-0x160-furniture-24 = sol meuble
place-0x160-furniture-25 = sol meuble
place-0x160-furniture-26 = sol meuble
place-0x160-furniture-27 = sol meuble
place-0x160-furniture-28 = sol meuble
place-0x160-furniture-29 = sol meuble
place-0x160-furniture-30 = sol meuble
place-0x160-furniture-31 = sol meuble
place-0x160-furniture-32 = sol meuble
place-0x160-furniture-33 = sol meuble
place-0x160-furniture-34 = sol meuble
place-0x160-furniture-35 = sol meuble
place-0x160-furniture-36 = sol meuble
place-0x160-furniture-37 = sol meuble
# audio guide: story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = audioguide
place-0x68-furniture-1 = audioguide
place-0x68-furniture-2 = audioguide
place-0x68-furniture-3 = audioguide
place-0x68-furniture-4 = audioguide
place-0x68-furniture-5 = audioguide
place-0x68-furniture-6 = audioguide
place-0x69-furniture-0 = audioguide
place-0x69-furniture-1 = audioguide
place-0x69-furniture-2 = audioguide
place-0x69-furniture-3 = audioguide
place-0x69-furniture-4 = audioguide
place-0x69-furniture-5 = audioguide
place-0x69-furniture-6 = audioguide
place-0x6a-furniture-0 = audioguide
place-0x6a-furniture-1 = audioguide
place-0x6a-furniture-2 = audioguide
place-0x6a-furniture-3 = audioguide
place-0x6a-furniture-4 = audioguide
place-0x6a-furniture-5 = audioguide
place-0x6a-furniture-6 = audioguide
place-0x6a-furniture-7 = audioguide
place-0x6a-furniture-8 = audioguide
place-0x6a-furniture-9 = audioguide
# statue of a Bisharp: story 215:21, Parfum Palace
place-0x120-furniture-7 = statue de Scalproie
place-0x120-furniture-8 = statue de Scalproie
place-0x120-furniture-9 = statue de Scalproie
place-0x120-furniture-10 = statue de Scalproie
place-0x120-furniture-11 = statue de Scalproie
place-0x120-furniture-12 = statue de Scalproie
place-0x120-furniture-13 = statue de Scalproie
place-0x120-furniture-14 = statue de Scalproie
place-0x125-furniture-2 = statue de Scalproie
place-0x125-furniture-3 = statue de Scalproie
place-0x127-furniture-2 = statue de Scalproie
place-0x127-furniture-3 = statue de Scalproie
place-0x12c-furniture-3 = statue de Scalproie
place-0x12c-furniture-4 = statue de Scalproie
# golden statue of a knight: story 215:22, Parfum Palace
place-0x120-furniture-15 = statue d’un chevalier d’or
place-0x120-furniture-16 = statue d’un chevalier d’or
place-0x120-furniture-17 = statue d’un chevalier d’or
place-0x120-furniture-18 = statue d’un chevalier d’or
place-0x120-furniture-19 = statue d’un chevalier d’or
place-0x120-furniture-20 = statue d’un chevalier d’or
place-0x120-furniture-21 = statue d’un chevalier d’or
place-0x120-furniture-22 = statue d’un chevalier d’or
place-0x120-furniture-23 = statue d’un chevalier d’or
place-0x120-furniture-24 = statue d’un chevalier d’or
place-0x125-furniture-4 = statue d’un chevalier d’or
place-0x125-furniture-5 = statue d’un chevalier d’or
place-0x127-furniture-4 = statue d’un chevalier d’or
place-0x127-furniture-5 = statue d’un chevalier d’or
# whiteboard: story 412:7, Santalune City, zone 0x3e
place-0x3e-furniture-0 = tableau
place-0x3e-furniture-1 = tableau
place-0x3e-furniture-2 = tableau
place-0x3e-furniture-3 = tableau
place-0x3e-furniture-4 = tableau
place-0x3e-furniture-5 = tableau
place-0x3e-furniture-6 = tableau
place-0x3e-furniture-7 = tableau
place-0x3e-furniture-8 = tableau
# statue of a Poké Ball: story 216:47, Parfum Palace's garden
place-0x12e-furniture-2 = statue de Poké Ball
place-0x12e-furniture-3 = statue de Poké Ball
place-0x12e-furniture-4 = statue de Poké Ball
place-0x12e-furniture-5 = statue de Poké Ball
place-0x12e-furniture-6 = statue de Poké Ball
place-0x12e-furniture-7 = statue de Poké Ball
# monument: story 226:7, the Battle Maison
place-0xf1-furniture-0 = monument
place-0xf1-furniture-1 = monument
place-0xf1-furniture-2 = monument
place-0xf1-furniture-3 = monument
place-0xf1-furniture-4 = monument
# portrait: story 203:0, Parfum Palace
place-0x12c-furniture-0 = portrait
place-0x12c-furniture-1 = portrait
place-0x12c-furniture-2 = portrait
# record machine: story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = appareil d’enregistrement
place-0x54-furniture-3 = appareil d’enregistrement
place-0x54-furniture-4 = appareil d’enregistrement
place-0x54-furniture-5 = appareil d’enregistrement
# PC: story 55:1, the bedroom's PC
place-0x2-furniture-0 = PC
# mirror: story 55:2
place-0x2-furniture-1 = miroir
# TV: story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = télévision
# TV: story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = télévision
# Wii U: story 55:0
place-0x2-furniture-4 = console Wii U
# statue: story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = statue
place-0x37-furniture-1 = statue
# pole: story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = barre
place-0x38-furniture-0 = barre
# photographs: story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = photographies
place-0x37-furniture-4 = photographies
place-0x37-furniture-5 = photographies
# photograph: story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = photographie
place-0x37-furniture-7 = photographie
# Lysandre Café: story 313:4, Lumiose City, zone 0x43: the plate of script 22, which no record shows, so no door is paired with it
place-0x43-warp-1 = Café Lysandre
