### What to call the people, Pokémon and things on X and Y's maps in Korean, and the use tiles,
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
sprite-0x7 = 엄마
# Looker: "It's me! Looker!" (system 102:16)
sprite-0x26 = 핸섬
# Alexa: "I'm Alexa. [...] Viola, is my younger sister." (story 421:10)
sprite-0x9a = 팬지
# Bonnie: "You can call me Bonnie." (story 404:0)
sprite-0x9b = 유리카
# the Mega Evolution guru: "People call me the Mega Evolution guru!" (story 281:6)
sprite-0x97 = 메가진화 아저씨
# the Game Director: "Hello! I am the Game Director." (story 272:0)
sprite-0x98 = 게임 디렉터
# Nurse: PCWOMAN1, at the counter in all 16 Pokémon Centers (spike 101)
sprite-0x3b = 간호사

# The game's own words for the story's people and for kinds of trainer, as `archives`
# reads them (findings, spikes 90 and 108): spikes/108_x_people.py table ko.
sprite-0x3 = 사나
sprite-0x4 = 티에르노
sprite-0x5 = 트로바
sprite-0x6 = 플라타느
sprite-0xa = 카르네
sprite-0xb = 간피
sprite-0xc = 파키라
sprite-0xd = 드라세나
sprite-0xe = 즈미
sprite-0xf = 비올라
sprite-0x10 = 자크로
sprite-0x11 = 코르니
sprite-0x12 = 후쿠지
sprite-0x13 = 시트론
sprite-0x14 = 마슈
sprite-0x15 = 고지카
sprite-0x16 = 우르프
sprite-0x17 = 플라드리
sprite-0x21 = AZ
sprite-0x93 = 칼름
sprite-0x94 = 세레나
sprite-0x9c = 플라드리
sprite-0x9f = 칼름
sprite-0xa0 = 세레나
sprite-0x1a = 플레어단 조무래기
sprite-0x1b = 플레어단 조무래기
sprite-0x28 = 보육원아
sprite-0x2a = 보육원아
sprite-0x2c = 스쿨걸
sprite-0x30 = 스쿨보이
sprite-0x34 = 투어객
sprite-0x35 = 백팩커
sprite-0x43 = 반바지 꼬마
sprite-0x44 = 애호가클럽
sprite-0x47 = 짧은 치마
sprite-0x4a = 배틀걸
sprite-0x4b = 롤러스케이트
sprite-0x4c = 롤러스케이트
sprite-0x4d = 낚시꾼
sprite-0x4e = 호프 트레이너
sprite-0x4f = 엘리트 트레이너
sprite-0x50 = 베테랑 트레이너
sprite-0x52 = 비키니 아가씨
sprite-0x55 = 투어객
sprite-0x57 = 포켓몬 브리더
sprite-0x58 = 초능력자
sprite-0x59 = 호프 트레이너
sprite-0x5b = 베테랑 트레이너
sprite-0x5d = 포켓몬 레인저
sprite-0x5f = 연구원
sprite-0x61 = 작업원
sprite-0x64 = 메르헨소녀
sprite-0x67 = 정원사
sprite-0x70 = 등산가
sprite-0xa3 = 수영팬티소년
sprite-0xa4 = 비키니 아가씨

# People the game names in its lines, never a trainer standing on the map: the sprite's
# id, where it stands, and the line naming them, the same line in every language
# (`spikes/218_x_people.py lines`). Aliana and Essentia are trainers, so the ROM names
# them once `xy.CHARACTERS` has them; until then, these.
# Mr. Bonding: KIZUNA: 14 zones; "They call me Mr. Bonding." (story 410:4)
sprite-0x77 = 유대아저씨
# Dexio: MYSTERYM2: Route 4, "I'm Dexio." (story 87:10), and six more
sprite-0x8b = 덱시오
# Sina: MYSTERYW2: Route 4, "My name's Sina!" (story 87:9); alone in Anistar, "I came in place of Dexio" (256:1)
sprite-0x8c = 지나
# Aliana: SCIENTIST5: the one scientist at the Kalos Power Plant, "My name is Aliana." (story 167:10)
sprite-0x20 = 아케비
# Essentia: C02GIRL_EX: Lysandre Labs and Lumiose, Looker's story; trainer 511 (inferred from the id)
sprite-0x9e = 에스프리
# Emma: C02GIRL: Lumiose, Looker's story; "Looker, you, Emma, Mimi..." (story 347:9) (inferred)
sprite-0x9d = 마티에르
# Sina: MYSTERYW: Geosenge's masked heroes, "Who are these Sina and Dexio you speak of?" (story 35:25) (inferred)
sprite-0x8 = 지나
# Dexio: MYSTERYM: likewise (inferred)
sprite-0x9 = 덱시오

