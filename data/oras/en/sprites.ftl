### What to call the people and things on ORAS's maps, when nothing better is known.
###
### A key is an overworld sprite, the one a move model is drawn with (findings, spikes 75 and 89),
### so a name here is everyone drawn as it, everywhere in the game. A name is looked for in order:
### what the player has called them, then what the game itself calls them, out of the player's own
### ROM and in their language (`games.gen6.archives`), and only then this file. So most of what is
### below is heard only when pa3ds cannot read the ROM (`pa3ds.core.names` has the rules).
###
### Each entry says where it came from, because a sprite is only as good as the reading behind it.

## sprites
# People the game has no list of: never a trainer, so the ROM names them nowhere. Read in replays,
# or from the ROM's own id for the sprite and where it stands (`spikes/86_door_kinds.py stands`).
# signs.ctm: "Now take care out there, honey!", in the player's house
sprite-0x118 = Mom
# opening.ctm: "Halloo! You over there! Please! Help me out!"
sprite-0x13e = Professor Birch
# HAKASE_RS: Birch on Route 101 in movie3 (spike 52)
sprite-0xb3 = Professor Birch
# wally.ctm: "Welcome to the Pokémon Center", in two Centers on two saves
sprite-0x12a = Nurse
# HAGI_RS: Dewford, Routes 104 and 109, the S.S. Tidal
sprite-0x11b = Mr. Briney
# KUSUNOKI_RS: Slateport and Lilycove
sprite-0x135 = Captain Stern
# SORAISI_RS: Fallarbor, Meteor Falls, Mossdeep
sprite-0x136 = Professor Cozmo
# MAYUMI_RS: Fallarbor and Route 114
sprite-0x161 = Lanette
# CHAMPFATHER_RS: Rustboro
sprite-0x11a = Mr. Stone
# HANDSOME_RS: the Battle Resort
sprite-0xd7 = Looker
# LEADER7_M_RS: Mossdeep; the ROM names only "Liza & Tate"
sprite-0xc1 = Tate
# LEADER7_F_RS: Mossdeep
sprite-0xc2 = Liza
# CONTEST_RS: beside Ali in every Contest Hall, "That performance by Lisia and Ali just now was amazing!" (story 39:7)
sprite-0xd2 = Lisia
# KIZUNA_RS: Mauville, "I'm Mr. Bonding, the expert on O-Powers!" (story 570:43)
sprite-0x134 = Mr. Bonding
# DIRECTOR_RS: Lilycove, "I'm the Game Director." (story 530:0)
sprite-0x139 = Game Director
# EV_POKE35: Lilycove's three; its model names Nidoran♂ too, and draws Wailmer's body
sprite-0x1030 = Wailmer

