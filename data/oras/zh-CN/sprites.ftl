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
sprite-0x118 = 妈妈
# opening.ctm: "Halloo! You over there! Please! Help me out!"
sprite-0x13e = 小田卷
# HAKASE_RS: Birch on Route 101 in movie3 (spike 52)
sprite-0xb3 = 小田卷
# wally.ctm: "Welcome to the Pokémon Center", in two Centers on two saves
sprite-0x12a = 乔伊小姐
# HAGI_RS: Dewford, Routes 104 and 109, the S.S. Tidal
sprite-0x11b = 哈奇
# KUSUNOKI_RS: Slateport and Lilycove
sprite-0x135 = 库斯诺吉船长
# SORAISI_RS: Fallarbor, Meteor Falls, Mossdeep
sprite-0x136 = 索蓝斯
# MAYUMI_RS: Fallarbor and Route 114
sprite-0x161 = 南木
# CHAMPFATHER_RS: Rustboro
sprite-0x11a = 兹伏奇
# HANDSOME_RS: the Battle Resort
sprite-0xd7 = 帅哥
# LEADER7_M_RS: Mossdeep; the ROM names only "Liza & Tate"
sprite-0xc1 = 枫
# LEADER7_F_RS: Mossdeep
sprite-0xc2 = 南
# CONTEST_RS: beside Ali in every Contest Hall, "That performance by Lisia and Ali just now was amazing!" (story 39:7)
sprite-0xd2 = 琉琪亚
# KIZUNA_RS: Mauville, "I'm Mr. Bonding, the expert on O-Powers!" (story 570:43)
sprite-0x134 = 邦迪先生
# DIRECTOR_RS: Lilycove, "I'm the Game Director." (story 530:0)
sprite-0x139 = 游戏总监
# EV_POKE35: Lilycove's three; its model names Nidoran♂ too, and draws Wailmer's body
sprite-0x1030 = 吼鲸王

# Shared looks the game never names, called what the ROM's own id for the sprite says they are
# (file 94, `spikes/85_rom_names.py sprite`), since everyone drawn as one hears the same word:
# WOMAN_RS is Mrs. Birch in Littleroot and a woman in 27 other zones. A `_ROOM_RS` sprite is
# the same look indoors. Every one here stands on ORAS's maps; STOPPER3, an invisible blocker
# outside Mauville Gym, is left out.
# BROTHER_RS: Littleroot and Route 101
sprite-0x119 = 青年
# CHILDBOY_RS
sprite-0x11c = 小男孩
# CHILDBOY_ROOM_RS
sprite-0x150 = 小男孩
# CHILDGIRL_RS
sprite-0x11d = 小女孩
# CHILDGIRL_ROOM_RS
sprite-0x151 = 小女孩
# OLDMAN_RS
sprite-0x11e = 老爷爷
# OLDMAN_ROOM_RS
sprite-0x152 = 老爷爷
# OLDWOMAN_RS
sprite-0x11f = 老奶奶
# OLDWOMAN_ROOM_RS
sprite-0x153 = 老奶奶
# BIGMAN1_RS
sprite-0x120 = 壮汉
# BIGMAN2_RS
sprite-0x121 = 壮汉
# BIGMAN1_ROOM_RS
sprite-0x154 = 壮汉
# BIGMAN2_ROOM_RS
sprite-0x155 = 壮汉
# BIGMAN, X's sprite, once in ORAS
sprite-0x70 = 壮汉
# OYAJI1_RS: Mauville, as are the four after it
sprite-0x122 = 中年男性
# OYAJI2_RS
sprite-0x123 = 中年男性
# OYAJI3_RS
sprite-0x124 = 中年男性
# OYAJI4_RS
sprite-0x125 = 中年男性
# OYAJI5_RS
sprite-0x126 = 中年男性
# WOMAN_RS
sprite-0x128 = 女性
# WOMAN2_RS
sprite-0x142 = 女性
# WOMAN_ROOM_RS
sprite-0x156 = 女性
# MAN_RS
sprite-0x132 = 男性
# MAN_ROOM_RS
sprite-0x157 = 男性
# MAN1, X's sprite, once in ORAS
sprite-0x31 = 男性
# GIRL_RS
sprite-0x131 = 少女
# COUPLE_M_RS: the young couples' man
sprite-0x10c = 青年
# COUPLE_F_RS: and woman
sprite-0x10d = 年轻女性
# TWIN1_RS
sprite-0x10a = 双胞胎
# TWIN2_RS
sprite-0x10b = 双胞胎
# KARAKURI_RS: Route 110 only, the Trick House
sprite-0x127 = 戏法大师
# SUNGLASS_RS
sprite-0x129 = 戴墨镜的男人
# SHOP_M_RS
sprite-0x12b = 店员
# INFORMATION_RS
sprite-0x12c = 接待员
# INFORMATIONW, X's sprite
sprite-0x3e = 接待员
# DANPEI_RS: in all eight Gym towns, as X's DANPEI is in all eight Gyms
sprite-0x12d = 道馆向导
# DELIVER1_RS: in all sixteen Pokémon Centers
sprite-0x12e = 配送员
# DELIVER2_RS: likewise
sprite-0x12f = 配送员
# ASSISTANT_M_RS
sprite-0x133 = 研究员
# ASSISTANTM, X's sprite
sprite-0x5f = 研究员
# KIMONO3_RS
sprite-0x137 = 和服女子
# OFFICEWORKER_RS
sprite-0x140 = 上班族
# GAMEBOY_RS
sprite-0x141 = 玩掌机的男孩
# REPORTER_RS
sprite-0x108 = 记者
# CAMERAMAN_RS
sprite-0x109 = 摄像师
# ELDER_RS: the elder of each pair of Teammates, Anna & Meg and three more (spike 193)
sprite-0x10e = 年长搭档
# JUNIOR_RS: and the younger
sprite-0x10f = 年少搭档