# Team Flare's admins and scientists: the admins by their trainers' class and name, as
# the grunts are; the scientists as story 378:5 calls them, not yet told apart.
# Team Flare Admin: FLAREKANBU1: the Power Plant, the Secret HQ; trainers "Admin" of class Team Flare
sprite-0x18 = 플레어단 간부
# Team Flare Admin: FLAREKANBU2: the Poké Ball Factory, the Secret HQ
sprite-0x19 = 플레어단 간부
# Team Flare Scientist: SCIENTIST1: Lysandre Café and Labs (story 378:5)
sprite-0x1c = 플레어단 과학자
# Team Flare Scientist: SCIENTIST2: Lysandre Labs, Frost Cavern
sprite-0x1d = 플레어단 과학자
# Team Flare Scientist: SCIENTIST3: Lysandre Labs, the Poké Ball Factory
sprite-0x1e = 플레어단 과학자
# Team Flare Scientist: SCIENTIST4: Lysandre Labs, the Poké Ball Factory
sprite-0x1f = 플레어단 과학자

# Shared looks the game never names, called what the sprite's id says they are (text
# file 76), in Omega Ruby's words for the same look where it has one. Left out:
# NONDRAW, STOPPER, the work objects WKOBJCODE00 to 04, and ENEMY3D and GIANTMAN02,
# which no line names.
# Little girl: CHILDGIRL1: Vaniville, Aquacorde
sprite-0x27 = 여자아이
# Little boy: CHILDBOY1: 7 zones
sprite-0x29 = 남자아이
# Girl: GIRL1: 10 zones
sprite-0x2b = 소녀
# Girl: GIRL3: 17 zones
sprite-0x2e = 소녀
# Boy: BOY1: 3 zones
sprite-0x2d = 소년
# Boy: BOY2: 13 zones
sprite-0x32 = 소년
# Woman: WOMAN1: 33 zones
sprite-0x2f = 여자
# Woman: WOMAN2: 34 zones, one Beauty among them
sprite-0x38 = 여자
# Man: MAN1: 36 zones
sprite-0x31 = 남자
# Middle-aged man: MIDDLEMAN2: 9 zones
sprite-0xa6 = 아저씨
# Gentleman: GENTLEMAN: 45 zones; X has no such class, Omega Ruby's class
sprite-0x37 = 신사
# Clerk: SHOPM1: 28 zones, the Poké Marts
sprite-0x3c = 점원
# Clerk: SHOPW1: 11 zones
sprite-0x3f = 점원
# Gym guide: DANPEI: in every Gym
sprite-0x3d = 체육관 가이드
# Receptionist: INFORMATIONW: 37 zones, the gates
sprite-0x3e = 접수원
# Delivery person: DELIVER: in all 17 Pokémon Centers
sprite-0x40 = 배달원
# Delivery person: DELIVER2: likewise
sprite-0xa1 = 배달원
# Cameraman: CAMERAMAN: 14 zones
sprite-0x42 = 카메라맨
# Twin: TWIN: two pairs of Twins among 10 zones
sprite-0x46 = 쌍둥이
# Twin: TWIN2: 4 zones
sprite-0xa2 = 쌍둥이
# Hairdresser: HAIRDRESSER: the salons, "we'll come up with the perfect hairstyle" (story 350:1)
sprite-0x41 = 미용사
# Café owner: MASTER: 18 zones, behind the counter in Pokémon Centers and Lumiose's cafés
sprite-0x48 = 카페 주인
# Cyclist: CYCLINGM: Cyllage, "If you're here for the Bicycle race" (story 310:0)
sprite-0x75 = 자전거 선수
# Mannequin: HEROINE_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x78 = 마네킹
# Mannequin: HERO_MANNEQUIN: a Lumiose boutique (id only)
sprite-0x96 = 마네킹
# Model: MODEL01: Route 7 (id only)
sprite-0xa7 = 모델
# Model: MODEL02: Lumiose's cafés and boulevards (id only)
sprite-0xa8 = 모델
# Model: MODEL03: likewise (id only)
sprite-0xa9 = 모델