# Shared looks the game never names, called what the ROM's own id for the sprite says they are
# (file 94, `spikes/85_rom_names.py sprite`), since everyone drawn as one hears the same word:
# WOMAN_RS is Mrs. Birch in Littleroot and a woman in 27 other zones. A `_ROOM_RS` sprite is
# the same look indoors. Every one here stands on ORAS's maps; STOPPER3, an invisible blocker
# outside Mauville Gym, is left out.
# BROTHER_RS: Littleroot and Route 101
sprite-0x119 = Young man
# CHILDBOY_RS
sprite-0x11c = Little boy
# CHILDBOY_ROOM_RS
sprite-0x150 = Little boy
# CHILDGIRL_RS
sprite-0x11d = Little girl
# CHILDGIRL_ROOM_RS
sprite-0x151 = Little girl
# OLDMAN_RS
sprite-0x11e = Old man
# OLDMAN_ROOM_RS
sprite-0x152 = Old man
# OLDWOMAN_RS
sprite-0x11f = Old woman
# OLDWOMAN_ROOM_RS
sprite-0x153 = Old woman
# BIGMAN1_RS
sprite-0x120 = Big man
# BIGMAN2_RS
sprite-0x121 = Big man
# BIGMAN1_ROOM_RS
sprite-0x154 = Big man
# BIGMAN2_ROOM_RS
sprite-0x155 = Big man
# BIGMAN, X's sprite, once in ORAS
sprite-0x70 = Big man
# OYAJI1_RS: Mauville, as are the four after it
sprite-0x122 = Middle-aged man
# OYAJI2_RS
sprite-0x123 = Middle-aged man
# OYAJI3_RS
sprite-0x124 = Middle-aged man
# OYAJI4_RS
sprite-0x125 = Middle-aged man
# OYAJI5_RS
sprite-0x126 = Middle-aged man
# WOMAN_RS
sprite-0x128 = Woman
# WOMAN2_RS
sprite-0x142 = Woman
# WOMAN_ROOM_RS
sprite-0x156 = Woman
# MAN_RS
sprite-0x132 = Man
# MAN_ROOM_RS
sprite-0x157 = Man
# MAN1, X's sprite, once in ORAS
sprite-0x31 = Man
# GIRL_RS
sprite-0x131 = Girl
# COUPLE_M_RS: the young couples' man
sprite-0x10c = Young man
# COUPLE_F_RS: and woman
sprite-0x10d = Young woman
# TWIN1_RS
sprite-0x10a = Twin
# TWIN2_RS
sprite-0x10b = Twin
# KARAKURI_RS: Route 110 only, the Trick House
sprite-0x127 = Trick Master
# SUNGLASS_RS
sprite-0x129 = Man in sunglasses
# SHOP_M_RS
sprite-0x12b = Clerk
# INFORMATION_RS
sprite-0x12c = Receptionist
# INFORMATIONW, X's sprite
sprite-0x3e = Receptionist
# DANPEI_RS: in all eight Gym towns, as X's DANPEI is in all eight Gyms
sprite-0x12d = Gym guide
# DELIVER1_RS: in all sixteen Pokémon Centers
sprite-0x12e = Delivery person
# DELIVER2_RS: likewise
sprite-0x12f = Delivery person
# ASSISTANT_M_RS
sprite-0x133 = Assistant
# ASSISTANTM, X's sprite
sprite-0x5f = Assistant
# KIMONO3_RS
sprite-0x137 = Woman in a kimono
# OFFICEWORKER_RS
sprite-0x140 = Office worker
# GAMEBOY_RS
sprite-0x141 = Boy playing a game
# REPORTER_RS
sprite-0x108 = Reporter
# CAMERAMAN_RS
sprite-0x109 = Cameraman
# ELDER_RS: the elder of each pair of Teammates, Anna & Meg and three more (spike 193)
sprite-0x10e = Older teammate
# JUNIOR_RS: and the younger
sprite-0x10f = Younger teammate

# Trainers' looks the ROM leaves unnamed, one trainer or none of a class standing as each, or a
# pair class beside the class (spike 90); called by the class their id and their trainers share.
# SCHOOL_M_RS
sprite-0xdd = Schoolkid
# SCHOOL_M_ROOM_RS
sprite-0x145 = Schoolkid
# SCHOOL_W_ROOM_RS
sprite-0x146 = Schoolkid
# TANPAN_ROOM_RS
sprite-0x143 = Youngster
# MINI_ROOM_RS
sprite-0x144 = Lass
# MOUNTMAN_ROOM_RS
sprite-0x147 = Hiker
# AROMA_ROOM_RS
sprite-0x148 = Aroma Lady
# SWIMMER_M_RS
sprite-0xe5 = Swimmer ♂
# UKIWA_W2_RS
sprite-0x15c = Tuber
# BREADER_M_RS
sprite-0xea = Pokémon Breeder
# BREADER_W_RS
sprite-0xeb = Pokémon Breeder
# FISHING_RS
sprite-0xef = Fisherman
# TRIRUN_M_RS
sprite-0x101 = Triathlete
# ELITE_M_ROOM_RS
sprite-0x149 = Ace Trainer
# ELITE_F_ROOM_RS
sprite-0x14a = Ace Trainer
# VETERAN_M_ROOM_RS
sprite-0x14b = Expert
# VETERAN_F_ROOM_RS
sprite-0x14c = Expert
# SCARY_M_ROOM_RS
sprite-0x14d = Street Thug
# SCARY_W_ROOM_RS
sprite-0x14e = Delinquent

# Things standing on the map that the game never names.
# omega_ruby_bush.ctm: "You can't pass because of prickly thorns, but a Pokémon may be able to cut the tree."
sprite-0x200b = bush
# omega_ruby_smash.ctm: "It's a cracked boulder. A Pokémon may be able to break it."
sprite-0x200c = cracked boulder
# omega_ruby_strength-boulders.ctm: "It's a big boulder, but a Pokémon may be able to push it aside."
sprite-0x2009 = big boulder