# Trainers' looks the ROM leaves unnamed, one trainer or none of a class standing as each, or a
# pair class beside the class (spike 90); called by the class their id and their trainers share.
# SCHOOL_M_RS
sprite-0xdd = 学生
# SCHOOL_M_ROOM_RS
sprite-0x145 = 学生
# SCHOOL_W_ROOM_RS
sprite-0x146 = 学生
# TANPAN_ROOM_RS
sprite-0x143 = 少年
# MINI_ROOM_RS
sprite-0x144 = 少女
# MOUNTMAN_ROOM_RS
sprite-0x147 = 登山男
# AROMA_ROOM_RS
sprite-0x148 = 芳香姐姐
# SWIMMER_M_RS
sprite-0xe5 = 泳裤小伙子
# UKIWA_W2_RS
sprite-0x15c = 浮潜者
# BREADER_M_RS
sprite-0xea = 宝可梦培育家
# BREADER_W_RS
sprite-0xeb = 宝可梦培育家
# FISHING_RS
sprite-0xef = 垂钓者
# TRIRUN_M_RS
sprite-0x101 = 铁人三项运动员
# ELITE_M_ROOM_RS
sprite-0x149 = 精英训练家
# ELITE_F_ROOM_RS
sprite-0x14a = 精英训练家
# VETERAN_M_ROOM_RS
sprite-0x14b = 老手
# VETERAN_F_ROOM_RS
sprite-0x14c = 老手
# SCARY_M_ROOM_RS
sprite-0x14d = 不良青年
# SCARY_W_ROOM_RS
sprite-0x14e = 不良少女

# Things standing on the map that the game never names.
# omega_ruby_bush.ctm: "You can't pass because of prickly thorns, but a Pokémon may be able to cut the tree."
sprite-0x200b = 灌木丛
# omega_ruby_smash.ctm: "It's a cracked boulder. A Pokémon may be able to break it."
sprite-0x200c = 有裂缝的岩石
# omega_ruby_strength-boulders.ctm: "It's a big boulder, but a Pokémon may be able to push it aside."
sprite-0x2009 = 巨大岩石