# Trainers' looks the ROM leaves unnamed: one trainer or none of the class the id says,
# or trainers of two classes standing as it (spike 90), called by the class.
# Punk Girl: BADGIRL: two Punk Girls and two Lumiose Gang Members; as BADMAN is Punk Guy
sprite-0x3a = 배드걸
# Poké Fan: MIDDLEWOMAN1: two Poké Fans, and a Poké Fan Family whose battles are not all doubles
sprite-0x45 = 애호가클럽
# Artist: ARTISTM: three Artists, and an Artist Family whose battles are not all doubles
sprite-0x53 = 예술가
# Artist: ARTISTW: one Artist
sprite-0x54 = 예술가
# Pokémon Breeder: BREEDERM: one Pokémon Breeder
sprite-0x56 = 포켓몬 브리더
# Scientist: ASSISTANTW: one Scientist, as ASSISTANTM is
sprite-0x60 = 연구원
# Worker: WORKMAN2: one Worker, as WORKMAN1 is
sprite-0x62 = 작업원
# Furisode Girl: KIMONO1: one Furisode Girl
sprite-0x68 = 기모노드레스
# Furisode Girl: KIMONO2: one Furisode Girl
sprite-0x69 = 기모노드레스
# Furisode Girl: KIMONO3: one Furisode Girl
sprite-0x6a = 기모노드레스
# Furisode Girl: KIMONO4: one Furisode Girl
sprite-0x6b = 기모노드레스
# Lady: LADY: 19 zones, no trainer
sprite-0x36 = 아기씨
# Waitress: WAITRESS: 24 zones, no trainer
sprite-0x49 = 웨이트리스
# Garçon: GARCON: 25 zones, no trainer
sprite-0x72 = 가르송
# Chef: COOK: 12 zones, no trainer
sprite-0x5e = 요리사
# Butler: BUTLER: 29 zones, the Pokémon Centers
sprite-0x65 = 집사
# Maid: MAID: 32 zones, the Pokémon Centers
sprite-0x66 = 메이드
# Sky Trainer: SKYM: 8 zones, no trainer
sprite-0x6c = 스카이 트레이너
# Sky Trainer: SKYW: 8 zones, no trainer
sprite-0x6d = 스카이 트레이너
# Fisherman: FISHING2: 4 zones, as FISHING is
sprite-0xa5 = 낚시꾼

# The game's own words that this table had not copied, as `archives` reads them: the
# kinds of trainer two or more of one class stand as (spikes 90 and 193).
# TOURISTM
sprite-0x33 = 투어객
# BADMAN
sprite-0x39 = 배드가이
# TRAINERM
sprite-0x5a = 엘리트 트레이너
# RANGERM
sprite-0x5c = 포켓몬 레인저
# OCCULTW
sprite-0x63 = 오컬트마니아
# FIGHTERM
sprite-0x71 = 태권왕

