### What to call the people, Pokémon and things on X and Y's maps in Simplified Chinese, and the
### use tiles, furniture and slots they stand among, when nothing better is known.
###
### The same as data/xy/en/sprites.ftl, in this language: a key is the sprite a move model is
### drawn with (findings, spikes 75, 89 and 100), and a name is looked for in order: what the
### player has called them, what the game calls them out of the player's own ROM, and only
### then this file.
###
### Each table names what the English one does (Orion, 2026-10-14). X and Y have no Chinese
### text, so no ROM can supply these words. Each one is the official Simplified Chinese name
### from a later game (Sun and Moon onward, as wiki.52poke.com lists them) where one exists,
### and pa3ds's own word otherwise, as the English one is for the Nurse, the looks named by
### their sprite's id, and the sparkles and cones. A look ORAS shares is called what
### data/oras/zh-CN/sprites.ftl calls it.
###
### Past [sprites], a key of [tiles] is a kind of use tile, a key of [furniture] the shared script a
### piece of furniture runs, and a key of [places] a slot, "zone:furniture:index" (spikes 83, 210,
### 211 and D).

## sprites
# People the ROM never lists as trainers, read off lines of its own text that name them, the
# same line in every language (spikes/108_x_people.py lines).
# Mom: "Mom's words echoed..." (system 140:43); ORAS's word
sprite-0x7 = 妈妈
# Looker: "It's me! Looker!" (system 102:16)
sprite-0x26 = 帅哥
# Alexa: "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = 三色堇
# Bonnie: "You can call me Bonnie." (story 404:0)
sprite-0x9b = 柚丽嘉
# the Mega Evolution guru: "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = 超级进化大叔
# the Game Director: "Hello! I am the Game Director." (story 272:0); ORAS's word
sprite-0x98 = 游戏总监
# Nurse: PCWOMAN1, at the counter in all 16 Pokémon Centers (spike 101); ORAS's word
sprite-0x3b = 乔伊小姐

# The story's people by the trainer each is, and kinds of trainer by their class (findings,
# spikes 90 and 108), in the official names 52poke lists for X and Y's trainers and classes.
sprite-0x3 = 莎娜
sprite-0x4 = 提耶鲁诺
sprite-0x5 = 多罗巴
sprite-0x6 = 布拉塔诺
sprite-0xa = 卡露妮
sprite-0xb = 雁铠
sprite-0xc = 帕琦拉
sprite-0xd = 朵拉塞娜
sprite-0xe = 志米
sprite-0xf = 紫罗兰
sprite-0x10 = 查克洛
sprite-0x11 = 可尔妮
sprite-0x12 = 福爷
sprite-0x13 = 希特隆
sprite-0x14 = 玛绣
sprite-0x15 = 葛吉花
sprite-0x16 = 得抚
sprite-0x17 = 弗拉达利
sprite-0x21 = AZ
sprite-0x93 = 卡鲁穆
sprite-0x94 = 莎莉娜
sprite-0x9c = 弗拉达利
sprite-0x9f = 卡鲁穆
sprite-0xa0 = 莎莉娜
sprite-0x1a = 闪焰队手下
sprite-0x1b = 闪焰队手下
sprite-0x28 = 幼儿园小朋友
sprite-0x2a = 幼儿园小朋友
sprite-0x2c = 女学生
sprite-0x30 = 男学生
sprite-0x34 = 团体游客
sprite-0x35 = 背包客
sprite-0x43 = 短裤小子
sprite-0x44 = 发烧友俱乐部
sprite-0x47 = 迷你裙
sprite-0x4a = 对战少女
sprite-0x4b = 轮滑手
sprite-0x4c = 轮滑手
sprite-0x4d = 垂钓者
sprite-0x4e = 新星训练家
sprite-0x4f = 精英训练家
sprite-0x50 = 资深训练家
sprite-0x52 = 比基尼姐姐
sprite-0x55 = 团体游客
sprite-0x57 = 宝可梦培育家
sprite-0x58 = 超能力者
sprite-0x59 = 新星训练家
sprite-0x5b = 资深训练家
sprite-0x5d = 宝可梦巡护员
sprite-0x5f = 研究员
sprite-0x61 = 工人
sprite-0x64 = 童话少女
sprite-0x67 = 园艺师
sprite-0x70 = 登山男
sprite-0xa3 = 泳裤小伙子
sprite-0xa4 = 比基尼姐姐

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# Mr. Bonding: KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4); ORAS's word
sprite-0x77 = 邦迪先生
# Dexio: MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = 德克希欧
# Sina: MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar (256:1)
sprite-0x8c = 吉娜
# Aliana: SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = 阿可碧
# Essentia: C02GIRL_EX: Lysandre Labs and Lumiose, Looker's story; trainer 511 (inferred from the id)
sprite-0x9e = 艾丝普莉
# Emma: C02GIRL: Lumiose, Looker's story; "Looker, you, Emma, Mimi..." (story 347:9) (inferred)
sprite-0x9d = 玛琪艾儿
# Sina: MYSTERYW: Geosenge's masked heroes, "Who are these Sina and Dexio" (story 35:25) (inferred)
sprite-0x8 = 吉娜
# Dexio: MYSTERYM: likewise (inferred)
sprite-0x9 = 德克希欧

