### What to call the people, Pokémon and things on X and Y's maps in Traditional Chinese, and the
### use tiles, furniture and slots they stand among, when nothing better is known.
###
### The same as data/xy/en/sprites.ftl, in this language: a key is the sprite a move model is
### drawn with (findings, spikes 75, 89 and 100), and a name is looked for in order: what the
### player has called them, what the game calls them out of the player's own ROM, and only
### then this file.
###
### Each table names what the English one does (Orion, 2026-10-14). X and Y have no Chinese
### text, so no ROM can supply these words. Each one is the official Traditional Chinese name
### from a later game (Sun and Moon onward, as wiki.52poke.com lists its Traditional names) where
### one exists, and pa3ds's own word in Taiwan's usage otherwise, as the English one is for the
### Nurse, the looks named by their sprite's id, and the sparkles and cones. A look ORAS shares is
### called what data/oras/zh-TW/sprites.ftl calls it.
###
### Past [sprites], a key of [tiles] is a kind of use tile, a key of [furniture] the shared script a
### piece of furniture runs, and a key of [places] a slot, "zone:furniture:index" (spikes 83, 210,
### 211 and D).

## sprites
# People the ROM never lists as trainers, read off lines of its own text that name them, the
# same line in every language (spikes/108_x_people.py lines).
# Mom: "Mom's words echoed..." (system 140:43); ORAS's word
sprite-0x7 = 媽媽
# Looker: "It's me! Looker!" (system 102:16)
sprite-0x26 = 帥哥
# Alexa: "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = 三色菫
# Bonnie: "You can call me Bonnie." (story 404:0)
sprite-0x9b = 柚麗嘉
# the Mega Evolution guru: "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = 超級進化大叔
# the Game Director: "Hello! I am the Game Director." (story 272:0); ORAS's word
sprite-0x98 = 遊戲總監
# Nurse: PCWOMAN1, at the counter in all 16 Pokémon Centers (spike 101); ORAS's word
sprite-0x3b = 喬伊小姐
# en masked hero, pa3ds's own word as no official one was found: MYSTERYW and MYSTERYM: Geosenge Town, the Kalos Power Plant and a room of Lysandre Labs, zone 0x61, where "We're the masked heroes!" (story 35:25)
sprite-0x8 = 蒙面英雄
sprite-0x9 = 蒙面英雄

# The story's people by the trainer each is, and kinds of trainer by their class (findings,
# spikes 90 and 108), in the official names 52poke lists for X and Y's trainers and classes.
sprite-0x3 = 莎娜
sprite-0x4 = 提耶魯諾
sprite-0x5 = 多羅巴
sprite-0x6 = 布拉塔諾
sprite-0xa = 卡露妮
sprite-0xb = 雁鎧
sprite-0xc = 帕琦拉
sprite-0xd = 朵拉塞娜
sprite-0xe = 志米
sprite-0xf = 紫羅蘭
sprite-0x10 = 查克洛
sprite-0x11 = 可爾妮
sprite-0x12 = 福爺
sprite-0x13 = 希特隆
sprite-0x14 = 瑪綉
sprite-0x15 = 葛吉花
sprite-0x16 = 得撫
sprite-0x17 = 弗拉達利
sprite-0x21 = AZ
sprite-0x93 = 卡魯穆
sprite-0x94 = 莎莉娜
sprite-0x9c = 弗拉達利
sprite-0x9f = 卡魯穆
sprite-0xa0 = 莎莉娜
sprite-0x1a = 閃焰隊手下
sprite-0x1b = 閃焰隊手下
sprite-0x28 = 幼稚園小朋友
sprite-0x2a = 幼稚園小朋友
sprite-0x2c = 女學生
sprite-0x30 = 男學生
sprite-0x34 = 團體觀光客
sprite-0x35 = 背包客
sprite-0x43 = 短褲小子
sprite-0x44 = 發燒友俱樂部
sprite-0x47 = 迷你裙
sprite-0x4a = 對戰少女
sprite-0x4b = 溜冰手
sprite-0x4c = 溜冰手
sprite-0x4d = 釣魚者
sprite-0x4e = 新星訓練家
sprite-0x4f = 精英訓練家
sprite-0x50 = 資深訓練家
sprite-0x52 = 比基尼姐姐
sprite-0x55 = 團體觀光客
sprite-0x57 = 寶可夢培育家
sprite-0x58 = 超能力者
sprite-0x59 = 新星訓練家
sprite-0x5b = 資深訓練家
sprite-0x5d = 寶可夢巡護員
sprite-0x5f = 研究員
sprite-0x61 = 工人
sprite-0x64 = 童話少女
sprite-0x67 = 園藝師
sprite-0x70 = 登山男
sprite-0xa3 = 泳褲小伙子
sprite-0xa4 = 比基尼姐姐

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# Mr. Bonding: KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4); ORAS's word
sprite-0x77 = 牽絆大叔
# Dexio: MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = 德克希歐
# Sina: MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar (256:1)
sprite-0x8c = 吉娜
# Aliana: SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = 阿可碧
# Essentia: C02GIRL_EX: Lysandre Labs and Lumiose, Looker's story; trainer 511 (inferred from the id)
sprite-0x9e = 艾絲普莉
# Emma: C02GIRL: Lumiose, Looker's story; "Looker, you, Emma, Mimi..." (story 347:9) (inferred)
sprite-0x9d = 瑪琪艾兒
# Sina: MYSTERYW: Geosenge's masked heroes, "Who are these Sina and Dexio" (story 35:25) (inferred)
sprite-0x8 = 吉娜
# Dexio: MYSTERYM: likewise (inferred)
sprite-0x9 = 德克希歐