# The Pokémon on the map, by the species each one's model draws, its tag read through
# the battle models (spike 218); with the ROM, its own species names. Mewtwo's model
# names none.
# Mewtwo: P_MODEL01: the Unknown Dungeon, "Mewtwo blended into the darkness of the cave" (story 185:1); its model names no species
sprite-0x1017 = 뮤츠
# EV_POKE01
sprite-0x1000 = 호바귀
# EV_POKE02
sprite-0x1001 = 파르토
# EV_POKE03
sprite-0x1002 = 지가르데
# EV_POKE04
sprite-0x1003 = 레오꼬
# EV_POKE05
sprite-0x1004 = 고고트
# RIDE_POKE02
sprite-0x1007 = 맘모꾸리
# RIDE_POKE04
sprite-0x1008 = 메이클
# EV_POKE07
sprite-0x1009 = 루카리오
# EV_POKE08
sprite-0x100a = 트리미앙
# EV_POKE10
sprite-0x100b = 화살꼬빈
# EV_POKE11
sprite-0x100c = 잠만보
# EV_POKE12
sprite-0x100d = 냐스퍼
# RIDE_POKE05
sprite-0x100f = 뿔카노
# EV_POKE09
sprite-0x1010 = 눈설왕
# EV_POKE14
sprite-0x1011 = 메이클
# EV_POKE15
sprite-0x1013 = 마릴

# Things standing on the map that the game never names, called by a word of the line the shared
# script they run shows when the player faces one (a/0/3/1 file 27, story 130; spike 219), as
# ORAS's are (spike 137).
# en bush: ITREE01, runs 0x2712: story 130:3
sprite-0x200b = 가시
# en cracked boulder: IROCK01, runs 0x2710: story 130:1
sprite-0x200c = 금이 간 바위
# en big boulder: KROCK01, runs 0x2714: story 130:5
sprite-0x2009 = 큰 바위
# en cracked wall: IWALL01, runs 0x2716: story 130:9
sprite-0x2034 = 금이 간 벽
# en rock: ISNOW01: ten on Route 17, where "all the rocks along the path are nothing against Mamoswine's strength!" (story 70:5)
sprite-0x2035 = 바위
# en rock: IROCK02: ten on Route 9, crossed on Rhyhorn; no line, its model is it0014_rock (inferred), the word of story 70:5
sprite-0x200d = 바위

# Things standing on the map, by the model their sprite's record in a/0/2/2 draws (spike 219), and
# called by a word of the line their zone's case shows, or the line their shared script does.
# en Central Kalos Pokédex award: AWARD04, case 0xd: story 55:4
sprite-0x203b = 센트럴칼로스도감 표창장
# en Coastal Kalos Pokédex award: AWARD03, case 0xe: story 55:5
sprite-0x203a = 코스트칼로스도감 표창장
# en Mountain Kalos Pokédex award: AWARD05, case 0xf: story 55:6
sprite-0x203c = 마운틴칼로스도감 표창장
# en Kalos region award: AWARD01, case 0x10: story 55:7
sprite-0x2036 = 칼로스지방 표창장
# en National Pokédex award: AWARD02, case 0x11: story 55:8
sprite-0x2037 = 전국도감 표창장
# en Berry tree: KINOMI, runs 0x2939: story 142:15, "There are small Berries growing high up in the tree!"; Route 15
sprite-0x200e = 열매 나무
# en Berry tree: KINOMI2, Routes 5 and 19
sprite-0x2011 = 열매 나무
# en Berry tree: KINOMI3, Routes 11 and 12
sprite-0x2012 = 열매 나무
# en Berry tree: KINOMI4, Routes 16 and 22
sprite-0x2013 = 열매 나무
# en Berry tree: KINOMI11, Route 7
sprite-0x201a = 열매 나무
# en Berry tree: KINOMI15, Route 18
sprite-0x201e = 열매 나무
# en Berry tree: KINOMI31, Route 21
sprite-0x202e = 열매 나무
# en Berry tree: KINOMI33, Route 10
sprite-0x2030 = 열매 나무
# en Berry tree: KINOMI34, Route 6
sprite-0x2031 = 열매 나무
# en Berry tree: KINOMI35, Routes 8 and 14
sprite-0x2032 = 열매 나무
# en Berry tree: KINOMI36, Route 20
sprite-0x2033 = 열매 나무

