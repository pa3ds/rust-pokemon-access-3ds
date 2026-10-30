### What to call the people, Pokémon and things on X and Y's maps in Japanese, and the use tiles,
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
sprite-0x7 = ママ
# Looker: "It's me! Looker!" (system 102:16)
sprite-0x26 = ハンサム
# Alexa: "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = パンジー
# Bonnie: "You can call me Bonnie." (story 404:0)
sprite-0x9b = ユリーカ
# the Mega Evolution guru: "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = メガシンカおやじ
# the Game Director: "Hello! I am the Game Director." (story 272:0)
sprite-0x98 = ゲーム　ディレクター
# Nurse: PCWOMAN1, at the counter in all 16 Pokémon Centers (spike 101)
sprite-0x3b = ナース

# The game's own words for the story's people and for kinds of trainer, as `archives`
# reads them (findings, spikes 90 and 108): spikes/108_x_people.py table ja.
sprite-0x3 = サナ
sprite-0x4 = ティエルノ
sprite-0x5 = トロバ
sprite-0x6 = プラターヌ
sprite-0xa = カルネ
sprite-0xb = ガンピ
sprite-0xc = パキラ
sprite-0xd = ドラセナ
sprite-0xe = ズミ
sprite-0xf = ビオラ
sprite-0x10 = ザクロ
sprite-0x11 = コルニ
sprite-0x12 = フクジ
sprite-0x13 = シトロン
sprite-0x14 = マーシュ
sprite-0x15 = ゴジカ
sprite-0x16 = ウルップ
sprite-0x17 = フラダリ
sprite-0x21 = ＡＺ
sprite-0x93 = カルム
sprite-0x94 = セレナ
sprite-0x9c = フラダリ
sprite-0x9f = カルム
sprite-0xa0 = セレナ
sprite-0x1a = フレアだん したっぱ
sprite-0x1b = フレアだん したっぱ
sprite-0x28 = えんじ
sprite-0x2a = えんじ
sprite-0x2c = スクールガール
sprite-0x30 = スクールボーイ
sprite-0x34 = ツアーきゃく
sprite-0x35 = バックパッカー
sprite-0x43 = たんぱんこぞう
sprite-0x44 = だいすきクラブ
sprite-0x47 = ミニスカート
sprite-0x4a = バトルガール
sprite-0x4b = ローラースケート
sprite-0x4c = ローラースケート
sprite-0x4d = つりびと
sprite-0x4e = ホープトレーナー
sprite-0x4f = エリートトレーナー
sprite-0x50 = ベテラントレーナー
sprite-0x52 = ビキニのおねえさん
sprite-0x55 = ツアーきゃく
sprite-0x57 = ポケモンブリーダー
sprite-0x58 = サイキッカー
sprite-0x59 = ホープトレーナー
sprite-0x5b = ベテラントレーナー
sprite-0x5d = ポケモンレンジャー
sprite-0x5f = けんきゅういん
sprite-0x61 = さぎょういん
sprite-0x64 = メルヘンしょうじょ
sprite-0x67 = にわし
sprite-0x70 = やまおとこ
sprite-0xa3 = かいパンやろう
sprite-0xa4 = ビキニのおねえさん

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# Mr. Bonding: KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4)
sprite-0x77 = キズナおやじ
# Dexio: MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = デクシオ
# Sina: MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar, "I came in place of Dexio" (256:1)
sprite-0x8c = ジーナ
# Aliana: SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = アケビ
# Essentia: C02GIRL_EX: Lysandre Labs and Lumiose, Looker's story; trainer 511 (inferred from the id)
sprite-0x9e = エスプリ
# Emma: C02GIRL: Lumiose, Looker's story; "Looker, you, Emma, Mimi..." (story 347:9) (inferred)
sprite-0x9d = マチエール
# Sina: MYSTERYW: Geosenge's masked heroes, "Who are these Sina and Dexio you speak of?" (story 35:25) (inferred)
sprite-0x8 = ジーナ
# Dexio: MYSTERYM: likewise (inferred)
sprite-0x9 = デクシオ