# Team Flare's admins and scientists: the admins by their trainers' class and name, as
# the grunts are; the scientists as story 378:5 calls them, not yet told apart.
# Team Flare Admin: FLAREKANBU1: the Power Plant, the Secret HQ; trainers "Admin" of class Team Flare
sprite-0x18 = 閃焰隊幹部
# Team Flare Admin: FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = 閃焰隊幹部
# Team Flare Scientist: SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = 閃焰隊科學家
# Team Flare Scientist: SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = 閃焰隊科學家
# Team Flare Scientist: SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = 閃焰隊科學家
# Team Flare Scientist: SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = 閃焰隊科學家

# Shared looks the game never names, called what the sprite's id says they are (text
# file 76), in ORAS's words for the same look where it has one. Left out:
# NONDRAW, STOPPER, the work objects WKOBJCODE00 to 04, and ENEMY3D and GIANTMAN02,
# which no line names.
# Little girl: CHILDGIRL1: Vaniville, Aquacorde
sprite-0x27 = 小女孩
# Little boy: CHILDBOY1: 7 zones
sprite-0x29 = 小男孩
# Girl: GIRL1: 10 zones
sprite-0x2b = 少女
# Girl: GIRL3: 17 zones
sprite-0x2e = 少女
# Boy: BOY1: 3 zones
sprite-0x2d = 少年
# Boy: BOY2: 13 zones
sprite-0x32 = 少年
# Woman: WOMAN1: 33 zones
sprite-0x2f = 女性
# Woman: WOMAN2: 34 zones, one Beauty among them
sprite-0x38 = 女性
# Man: MAN1: 36 zones
sprite-0x31 = 男性
# Middle-aged man: MIDDLEMAN2: 9 zones
sprite-0xa6 = 中年男性
# Gentleman: GENTLEMAN: 45 zones; X has no such class, Omega Ruby's class
sprite-0x37 = 紳士
# Clerk: SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = 店員
# Clerk: SHOPW1: 11 zones
sprite-0x3f = 店員
# Gym guide: DANPEI: in every Gym
sprite-0x3d = 道館嚮導
# Receptionist: INFORMATIONW: 37 zones, the gates
sprite-0x3e = 接待員
# Delivery person: DELIVER: in all 17 Pokémon Centers
sprite-0x40 = 配送員
# Delivery person: DELIVER2: likewise
sprite-0xa1 = 配送員
# Cameraman: CAMERAMAN: 14 zones
sprite-0x42 = 攝影師
# Twin: TWIN: two pairs of Twins among 10 zones
sprite-0x46 = 雙胞胎
# Twin: TWIN2: 4 zones
sprite-0xa2 = 雙胞胎
# Hairdresser: HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = 美髮師
# Café owner: MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = 咖啡店老闆
# Cyclist: CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = 自行車手
# Mannequin: HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = 人形模特兒
# Mannequin: HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = 人形模特兒
# Model: MODEL01: Route 7 (id only)
sprite-0xa7 = 模特兒
# Model: MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = 模特兒
# Model: MODEL03: likewise (id only)
sprite-0xa9 = 模特兒

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# Punk Girl: BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = 壞女孩
# Poké Fan: MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = 發燒友俱樂部
# Artist: ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = 藝術家
# Artist: ARTISTW: one Artist
sprite-0x54 = 藝術家
# Pokémon Breeder: BREEDERM: one Pokémon Breeder
sprite-0x56 = 寶可夢培育家
# Scientist: ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = 研究員
# Worker: WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = 工人
# Furisode Girl: KIMONO1: one Furisode Girl
sprite-0x68 = 長袖和服少女
# Furisode Girl: KIMONO2: one Furisode Girl
sprite-0x69 = 長袖和服少女
# Furisode Girl: KIMONO3: one Furisode Girl
sprite-0x6a = 長袖和服少女
# Furisode Girl: KIMONO4: one Furisode Girl
sprite-0x6b = 長袖和服少女
# Lady: LADY: 19 zones, no trainer
sprite-0x36 = 千金小姐
# Waitress: WAITRESS: 24 zones, no trainer
sprite-0x49 = 女服務生
# Garçon: GARCON: 25 zones, no trainer
sprite-0x72 = 男服務生
# Chef: COOK: 12 zones, no trainer
sprite-0x5e = 廚師
# Butler: BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = 管家
# Maid: MAID: 32 zones, the Pokémon Centers
sprite-0x66 = 女僕
# Sky Trainer: SKYM: 8 zones, no trainer
sprite-0x6c = 天空訓練家
# Sky Trainer: SKYW: 8 zones, no trainer
sprite-0x6d = 天空訓練家
# Fisherman: FISHING2: 4 zones, as FISHING is
sprite-0xa5 = 釣魚者

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = 團體觀光客
# BADMAN
sprite-0x39 = 壞男孩
# TRAINERM
sprite-0x5a = 精英訓練家
# RANGERM
sprite-0x5c = 寶可夢巡護員
# OCCULTW
sprite-0x63 = 靈異迷
# FIGHTERM
sprite-0x71 = 空手道王

