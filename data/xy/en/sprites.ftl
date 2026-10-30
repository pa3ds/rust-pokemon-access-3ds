### What to call the people, Pokémon and things on X and Y's maps, and the use tiles, furniture and
### slots they stand among, when nothing better is known.
###
### A key is an overworld sprite, the one a move model is drawn with (findings, spikes 75, 89 and
### 100), so a name here is everyone drawn as it, everywhere in the game. A name is looked for in
### order: what the player has called them, then what the game itself calls them, out of the
### player's own ROM and in their language (`games.xy`, `games.gen6.archives`), and only then this
### file. So most of what is below is heard only when pa3ds cannot read the ROM (`pa3ds.core.names` has
### the rules).
###
### Past [sprites], a key of [tiles] is a kind of use tile, a key of [furniture] the shared script a
### piece of furniture runs, and a key of [places] a slot, "zone:furniture:index" (spikes 83, 210,
### 211 and D).
###
### Each entry says where it came from, because a sprite is only as good as the reading behind it.

## sprites
# People the game has no list of: never a trainer, so the ROM names them nowhere. Each is the
# sprite's own id, where it stands, and a line of the ROM's text naming them, the same line in
# every language (`spikes/108_x_people.py lines`).
# MAMA, alone in the player's house: "Mom's words echoed..." (system 140:43)
sprite-0x7 = Mom
# HANDSOME, his Japanese name: Lumiose and Lysandre Labs; "It's me! Looker!" (system 102:16)
sprite-0x26 = Looker
# LEADER1_SIS: Santalune, where "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = Alexa
# LEADER5_SIS: Prism Tower, where "You can call me Bonnie." (story 404:0)
sprite-0x9b = Bonnie
# MEGAG: the Tower of Mastery, where "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = Mega Evolution guru
# DIRECTOR: Coumarine, where "Hello! I am the Game Director." (story 272:0)
sprite-0x98 = Game Director
# PCWOMAN1: at the counter in all 16 Pokémon Centers (spike 101); also the Battle Maison's receptionist
sprite-0x3b = Nurse
# MYSTERYW and MYSTERYM: Geosenge Town, the Kalos Power Plant and a room of Lysandre Labs, zone 0x61, where "We're the masked heroes!" (story 35:25)
sprite-0x8 = masked hero
sprite-0x9 = masked hero

# The game's own words, copied from the ROM's English text by `spikes/108_x_people.py table`: the
# story's people by the trainer each is (`xy.CHARACTERS`), and every other sprite that two
# trainers or more of one class stand as, by that class (findings, spikes 90 and 108). With the
# ROM, these come from it instead.
# FRIEND1
sprite-0x3 = Shauna
# FRIEND2
sprite-0x4 = Tierno
# FRIEND3
sprite-0x5 = Trevor
# HAKASE1
sprite-0x6 = Sycamore
# CHAMPION
sprite-0xa = Diantha
# BIGFOUR1
sprite-0xb = Wikstrom
# BIGFOUR2
sprite-0xc = Malva
# BIGFOUR3
sprite-0xd = Drasna
# BIGFOUR4
sprite-0xe = Siebold
# LEADER1
sprite-0xf = Viola
# LEADER2
sprite-0x10 = Grant
# LEADER3
sprite-0x11 = Korrina
# LEADER4
sprite-0x12 = Ramos
# LEADER5
sprite-0x13 = Clemont
# LEADER6
sprite-0x14 = Valerie
# LEADER7
sprite-0x15 = Olympia
# LEADER8
sprite-0x16 = Wulfric
# FLAREBOSS
sprite-0x17 = Lysandre
# AZ
sprite-0x21 = AZ
# RIVALM
sprite-0x93 = Calem
# RIVALW
sprite-0x94 = Serena
# FLAREBOSS2
sprite-0x9c = Lysandre
# RIVALM_EX
sprite-0x9f = Calem
# RIVALW_EX
sprite-0xa0 = Serena
# FLAREM
sprite-0x1a = Team Flare Grunt
# FLAREW
sprite-0x1b = Team Flare Grunt
# BABYGIRL1
sprite-0x28 = Preschooler
# BABYBOY1
sprite-0x2a = Preschooler
# CHILDGIRL2
sprite-0x2c = Schoolgirl
# CHILDBOY3
sprite-0x30 = Schoolboy
# TOURISTW
sprite-0x34 = Tourist
# BACKPACKERM
sprite-0x35 = Backpacker
# CHILDBOY2
sprite-0x43 = Youngster
# MIDDLEMAN1
sprite-0x44 = Poké Fan
# GIRL2
sprite-0x47 = Lass
# FIGHTERW
sprite-0x4a = Battle Girl
# ROLLERW
sprite-0x4b = Roller Skater
# ROLLERM
sprite-0x4c = Roller Skater
# FISHING
sprite-0x4d = Fisherman
# HOPEW
sprite-0x4e = Rising Star
# TRAINERW
sprite-0x4f = Ace Trainer
# VETERANW
sprite-0x50 = Veteran
# SWIMMERW
sprite-0x52 = Swimmer ♀
# TRAVELERW
sprite-0x55 = Tourist
# BREEDERW
sprite-0x57 = Pokémon Breeder
# ESPM
sprite-0x58 = Psychic
# HOPEM
sprite-0x59 = Rising Star
# VETERANM
sprite-0x5b = Veteran
# RANGERW
sprite-0x5d = Pokémon Ranger
# ASSISTANTM
sprite-0x5f = Scientist
# WORKMAN1
sprite-0x61 = Worker
# FAIRYTALEW
sprite-0x64 = Fairy Tale Girl
# GARDENER
sprite-0x67 = Gardener
# BIGMAN
sprite-0x70 = Hiker
# SWIMMERM2
sprite-0xa3 = Swimmer ♂
# SWIMMERW2
sprite-0xa4 = Swimmer ♀

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4)
sprite-0x77 = Mr. Bonding
# MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = Dexio
# MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar, "I came in place of Dexio" (256:1)
sprite-0x8c = Sina
# SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = Aliana
# C02GIRL_EX: Lysandre Labs and Lumiose, Looker's story; trainer 511 (inferred from the id)
sprite-0x9e = Essentia
# C02GIRL: Lumiose, Looker's story; "Looker, you, Emma, Mimi..." (story 347:9) (inferred)
sprite-0x9d = Emma
# MYSTERYW: Geosenge's masked heroes, "Who are these Sina and Dexio you speak of?" (story 35:25) (inferred)
sprite-0x8 = Sina
# MYSTERYM: likewise (inferred)
sprite-0x9 = Dexio