# Things the game never names, called what the ROM's own id for the sprite says they are.
# en sparkle: KIRAKIRA, runs 0x25e9: five in Lumiose and Prism Tower; its model's joints are kira00 to kira03
sprite-0x203d = 반짝임
# en sparkle: KIRAKIRA2: twelve in twelve zones, each case handing over one Mega Stone, items 665 to 683 (text 96)
sprite-0x2042 = 반짝임
# en cone: CONE, model com_bm_skatecone: two across a Lumiose street, "There is a fence blocking the road." (story 317:18), and seven in Lysandre Labs; the word of story 317:18
sprite-0x203e = 교통콘
# en shaking grass: TOBI_KUSA, no script: three on Route 6 and three on Route 18, rustling till a wild Pokémon "leaped out of the tall grass" (text 124:261)
sprite-0x2002 = 흔들리는 풀숲
# en lurking Pokémon: ENEMY3D, shared script 2000: nine in the Glittering Cave, drawn as nothing, a wild battle as the player comes up to one (spike 239)
sprite-0x95 = 숨어 있는 포켓몬

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
tile-0xd8 = 포켓몬 상품
# Pokémon books: text 132:1, books and photo albums
tile-0xd9 = 포켓몬 책
# cabinet: text 132:2, ORAS's line and words
tile-0xda = 수납 가구
# useful items: text 132:3
tile-0xdb = 편리해 보이는 상품
# Pokémon novelties: text 132:4
tile-0xdc = 많은 포켓몬 상품
# trash can: text 132:6
tile-0xdd = 쓰레기통
# books for Pokémon: text 132:5, "Are these books for Pokémon to read?"
tile-0xde = 포켓몬이 읽는 책