# Team Flare's admins and scientists: the admins by their trainers' class and name, as
# the grunts are; the scientists as story 378:5 calls them, not yet told apart.
# Team Flare Admin: FLAREKANBU1: the Power Plant, the Secret HQ; trainers "Admin" of class Team Flare
sprite-0x18 = フレアだん かんぶ
# Team Flare Admin: FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = フレアだん かんぶ
# Team Flare Scientist: SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = フレアだん かがくしゃ
# Team Flare Scientist: SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = フレアだん かがくしゃ
# Team Flare Scientist: SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = フレアだん かがくしゃ
# Team Flare Scientist: SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = フレアだん かがくしゃ

# Shared looks the game never names, called what the sprite's id says they are (text
# file 76), in Omega Ruby's words for the same look where it has one. Left out:
# NONDRAW, STOPPER, the work objects WKOBJCODE00 to 04, and ENEMY3D and GIANTMAN02,
# which no line names.
# Little girl: CHILDGIRL1: Vaniville, Aquacorde
sprite-0x27 = おんなのこ
# Little boy: CHILDBOY1: 7 zones
sprite-0x29 = おとこのこ
# Girl: GIRL1: 10 zones
sprite-0x2b = しょうじょ
# Girl: GIRL3: 17 zones
sprite-0x2e = しょうじょ
# Boy: BOY1: 3 zones
sprite-0x2d = しょうねん
# Boy: BOY2: 13 zones
sprite-0x32 = しょうねん
# Woman: WOMAN1: 33 zones
sprite-0x2f = おんなのひと
# Woman: WOMAN2: 34 zones, one Beauty among them
sprite-0x38 = おんなのひと
# Man: MAN1: 36 zones
sprite-0x31 = おとこのひと
# Middle-aged man: MIDDLEMAN2: 9 zones
sprite-0xa6 = おじさん
# Gentleman: GENTLEMAN: 45 zones; X has no such class, Omega Ruby's class
sprite-0x37 = ジェントルマン
# Clerk: SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = てんいん
# Clerk: SHOPW1: 11 zones
sprite-0x3f = てんいん
# Gym guide: DANPEI: in every Gym
sprite-0x3d = ジムガイド
# Receptionist: INFORMATIONW: 37 zones, the gates
sprite-0x3e = うけつけ
# Delivery person: DELIVER: in all 17 Pokémon Centers
sprite-0x40 = はいたついん
# Delivery person: DELIVER2: likewise
sprite-0xa1 = はいたついん
# Cameraman: CAMERAMAN: 14 zones
sprite-0x42 = カメラマン
# Twin: TWIN: two pairs of Twins among 10 zones
sprite-0x46 = ふたご
# Twin: TWIN2: 4 zones
sprite-0xa2 = ふたご
# Hairdresser: HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = びようし
# Café owner: MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = カフェの　マスター
# Cyclist: CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = サイクリスト
# Mannequin: HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = マネキン
# Mannequin: HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = マネキン
# Model: MODEL01: Route 7 (id only)
sprite-0xa7 = モデル
# Model: MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = モデル
# Model: MODEL03: likewise (id only)
sprite-0xa9 = モデル

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# Punk Girl: BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = バッドガール
# Poké Fan: MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = だいすきクラブ
# Artist: ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = げいじゅつか
# Artist: ARTISTW: one Artist
sprite-0x54 = げいじゅつか
# Pokémon Breeder: BREEDERM: one Pokémon Breeder
sprite-0x56 = ポケモンブリーダー
# Scientist: ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = けんきゅういん
# Worker: WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = さぎょういん
# Furisode Girl: KIMONO1: one Furisode Girl
sprite-0x68 = ふりそで
# Furisode Girl: KIMONO2: one Furisode Girl
sprite-0x69 = ふりそで
# Furisode Girl: KIMONO3: one Furisode Girl
sprite-0x6a = ふりそで
# Furisode Girl: KIMONO4: one Furisode Girl
sprite-0x6b = ふりそで
# Lady: LADY: 19 zones, no trainer
sprite-0x36 = おじょうさま
# Waitress: WAITRESS: 24 zones, no trainer
sprite-0x49 = ウエイトレス
# Garçon: GARCON: 25 zones, no trainer
sprite-0x72 = ギャルソン
# Chef: COOK: 12 zones, no trainer
sprite-0x5e = りょうりにん
# Butler: BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = しつじ
# Maid: MAID: 32 zones, the Pokémon Centers
sprite-0x66 = メイド
# Sky Trainer: SKYM: 8 zones, no trainer
sprite-0x6c = スカイトレーナー
# Sky Trainer: SKYW: 8 zones, no trainer
sprite-0x6d = スカイトレーナー
# Fisherman: FISHING2: 4 zones, as FISHING is
sprite-0xa5 = つりびと

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = ツアーきゃく
# BADMAN
sprite-0x39 = バッドガイ
# TRAINERM
sprite-0x5a = エリートトレーナー
# RANGERM
sprite-0x5c = ポケモンレンジャー
# OCCULTW
sprite-0x63 = オカルトマニア
# FIGHTERM
sprite-0x71 = カラテおう