# Team Flare's admins and scientists: the admins by their trainers' class and name, as
# the grunts are; the scientists as story 378:5 calls them, not yet told apart.
# FLAREKANBU1: the Power Plant, the Secret HQ; trainers "Admin" of class Team Flare
sprite-0x18 = Team Flare Admin
# FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = Team Flare Admin
# SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = Team Flare Scientist
# SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = Team Flare Scientist
# SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = Team Flare Scientist
# SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = Team Flare Scientist

# Shared looks the game never names, called what the sprite's id says they are (text
# file 76), in Omega Ruby's words for the same look where it has one. Left out:
# NONDRAW, STOPPER, the work objects WKOBJCODE00 to 04, and ENEMY3D and GIANTMAN02,
# which no line names.
# CHILDGIRL1: Vaniville, Aquacorde
sprite-0x27 = Little girl
# CHILDBOY1: 7 zones
sprite-0x29 = Little boy
# GIRL1: 10 zones
sprite-0x2b = Girl
# GIRL3: 17 zones
sprite-0x2e = Girl
# BOY1: 3 zones
sprite-0x2d = Boy
# BOY2: 13 zones
sprite-0x32 = Boy
# WOMAN1: 33 zones
sprite-0x2f = Woman
# WOMAN2: 34 zones, one Beauty among them
sprite-0x38 = Woman
# MAN1: 36 zones
sprite-0x31 = Man
# MIDDLEMAN2: 9 zones
sprite-0xa6 = Middle-aged man
# GENTLEMAN: 45 zones; X has no such class, Omega Ruby's class
sprite-0x37 = Gentleman
# SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = Clerk
# SHOPW1: 11 zones
sprite-0x3f = Clerk
# DANPEI: in every Gym
sprite-0x3d = Gym guide
# INFORMATIONW: 37 zones, the gates
sprite-0x3e = Receptionist
# DELIVER: in all 17 Pokémon Centers
sprite-0x40 = Delivery person
# DELIVER2: likewise
sprite-0xa1 = Delivery person
# CAMERAMAN: 14 zones
sprite-0x42 = Cameraman
# TWIN: two pairs of Twins among 10 zones
sprite-0x46 = Twin
# TWIN2: 4 zones
sprite-0xa2 = Twin
# HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = Hairdresser
# MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = Café owner
# CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = Cyclist
# HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = Mannequin
# HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = Mannequin
# MODEL01: Route 7 (id only)
sprite-0xa7 = Model
# MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = Model
# MODEL03: likewise (id only)
sprite-0xa9 = Model

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = Punk Girl
# MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = Poké Fan
# ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = Artist
# ARTISTW: one Artist
sprite-0x54 = Artist
# BREEDERM: one Pokémon Breeder
sprite-0x56 = Pokémon Breeder
# ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = Scientist
# WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = Worker
# KIMONO1: one Furisode Girl
sprite-0x68 = Furisode Girl
# KIMONO2: one Furisode Girl
sprite-0x69 = Furisode Girl
# KIMONO3: one Furisode Girl
sprite-0x6a = Furisode Girl
# KIMONO4: one Furisode Girl
sprite-0x6b = Furisode Girl
# LADY: 19 zones, no trainer
sprite-0x36 = Lady
# WAITRESS: 24 zones, no trainer
sprite-0x49 = Waitress
# GARCON: 25 zones, no trainer
sprite-0x72 = Garçon
# COOK: 12 zones, no trainer
sprite-0x5e = Chef
# BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = Butler
# MAID: 32 zones, the Pokémon Centers
sprite-0x66 = Maid
# SKYM: 8 zones, no trainer
sprite-0x6c = Sky Trainer
# SKYW: 8 zones, no trainer
sprite-0x6d = Sky Trainer
# FISHING2: 4 zones, as FISHING is
sprite-0xa5 = Fisherman

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = Tourist
# BADMAN
sprite-0x39 = Punk Guy
# TRAINERM
sprite-0x5a = Ace Trainer
# RANGERM
sprite-0x5c = Pokémon Ranger
# OCCULTW
sprite-0x63 = Hex Maniac
# FIGHTERM
sprite-0x71 = Black Belt