# The Pokémon on the map, by the species each one's model draws, its tag read through
# the battle models (spike 218); with the ROM, its own species names. Mewtwo's model
# names none.
# Mewtwo: P_MODEL01: the Unknown Dungeon (story 185:1); its model names no species
sprite-0x1017 = 超夢
# EV_POKE01
sprite-0x1000 = 南瓜精
# EV_POKE02
sprite-0x1001 = 掘地兔
# EV_POKE03
sprite-0x1002 = 基格爾德
# EV_POKE04
sprite-0x1003 = 小獅獅
# EV_POKE05
sprite-0x1004 = 坐騎山羊
# RIDE_POKE02
sprite-0x1007 = 象牙豬
# RIDE_POKE04
sprite-0x1008 = 坐騎小羊
# EV_POKE07
sprite-0x1009 = 路卡利歐
# EV_POKE08
sprite-0x100a = 多麗米亞
# EV_POKE10
sprite-0x100b = 小箭雀
# EV_POKE11
sprite-0x100c = 卡比獸
# EV_POKE12
sprite-0x100d = 妙喵
# RIDE_POKE05
sprite-0x100f = 獨角犀牛
# EV_POKE09
sprite-0x1010 = 暴雪王
# EV_POKE14
sprite-0x1011 = 坐騎小羊
# EV_POKE15
sprite-0x1013 = 瑪力露