# Things standing on the map, by the model their sprite's record in a/1/3/7 draws (spike 211), and
# called what the game calls that thing among the Secret Base decorations (text 56) or the items
# (text 114).
# KIRAKIRA: the sparkle at each of the 28 places a Mega Stone lies, a script each from 0x41f
# to 0x43a (or_mega_stone: "Raiden found a Sablenite!"); the word is story 330:64's
sprite-0x203d = 超级石
# DOLL_RURIRI: text 56:111, in both houses in Littleroot
sprite-0x204a = 露力丽玩偶
# DOLL_ENEKO: text 56:112
sprite-0x204b = 向尾喵玩偶
# DOLL_TIRUTTO: text 56:113
sprite-0x204c = 青绵鸟玩偶
# DOLL_RAPURASU: text 56:120
sprite-0x204d = 拉普拉斯玩偶
# DOLL_KABIGON: text 56:118
sprite-0x204e = 卡比兽玩偶
# AIIRONOTAMA: text 114:535, on Mt. Pyre's summit
sprite-0x204f = 蓝色宝珠
# BENIIRONOTAMA: text 114:534, beside it
sprite-0x2050 = 红色宝珠
# MAGMASUIT: text 114:741, in the Magma Hideout
sprite-0x2056 = 熔岩队制服
# AQUASUIT: text 114:742, in the Aqua Hideout
sprite-0x2057 = 海洋队制服
# ASASE_SHIO: text 114:70, in Shoal Cave
sprite-0x2058 = 浅滩海盐
# ASASE_KAIGARA: text 114:71, in Shoal Cave
sprite-0x2059 = 浅滩贝壳
# LETTER: in Steven's house; the item, text 114:725, is the one to him
sprite-0x205b = 信件

# The game's own words, copied from the ROM's English text by `spikes/90_sprite_trainers.py`: the
# story's people by the trainer each is, and every other sprite that two trainers or more of one
# class stand as, by that class (findings, spike 90). With the ROM, these come from it instead.
# RIVAL_RS
sprite-0xb4 = 满充
# RIVAL_EX_RS
sprite-0xb5 = 满充
# CHAMPION_RS
sprite-0xb6 = 大吾
# BIGFOUR1_RS
sprite-0xb7 = 花月
# BIGFOUR2_RS
sprite-0xb8 = 芙蓉
# BIGFOUR3_RS
sprite-0xb9 = 波妮
# BIGFOUR4_RS
sprite-0xba = 源治
# LEADER1_RS
sprite-0xbb = 杜鹃
# LEADER2_RS
sprite-0xbc = 藤树
# LEADER3_RS
sprite-0xbd = 铁旋
# LEADER4_RS
sprite-0xbe = 亚莎
# LEADER5_RS
sprite-0xbf = 千里
# LEADER6_RS
sprite-0xc0 = 娜琪
# LEADER8_RS
sprite-0xc3 = 米可利
# MAGMABOSS_RS
sprite-0xc4 = 赤焰松
# MAGMAKANBU_M_RS
sprite-0xc5 = 篝火
# MAGMAKANBU_F_RS
sprite-0xc6 = 泉
# AQUABOSS_RS
sprite-0xc9 = 水梧桐
# AQUAKANBU_M_RS
sprite-0xca = 阿泉
# AQUAKANBU_F_RS
sprite-0xcb = 阿泉
# HIMITU_RS
sprite-0xd3 = 阿睿
# ZAZAKO_RS: Lorekeeper Zinnia, outside Sky Pillar (spike 193)
sprite-0xd4 = 兹霓亚
# SUPPORT_F_RS, drawn into work object 0x7a (spike 89)
sprite-0x13a = 小遥
# SUPPORT_M_RS, likewise
sprite-0x13b = 祐树
# SUPPORT_F_BIKE_RS, by the ROM's own id; no replay yet
sprite-0x13c = 小遥
# SUPPORT_M_BIKE_RS, likewise
sprite-0x13d = 祐树
# LEADER5A_RS: wally.ctm, "thanked as Mr. Norman"
sprite-0x13f = 千里
# LEADER2A_RS
sprite-0x15d = 藤树
# BIGFOUR2A_RS
sprite-0x15e = 芙蓉
# LEADER6A_RS
sprite-0x15f = 娜琪
# BIGFOUR1A_RS
sprite-0x160 = 花月
# MAGMA_M_RS
sprite-0xc7 = 熔岩队手下
# MAGMA_F_RS
sprite-0xc8 = 熔岩队手下
# AQUA_M_RS
sprite-0xcc = 海洋队手下
# AQUA_F_RS
sprite-0xcd = 海洋队手下
# TANPAN_RS
sprite-0xda = 少年
# MINI_RS
sprite-0xdb = 少女
# MUSITORI_RS
sprite-0xdc = 捕虫少年
# SCHOOL_W_RS
sprite-0xde = 学生
# CAMPBOY_RS
sprite-0xdf = 露营少年
# PICNICGIRL_RS
sprite-0xe0 = 野餐女孩
# RICHBOY_RS
sprite-0xe1 = 富家少爷
# RICHGIRL_RS
sprite-0xe2 = 富家小姐
# UKIWA_M_RS
sprite-0xe3 = 浮潜者
# UKIWA_W_RS
sprite-0xe4 = 浮潜者
# SWIMMER_W_RS
sprite-0xe6 = 泳装女孩
# GENTLEMAN_RS
sprite-0xe7 = 绅士
# MARINER_RS
sprite-0xe8 = 水手
# MOUNTMAN_RS
sprite-0xe9 = 登山男
# FIGHTER_W_RS
sprite-0xed = 格斗少女
# COLLECTOR_RS
sprite-0xee = 收藏家
# MONSTERMANIA_RS
sprite-0xf0 = 怪兽狂
# BUGMANIA_RS
sprite-0xf1 = 虫狂
# RUINMANIA_RS
sprite-0xf2 = 遗迹狂
# OCCULT_RS
sprite-0xf3 = 超能力少女
# ESP_M_RS
sprite-0xf4 = 超能力者
# NINJA_RS
sprite-0xf5 = 忍者小子
# CAMPFIRE_RS
sprite-0xf6 = 烤火人
# BIRDTAMER_RS
sprite-0xf7 = 养鸟人
# DRAGONTAMER_RS
sprite-0xf8 = 龙使
# GUITAR_RS
sprite-0xf9 = 吉他手
# RANGER_M_RS
sprite-0xfa = 宝可梦巡护员
# RANGER_W_RS
sprite-0xfb = 宝可梦巡护员
# ADULT_RS
sprite-0xfc = 大姐姐
# PARASOL_RS
sprite-0xfd = 遮阳伞姐姐
# AROMA_RS
sprite-0xfe = 芳香姐姐
# DAISUKI_M_RS
sprite-0xff = 宝可梦爱好者
# DAISUKI_W_RS
sprite-0x100 = 宝可梦爱好者
# TRIBIKE_M_RS
sprite-0x102 = 铁人三项运动员
# TRISWIM_M_RS
sprite-0x103 = 铁人三项运动员
# ELITE_M_RS
sprite-0x104 = 精英训练家
# SWIMMER_W2_RS
sprite-0x159 = 泳装女孩
# FIGHTER_M_RS
sprite-0xec = 空手道王
# ELITE_F_RS
sprite-0x105 = 精英训练家
# VETERAN_M_RS
sprite-0x106 = 老手
# FAIRYTALE_RS, beside two pairs of Mysterious Sisters (spike 193)
sprite-0x110 = 童话少女
# VETERAN_F_RS
sprite-0x107 = 老手
# NAMINORI_M_RS
sprite-0x111 = 精英训练家
# NAMINORI_F_RS
sprite-0x112 = 精英训练家
# BACKPACKER_M_RS
sprite-0x113 = 背包客
# SCARY_M_RS
sprite-0x114 = 不良青年
# SCARY_W_RS
sprite-0x115 = 不良少女
# DIVER_M_RS
sprite-0x116 = 水肺潜水员
# DIVER_F_RS
sprite-0x117 = 自由潜水员
# FISHING2_RS
sprite-0x158 = 垂钓者
# SWIMMER_M2_RS
sprite-0x15a = 泳裤小伙子
# UKIWA_M2_RS
sprite-0x15b = 浮潜者