# The Pokémon on the map, by the species each one's model draws, its tag read through
# the battle models (spike 218); with the ROM, its own species names. Mewtwo's model
# names none.
# P_MODEL01: the Unknown Dungeon, "Mewtwo blended into the darkness of the cave" (story 185:1); its model names no species
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
# ITREE01, runs 0x2712: story 130:3, "You can't pass because of prickly thorns, but a Pokémon may be able to cut the tree."; 25 in 12 zones, Route 3 to Route 22 and the Pokémon Village
sprite-0x200b = bush
# IROCK01, runs 0x2710: story 130:1, "It's a cracked boulder. A Pokémon may be able to break it."; 61 in 13 zones, Ambrette Town to Victory Road
sprite-0x200c = cracked boulder
# KROCK01, runs 0x2714: story 130:5, "It's a big boulder, but a Pokémon may be able to push it aside."; 25 in 12 zones
sprite-0x2009 = big boulder
# IWALL01, runs 0x2716: story 130:9, "It's a cracked wall. A Pokémon may be able to break it."; 3 on Route 15, 2 in Victory Road, 7 in the Lost Hotel
sprite-0x2034 = cracked wall
# ISNOW01: ten on Route 17, where "all the rocks along the path are nothing against Mamoswine's strength!" (story 70:5)
sprite-0x2035 = rock
# IROCK02: ten on Route 9, crossed on Rhyhorn; no line, its model is it0014_rock (inferred), the word of story 70:5
sprite-0x200d = rock

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# AWARD04, case 0xd: story 55:4, "It's an award for completing the Central Kalos Pokédex."; the bedroom, (12, 10)
sprite-0x203b = Central Kalos Pokédex award
# AWARD03, case 0xe: story 55:5; (11, 10)
sprite-0x203a = Coastal Kalos Pokédex award
# AWARD05, case 0xf: story 55:6; (13, 10)
sprite-0x203c = Mountain Kalos Pokédex award
# AWARD01, case 0x10: story 55:7, "...for catching all of the Pokémon in the Kalos region."; (14, 10)
sprite-0x2036 = Kalos region award
# AWARD02, case 0x11: story 55:8; (15, 10)
sprite-0x2037 = National Pokédex award
# KINOMI, runs 0x2939: story 142:15, "There are small Berries growing high up in the tree!"; Route 15
sprite-0x200e = Berry tree
# KINOMI2, Routes 5 and 19
sprite-0x2011 = Berry tree
# KINOMI3, Routes 11 and 12
sprite-0x2012 = Berry tree
# KINOMI4, Routes 16 and 22
sprite-0x2013 = Berry tree
# KINOMI11, Route 7
sprite-0x201a = Berry tree
# KINOMI15, Route 18
sprite-0x201e = Berry tree
# KINOMI31, Route 21
sprite-0x202e = Berry tree
# KINOMI33, Route 10
sprite-0x2030 = Berry tree
# KINOMI34, Route 6
sprite-0x2031 = Berry tree
# KINOMI35, Routes 8 and 14
sprite-0x2032 = Berry tree
# KINOMI36, Route 20
sprite-0x2033 = Berry tree