# Things standing on the map that the game never names, called by a word of the line the shared
# script they run shows when the player faces one (a/0/3/1 file 27, story 130; spike 219), as
# ORAS's are (spike 137), in ORAS's words where it has them.
# en bush: ITREE01, runs 0x2712: story 130:3
sprite-0x200b = 灌木叢
# en cracked boulder: IROCK01, runs 0x2710: story 130:1
sprite-0x200c = 有裂縫的岩石
# en big boulder: KROCK01, runs 0x2714: story 130:5
sprite-0x2009 = 巨大岩石
# en cracked wall: IWALL01, runs 0x2716: story 130:9
sprite-0x2034 = 有裂縫的牆壁
# en rock: ISNOW01: ten on Route 17 (story 70:5)
sprite-0x2035 = 岩石
# en rock: IROCK02: ten on Route 9, crossed on Rhyhorn; no line, the word of story 70:5
sprite-0x200d = 岩石

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# en Central Kalos Pokédex award: AWARD04, case 0xd: story 55:4
sprite-0x203b = 卡洛斯中央地區圖鑑獎狀
# en Coastal Kalos Pokédex award: AWARD03, case 0xe: story 55:5
sprite-0x203a = 卡洛斯海岸地區圖鑑獎狀
# en Mountain Kalos Pokédex award: AWARD05, case 0xf: story 55:6
sprite-0x203c = 卡洛斯山岳地區圖鑑獎狀
# en Kalos region award: AWARD01, case 0x10: story 55:7
sprite-0x2036 = 卡洛斯地區獎狀
# en National Pokédex award: AWARD02, case 0x11: story 55:8
sprite-0x2037 = 全國圖鑑獎狀
# en Berry tree: KINOMI, runs 0x2939: story 142:15; Route 15
sprite-0x200e = 樹果樹
# en Berry tree: KINOMI2, Routes 5 and 19
sprite-0x2011 = 樹果樹
# en Berry tree: KINOMI3, Routes 11 and 12
sprite-0x2012 = 樹果樹
# en Berry tree: KINOMI4, Routes 16 and 22
sprite-0x2013 = 樹果樹
# en Berry tree: KINOMI11, Route 7
sprite-0x201a = 樹果樹
# en Berry tree: KINOMI15, Route 18
sprite-0x201e = 樹果樹
# en Berry tree: KINOMI31, Route 21
sprite-0x202e = 樹果樹
# en Berry tree: KINOMI33, Route 10
sprite-0x2030 = 樹果樹
# en Berry tree: KINOMI34, Route 6
sprite-0x2031 = 樹果樹
# en Berry tree: KINOMI35, Routes 8 and 14
sprite-0x2032 = 樹果樹
# en Berry tree: KINOMI36, Route 20
sprite-0x2033 = 樹果樹