# The Pokémon on the map, by the Pokémon each one's model draws (spike 193); with the ROM, its
# own species names.
# RIDE_POKE01
sprite-0x1006 = 独角犀牛
# EV_POKE07
sprite-0x1009 = 路卡利欧
# RIDE_POKE05
sprite-0x100f = 独角犀牛
# EV_POKE15
sprite-0x1013 = 玛力露
# RIDE_POKE01_EX
sprite-0x1014 = 独角犀牛
# EV_POKE20
sprite-0x101b = 固拉多
# EV_POKE21
sprite-0x101d = 盖欧卡
# EV_POKE22
sprite-0x101f = 烈空坐
# EV_POKE23
sprite-0x1021 = 拉帝亚斯
# EV_POKE24
sprite-0x1022 = 拉帝欧斯
# RIDE_POKE13
sprite-0x1023 = 拉帝亚斯
# RIDE_POKE14
sprite-0x1024 = 拉帝欧斯
# EV_POKE25
sprite-0x1025 = 豪力
# EV_POKE26
sprite-0x1026 = 长翅鸥
# EV_POKE28
sprite-0x1029 = 变隐龙
# EV_POKE29
sprite-0x102a = 雷吉洛克
# EV_POKE30
sprite-0x102b = 雷吉艾斯
# EV_POKE31
sprite-0x102c = 雷吉斯奇鲁
# EV_POKE32
sprite-0x102d = 土狼犬
# EV_POKE33
sprite-0x102e = 向尾喵
# EV_POKE34
sprite-0x102f = 蛇纹熊
# EV_POKE36
sprite-0x1031 = 正电拍拍
# EV_POKE37
sprite-0x1032 = 负电拍拍
# EV_POKE38
sprite-0x1033 = 玛沙那
# EV_POKE39
sprite-0x1034 = 蘑蘑菇
# EV_POKE40
sprite-0x1035 = 懒人獭
# EV_POKE41
sprite-0x1036 = 毒蔷薇
# EV_POKE42
sprite-0x1037 = 傲骨燕
# EV_POKE44
sprite-0x1039 = 咕妞妞
# EV_POKE45
sprite-0x103a = 落雷兽
# EV_POKE46
sprite-0x103b = 七夕青鸟
# EV_POKE47
sprite-0x103c = 土狼犬
# EV_POKE48
sprite-0x103d = 固拉多
# EV_POKE49
sprite-0x103e = 盖欧卡
# EV_POKE50
sprite-0x103f = 豪力