# Team Flare's admins and scientists: the admins by their trainers' class and name, as
# the grunts are; the scientists as story 378:5 calls them, not yet told apart.
# Team Flare Admin: FLAREKANBU1: the Power Plant, the Secret HQ; trainers "Admin" of class Team Flare
sprite-0x18 = 闪焰队干部
# Team Flare Admin: FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = 闪焰队干部
# Team Flare Scientist: SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = 闪焰队科学家
# Team Flare Scientist: SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = 闪焰队科学家
# Team Flare Scientist: SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = 闪焰队科学家
# Team Flare Scientist: SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = 闪焰队科学家

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
sprite-0x37 = 绅士
# Clerk: SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = 店员
# Clerk: SHOPW1: 11 zones
sprite-0x3f = 店员
# Gym guide: DANPEI: in every Gym
sprite-0x3d = 道馆向导
# Receptionist: INFORMATIONW: 37 zones, the gates
sprite-0x3e = 接待员
# Delivery person: DELIVER: in all 17 Pokémon Centers
sprite-0x40 = 配送员
# Delivery person: DELIVER2: likewise
sprite-0xa1 = 配送员
# Cameraman: CAMERAMAN: 14 zones
sprite-0x42 = 摄像师
# Twin: TWIN: two pairs of Twins among 10 zones
sprite-0x46 = 双胞胎
# Twin: TWIN2: 4 zones
sprite-0xa2 = 双胞胎
# Hairdresser: HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = 美发师
# Café owner: MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = 咖啡店老板
# Cyclist: CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = 自行车手
# Mannequin: HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = 人体模特
# Mannequin: HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = 人体模特
# Model: MODEL01: Route 7 (id only)
sprite-0xa7 = 模特
# Model: MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = 模特
# Model: MODEL03: likewise (id only)
sprite-0xa9 = 模特

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# Punk Girl: BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = 坏女孩
# Poké Fan: MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = 发烧友俱乐部
# Artist: ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = 艺术家
# Artist: ARTISTW: one Artist
sprite-0x54 = 艺术家
# Pokémon Breeder: BREEDERM: one Pokémon Breeder
sprite-0x56 = 宝可梦培育家
# Scientist: ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = 研究员
# Worker: WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = 工人
# Furisode Girl: KIMONO1: one Furisode Girl
sprite-0x68 = 长袖和服少女
# Furisode Girl: KIMONO2: one Furisode Girl
sprite-0x69 = 长袖和服少女
# Furisode Girl: KIMONO3: one Furisode Girl
sprite-0x6a = 长袖和服少女
# Furisode Girl: KIMONO4: one Furisode Girl
sprite-0x6b = 长袖和服少女
# Lady: LADY: 19 zones, no trainer
sprite-0x36 = 千金小姐
# Waitress: WAITRESS: 24 zones, no trainer
sprite-0x49 = 女服务生
# Garçon: GARCON: 25 zones, no trainer
sprite-0x72 = 男服务生
# Chef: COOK: 12 zones, no trainer
sprite-0x5e = 厨师
# Butler: BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = 管家
# Maid: MAID: 32 zones, the Pokémon Centers
sprite-0x66 = 女仆
# Sky Trainer: SKYM: 8 zones, no trainer
sprite-0x6c = 天空训练家
# Sky Trainer: SKYW: 8 zones, no trainer
sprite-0x6d = 天空训练家
# Fisherman: FISHING2: 4 zones, as FISHING is
sprite-0xa5 = 垂钓者

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = 团体游客
# BADMAN
sprite-0x39 = 坏男孩
# TRAINERM
sprite-0x5a = 精英训练家
# RANGERM
sprite-0x5c = 宝可梦巡护员
# OCCULTW
sprite-0x63 = 灵异迷
# FIGHTERM
sprite-0x71 = 空手道王