# Things standing on the map, by the model their sprite's record in a/1/3/7 draws (spike 211), and
# called what the game calls that thing among the Secret Base decorations (text 56) or the items
# (text 114).
# KIRAKIRA: the sparkle at each of the 28 places a Mega Stone lies, a script each from 0x41f
# to 0x43a (or_mega_stone: "Raiden found a Sablenite!"); the word is story 330:64's
sprite-0x203d = Mega Stone
# DOLL_RURIRI: text 56:111, in both houses in Littleroot
sprite-0x204a = Azurill Doll
# DOLL_ENEKO: text 56:112
sprite-0x204b = Skitty Doll
# DOLL_TIRUTTO: text 56:113
sprite-0x204c = Swablu Doll
# DOLL_RAPURASU: text 56:120
sprite-0x204d = Lapras Doll
# DOLL_KABIGON: text 56:118
sprite-0x204e = Snorlax Doll
# AIIRONOTAMA: text 114:535, on Mt. Pyre's summit
sprite-0x204f = Blue Orb
# BENIIRONOTAMA: text 114:534, beside it
sprite-0x2050 = Red Orb
# MAGMASUIT: text 114:741, in the Magma Hideout
sprite-0x2056 = Magma Suit
# AQUASUIT: text 114:742, in the Aqua Hideout
sprite-0x2057 = Aqua Suit
# ASASE_SHIO: text 114:70, in Shoal Cave
sprite-0x2058 = Shoal Salt
# ASASE_KAIGARA: text 114:71, in Shoal Cave
sprite-0x2059 = Shoal Shell
# LETTER: in Steven's house; the item, text 114:725, is the one to him
sprite-0x205b = letter