## furniture
# What a piece of furniture is, by the game's shared script it runs (X's are in a/0/3/1): a record's
# own number from 0x7d0 up, or the one its zone's script runs for it (spike 220). The elevators, photo
# spots, trash cans and Berry trees carry theirs as their own number; the rest are named by the line
# they show, or by a line of another message of the ROM's where theirs has no word.
# elevator: story 137:21 and 145:11, the elevators' own file: Lumiose City and Lysandre Labs, two records a door
furniture-0x27da = 엘리베이터
furniture-0x27db = 엘리베이터
furniture-0x27dc = 엘리베이터
furniture-0x27dd = 엘리베이터
furniture-0x27de = 엘리베이터
furniture-0x27df = 엘리베이터
furniture-0x27e0 = 엘리베이터
furniture-0x27e1 = 엘리베이터
furniture-0x27e2 = 엘리베이터
furniture-0x27e3 = 엘리베이터
furniture-0x27e4 = 엘리베이터
furniture-0x27e5 = 엘리베이터
furniture-0x27e6 = 엘리베이터
furniture-0x27e7 = 엘리베이터
furniture-0x27e8 = 엘리베이터
furniture-0x27e9 = 엘리베이터
furniture-0x27ea = 엘리베이터
furniture-0x27eb = 엘리베이터
furniture-0x27ec = 엘리베이터
furniture-0x27ed = 엘리베이터
furniture-0x27ee = 엘리베이터
furniture-0x27ef = 엘리베이터
furniture-0x27f0 = 엘리베이터
furniture-0x27f1 = 엘리베이터
furniture-0x27f2 = 엘리베이터
furniture-0x27f3 = 엘리베이터
furniture-0x27f4 = 엘리베이터
furniture-0x27f5 = 엘리베이터
furniture-0x27f6 = 엘리베이터
furniture-0x27f7 = 엘리베이터
furniture-0x27f8 = 엘리베이터
furniture-0x27f9 = 엘리베이터
furniture-0x27fa = 엘리베이터
furniture-0x27fb = 엘리베이터
furniture-0x27fc = 엘리베이터
furniture-0x27fd = 엘리베이터
furniture-0x27fe = 엘리베이터
furniture-0x27ff = 엘리베이터
furniture-0x2800 = 엘리베이터
furniture-0x2801 = 엘리베이터
furniture-0x2802 = 엘리베이터
furniture-0x2803 = 엘리베이터
furniture-0x2804 = 엘리베이터
furniture-0x2805 = 엘리베이터
furniture-0x2806 = 엘리베이터
furniture-0x2807 = 엘리베이터
furniture-0x2808 = 엘리베이터
furniture-0x2809 = 엘리베이터
# meteor: story 221:8, Ambrette Town
furniture-0x281e = 운석
# cardboard boxes: story 220:0, Lumiose City, zone 0x54
furniture-0x284b = 박스
# photo spot: story 222:0, one in each of 12 places
furniture-0x28c8 = 사진 간판
furniture-0x28c9 = 사진 간판
furniture-0x28ca = 사진 간판
furniture-0x28cb = 사진 간판
furniture-0x28cc = 사진 간판
furniture-0x28cd = 사진 간판
furniture-0x28ce = 사진 간판
furniture-0x28cf = 사진 간판
furniture-0x28d0 = 사진 간판
furniture-0x28d1 = 사진 간판
furniture-0x28d2 = 사진 간판
furniture-0x28d3 = 사진 간판
# trash can: story 142:14, Pokémon Village and the Lost Hotel
furniture-0x28f0 = 쓰레기통
furniture-0x28f1 = 쓰레기통
furniture-0x28f2 = 쓰레기통
furniture-0x28f3 = 쓰레기통
furniture-0x28f4 = 쓰레기통
furniture-0x28f5 = 쓰레기통
furniture-0x28f6 = 쓰레기통
furniture-0x28f7 = 쓰레기통
furniture-0x28f8 = 쓰레기통
furniture-0x28f9 = 쓰레기통
# Berry tree: story 142:15, its Berries and its tree, Routes 5 to 22
furniture-0x2922 = 열매 나무
furniture-0x2923 = 열매 나무
furniture-0x2924 = 열매 나무
furniture-0x2925 = 열매 나무
furniture-0x2926 = 열매 나무
furniture-0x2927 = 열매 나무
furniture-0x2928 = 열매 나무
furniture-0x2929 = 열매 나무
furniture-0x292a = 열매 나무
furniture-0x292b = 열매 나무
furniture-0x292c = 열매 나무
furniture-0x292d = 열매 나무
furniture-0x292e = 열매 나무
furniture-0x292f = 열매 나무
furniture-0x2930 = 열매 나무
# fitting room: text 142:85, the 17 zones of the Pokémon Centers' PC
furniture-0x29d2 = 피팅룸
# fitting room: text 142:85, the 7 boutiques' fitting room, called as the Pokémon Centers' is
furniture-0x29d3 = 피팅룸
# clothes: text 142:60 and 64, the boutiques' displays
furniture-0x29d4 = 옷
# bed: story 398:19, zones 0x88 to 0x8b; French story 204:1
furniture-0x2a2d = 침대
# taxi stand: story 399:138, Lumiose's 11 taxi stands
furniture-0x2a3a = 택시 타는 곳
# Gogoat Shuttle: story 399:218, Lumiose's 12 stops
furniture-0x2a3b = 고고셔틀
furniture-0x2a3c = 고고셔틀
furniture-0x2a3d = 고고셔틀
furniture-0x2a3e = 고고셔틀
furniture-0x2a3f = 고고셔틀
furniture-0x2a40 = 고고셔틀
furniture-0x2a41 = 고고셔틀
furniture-0x2a42 = 고고셔틀
furniture-0x2a43 = 고고셔틀
furniture-0x2a44 = 고고셔틀
furniture-0x2a45 = 고고셔틀
furniture-0x2a46 = 고고셔틀

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spikes 211 and 220). A slot is
# in a group where its message opens as the group's does in any one language. The first five
# slots, zone 0x2, are the player's house.
# soft soil: story 127:1, Route 7, zone 0x160
place-0x160-furniture-2 = 푹신푹신한 땅
place-0x160-furniture-3 = 푹신푹신한 땅
place-0x160-furniture-4 = 푹신푹신한 땅
place-0x160-furniture-5 = 푹신푹신한 땅
place-0x160-furniture-6 = 푹신푹신한 땅
place-0x160-furniture-7 = 푹신푹신한 땅
place-0x160-furniture-8 = 푹신푹신한 땅
place-0x160-furniture-9 = 푹신푹신한 땅
place-0x160-furniture-10 = 푹신푹신한 땅
place-0x160-furniture-11 = 푹신푹신한 땅
place-0x160-furniture-12 = 푹신푹신한 땅
place-0x160-furniture-13 = 푹신푹신한 땅
place-0x160-furniture-14 = 푹신푹신한 땅
place-0x160-furniture-15 = 푹신푹신한 땅
place-0x160-furniture-16 = 푹신푹신한 땅
place-0x160-furniture-17 = 푹신푹신한 땅
place-0x160-furniture-18 = 푹신푹신한 땅
place-0x160-furniture-19 = 푹신푹신한 땅
place-0x160-furniture-20 = 푹신푹신한 땅
place-0x160-furniture-21 = 푹신푹신한 땅
place-0x160-furniture-22 = 푹신푹신한 땅
place-0x160-furniture-23 = 푹신푹신한 땅
place-0x160-furniture-24 = 푹신푹신한 땅
place-0x160-furniture-25 = 푹신푹신한 땅
place-0x160-furniture-26 = 푹신푹신한 땅
place-0x160-furniture-27 = 푹신푹신한 땅
place-0x160-furniture-28 = 푹신푹신한 땅
place-0x160-furniture-29 = 푹신푹신한 땅
place-0x160-furniture-30 = 푹신푹신한 땅
place-0x160-furniture-31 = 푹신푹신한 땅
place-0x160-furniture-32 = 푹신푹신한 땅
place-0x160-furniture-33 = 푹신푹신한 땅
place-0x160-furniture-34 = 푹신푹신한 땅
place-0x160-furniture-35 = 푹신푹신한 땅
place-0x160-furniture-36 = 푹신푹신한 땅
place-0x160-furniture-37 = 푹신푹신한 땅
# audio guide: story 370:9, Lumiose City, zones 0x68 to 0x6a
place-0x68-furniture-0 = 가이드 박스
place-0x68-furniture-1 = 가이드 박스
place-0x68-furniture-2 = 가이드 박스
place-0x68-furniture-3 = 가이드 박스
place-0x68-furniture-4 = 가이드 박스
place-0x68-furniture-5 = 가이드 박스
place-0x68-furniture-6 = 가이드 박스
place-0x69-furniture-0 = 가이드 박스
place-0x69-furniture-1 = 가이드 박스
place-0x69-furniture-2 = 가이드 박스
place-0x69-furniture-3 = 가이드 박스
place-0x69-furniture-4 = 가이드 박스
place-0x69-furniture-5 = 가이드 박스
place-0x69-furniture-6 = 가이드 박스
place-0x6a-furniture-0 = 가이드 박스
place-0x6a-furniture-1 = 가이드 박스
place-0x6a-furniture-2 = 가이드 박스
place-0x6a-furniture-3 = 가이드 박스
place-0x6a-furniture-4 = 가이드 박스
place-0x6a-furniture-5 = 가이드 박스
place-0x6a-furniture-6 = 가이드 박스
place-0x6a-furniture-7 = 가이드 박스
place-0x6a-furniture-8 = 가이드 박스
place-0x6a-furniture-9 = 가이드 박스
# statue of a Bisharp: story 215:21, Parfum Palace
place-0x120-furniture-7 = 절각참의 상
place-0x120-furniture-8 = 절각참의 상
place-0x120-furniture-9 = 절각참의 상
place-0x120-furniture-10 = 절각참의 상
place-0x120-furniture-11 = 절각참의 상
place-0x120-furniture-12 = 절각참의 상
place-0x120-furniture-13 = 절각참의 상
place-0x120-furniture-14 = 절각참의 상
place-0x125-furniture-2 = 절각참의 상
place-0x125-furniture-3 = 절각참의 상
place-0x127-furniture-2 = 절각참의 상
place-0x127-furniture-3 = 절각참의 상
place-0x12c-furniture-3 = 절각참의 상
place-0x12c-furniture-4 = 절각참의 상
# golden statue of a knight: story 215:22, Parfum Palace
place-0x120-furniture-15 = 황금의 기사상
place-0x120-furniture-16 = 황금의 기사상
place-0x120-furniture-17 = 황금의 기사상
place-0x120-furniture-18 = 황금의 기사상
place-0x120-furniture-19 = 황금의 기사상
place-0x120-furniture-20 = 황금의 기사상
place-0x120-furniture-21 = 황금의 기사상
place-0x120-furniture-22 = 황금의 기사상
place-0x120-furniture-23 = 황금의 기사상
place-0x120-furniture-24 = 황금의 기사상
place-0x125-furniture-4 = 황금의 기사상
place-0x125-furniture-5 = 황금의 기사상
place-0x127-furniture-4 = 황금의 기사상
place-0x127-furniture-5 = 황금의 기사상
# whiteboard: story 412:7, Santalune City, zone 0x3e
place-0x3e-furniture-0 = 화이트보드
place-0x3e-furniture-1 = 화이트보드
place-0x3e-furniture-2 = 화이트보드
place-0x3e-furniture-3 = 화이트보드
place-0x3e-furniture-4 = 화이트보드
place-0x3e-furniture-5 = 화이트보드
place-0x3e-furniture-6 = 화이트보드
place-0x3e-furniture-7 = 화이트보드
place-0x3e-furniture-8 = 화이트보드
# statue of a Poké Ball: story 216:47, Parfum Palace's garden
place-0x12e-furniture-2 = 몬스터볼의 상
place-0x12e-furniture-3 = 몬스터볼의 상
place-0x12e-furniture-4 = 몬스터볼의 상
place-0x12e-furniture-5 = 몬스터볼의 상
place-0x12e-furniture-6 = 몬스터볼의 상
place-0x12e-furniture-7 = 몬스터볼의 상
# monument: story 226:7, the Battle Maison
place-0xf1-furniture-0 = 기념비
place-0xf1-furniture-1 = 기념비
place-0xf1-furniture-2 = 기념비
place-0xf1-furniture-3 = 기념비
place-0xf1-furniture-4 = 기념비
# portrait: story 203:0, Parfum Palace
place-0x12c-furniture-0 = 초상화
place-0x12c-furniture-1 = 초상화
place-0x12c-furniture-2 = 초상화
# record machine: story 390:20, Lumiose City, zone 0x54
place-0x54-furniture-2 = 레코드머신
place-0x54-furniture-3 = 레코드머신
place-0x54-furniture-4 = 레코드머신
place-0x54-furniture-5 = 레코드머신
# PC: story 55:1, the bedroom's PC
place-0x2-furniture-0 = PC
# mirror: story 55:2
place-0x2-furniture-1 = 거울
# TV: story 55:38 shows a film; the word is story 390:19's
place-0x2-furniture-2 = TV
# TV: story 55:39 shows a film, on the ground floor; story 390:19
place-0x2-furniture-3 = TV
# Wii U: story 55:0
place-0x2-furniture-4 = Wii U
# statue: story 420:3, Santalune Gym, zone 0x37
place-0x37-furniture-0 = 수수께끼상
place-0x37-furniture-1 = 수수께끼상
# pole: story 420:1, Santalune Gym, zones 0x37 and 0x38
place-0x37-furniture-2 = 봉
place-0x38-furniture-0 = 봉
# photographs: story 420:9, Santalune Gym, zone 0x37
place-0x37-furniture-3 = 사진
place-0x37-furniture-4 = 사진
place-0x37-furniture-5 = 사진
# photograph: story 420:7 and 420:8, Santalune Gym, zone 0x37
place-0x37-furniture-6 = 사진
place-0x37-furniture-7 = 사진
# Lysandre Café: story 313:4, Lumiose City, zone 0x43: the plate of script 22, which no record shows, so no door is paired with it
place-0x43-warp-1 = 플라드리 카페