# The Pokémon on the map, by the species each one's model draws, its tag read through
# the battle models (spike 218); with the ROM, its own species names. Mewtwo's model
# names none.
# Mewtwo: P_MODEL01: the Unknown Dungeon (story 185:1); its model names no species
sprite-0x1017 = 超梦
# EV_POKE01
sprite-0x1000 = 南瓜精
# EV_POKE02
sprite-0x1001 = 掘地兔
# EV_POKE03
sprite-0x1002 = 基格尔德
# EV_POKE04
sprite-0x1003 = 小狮狮
# EV_POKE05
sprite-0x1004 = 坐骑山羊
# RIDE_POKE02
sprite-0x1007 = 象牙猪
# RIDE_POKE04
sprite-0x1008 = 坐骑小羊
# EV_POKE07
sprite-0x1009 = 路卡利欧
# EV_POKE08
sprite-0x100a = 多丽米亚
# EV_POKE10
sprite-0x100b = 小箭雀
# EV_POKE11
sprite-0x100c = 卡比兽
# EV_POKE12
sprite-0x100d = 妙喵
# RIDE_POKE05
sprite-0x100f = 独角犀牛
# EV_POKE09
sprite-0x1010 = 暴雪王
# EV_POKE14
sprite-0x1011 = 坐骑小羊
# EV_POKE15
sprite-0x1013 = 玛力露

# Things standing on the map that the game never names, called by a word of the line the shared
# script they run shows when the player faces one (a/0/3/1 file 27, story 130; spike 219), as
# ORAS's are (spike 137), in ORAS's words where it has them.
# en bush: ITREE01, runs 0x2712: story 130:3
sprite-0x200b = 灌木丛
# en cracked boulder: IROCK01, runs 0x2710: story 130:1
sprite-0x200c = 有裂缝的岩石
# en big boulder: KROCK01, runs 0x2714: story 130:5
sprite-0x2009 = 巨大岩石
# en cracked wall: IWALL01, runs 0x2716: story 130:9
sprite-0x2034 = 有裂缝的墙壁
# en rock: ISNOW01: ten on Route 17 (story 70:5)
sprite-0x2035 = 岩石
# en rock: IROCK02: ten on Route 9, crossed on Rhyhorn; no line, the word of story 70:5
sprite-0x200d = 岩石

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# en Central Kalos Pokédex award: AWARD04, case 0xd: story 55:4
sprite-0x203b = 卡洛斯中央地区图鉴奖状
# en Coastal Kalos Pokédex award: AWARD03, case 0xe: story 55:5
sprite-0x203a = 卡洛斯海岸地区图鉴奖状
# en Mountain Kalos Pokédex award: AWARD05, case 0xf: story 55:6
sprite-0x203c = 卡洛斯山岳地区图鉴奖状
# en Kalos region award: AWARD01, case 0x10: story 55:7
sprite-0x2036 = 卡洛斯地区奖状
# en National Pokédex award: AWARD02, case 0x11: story 55:8
sprite-0x2037 = 全国图鉴奖状
# en Berry tree: KINOMI, runs 0x2939: story 142:15; Route 15
sprite-0x200e = 树果树
# en Berry tree: KINOMI2, Routes 5 and 19
sprite-0x2011 = 树果树
# en Berry tree: KINOMI3, Routes 11 and 12
sprite-0x2012 = 树果树
# en Berry tree: KINOMI4, Routes 16 and 22
sprite-0x2013 = 树果树
# en Berry tree: KINOMI11, Route 7
sprite-0x201a = 树果树
# en Berry tree: KINOMI15, Route 18
sprite-0x201e = 树果树
# en Berry tree: KINOMI31, Route 21
sprite-0x202e = 树果树
# en Berry tree: KINOMI33, Route 10
sprite-0x2030 = 树果树
# en Berry tree: KINOMI34, Route 6
sprite-0x2031 = 树果树
# en Berry tree: KINOMI35, Routes 8 and 14
sprite-0x2032 = 树果树
# en Berry tree: KINOMI36, Route 20
sprite-0x2033 = 树果树