# The game's own words, copied from the ROM's English text by `spikes/90_sprite_trainers.py`: the
# story's people by the trainer each is, and every other sprite that two trainers or more of one
# class stand as, by that class (findings, spike 90). With the ROM, these come from it instead.
# RIVAL_RS
sprite-0xb4 = Wally
# RIVAL_EX_RS
sprite-0xb5 = Wally
# CHAMPION_RS
sprite-0xb6 = Steven
# BIGFOUR1_RS
sprite-0xb7 = Sidney
# BIGFOUR2_RS
sprite-0xb8 = Phoebe
# BIGFOUR3_RS
sprite-0xb9 = Glacia
# BIGFOUR4_RS
sprite-0xba = Drake
# LEADER1_RS
sprite-0xbb = Roxanne
# LEADER2_RS
sprite-0xbc = Brawly
# LEADER3_RS
sprite-0xbd = Wattson
# LEADER4_RS
sprite-0xbe = Flannery
# LEADER5_RS
sprite-0xbf = Norman
# LEADER6_RS
sprite-0xc0 = Winona
# LEADER8_RS
sprite-0xc3 = Wallace
# MAGMABOSS_RS
sprite-0xc4 = Maxie
# MAGMAKANBU_M_RS
sprite-0xc5 = Tabitha
# MAGMAKANBU_F_RS
sprite-0xc6 = Courtney
# AQUABOSS_RS
sprite-0xc9 = Archie
# AQUAKANBU_M_RS
sprite-0xca = Matt
# AQUAKANBU_F_RS
sprite-0xcb = Shelly
# HIMITU_RS
sprite-0xd3 = Aarune
# ZAZAKO_RS: Lorekeeper Zinnia, outside Sky Pillar (spike 193)
sprite-0xd4 = Zinnia
# SUPPORT_F_RS, drawn into work object 0x7a (spike 89)
sprite-0x13a = May
# SUPPORT_M_RS, likewise
sprite-0x13b = Brendan
# SUPPORT_F_BIKE_RS, by the ROM's own id; no replay yet
sprite-0x13c = May
# SUPPORT_M_BIKE_RS, likewise
sprite-0x13d = Brendan
# LEADER5A_RS: wally.ctm, "thanked as Mr. Norman"
sprite-0x13f = Norman
# LEADER2A_RS
sprite-0x15d = Brawly
# BIGFOUR2A_RS
sprite-0x15e = Phoebe
# LEADER6A_RS
sprite-0x15f = Winona
# BIGFOUR1A_RS
sprite-0x160 = Sidney
# MAGMA_M_RS
sprite-0xc7 = Team Magma Grunt
# MAGMA_F_RS
sprite-0xc8 = Team Magma Grunt
# AQUA_M_RS
sprite-0xcc = Team Aqua Grunt
# AQUA_F_RS
sprite-0xcd = Team Aqua Grunt
# TANPAN_RS
sprite-0xda = Youngster
# MINI_RS
sprite-0xdb = Lass
# MUSITORI_RS
sprite-0xdc = Bug Catcher
# SCHOOL_W_RS
sprite-0xde = Schoolkid
# CAMPBOY_RS
sprite-0xdf = Camper
# PICNICGIRL_RS
sprite-0xe0 = Picnicker
# RICHBOY_RS
sprite-0xe1 = Rich Boy
# RICHGIRL_RS
sprite-0xe2 = Lady
# UKIWA_M_RS
sprite-0xe3 = Tuber
# UKIWA_W_RS
sprite-0xe4 = Tuber
# SWIMMER_W_RS
sprite-0xe6 = Swimmer ♀
# GENTLEMAN_RS
sprite-0xe7 = Gentleman
# MARINER_RS
sprite-0xe8 = Sailor
# MOUNTMAN_RS
sprite-0xe9 = Hiker
# FIGHTER_W_RS
sprite-0xed = Battle Girl
# COLLECTOR_RS
sprite-0xee = Collector
# MONSTERMANIA_RS
sprite-0xf0 = Poké Maniac
# BUGMANIA_RS
sprite-0xf1 = Bug Maniac
# RUINMANIA_RS
sprite-0xf2 = Ruin Maniac
# OCCULT_RS
sprite-0xf3 = Hex Maniac
# ESP_M_RS
sprite-0xf4 = Psychic
# NINJA_RS
sprite-0xf5 = Ninja Boy
# CAMPFIRE_RS
sprite-0xf6 = Kindler
# BIRDTAMER_RS
sprite-0xf7 = Bird Keeper
# DRAGONTAMER_RS
sprite-0xf8 = Dragon Tamer
# GUITAR_RS
sprite-0xf9 = Guitarist
# RANGER_M_RS
sprite-0xfa = Pokémon Ranger
# RANGER_W_RS
sprite-0xfb = Pokémon Ranger
# ADULT_RS
sprite-0xfc = Beauty
# PARASOL_RS
sprite-0xfd = Parasol Lady
# AROMA_RS
sprite-0xfe = Aroma Lady
# DAISUKI_M_RS
sprite-0xff = Poké Fan
# DAISUKI_W_RS
sprite-0x100 = Poké Fan
# TRIBIKE_M_RS
sprite-0x102 = Triathlete
# TRISWIM_M_RS
sprite-0x103 = Triathlete
# ELITE_M_RS
sprite-0x104 = Ace Trainer
# SWIMMER_W2_RS
sprite-0x159 = Swimmer ♀
# FIGHTER_M_RS
sprite-0xec = Black Belt
# ELITE_F_RS
sprite-0x105 = Ace Trainer
# VETERAN_M_RS
sprite-0x106 = Expert
# FAIRYTALE_RS, beside two pairs of Mysterious Sisters (spike 193)
sprite-0x110 = Fairy Tale Girl
# VETERAN_F_RS
sprite-0x107 = Expert
# NAMINORI_M_RS
sprite-0x111 = Ace Trainer
# NAMINORI_F_RS
sprite-0x112 = Ace Trainer
# BACKPACKER_M_RS
sprite-0x113 = Backpacker
# SCARY_M_RS
sprite-0x114 = Street Thug
# SCARY_W_RS
sprite-0x115 = Delinquent
# DIVER_M_RS
sprite-0x116 = Scuba Diver
# DIVER_F_RS
sprite-0x117 = Free Diver
# FISHING2_RS
sprite-0x158 = Fisherman
# SWIMMER_M2_RS
sprite-0x15a = Swimmer ♂
# UKIWA_M2_RS
sprite-0x15b = Tuber