# Things the game never names, called what the ROM's own id for the sprite says they are.
# KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower; its model's joints are kira00 to kira03
sprite-0x203d = sparkle
# KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone, items 665 to 683 (text 96)
sprite-0x2042 = sparkle
# CONE, model com_bm_skatecone: two across a Lumiose street, "There is a fence blocking the road." (story 317:18), and seven in Lysandre Labs
sprite-0x203e = cone
# TOBI_KUSA, no script: three on Route 6 and three on Route 18, rustling till a wild Pokémon "leaped out of the tall grass" (text 124:261)
sprite-0x2002 = shaking grass
# ENEMY3D, shared script 2000: nine in the Glittering Cave, drawn as nothing, a wild battle as the player comes up to one (spike 239)
sprite-0x95 = lurking Pokémon

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 132, which says what is there; the word here is taken from
# that line (spike 220). The PC's word is read from the ROM ahead of this one, and the TV's is story
# 390:19's, since X has no line naming a TV alone.
# the PC's own lines (archives.PC_LINES)
tile-0xd5 = PC
# story 390:19, "a TV and a washing machine": X has no line naming a TV alone
tile-0xd7 = TV
# text 132:0
tile-0xd8 = Pokémon knickknacks
# text 132:1, books and photo albums
tile-0xd9 = Pokémon books
# text 132:2, ORAS's line and words
tile-0xda = cabinet
# text 132:3
tile-0xdb = useful items
# text 132:4
tile-0xdc = Pokémon novelties
# text 132:6
tile-0xdd = trash can
# text 132:5, "Are these books for Pokémon to read?"
tile-0xde = books for Pokémon

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# story 137:21 and 145:11, the elevators' own file: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = elevator
furniture-0x27db = elevator
furniture-0x27dc = elevator
furniture-0x27dd = elevator
furniture-0x27de = elevator
furniture-0x27df = elevator
furniture-0x27e0 = elevator
furniture-0x27e1 = elevator
furniture-0x27e2 = elevator
furniture-0x27e3 = elevator
furniture-0x27e4 = elevator
furniture-0x27e5 = elevator
furniture-0x27e6 = elevator
furniture-0x27e7 = elevator
furniture-0x27e8 = elevator
furniture-0x27e9 = elevator
furniture-0x27ea = elevator
furniture-0x27eb = elevator
furniture-0x27ec = elevator
furniture-0x27ed = elevator
furniture-0x27ee = elevator
furniture-0x27ef = elevator
furniture-0x27f0 = elevator
furniture-0x27f1 = elevator
furniture-0x27f2 = elevator
furniture-0x27f3 = elevator
furniture-0x27f4 = elevator
furniture-0x27f5 = elevator
furniture-0x27f6 = elevator
furniture-0x27f7 = elevator
furniture-0x27f8 = elevator
furniture-0x27f9 = elevator
furniture-0x27fa = elevator
furniture-0x27fb = elevator
furniture-0x27fc = elevator
furniture-0x27fd = elevator
furniture-0x27fe = elevator
furniture-0x27ff = elevator
furniture-0x2800 = elevator
furniture-0x2801 = elevator
furniture-0x2802 = elevator
furniture-0x2803 = elevator
furniture-0x2804 = elevator
furniture-0x2805 = elevator
furniture-0x2806 = elevator
furniture-0x2807 = elevator
furniture-0x2808 = elevator
furniture-0x2809 = elevator
# story 221:8, Ambrette Town
furniture-0x281e = meteor
# story 220:0, Lumiose City, zone 0x54
furniture-0x284b = cardboard boxes
# story 222:0, one in each of 12 places
furniture-0x28c8 = photo spot
furniture-0x28c9 = photo spot
furniture-0x28ca = photo spot
furniture-0x28cb = photo spot
furniture-0x28cc = photo spot
furniture-0x28cd = photo spot
furniture-0x28ce = photo spot
furniture-0x28cf = photo spot
furniture-0x28d0 = photo spot
furniture-0x28d1 = photo spot
furniture-0x28d2 = photo spot
furniture-0x28d3 = photo spot
# story 142:14, Pokémon Village and the Lost Hotel
furniture-0x28f0 = trash can
furniture-0x28f1 = trash can
furniture-0x28f2 = trash can
furniture-0x28f3 = trash can
furniture-0x28f4 = trash can
furniture-0x28f5 = trash can
furniture-0x28f6 = trash can
furniture-0x28f7 = trash can
furniture-0x28f8 = trash can
furniture-0x28f9 = trash can
# story 142:15, its Berries and its tree, Routes 5 to 22
furniture-0x2922 = Berry tree
furniture-0x2923 = Berry tree
furniture-0x2924 = Berry tree
furniture-0x2925 = Berry tree
furniture-0x2926 = Berry tree
furniture-0x2927 = Berry tree
furniture-0x2928 = Berry tree
furniture-0x2929 = Berry tree
furniture-0x292a = Berry tree
furniture-0x292b = Berry tree
furniture-0x292c = Berry tree
furniture-0x292d = Berry tree
furniture-0x292e = Berry tree
furniture-0x292f = Berry tree
furniture-0x2930 = Berry tree
# text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = fitting room
# text 142:225, the 7 boutiques' fitting room, its clerk's word
furniture-0x29d3 = fitting room
# text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = clothes
# story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = bed
# story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = taxi stand
# story 399:218, Lumiose's 12 stops
furniture-0x2a3b = Gogoat Shuttle
furniture-0x2a3c = Gogoat Shuttle
furniture-0x2a3d = Gogoat Shuttle
furniture-0x2a3e = Gogoat Shuttle
furniture-0x2a3f = Gogoat Shuttle
furniture-0x2a40 = Gogoat Shuttle
furniture-0x2a41 = Gogoat Shuttle
furniture-0x2a42 = Gogoat Shuttle
furniture-0x2a43 = Gogoat Shuttle
furniture-0x2a44 = Gogoat Shuttle
furniture-0x2a45 = Gogoat Shuttle
furniture-0x2a46 = Gogoat Shuttle

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = soft soil
place-0x160-furniture-3 = soft soil
place-0x160-furniture-4 = soft soil
place-0x160-furniture-5 = soft soil
place-0x160-furniture-6 = soft soil
place-0x160-furniture-7 = soft soil
place-0x160-furniture-8 = soft soil
place-0x160-furniture-9 = soft soil
place-0x160-furniture-10 = soft soil
place-0x160-furniture-11 = soft soil
place-0x160-furniture-12 = soft soil
place-0x160-furniture-13 = soft soil
place-0x160-furniture-14 = soft soil
place-0x160-furniture-15 = soft soil
place-0x160-furniture-16 = soft soil
place-0x160-furniture-17 = soft soil
place-0x160-furniture-18 = soft soil
place-0x160-furniture-19 = soft soil
place-0x160-furniture-20 = soft soil
place-0x160-furniture-21 = soft soil
place-0x160-furniture-22 = soft soil
place-0x160-furniture-23 = soft soil
place-0x160-furniture-24 = soft soil
place-0x160-furniture-25 = soft soil
place-0x160-furniture-26 = soft soil
place-0x160-furniture-27 = soft soil
place-0x160-furniture-28 = soft soil
place-0x160-furniture-29 = soft soil
place-0x160-furniture-30 = soft soil
place-0x160-furniture-31 = soft soil
place-0x160-furniture-32 = soft soil
place-0x160-furniture-33 = soft soil
place-0x160-furniture-34 = soft soil
place-0x160-furniture-35 = soft soil
place-0x160-furniture-36 = soft soil
place-0x160-furniture-37 = soft soil
# story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = audio guide
place-0x68-furniture-1 = audio guide
place-0x68-furniture-2 = audio guide
place-0x68-furniture-3 = audio guide
place-0x68-furniture-4 = audio guide
place-0x68-furniture-5 = audio guide
place-0x68-furniture-6 = audio guide
place-0x69-furniture-0 = audio guide
place-0x69-furniture-1 = audio guide
place-0x69-furniture-2 = audio guide
place-0x69-furniture-3 = audio guide
place-0x69-furniture-4 = audio guide
place-0x69-furniture-5 = audio guide
place-0x69-furniture-6 = audio guide
place-0x6a-furniture-0 = audio guide
place-0x6a-furniture-1 = audio guide
place-0x6a-furniture-2 = audio guide
place-0x6a-furniture-3 = audio guide
place-0x6a-furniture-4 = audio guide
place-0x6a-furniture-5 = audio guide
place-0x6a-furniture-6 = audio guide
place-0x6a-furniture-7 = audio guide
place-0x6a-furniture-8 = audio guide
place-0x6a-furniture-9 = audio guide
# story 215:21, Parfum Palace
place-0x120-furniture-7 = statue of a Bisharp
place-0x120-furniture-8 = statue of a Bisharp
place-0x120-furniture-9 = statue of a Bisharp
place-0x120-furniture-10 = statue of a Bisharp
place-0x120-furniture-11 = statue of a Bisharp
place-0x120-furniture-12 = statue of a Bisharp
place-0x120-furniture-13 = statue of a Bisharp
place-0x120-furniture-14 = statue of a Bisharp
place-0x125-furniture-2 = statue of a Bisharp
place-0x125-furniture-3 = statue of a Bisharp
place-0x127-furniture-2 = statue of a Bisharp
place-0x127-furniture-3 = statue of a Bisharp
place-0x12c-furniture-3 = statue of a Bisharp
place-0x12c-furniture-4 = statue of a Bisharp
# story 215:22, Parfum Palace
place-0x120-furniture-15 = golden statue of a knight
place-0x120-furniture-16 = golden statue of a knight
place-0x120-furniture-17 = golden statue of a knight
place-0x120-furniture-18 = golden statue of a knight
place-0x120-furniture-19 = golden statue of a knight
place-0x120-furniture-20 = golden statue of a knight
place-0x120-furniture-21 = golden statue of a knight
place-0x120-furniture-22 = golden statue of a knight
place-0x120-furniture-23 = golden statue of a knight
place-0x120-furniture-24 = golden statue of a knight
place-0x125-furniture-4 = golden statue of a knight
place-0x125-furniture-5 = golden statue of a knight
place-0x127-furniture-4 = golden statue of a knight
place-0x127-furniture-5 = golden statue of a knight
# story 412:7, Santalune City, zone 0x3e
place-0x3e-furniture-0 = whiteboard
place-0x3e-furniture-1 = whiteboard
place-0x3e-furniture-2 = whiteboard
place-0x3e-furniture-3 = whiteboard
place-0x3e-furniture-4 = whiteboard
place-0x3e-furniture-5 = whiteboard
place-0x3e-furniture-6 = whiteboard
place-0x3e-furniture-7 = whiteboard
place-0x3e-furniture-8 = whiteboard
# story 216:47, Parfum Palace's garden
place-0x12e-furniture-2 = statue of a Poké Ball
place-0x12e-furniture-3 = statue of a Poké Ball
place-0x12e-furniture-4 = statue of a Poké Ball
place-0x12e-furniture-5 = statue of a Poké Ball
place-0x12e-furniture-6 = statue of a Poké Ball
place-0x12e-furniture-7 = statue of a Poké Ball
# story 226:7, the Battle Maison
place-0xf1-furniture-0 = monument
place-0xf1-furniture-1 = monument
place-0xf1-furniture-2 = monument
place-0xf1-furniture-3 = monument
place-0xf1-furniture-4 = monument
# story 203:0, Parfum Palace
place-0x12c-furniture-0 = portrait
place-0x12c-furniture-1 = portrait
place-0x12c-furniture-2 = portrait
# story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = record machine
place-0x54-furniture-3 = record machine
place-0x54-furniture-4 = record machine
place-0x54-furniture-5 = record machine
# story 55:1, the bedroom's PC
place-0x2-furniture-0 = PC
# story 55:2
place-0x2-furniture-1 = mirror
# story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = TV
# story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = TV
# story 55:0
place-0x2-furniture-4 = Wii U
# story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = statue
place-0x37-furniture-1 = statue
# story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = pole
place-0x38-furniture-0 = pole
# story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = photographs
place-0x37-furniture-4 = photographs
place-0x37-furniture-5 = photographs
# story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = photograph
place-0x37-furniture-7 = photograph
# story 313:4, Lumiose City, zone 0x43: the plate of script 22, which no record shows, so no door is paired with it
place-0x43-warp-1 = Lysandre Café
# bed: story 374:0, Lysandre Labs, zone 0x64: "The bed looks nice and comfortable."
place-0x64-furniture-0 = bed
place-0x64-furniture-1 = bed
place-0x64-furniture-2 = bed
place-0x64-furniture-3 = bed
place-0x64-furniture-4 = bed
place-0x64-furniture-5 = bed