# The Pokémon on the map, by the species each one's model draws, its tag read through
# the battle models (spike 218); with the ROM, its own species names. Mewtwo's model
# names none.
# Mewtwo: P_MODEL01: the Unknown Dungeon, "Mewtwo blended into the darkness of the cave" (story 185:1); its model names no species
sprite-0x1017 = ミュウツー
# EV_POKE01
sprite-0x1000 = バケッチャ
# EV_POKE02
sprite-0x1001 = ホルード
# EV_POKE03
sprite-0x1002 = ジガルデ
# EV_POKE04
sprite-0x1003 = シシコ
# EV_POKE05
sprite-0x1004 = ゴーゴート
# RIDE_POKE02
sprite-0x1007 = マンムー
# RIDE_POKE04
sprite-0x1008 = メェークル
# EV_POKE07
sprite-0x1009 = ルカリオ
# EV_POKE08
sprite-0x100a = トリミアン
# EV_POKE10
sprite-0x100b = ヤヤコマ
# EV_POKE11
sprite-0x100c = カビゴン
# EV_POKE12
sprite-0x100d = ニャスパー
# RIDE_POKE05
sprite-0x100f = サイホーン
# EV_POKE09
sprite-0x1010 = ユキノオー
# EV_POKE14
sprite-0x1011 = メェークル
# EV_POKE15
sprite-0x1013 = マリル

# Things standing on the map that the game never names, called by a word of the line the shared
# script they run shows when the player faces one (a/0/3/1 file 27, story 130; spike 219), as
# ORAS's are (spike 137).
# en bush: ITREE01, runs 0x2712: story 130:3
sprite-0x200b = トゲ
# en cracked boulder: IROCK01, runs 0x2710: story 130:1
sprite-0x200c = ひびが　はいった　いわ
# en big boulder: KROCK01, runs 0x2714: story 130:5
sprite-0x2009 = おおきな　いわ
# en cracked wall: IWALL01, runs 0x2716: story 130:9
sprite-0x2034 = ひびが　はいった　かべ
# en rock: ISNOW01: ten on Route 17, where "all the rocks along the path are nothing against Mamoswine's strength!" (story 70:5)
sprite-0x2035 = いわ
# en rock: IROCK02: ten on Route 9, crossed on Rhyhorn; no line, its model is it0014_rock (inferred), the word of story 70:5
sprite-0x200d = いわ

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# en Central Kalos Pokédex award: AWARD04, case 0xd: story 55:4
sprite-0x203b = セントラルカロスずかんの　ひょうしょうじょう
# en Coastal Kalos Pokédex award: AWARD03, case 0xe: story 55:5
sprite-0x203a = コーストカロスずかんの　ひょうしょうじょう
# en Mountain Kalos Pokédex award: AWARD05, case 0xf: story 55:6
sprite-0x203c = マウンテンカロスずかんの　ひょうしょうじょう
# en Kalos region award: AWARD01, case 0x10: story 55:7
sprite-0x2036 = カロスちほうの　ひょうしょうじょう
# en National Pokédex award: AWARD02, case 0x11: story 55:8
sprite-0x2037 = ぜんこくずかんの　ひょうしょうじょう
# en Berry tree: KINOMI, runs 0x2939: story 142:15, "There are small Berries growing high up in the tree!"; Route 15
sprite-0x200e = みの　なる　き
# en Berry tree: KINOMI2, Routes 5 and 19
sprite-0x2011 = みの　なる　き
# en Berry tree: KINOMI3, Routes 11 and 12
sprite-0x2012 = みの　なる　き
# en Berry tree: KINOMI4, Routes 16 and 22
sprite-0x2013 = みの　なる　き
# en Berry tree: KINOMI11, Route 7
sprite-0x201a = みの　なる　き
# en Berry tree: KINOMI15, Route 18
sprite-0x201e = みの　なる　き
# en Berry tree: KINOMI31, Route 21
sprite-0x202e = みの　なる　き
# en Berry tree: KINOMI33, Route 10
sprite-0x2030 = みの　なる　き
# en Berry tree: KINOMI34, Route 6
sprite-0x2031 = みの　なる　き
# en Berry tree: KINOMI35, Routes 8 and 14
sprite-0x2032 = みの　なる　き
# en Berry tree: KINOMI36, Route 20
sprite-0x2033 = みの　なる　き