## tiles
# What each kind of use tile is: a cell of the map the player faces and presses A at, with no
# record behind it (spike 83). The game runs one script for each kind wherever it stands, and each
# of those shows one line of text file 155, which says what is there; the word here is taken from
# that line (spike 210). The PC's and the TV's are the game's own names for them. The PC's word is
# read from the ROM ahead of this one.
# the PC's own lines (archives.PC_LINES)
tile-0xd5 = 电脑
# text 56:138, the TV among the Secret Base decorations
tile-0xd7 = 电视
# text 155:0
tile-0xd8 = 宝可梦摆件
# text 155:9
tile-0xd9 = 宝可梦杂志
# text 155:2
tile-0xda = 柜子
# text 155:3
tile-0xdb = 日用道具
# text 155:4
tile-0xdc = 宝可梦周边
# text 155:6
tile-0xdd = 垃圾桶
# text 155:7
tile-0xde = 宝可梦绘本
# text 155:10
tile-0xdf = 花瓶

## furniture
# What a piece of furniture is, by the game's shared script it runs: a record's own number from
# 0x7d0 up, or the one its zone's script runs for it (spike 210). Every vending machine runs one of
# 0x28f0 to 0x28f4, and is called what the game calls the one among the Secret Base decorations
# (text 56:144); the wardrobes say so (story 460:35).
# text 56:144
furniture-0x28f0 = 自动售货机
# text 56:144
furniture-0x28f1 = 自动售货机
# text 56:144
furniture-0x28f2 = 自动售货机
# text 56:144
furniture-0x28f3 = 自动售货机
# text 56:144
furniture-0x28f4 = 自动售货机
# story 460:35
furniture-0x284e = 衣柜
# The elevators' word is the Mauville Hills computer's, story 264:15; the bedrooms', the lines the
# player's room shows.
# Mauville's and the Lilycove Department Store's doors
furniture-0x27da = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27db = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27dc = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27dd = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27de = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27df = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e0 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e1 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e2 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e3 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e4 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e5 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e6 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e7 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e8 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27e9 = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27ea = 电梯
# Mauville's and the Lilycove Department Store's doors
furniture-0x27eb = 电梯
# story 253:36, the player's room and the neighbour's
furniture-0x2956 = 时钟
# story 253:43, both rooms
furniture-0x2961 = Wii U
# text 114:442, the item; both rooms
furniture-0x2962 = 城镇地图
# story 253:41, both rooms
furniture-0x2963 = 笔记本
# Mom's, in both houses
furniture-0x2965 = 笔记本