# The Pokémon on the map, by the Pokémon each one's model draws (spike 193); with the ROM, its
# own species names.
# RIDE_POKE01
sprite-0x1006 = Rhyhorn
# EV_POKE07
sprite-0x1009 = Lucario
# RIDE_POKE05
sprite-0x100f = Rhyhorn
# EV_POKE15
sprite-0x1013 = Marill
# RIDE_POKE01_EX
sprite-0x1014 = Rhyhorn
# EV_POKE20
sprite-0x101b = Groudon
# EV_POKE21
sprite-0x101d = Kyogre
# EV_POKE22
sprite-0x101f = Rayquaza
# EV_POKE23
sprite-0x1021 = Latias
# EV_POKE24
sprite-0x1022 = Latios
# RIDE_POKE13
sprite-0x1023 = Latias
# RIDE_POKE14
sprite-0x1024 = Latios
# EV_POKE25
sprite-0x1025 = Machoke
# EV_POKE26
sprite-0x1026 = Wingull
# EV_POKE28
sprite-0x1029 = Kecleon
# EV_POKE29
sprite-0x102a = Regirock
# EV_POKE30
sprite-0x102b = Regice
# EV_POKE31
sprite-0x102c = Registeel
# EV_POKE32
sprite-0x102d = Poochyena
# EV_POKE33
sprite-0x102e = Skitty
# EV_POKE34
sprite-0x102f = Zigzagoon
# EV_POKE36
sprite-0x1031 = Plusle
# EV_POKE37
sprite-0x1032 = Minun
# EV_POKE38
sprite-0x1033 = Meditite
# EV_POKE39
sprite-0x1034 = Shroomish
# EV_POKE40
sprite-0x1035 = Slakoth
# EV_POKE41
sprite-0x1036 = Roselia
# EV_POKE42
sprite-0x1037 = Taillow
# EV_POKE44
sprite-0x1039 = Whismur
# EV_POKE45
sprite-0x103a = Electrike
# EV_POKE46
sprite-0x103b = Altaria
# EV_POKE47
sprite-0x103c = Poochyena
# EV_POKE48
sprite-0x103d = Groudon
# EV_POKE49
sprite-0x103e = Kyogre
# EV_POKE50
sprite-0x103f = Machoke

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 155, which says what is there; the word here is taken from
# that line (spike 210). The PC's and the TV's are the game's own names for them. The PC's word is
# read from the ROM ahead of this one.
# the PC's own lines (archives.PC_LINES)
tile-0xd5 = PC
# text 56:138, the TV among the Secret Base decorations
tile-0xd7 = TV
# text 155:0
tile-0xd8 = Pokémon knickknacks
# text 155:9
tile-0xd9 = Pokémon magazines
# text 155:2
tile-0xda = cabinet
# text 155:3
tile-0xdb = useful items
# text 155:4
tile-0xdc = Pokémon novelties
# text 155:6
tile-0xdd = trash can
# text 155:7
tile-0xde = Pokémon picture books
# text 155:10
tile-0xdf = vase