# Things the game never names, called what the ROM's own id for the sprite says they are.
# en sparkle: KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower
sprite-0x203d = 閃光
# en sparkle: KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone
sprite-0x2042 = 閃光
# en cone: CONE, model com_bm_skatecone: two across a Lumiose street, seven in Lysandre Labs
sprite-0x203e = 三角錐
# en shaking grass: TOBI_KUSA, no script: three on Route 6 and three on Route 18
sprite-0x2002 = 搖晃的草叢
# en lurking Pokémon: ENEMY3D, shared script 2000: nine in the Glittering Cave (spike 239)
sprite-0x95 = 潛伏的寶可夢

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 132, which says what is there; the word here is taken from
# that line (spike 220), in ORAS's word where the English is ORAS's too.
# PC: the PC's own lines (archives.PC_LINES)
tile-0xd5 = 電腦
# TV: story 390:19, "a TV and a washing machine": X has no line naming a TV alone
tile-0xd7 = 電視
# Pokémon knickknacks: text 132:0
tile-0xd8 = 寶可夢擺飾
# Pokémon books: text 132:1, books and photo albums
tile-0xd9 = 寶可夢書籍
# cabinet: text 132:2, ORAS's line and words
tile-0xda = 櫃子
# useful items: text 132:3
tile-0xdb = 日用道具
# Pokémon novelties: text 132:4
tile-0xdc = 寶可夢周邊
# trash can: text 132:6
tile-0xdd = 垃圾桶
# books for Pokémon: text 132:5, "Are these books for Pokémon to read?"
tile-0xde = 給寶可夢看的書

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# elevator: story 137:21 and 145:11: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = 電梯
furniture-0x27db = 電梯
furniture-0x27dc = 電梯
furniture-0x27dd = 電梯
furniture-0x27de = 電梯
furniture-0x27df = 電梯
furniture-0x27e0 = 電梯
furniture-0x27e1 = 電梯
furniture-0x27e2 = 電梯
furniture-0x27e3 = 電梯
furniture-0x27e4 = 電梯
furniture-0x27e5 = 電梯
furniture-0x27e6 = 電梯
furniture-0x27e7 = 電梯
furniture-0x27e8 = 電梯
furniture-0x27e9 = 電梯
furniture-0x27ea = 電梯
furniture-0x27eb = 電梯
furniture-0x27ec = 電梯
furniture-0x27ed = 電梯
furniture-0x27ee = 電梯
furniture-0x27ef = 電梯
furniture-0x27f0 = 電梯
furniture-0x27f1 = 電梯
furniture-0x27f2 = 電梯
furniture-0x27f3 = 電梯
furniture-0x27f4 = 電梯
furniture-0x27f5 = 電梯
furniture-0x27f6 = 電梯
furniture-0x27f7 = 電梯
furniture-0x27f8 = 電梯
furniture-0x27f9 = 電梯
furniture-0x27fa = 電梯
furniture-0x27fb = 電梯
furniture-0x27fc = 電梯
furniture-0x27fd = 電梯
furniture-0x27fe = 電梯
furniture-0x27ff = 電梯
furniture-0x2800 = 電梯
furniture-0x2801 = 電梯
furniture-0x2802 = 電梯
furniture-0x2803 = 電梯
furniture-0x2804 = 電梯
furniture-0x2805 = 電梯
furniture-0x2806 = 電梯
furniture-0x2807 = 電梯
furniture-0x2808 = 電梯
furniture-0x2809 = 電梯
# meteor: story 221:8, Ambrette Town
furniture-0x281e = 隕石
# cardboard boxes: story 220:0, Lumiose City, zone 0x54
furniture-0x284b = 紙箱
# photo spot: story 222:0, one in each of 12 places
furniture-0x28c8 = 拍照點
furniture-0x28c9 = 拍照點
furniture-0x28ca = 拍照點
furniture-0x28cb = 拍照點
furniture-0x28cc = 拍照點
furniture-0x28cd = 拍照點
furniture-0x28ce = 拍照點
furniture-0x28cf = 拍照點
furniture-0x28d0 = 拍照點
furniture-0x28d1 = 拍照點
furniture-0x28d2 = 拍照點
furniture-0x28d3 = 拍照點
# trash can: story 142:14, Pokémon Village and the Lost Hotel
furniture-0x28f0 = 垃圾桶
furniture-0x28f1 = 垃圾桶
furniture-0x28f2 = 垃圾桶
furniture-0x28f3 = 垃圾桶
furniture-0x28f4 = 垃圾桶
furniture-0x28f5 = 垃圾桶
furniture-0x28f6 = 垃圾桶
furniture-0x28f7 = 垃圾桶
furniture-0x28f8 = 垃圾桶
furniture-0x28f9 = 垃圾桶
# Berry tree: story 142:15, its Berries and its tree, Routes 5 to 22
furniture-0x2922 = 樹果樹
furniture-0x2923 = 樹果樹
furniture-0x2924 = 樹果樹
furniture-0x2925 = 樹果樹
furniture-0x2926 = 樹果樹
furniture-0x2927 = 樹果樹
furniture-0x2928 = 樹果樹
furniture-0x2929 = 樹果樹
furniture-0x292a = 樹果樹
furniture-0x292b = 樹果樹
furniture-0x292c = 樹果樹
furniture-0x292d = 樹果樹
furniture-0x292e = 樹果樹
furniture-0x292f = 樹果樹
furniture-0x2930 = 樹果樹
# fitting room: text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = 試衣間
# fitting room: text 142:85, the 7 boutiques' fitting room, called as the Pokémon Centers' is
furniture-0x29d3 = 試衣間
# clothes: text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = 衣服
# bed: story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = 床
# taxi stand: story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = 計程車招呼站
# Gogoat Shuttle: story 399:218, Lumiose's 12 stops
furniture-0x2a3b = 坐騎山羊接駁車
furniture-0x2a3c = 坐騎山羊接駁車
furniture-0x2a3d = 坐騎山羊接駁車
furniture-0x2a3e = 坐騎山羊接駁車
furniture-0x2a3f = 坐騎山羊接駁車
furniture-0x2a40 = 坐騎山羊接駁車
furniture-0x2a41 = 坐騎山羊接駁車
furniture-0x2a42 = 坐騎山羊接駁車
furniture-0x2a43 = 坐騎山羊接駁車
furniture-0x2a44 = 坐騎山羊接駁車
furniture-0x2a45 = 坐騎山羊接駁車
furniture-0x2a46 = 坐騎山羊接駁車

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# soft soil: story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = 鬆軟的土壤
place-0x160-furniture-3 = 鬆軟的土壤
place-0x160-furniture-4 = 鬆軟的土壤
place-0x160-furniture-5 = 鬆軟的土壤
place-0x160-furniture-6 = 鬆軟的土壤
place-0x160-furniture-7 = 鬆軟的土壤
place-0x160-furniture-8 = 鬆軟的土壤
place-0x160-furniture-9 = 鬆軟的土壤
place-0x160-furniture-10 = 鬆軟的土壤
place-0x160-furniture-11 = 鬆軟的土壤
place-0x160-furniture-12 = 鬆軟的土壤
place-0x160-furniture-13 = 鬆軟的土壤
place-0x160-furniture-14 = 鬆軟的土壤
place-0x160-furniture-15 = 鬆軟的土壤
place-0x160-furniture-16 = 鬆軟的土壤
place-0x160-furniture-17 = 鬆軟的土壤
place-0x160-furniture-18 = 鬆軟的土壤
place-0x160-furniture-19 = 鬆軟的土壤
place-0x160-furniture-20 = 鬆軟的土壤
place-0x160-furniture-21 = 鬆軟的土壤
place-0x160-furniture-22 = 鬆軟的土壤
place-0x160-furniture-23 = 鬆軟的土壤
place-0x160-furniture-24 = 鬆軟的土壤
place-0x160-furniture-25 = 鬆軟的土壤
place-0x160-furniture-26 = 鬆軟的土壤
place-0x160-furniture-27 = 鬆軟的土壤
place-0x160-furniture-28 = 鬆軟的土壤
place-0x160-furniture-29 = 鬆軟的土壤
place-0x160-furniture-30 = 鬆軟的土壤
place-0x160-furniture-31 = 鬆軟的土壤
place-0x160-furniture-32 = 鬆軟的土壤
place-0x160-furniture-33 = 鬆軟的土壤
place-0x160-furniture-34 = 鬆軟的土壤
place-0x160-furniture-35 = 鬆軟的土壤
place-0x160-furniture-36 = 鬆軟的土壤
place-0x160-furniture-37 = 鬆軟的土壤
# audio guide: story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = 語音導覽
place-0x68-furniture-1 = 語音導覽
place-0x68-furniture-2 = 語音導覽
place-0x68-furniture-3 = 語音導覽
place-0x68-furniture-4 = 語音導覽
place-0x68-furniture-5 = 語音導覽
place-0x68-furniture-6 = 語音導覽
place-0x69-furniture-0 = 語音導覽
place-0x69-furniture-1 = 語音導覽
place-0x69-furniture-2 = 語音導覽
place-0x69-furniture-3 = 語音導覽
place-0x69-furniture-4 = 語音導覽
place-0x69-furniture-5 = 語音導覽
place-0x69-furniture-6 = 語音導覽
place-0x6a-furniture-0 = 語音導覽
place-0x6a-furniture-1 = 語音導覽
place-0x6a-furniture-2 = 語音導覽
place-0x6a-furniture-3 = 語音導覽
place-0x6a-furniture-4 = 語音導覽
place-0x6a-furniture-5 = 語音導覽
place-0x6a-furniture-6 = 語音導覽
place-0x6a-furniture-7 = 語音導覽
place-0x6a-furniture-8 = 語音導覽
place-0x6a-furniture-9 = 語音導覽
# statue of a Bisharp: story 215:21, Parfum Palace
place-0x120-furniture-7 = 劈斬司令雕像
place-0x120-furniture-8 = 劈斬司令雕像
place-0x120-furniture-9 = 劈斬司令雕像
place-0x120-furniture-10 = 劈斬司令雕像
place-0x120-furniture-11 = 劈斬司令雕像
place-0x120-furniture-12 = 劈斬司令雕像
place-0x120-furniture-13 = 劈斬司令雕像
place-0x120-furniture-14 = 劈斬司令雕像
place-0x125-furniture-2 = 劈斬司令雕像
place-0x125-furniture-3 = 劈斬司令雕像
place-0x127-furniture-2 = 劈斬司令雕像
place-0x127-furniture-3 = 劈斬司令雕像
place-0x12c-furniture-3 = 劈斬司令雕像
place-0x12c-furniture-4 = 劈斬司令雕像
# golden statue of a knight: story 215:22, Parfum Palace
place-0x120-furniture-15 = 黃金騎士像
place-0x120-furniture-16 = 黃金騎士像
place-0x120-furniture-17 = 黃金騎士像
place-0x120-furniture-18 = 黃金騎士像
place-0x120-furniture-19 = 黃金騎士像
place-0x120-furniture-20 = 黃金騎士像
place-0x120-furniture-21 = 黃金騎士像
place-0x120-furniture-22 = 黃金騎士像
place-0x120-furniture-23 = 黃金騎士像
place-0x120-furniture-24 = 黃金騎士像
place-0x125-furniture-4 = 黃金騎士像
place-0x125-furniture-5 = 黃金騎士像
place-0x127-furniture-4 = 黃金騎士像
place-0x127-furniture-5 = 黃金騎士像
# whiteboard: story 412:7, Santalune City, zone 0x3e
place-0x3e-furniture-0 = 白板
place-0x3e-furniture-1 = 白板
place-0x3e-furniture-2 = 白板
place-0x3e-furniture-3 = 白板
place-0x3e-furniture-4 = 白板
place-0x3e-furniture-5 = 白板
place-0x3e-furniture-6 = 白板
place-0x3e-furniture-7 = 白板
place-0x3e-furniture-8 = 白板
# statue of a Poké Ball: story 216:47, Parfum Palace's garden
place-0x12e-furniture-2 = 精靈球雕像
place-0x12e-furniture-3 = 精靈球雕像
place-0x12e-furniture-4 = 精靈球雕像
place-0x12e-furniture-5 = 精靈球雕像
place-0x12e-furniture-6 = 精靈球雕像
place-0x12e-furniture-7 = 精靈球雕像
# monument: story 226:7, the Battle Maison
place-0xf1-furniture-0 = 紀念碑
place-0xf1-furniture-1 = 紀念碑
place-0xf1-furniture-2 = 紀念碑
place-0xf1-furniture-3 = 紀念碑
place-0xf1-furniture-4 = 紀念碑
# portrait: story 203:0, Parfum Palace
place-0x12c-furniture-0 = 肖像畫
place-0x12c-furniture-1 = 肖像畫
place-0x12c-furniture-2 = 肖像畫
# record machine: story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = 唱片機
place-0x54-furniture-3 = 唱片機
place-0x54-furniture-4 = 唱片機
place-0x54-furniture-5 = 唱片機
# PC: story 55:1, the bedroom's PC
place-0x2-furniture-0 = 電腦
# mirror: story 55:2
place-0x2-furniture-1 = 鏡子
# TV: story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = 電視
# TV: story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = 電視
# Wii U: story 55:0
place-0x2-furniture-4 = Wii U
# statue: story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = 雕像
place-0x37-furniture-1 = 雕像
# pole: story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = 桿子
place-0x38-furniture-0 = 桿子
# photographs: story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = 照片
place-0x37-furniture-4 = 照片
place-0x37-furniture-5 = 照片
# photograph: story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = 照片
place-0x37-furniture-7 = 照片
# Lysandre Café: story 313:4, Lumiose City, zone 0x43; the official Traditional name (wiki.52poke.com/wiki/弗拉达利咖啡店)
place-0x43-warp-1 = 弗拉達利咖啡店
# bed: story 374:0, Lysandre Labs, zone 0x64: "The bed looks nice and comfortable."; pa3ds's own word
place-0x64-furniture-0 = 床
place-0x64-furniture-1 = 床
place-0x64-furniture-2 = 床
place-0x64-furniture-3 = 床
place-0x64-furniture-4 = 床
place-0x64-furniture-5 = 床