# Things the game never names, called what the ROM's own id for the sprite says they are.
# en sparkle: KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower
sprite-0x203d = 闪光
# en sparkle: KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone
sprite-0x2042 = 闪光
# en cone: CONE, model com_bm_skatecone: two across a Lumiose street, seven in Lysandre Labs
sprite-0x203e = 三角锥
# en shaking grass: TOBI_KUSA, no script: three on Route 6 and three on Route 18
sprite-0x2002 = 摇晃的草丛
# en lurking Pokémon: ENEMY3D, shared script 2000: nine in the Glittering Cave (spike 239)
sprite-0x95 = 潜伏的宝可梦

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 132, which says what is there; the word here is taken from
# that line (spike 220), in ORAS's word where the English is ORAS's too.
# PC: the PC's own lines (archives.PC_LINES)
tile-0xd5 = 电脑
# TV: story 390:19, "a TV and a washing machine": X has no line naming a TV alone
tile-0xd7 = 电视
# Pokémon knickknacks: text 132:0
tile-0xd8 = 宝可梦摆件
# Pokémon books: text 132:1, books and photo albums
tile-0xd9 = 宝可梦书籍
# cabinet: text 132:2, ORAS's line and words
tile-0xda = 柜子
# useful items: text 132:3
tile-0xdb = 日用道具
# Pokémon novelties: text 132:4
tile-0xdc = 宝可梦周边
# trash can: text 132:6
tile-0xdd = 垃圾桶
# books for Pokémon: text 132:5, "Are these books for Pokémon to read?"
tile-0xde = 给宝可梦看的书

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# elevator: story 137:21 and 145:11: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = 电梯
furniture-0x27db = 电梯
furniture-0x27dc = 电梯
furniture-0x27dd = 电梯
furniture-0x27de = 电梯
furniture-0x27df = 电梯
furniture-0x27e0 = 电梯
furniture-0x27e1 = 电梯
furniture-0x27e2 = 电梯
furniture-0x27e3 = 电梯
furniture-0x27e4 = 电梯
furniture-0x27e5 = 电梯
furniture-0x27e6 = 电梯
furniture-0x27e7 = 电梯
furniture-0x27e8 = 电梯
furniture-0x27e9 = 电梯
furniture-0x27ea = 电梯
furniture-0x27eb = 电梯
furniture-0x27ec = 电梯
furniture-0x27ed = 电梯
furniture-0x27ee = 电梯
furniture-0x27ef = 电梯
furniture-0x27f0 = 电梯
furniture-0x27f1 = 电梯
furniture-0x27f2 = 电梯
furniture-0x27f3 = 电梯
furniture-0x27f4 = 电梯
furniture-0x27f5 = 电梯
furniture-0x27f6 = 电梯
furniture-0x27f7 = 电梯
furniture-0x27f8 = 电梯
furniture-0x27f9 = 电梯
furniture-0x27fa = 电梯
furniture-0x27fb = 电梯
furniture-0x27fc = 电梯
furniture-0x27fd = 电梯
furniture-0x27fe = 电梯
furniture-0x27ff = 电梯
furniture-0x2800 = 电梯
furniture-0x2801 = 电梯
furniture-0x2802 = 电梯
furniture-0x2803 = 电梯
furniture-0x2804 = 电梯
furniture-0x2805 = 电梯
furniture-0x2806 = 电梯
furniture-0x2807 = 电梯
furniture-0x2808 = 电梯
furniture-0x2809 = 电梯
# meteor: story 221:8, Ambrette Town
furniture-0x281e = 陨石
# cardboard boxes: story 220:0, Lumiose City, zone 0x54
furniture-0x284b = 纸箱
# photo spot: story 222:0, one in each of 12 places
furniture-0x28c8 = 拍照点
furniture-0x28c9 = 拍照点
furniture-0x28ca = 拍照点
furniture-0x28cb = 拍照点
furniture-0x28cc = 拍照点
furniture-0x28cd = 拍照点
furniture-0x28ce = 拍照点
furniture-0x28cf = 拍照点
furniture-0x28d0 = 拍照点
furniture-0x28d1 = 拍照点
furniture-0x28d2 = 拍照点
furniture-0x28d3 = 拍照点
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
furniture-0x2922 = 树果树
furniture-0x2923 = 树果树
furniture-0x2924 = 树果树
furniture-0x2925 = 树果树
furniture-0x2926 = 树果树
furniture-0x2927 = 树果树
furniture-0x2928 = 树果树
furniture-0x2929 = 树果树
furniture-0x292a = 树果树
furniture-0x292b = 树果树
furniture-0x292c = 树果树
furniture-0x292d = 树果树
furniture-0x292e = 树果树
furniture-0x292f = 树果树
furniture-0x2930 = 树果树
# fitting room: text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = 试衣间
# clothes: text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = 衣服
# bed: story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = 床
# taxi stand: story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = 出租车站
# Gogoat Shuttle: story 399:218, Lumiose's 12 stops
furniture-0x2a3b = 坐骑山羊接驳车
furniture-0x2a3c = 坐骑山羊接驳车
furniture-0x2a3d = 坐骑山羊接驳车
furniture-0x2a3e = 坐骑山羊接驳车
furniture-0x2a3f = 坐骑山羊接驳车
furniture-0x2a40 = 坐骑山羊接驳车
furniture-0x2a41 = 坐骑山羊接驳车
furniture-0x2a42 = 坐骑山羊接驳车
furniture-0x2a43 = 坐骑山羊接驳车
furniture-0x2a44 = 坐骑山羊接驳车
furniture-0x2a45 = 坐骑山羊接驳车
furniture-0x2a46 = 坐骑山羊接驳车

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# soft soil: story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = 松软的土壤
place-0x160-furniture-3 = 松软的土壤
place-0x160-furniture-4 = 松软的土壤
place-0x160-furniture-5 = 松软的土壤
place-0x160-furniture-6 = 松软的土壤
place-0x160-furniture-7 = 松软的土壤
place-0x160-furniture-8 = 松软的土壤
place-0x160-furniture-9 = 松软的土壤
place-0x160-furniture-10 = 松软的土壤
place-0x160-furniture-11 = 松软的土壤
place-0x160-furniture-12 = 松软的土壤
place-0x160-furniture-13 = 松软的土壤
place-0x160-furniture-14 = 松软的土壤
place-0x160-furniture-15 = 松软的土壤
place-0x160-furniture-16 = 松软的土壤
place-0x160-furniture-17 = 松软的土壤
place-0x160-furniture-18 = 松软的土壤
place-0x160-furniture-19 = 松软的土壤
place-0x160-furniture-20 = 松软的土壤
place-0x160-furniture-21 = 松软的土壤
place-0x160-furniture-22 = 松软的土壤
place-0x160-furniture-23 = 松软的土壤
place-0x160-furniture-24 = 松软的土壤
place-0x160-furniture-25 = 松软的土壤
place-0x160-furniture-26 = 松软的土壤
place-0x160-furniture-27 = 松软的土壤
place-0x160-furniture-28 = 松软的土壤
place-0x160-furniture-29 = 松软的土壤
place-0x160-furniture-30 = 松软的土壤
place-0x160-furniture-31 = 松软的土壤
place-0x160-furniture-32 = 松软的土壤
place-0x160-furniture-33 = 松软的土壤
place-0x160-furniture-34 = 松软的土壤
place-0x160-furniture-35 = 松软的土壤
place-0x160-furniture-36 = 松软的土壤
place-0x160-furniture-37 = 松软的土壤
# audio guide: story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = 语音导览
place-0x68-furniture-1 = 语音导览
place-0x68-furniture-2 = 语音导览
place-0x68-furniture-3 = 语音导览
place-0x68-furniture-4 = 语音导览
place-0x68-furniture-5 = 语音导览
place-0x68-furniture-6 = 语音导览
place-0x69-furniture-0 = 语音导览
place-0x69-furniture-1 = 语音导览
place-0x69-furniture-2 = 语音导览
place-0x69-furniture-3 = 语音导览
place-0x69-furniture-4 = 语音导览
place-0x69-furniture-5 = 语音导览
place-0x69-furniture-6 = 语音导览
place-0x6a-furniture-0 = 语音导览
place-0x6a-furniture-1 = 语音导览
place-0x6a-furniture-2 = 语音导览
place-0x6a-furniture-3 = 语音导览
place-0x6a-furniture-4 = 语音导览
place-0x6a-furniture-5 = 语音导览
place-0x6a-furniture-6 = 语音导览
place-0x6a-furniture-7 = 语音导览
place-0x6a-furniture-8 = 语音导览
place-0x6a-furniture-9 = 语音导览
# statue of a Bisharp: story 215:21, Parfum Palace
place-0x120-furniture-7 = 劈斩司令雕像
place-0x120-furniture-8 = 劈斩司令雕像
place-0x120-furniture-9 = 劈斩司令雕像
place-0x120-furniture-10 = 劈斩司令雕像
place-0x120-furniture-11 = 劈斩司令雕像
place-0x120-furniture-12 = 劈斩司令雕像
place-0x120-furniture-13 = 劈斩司令雕像
place-0x120-furniture-14 = 劈斩司令雕像
place-0x125-furniture-2 = 劈斩司令雕像
place-0x125-furniture-3 = 劈斩司令雕像
place-0x127-furniture-2 = 劈斩司令雕像
place-0x127-furniture-3 = 劈斩司令雕像
place-0x12c-furniture-3 = 劈斩司令雕像
place-0x12c-furniture-4 = 劈斩司令雕像
# golden statue of a knight: story 215:22, Parfum Palace
place-0x120-furniture-15 = 黄金骑士像
place-0x120-furniture-16 = 黄金骑士像
place-0x120-furniture-17 = 黄金骑士像
place-0x120-furniture-18 = 黄金骑士像
place-0x120-furniture-19 = 黄金骑士像
place-0x120-furniture-20 = 黄金骑士像
place-0x120-furniture-21 = 黄金骑士像
place-0x120-furniture-22 = 黄金骑士像
place-0x120-furniture-23 = 黄金骑士像
place-0x120-furniture-24 = 黄金骑士像
place-0x125-furniture-4 = 黄金骑士像
place-0x125-furniture-5 = 黄金骑士像
place-0x127-furniture-4 = 黄金骑士像
place-0x127-furniture-5 = 黄金骑士像
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
place-0x12e-furniture-2 = 精灵球雕像
place-0x12e-furniture-3 = 精灵球雕像
place-0x12e-furniture-4 = 精灵球雕像
place-0x12e-furniture-5 = 精灵球雕像
place-0x12e-furniture-6 = 精灵球雕像
place-0x12e-furniture-7 = 精灵球雕像
# monument: story 226:7, the Battle Maison
place-0xf1-furniture-0 = 纪念碑
place-0xf1-furniture-1 = 纪念碑
place-0xf1-furniture-2 = 纪念碑
place-0xf1-furniture-3 = 纪念碑
place-0xf1-furniture-4 = 纪念碑
# portrait: story 203:0, Parfum Palace
place-0x12c-furniture-0 = 肖像画
place-0x12c-furniture-1 = 肖像画
place-0x12c-furniture-2 = 肖像画
# record machine: story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = 唱片机
place-0x54-furniture-3 = 唱片机
place-0x54-furniture-4 = 唱片机
place-0x54-furniture-5 = 唱片机
# PC: story 55:1, the bedroom's PC
place-0x2-furniture-0 = 电脑
# mirror: story 55:2
place-0x2-furniture-1 = 镜子
# TV: story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = 电视
# TV: story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = 电视
# Wii U: story 55:0
place-0x2-furniture-4 = Wii U
# statue: story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = 雕像
place-0x37-furniture-1 = 雕像
# pole: story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = 杆子
place-0x38-furniture-0 = 杆子
# photographs: story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = 照片
place-0x37-furniture-4 = 照片
place-0x37-furniture-5 = 照片
# photograph: story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = 照片
place-0x37-furniture-7 = 照片