## furniture
# What a piece of furniture is, by the game's shared script it runs: a record's own number from
# 0x7d0 up, or the one its zone's script runs for it (spike 210). Every vending machine runs one of
# 0x28f0 to 0x28f4, and is called what the game calls the one among the Secret Base decorations
# (text 56:144); the wardrobes say so (story 460:35).
# text 56:144
furniture-0x28f0 = vending machine
# text 56:144
furniture-0x28f1 = vending machine
# text 56:144
furniture-0x28f2 = vending machine
# text 56:144
furniture-0x28f3 = vending machine
# text 56:144
furniture-0x28f4 = vending machine
# story 460:35
furniture-0x284e = wardrobe
# The elevators' word is the Mauville Hills computer's, story 264:15; the bedrooms', the lines the
# player's room shows.
# Mauville's and the Lilycove Department Store's doors
furniture-0x27da = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27db = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27dc = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27dd = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27de = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27df = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e0 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e1 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e2 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e3 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e4 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e5 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e6 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e7 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e8 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e9 = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27ea = elevator
# Mauville's and the Lilycove Department Store's doors
furniture-0x27eb = elevator
# story 253:36, the player's room and the neighbour's
furniture-0x2956 = clock
# story 253:43, both rooms
furniture-0x2961 = Wii U
# text 114:442, the item; both rooms
furniture-0x2962 = Town Map
# story 253:41, both rooms
furniture-0x2963 = notebook
# Mom's, in both houses
furniture-0x2965 = notebook

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spike 211). A slot is in a
# group where its message opens as the group's does in any one language; Omega Ruby and Alpha
# Sapphire have the same.
# story 341:8, "There's a bed.", in the Magma Hideout
place-0xaa-furniture-0 = bed
place-0xaa-furniture-1 = bed
place-0xaa-furniture-2 = bed
place-0xaa-furniture-3 = bed
place-0xaa-furniture-4 = bed
place-0xaa-furniture-5 = bed
place-0xaa-furniture-6 = bed
place-0xaa-furniture-7 = bed
# story 420:16, in the Aqua Hideout
place-0x5d-furniture-0 = bed
place-0x5d-furniture-1 = bed
place-0x5d-furniture-2 = bed
place-0x5d-furniture-3 = bed
place-0x5d-furniture-4 = bed
place-0x5d-furniture-5 = bed
place-0x5d-furniture-6 = bed
place-0x5d-furniture-7 = bed
# story 169:14, at the Weather Institute
place-0x188-furniture-0 = bed
place-0x188-furniture-1 = bed
# story 70:0, "It's a bed.", on the S.S. Tidal
place-0xda-furniture-0 = bed
# story 58:5, "Aha! Pokémon Journal", in the Pokémon Centers
place-0xe4-furniture-0 = Pokémon Journal
place-0xe8-furniture-0 = Pokémon Journal
place-0xec-furniture-0 = Pokémon Journal
place-0xf0-furniture-0 = Pokémon Journal
place-0xf6-furniture-0 = Pokémon Journal
place-0xfd-furniture-0 = Pokémon Journal
place-0x103-furniture-0 = Pokémon Journal
place-0x109-furniture-0 = Pokémon Journal
place-0x116-furniture-0 = Pokémon Journal
place-0x11c-furniture-0 = Pokémon Journal
place-0x12d-furniture-0 = Pokémon Journal
place-0x135-furniture-0 = Pokémon Journal
place-0x14a-furniture-0 = Pokémon Journal
place-0x155-furniture-0 = Pokémon Journal
place-0x161-furniture-0 = Pokémon Journal
place-0x206-furniture-0 = Pokémon Journal
# story 587:13, "Hoenn's Local Heroes / The Hoenn Rangers Coexistence Force!": posters, figures, a face board
place-0x115-furniture-5 = Hoenn Rangers
place-0x115-furniture-6 = Hoenn Rangers
place-0x143-furniture-4 = Hoenn Rangers
place-0x143-furniture-5 = Hoenn Rangers
place-0x148-furniture-0 = Hoenn Rangers
place-0x151-furniture-1 = Hoenn Rangers
place-0x151-furniture-2 = Hoenn Rangers
place-0x188-furniture-4 = Hoenn Rangers
place-0x188-furniture-5 = Hoenn Rangers
place-0x1c2-furniture-10 = Hoenn Rangers
place-0x1c2-furniture-11 = Hoenn Rangers
place-0x20d-furniture-0 = Hoenn Rangers
# story 366:5, "The door is locked.", in Sea Mauville
place-0x91-furniture-1 = locked door
place-0x92-furniture-1 = locked door
place-0x92-furniture-2 = locked door
place-0x92-furniture-3 = locked door
place-0x92-furniture-4 = locked door
# story 206:0, the Trick House's scroll doors
place-0x175-furniture-0 = locked door
place-0x176-furniture-0 = locked door
place-0x177-furniture-0 = locked door
place-0x178-furniture-0 = locked door
place-0x179-furniture-0 = locked door
place-0x17a-furniture-0 = locked door
# story 341:0, "PROJECT AZOTH / File No. 1", in both hideouts
place-0xaa-furniture-8 = Project AZOTH
place-0xaa-furniture-9 = Project AZOTH
place-0xaa-furniture-10 = Project AZOTH
place-0xaa-furniture-11 = Project AZOTH
place-0xab-furniture-1 = Project AZOTH
place-0x5d-furniture-8 = Project AZOTH
place-0x5d-furniture-9 = Project AZOTH
place-0x5d-furniture-10 = Project AZOTH
place-0x5d-furniture-11 = Project AZOTH
place-0x5e-furniture-1 = Project AZOTH
# story 338:0, "It's the Team Magma flag!"
place-0xad-furniture-0 = Team Magma flag
place-0xad-furniture-1 = Team Magma flag
place-0xae-furniture-2 = Team Magma flag
# story 417:4, "It's the Team Aqua flag!"
place-0x60-furniture-4 = Team Aqua flag
place-0x60-furniture-5 = Team Aqua flag
place-0x61-furniture-2 = Team Aqua flag
# story 65:25, "It's an Azurill Doll!", where the doll stands
place-0xdf-furniture-0 = Azurill Doll
place-0xe1-furniture-0 = Azurill Doll
# in the moving truck
place-0x18f-furniture-0 = Azurill Doll