# Things the game never names, called what the ROM's own id for the sprite says they are.
# en sparkle: KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower; its model's joints are kira00 to kira03
sprite-0x203d = キラキラ
# en sparkle: KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone, items 665 to 683 (text 96)
sprite-0x2042 = キラキラ
# en cone: CONE, model com_bm_skatecone: two across a Lumiose street, "There is a fence blocking the road." (story 317:18), and seven in Lysandre Labs
sprite-0x203e = コーン
# en shaking grass: TOBI_KUSA, no script: three on Route 6 and three on Route 18, rustling till a wild Pokémon "leaped out of the tall grass" (text 124:261)
sprite-0x2002 = 揺れる草むら
# en lurking Pokémon: ENEMY3D, shared script 2000: nine in the Glittering Cave, drawn as nothing, a wild battle as the player comes up to one (spike 239)
sprite-0x95 = 潜んでいるポケモン

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 132, which says what is there; the word here is taken from
# that line (spike 220). The PC's word is read from the ROM ahead of this one, and the TV's is story
# 390:19's, since X has no line naming a TV alone.
# PC: the PC's own lines (archives.PC_LINES)
tile-0xd5 = パソコン
# TV: story 390:19, "a TV and a washing machine": X has no line naming a TV alone
tile-0xd7 = テレビ
# Pokémon knickknacks: text 132:0
tile-0xd8 = ポケモン　グッズ
# Pokémon books: text 132:1, books and photo albums
tile-0xd9 = ポケモンの　ほん
# cabinet: text 132:2, ORAS's line and words
tile-0xda = しゅうのうかぐ
# useful items: text 132:3
tile-0xdb = べんりそうな　グッズ
# Pokémon novelties: text 132:4
tile-0xdc = たくさんの　ポケモン　グッズ
# trash can: text 132:6
tile-0xdd = ゴミばこ
# books for Pokémon: text 132:5, "Are these books for Pokémon to read?"
tile-0xde = ポケモンが　よむ　ほん

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# elevator: story 137:21 and 145:11, the elevators' own file: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = エレベータ
furniture-0x27db = エレベータ
furniture-0x27dc = エレベータ
furniture-0x27dd = エレベータ
furniture-0x27de = エレベータ
furniture-0x27df = エレベータ
furniture-0x27e0 = エレベータ
furniture-0x27e1 = エレベータ
furniture-0x27e2 = エレベータ
furniture-0x27e3 = エレベータ
furniture-0x27e4 = エレベータ
furniture-0x27e5 = エレベータ
furniture-0x27e6 = エレベータ
furniture-0x27e7 = エレベータ
furniture-0x27e8 = エレベータ
furniture-0x27e9 = エレベータ
furniture-0x27ea = エレベータ
furniture-0x27eb = エレベータ
furniture-0x27ec = エレベータ
furniture-0x27ed = エレベータ
furniture-0x27ee = エレベータ
furniture-0x27ef = エレベータ
furniture-0x27f0 = エレベータ
furniture-0x27f1 = エレベータ
furniture-0x27f2 = エレベータ
furniture-0x27f3 = エレベータ
furniture-0x27f4 = エレベータ
furniture-0x27f5 = エレベータ
furniture-0x27f6 = エレベータ
furniture-0x27f7 = エレベータ
furniture-0x27f8 = エレベータ
furniture-0x27f9 = エレベータ
furniture-0x27fa = エレベータ
furniture-0x27fb = エレベータ
furniture-0x27fc = エレベータ
furniture-0x27fd = エレベータ
furniture-0x27fe = エレベータ
furniture-0x27ff = エレベータ
furniture-0x2800 = エレベータ
furniture-0x2801 = エレベータ
furniture-0x2802 = エレベータ
furniture-0x2803 = エレベータ
furniture-0x2804 = エレベータ
furniture-0x2805 = エレベータ
furniture-0x2806 = エレベータ
furniture-0x2807 = エレベータ
furniture-0x2808 = エレベータ
furniture-0x2809 = エレベータ
# meteor: story 221:8, Ambrette Town
furniture-0x281e = いんせき
# cardboard boxes: story 220:0, Lumiose City, zone 0x54
furniture-0x284b = ダンボール
# photo spot: story 222:0, one in each of 12 places
furniture-0x28c8 = しゃしんかんばん
furniture-0x28c9 = しゃしんかんばん
furniture-0x28ca = しゃしんかんばん
furniture-0x28cb = しゃしんかんばん
furniture-0x28cc = しゃしんかんばん
furniture-0x28cd = しゃしんかんばん
furniture-0x28ce = しゃしんかんばん
furniture-0x28cf = しゃしんかんばん
furniture-0x28d0 = しゃしんかんばん
furniture-0x28d1 = しゃしんかんばん
furniture-0x28d2 = しゃしんかんばん
furniture-0x28d3 = しゃしんかんばん
# trash can: story 142:14, Pokémon Village and the Lost Hotel
furniture-0x28f0 = ゴミばこ
furniture-0x28f1 = ゴミばこ
furniture-0x28f2 = ゴミばこ
furniture-0x28f3 = ゴミばこ
furniture-0x28f4 = ゴミばこ
furniture-0x28f5 = ゴミばこ
furniture-0x28f6 = ゴミばこ
furniture-0x28f7 = ゴミばこ
furniture-0x28f8 = ゴミばこ
furniture-0x28f9 = ゴミばこ
# Berry tree: story 142:15, its Berries and its tree, Routes 5 to 22
furniture-0x2922 = みの　なる　き
furniture-0x2923 = みの　なる　き
furniture-0x2924 = みの　なる　き
furniture-0x2925 = みの　なる　き
furniture-0x2926 = みの　なる　き
furniture-0x2927 = みの　なる　き
furniture-0x2928 = みの　なる　き
furniture-0x2929 = みの　なる　き
furniture-0x292a = みの　なる　き
furniture-0x292b = みの　なる　き
furniture-0x292c = みの　なる　き
furniture-0x292d = みの　なる　き
furniture-0x292e = みの　なる　き
furniture-0x292f = みの　なる　き
furniture-0x2930 = みの　なる　き
# fitting room: text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = フィッティングルーム
# fitting room: text 142:85, the 7 boutiques' fitting room, called as the Pokémon Centers' is
furniture-0x29d3 = フィッティングルーム
# clothes: text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = ふく
# bed: story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = ベッド
# taxi stand: story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = タクシーのりば
# Gogoat Shuttle: story 399:218, Lumiose's 12 stops
furniture-0x2a3b = ゴーゴーシャトル
furniture-0x2a3c = ゴーゴーシャトル
furniture-0x2a3d = ゴーゴーシャトル
furniture-0x2a3e = ゴーゴーシャトル
furniture-0x2a3f = ゴーゴーシャトル
furniture-0x2a40 = ゴーゴーシャトル
furniture-0x2a41 = ゴーゴーシャトル
furniture-0x2a42 = ゴーゴーシャトル
furniture-0x2a43 = ゴーゴーシャトル
furniture-0x2a44 = ゴーゴーシャトル
furniture-0x2a45 = ゴーゴーシャトル
furniture-0x2a46 = ゴーゴーシャトル

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# soft soil: story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = ふかふかの　つち
place-0x160-furniture-3 = ふかふかの　つち
place-0x160-furniture-4 = ふかふかの　つち
place-0x160-furniture-5 = ふかふかの　つち
place-0x160-furniture-6 = ふかふかの　つち
place-0x160-furniture-7 = ふかふかの　つち
place-0x160-furniture-8 = ふかふかの　つち
place-0x160-furniture-9 = ふかふかの　つち
place-0x160-furniture-10 = ふかふかの　つち
place-0x160-furniture-11 = ふかふかの　つち
place-0x160-furniture-12 = ふかふかの　つち
place-0x160-furniture-13 = ふかふかの　つち
place-0x160-furniture-14 = ふかふかの　つち
place-0x160-furniture-15 = ふかふかの　つち
place-0x160-furniture-16 = ふかふかの　つち
place-0x160-furniture-17 = ふかふかの　つち
place-0x160-furniture-18 = ふかふかの　つち
place-0x160-furniture-19 = ふかふかの　つち
place-0x160-furniture-20 = ふかふかの　つち
place-0x160-furniture-21 = ふかふかの　つち
place-0x160-furniture-22 = ふかふかの　つち
place-0x160-furniture-23 = ふかふかの　つち
place-0x160-furniture-24 = ふかふかの　つち
place-0x160-furniture-25 = ふかふかの　つち
place-0x160-furniture-26 = ふかふかの　つち
place-0x160-furniture-27 = ふかふかの　つち
place-0x160-furniture-28 = ふかふかの　つち
place-0x160-furniture-29 = ふかふかの　つち
place-0x160-furniture-30 = ふかふかの　つち
place-0x160-furniture-31 = ふかふかの　つち
place-0x160-furniture-32 = ふかふかの　つち
place-0x160-furniture-33 = ふかふかの　つち
place-0x160-furniture-34 = ふかふかの　つち
place-0x160-furniture-35 = ふかふかの　つち
place-0x160-furniture-36 = ふかふかの　つち
place-0x160-furniture-37 = ふかふかの　つち
# audio guide: story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = ガイドボックス
place-0x68-furniture-1 = ガイドボックス
place-0x68-furniture-2 = ガイドボックス
place-0x68-furniture-3 = ガイドボックス
place-0x68-furniture-4 = ガイドボックス
place-0x68-furniture-5 = ガイドボックス
place-0x68-furniture-6 = ガイドボックス
place-0x69-furniture-0 = ガイドボックス
place-0x69-furniture-1 = ガイドボックス
place-0x69-furniture-2 = ガイドボックス
place-0x69-furniture-3 = ガイドボックス
place-0x69-furniture-4 = ガイドボックス
place-0x69-furniture-5 = ガイドボックス
place-0x69-furniture-6 = ガイドボックス
place-0x6a-furniture-0 = ガイドボックス
place-0x6a-furniture-1 = ガイドボックス
place-0x6a-furniture-2 = ガイドボックス
place-0x6a-furniture-3 = ガイドボックス
place-0x6a-furniture-4 = ガイドボックス
place-0x6a-furniture-5 = ガイドボックス
place-0x6a-furniture-6 = ガイドボックス
place-0x6a-furniture-7 = ガイドボックス
place-0x6a-furniture-8 = ガイドボックス
place-0x6a-furniture-9 = ガイドボックス
# statue of a Bisharp: story 215:21, Parfum Palace
place-0x120-furniture-7 = キリキザンの　ぞう
place-0x120-furniture-8 = キリキザンの　ぞう
place-0x120-furniture-9 = キリキザンの　ぞう
place-0x120-furniture-10 = キリキザンの　ぞう
place-0x120-furniture-11 = キリキザンの　ぞう
place-0x120-furniture-12 = キリキザンの　ぞう
place-0x120-furniture-13 = キリキザンの　ぞう
place-0x120-furniture-14 = キリキザンの　ぞう
place-0x125-furniture-2 = キリキザンの　ぞう
place-0x125-furniture-3 = キリキザンの　ぞう
place-0x127-furniture-2 = キリキザンの　ぞう
place-0x127-furniture-3 = キリキザンの　ぞう
place-0x12c-furniture-3 = キリキザンの　ぞう
place-0x12c-furniture-4 = キリキザンの　ぞう
# golden statue of a knight: story 215:22, Parfum Palace
place-0x120-furniture-15 = おうごんの　きしぞう
place-0x120-furniture-16 = おうごんの　きしぞう
place-0x120-furniture-17 = おうごんの　きしぞう
place-0x120-furniture-18 = おうごんの　きしぞう
place-0x120-furniture-19 = おうごんの　きしぞう
place-0x120-furniture-20 = おうごんの　きしぞう
place-0x120-furniture-21 = おうごんの　きしぞう
place-0x120-furniture-22 = おうごんの　きしぞう
place-0x120-furniture-23 = おうごんの　きしぞう
place-0x120-furniture-24 = おうごんの　きしぞう
place-0x125-furniture-4 = おうごんの　きしぞう
place-0x125-furniture-5 = おうごんの　きしぞう
place-0x127-furniture-4 = おうごんの　きしぞう
place-0x127-furniture-5 = おうごんの　きしぞう
# whiteboard: story 412:7, Santalune City, zone 0x3e
place-0x3e-furniture-0 = ホワイトボード
place-0x3e-furniture-1 = ホワイトボード
place-0x3e-furniture-2 = ホワイトボード
place-0x3e-furniture-3 = ホワイトボード
place-0x3e-furniture-4 = ホワイトボード
place-0x3e-furniture-5 = ホワイトボード
place-0x3e-furniture-6 = ホワイトボード
place-0x3e-furniture-7 = ホワイトボード
place-0x3e-furniture-8 = ホワイトボード
# statue of a Poké Ball: story 216:47, Parfum Palace's garden
place-0x12e-furniture-2 = モンスターボールの　ぞう
place-0x12e-furniture-3 = モンスターボールの　ぞう
place-0x12e-furniture-4 = モンスターボールの　ぞう
place-0x12e-furniture-5 = モンスターボールの　ぞう
place-0x12e-furniture-6 = モンスターボールの　ぞう
place-0x12e-furniture-7 = モンスターボールの　ぞう
# monument: story 226:7, the Battle Maison
place-0xf1-furniture-0 = モニュメント
place-0xf1-furniture-1 = モニュメント
place-0xf1-furniture-2 = モニュメント
place-0xf1-furniture-3 = モニュメント
place-0xf1-furniture-4 = モニュメント
# portrait: story 203:0, Parfum Palace
place-0x12c-furniture-0 = しょうぞう
place-0x12c-furniture-1 = しょうぞう
place-0x12c-furniture-2 = しょうぞう
# record machine: story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = レコードマシン
place-0x54-furniture-3 = レコードマシン
place-0x54-furniture-4 = レコードマシン
place-0x54-furniture-5 = レコードマシン
# PC: story 55:1, the bedroom's PC
place-0x2-furniture-0 = パソコン
# mirror: story 55:2
place-0x2-furniture-1 = かがみ
# TV: story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = テレビ
# TV: story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = テレビ
# Wii U: story 55:0
place-0x2-furniture-4 = Ｗｉｉ Ｕ
# statue: story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = ナゾの　ぞう
place-0x37-furniture-1 = ナゾの　ぞう
# pole: story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = バー
place-0x38-furniture-0 = バー
# photographs: story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = しゃしん
place-0x37-furniture-4 = しゃしん
place-0x37-furniture-5 = しゃしん
# photograph: story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = しゃしん
place-0x37-furniture-7 = しゃしん
# Lysandre Café: story 313:4, Lumiose City, zone 0x43: the plate of script 22, which no record shows, so no door is paired with it
place-0x43-warp-1 = フラダリカフェ