## places
# Furniture the game shows alike in many zones, each with a message of its own zone's, so named by
# its slot as the player's names file names one: zone, furniture, index (spike 211). A slot is in a
# group where its message opens as the group's does in any one language; Omega Ruby and Alpha
# Sapphire have the same.
# story 341:8, "There's a bed.", in the Magma Hideout
place-0xaa-furniture-0 = 床
place-0xaa-furniture-1 = 床
place-0xaa-furniture-2 = 床
place-0xaa-furniture-3 = 床
place-0xaa-furniture-4 = 床
place-0xaa-furniture-5 = 床
place-0xaa-furniture-6 = 床
place-0xaa-furniture-7 = 床
# story 420:16, in the Aqua Hideout
place-0x5d-furniture-0 = 床
place-0x5d-furniture-1 = 床
place-0x5d-furniture-2 = 床
place-0x5d-furniture-3 = 床
place-0x5d-furniture-4 = 床
place-0x5d-furniture-5 = 床
place-0x5d-furniture-6 = 床
place-0x5d-furniture-7 = 床
# story 169:14, at the Weather Institute
place-0x188-furniture-0 = 床
place-0x188-furniture-1 = 床
# story 70:0, "It's a bed.", on the S.S. Tidal
place-0xda-furniture-0 = 床
# story 58:5, "Aha! Pokémon Journal", in the Pokémon Centers
place-0xe4-furniture-0 = 宝可梦通讯
place-0xe8-furniture-0 = 宝可梦通讯
place-0xec-furniture-0 = 宝可梦通讯
place-0xf0-furniture-0 = 宝可梦通讯
place-0xf6-furniture-0 = 宝可梦通讯
place-0xfd-furniture-0 = 宝可梦通讯
place-0x103-furniture-0 = 宝可梦通讯
place-0x109-furniture-0 = 宝可梦通讯
place-0x116-furniture-0 = 宝可梦通讯
place-0x11c-furniture-0 = 宝可梦通讯
place-0x12d-furniture-0 = 宝可梦通讯
place-0x135-furniture-0 = 宝可梦通讯
place-0x14a-furniture-0 = 宝可梦通讯
place-0x155-furniture-0 = 宝可梦通讯
place-0x161-furniture-0 = 宝可梦通讯
place-0x206-furniture-0 = 宝可梦通讯
# story 587:13, "Hoenn's Local Heroes / The Hoenn Rangers Coexistence Force!": posters, figures, a face board
place-0x115-furniture-5 = 丰缘巡护队
place-0x115-furniture-6 = 丰缘巡护队
place-0x143-furniture-4 = 丰缘巡护队
place-0x143-furniture-5 = 丰缘巡护队
place-0x148-furniture-0 = 丰缘巡护队
place-0x151-furniture-1 = 丰缘巡护队
place-0x151-furniture-2 = 丰缘巡护队
place-0x188-furniture-4 = 丰缘巡护队
place-0x188-furniture-5 = 丰缘巡护队
place-0x1c2-furniture-10 = 丰缘巡护队
place-0x1c2-furniture-11 = 丰缘巡护队
place-0x20d-furniture-0 = 丰缘巡护队
# story 206:0, the Trick House's scroll doors
place-0x175-furniture-0 = 上锁的门
place-0x176-furniture-0 = 上锁的门
place-0x177-furniture-0 = 上锁的门
place-0x178-furniture-0 = 上锁的门
place-0x179-furniture-0 = 上锁的门
place-0x17a-furniture-0 = 上锁的门
# story 341:0, "PROJECT AZOTH / File No. 1", in both hideouts
place-0xaa-furniture-8 = 阿佐特计划
place-0xaa-furniture-9 = 阿佐特计划
place-0xaa-furniture-10 = 阿佐特计划
place-0xaa-furniture-11 = 阿佐特计划
place-0xab-furniture-1 = 阿佐特计划
place-0x5d-furniture-8 = 阿佐特计划
place-0x5d-furniture-9 = 阿佐特计划
place-0x5d-furniture-10 = 阿佐特计划
place-0x5d-furniture-11 = 阿佐特计划
place-0x5e-furniture-1 = 阿佐特计划
# story 338:0, "It's the Team Magma flag!"
place-0xad-furniture-0 = 熔岩队旗帜
place-0xad-furniture-1 = 熔岩队旗帜
place-0xae-furniture-2 = 熔岩队旗帜
# story 417:4, "It's the Team Aqua flag!"
place-0x60-furniture-4 = 海洋队旗帜
place-0x60-furniture-5 = 海洋队旗帜
place-0x61-furniture-2 = 海洋队旗帜
# story 65:25, "It's an Azurill Doll!", where the doll stands
place-0xdf-furniture-0 = 露力丽玩偶
place-0xe1-furniture-0 = 露力丽玩偶
# in the moving truck
place-0x18f-furniture-0 = 露力丽玩偶
# The rocks that evolve Eevee, which no text names: story 378:0, in Shoal Cave's ice room
place-0x86-furniture-0 = 被冰覆盖的岩石
# story 432:45, in Petalburg Woods
place-0x52-furniture-2 = 被青苔覆盖的岩石
