### What to call the people and things on Sun and Moon's maps that the ROM's own rules leave unnamed, in zh-CN.
###
### A key is the model a person is drawn as, model n file n of a/1/9/6 (findings, spike
### 291). A name is looked for in order: what the player has called them, then what the
### game itself calls them, out of the player's own ROM (`sm.gametext`), and only then
### this file. Each table names what the English one does, and each entry says where
### it came from: the ROM's own class and trainer names in this language, ORAS's table's
### word for townsfolk, the story text's, or pa3ds's own. Townsfolk are named by their
### look (their textures) and what they say, so their words are the least sure here.

## sprites
# The story's people, who battle never or as another model.
# Mom: model 0x21, tr0032, standing only in Route 1's zones, the player's house and its rooms, with her Meowth; the word X's table calls the player's mother in this language
sprite-0x21 = 妈妈
# Professor Kukui: model 0x5, tr0004, a lab coat, glasses and a cap, on Route 1 by his lab and at the Pokémon League, his battles drawn as 0x11b; "That's got to be Professor Kukui"
sprite-0x5 = 库库伊博士
# Lillie: model 0x2, tr0001, the white dress, the sun hat and braided blond hair; "Please, you have to help save Nebby!" on Mahalo Trail; "You can call me Lillie."
sprite-0x2 = 莉莉艾
# Lillie: model 0x3, tr0002, her later outfit, a white top, a ponytail and a pink skirt, on Poni and at the altars
sprite-0x3 = 莉莉艾
# The story's people the ROM names, whose look it cannot tell from a crowd's: the ROM's words
# Ilima: class 151, no trainers, every byte but its model Ilima's class 38's; pink hair, at his trial in Verdant Cavern and in Hau'oli
sprite-0x14 = 队长 伊利马
# Lana: class 152, no trainers, Lana's class 44's but its model; Brooklet Hill, her trial
sprite-0x16 = 队长 水莲
# Kiawe: class 43, Kiawe the one trainer; Wela Volcano Park, Paniola
sprite-0x15 = 队长 卡奇
# Mallow: class 45, Mallow the one trainer; Lush Jungle, her trial
sprite-0x17 = 队长 玛奥
# Sophocles: class 142, Sophocles the one trainer; Hokulani Observatory
sprite-0x18 = 队长 马玛内
# Mina: class 153, Mina the one trainer; Poni Gauntlet, Vast Poni Canyon
sprite-0x1a = 队长 茉莉
# Nanu: class 50, no trainers, Nanu's class 141's but its model; Po Town, Malie, the altars
sprite-0x1d = 岛屿之王 默丹
# Lusamine: class 71, Lusamine the one trainer; Aether Paradise, the altars
sprite-0x7 = 以太理事长 露莎米奈
# Guzma: class 140, Guzma the one trainer; Hau'oli, Route 2
sprite-0xf = 骷髅队老大 古兹马
# Plumeria: class 79, Plumeria the one trainer; Route 13
sprite-0x11 = 骷髅队干部 布尔美丽
# Ryuki: class 143, Ryuki the one trainer
sprite-0x20 = 龙葵
# Red: class 83, Red the one trainer; the Battle Tree
sprite-0x24 = 赤红
# Blue: class 84, Blue the one trainer; the Battle Tree
sprite-0x25 = 青绿
# Sina: class 166, Sina the one trainer; Heahea, Route 16
sprite-0x27 = 吉娜
# Anabel: class 171, Anabel the one trainer; Routes 2, 8 and 13, beside Looker
sprite-0x2a = 莉拉
# The Royal: class 92, The Royal the one trainer; the Battle Royal Dome
sprite-0x2e = 皇家蒙面人
# Looks many share: the ROM's word for the class
# Trial Guide: class 42, the Trial Guide Ben its one trainer, but standing at every island's trials
sprite-0x6b = 考验支援者
# Bellhop: class 55, the Bellhop Jody its one trainer, but in both hotels
sprite-0x60 = 行李员
# Firefighter: class 56, the Firefighter Alex its one trainer, but in Heahea, Malie and Route 13
sprite-0x61 = 消防员
# Worker: class 58's word: a uniform; "we've compacted the soil with a good Bulldoze", the roads City Hall keeps
sprite-0x3f = 工人
# Police Officer: class 52's word: a police uniform and a riding helmet; "I can patrol even the peak of a mountain"
sprite-0xbb = 警察先生
# Townsfolk, by their look and their words: ORAS's table's word
# Clerk: a blue uniform shirt; the shops', the mall's and the ferry terminals' staff
sprite-0x40 = 店员
# Receptionist: a dark suit and tie; "Welcome to the mall!", the hotel's, the Battle Royal Dome's
sprite-0x41 = 接待员
# Old man: white hair, a lined forehead; "Opening a window lets night come creeping in"
sprite-0x3d = 老爷爷
# Old man: grey hair and a moustache; Iki Town's talk of "Old Hala"
sprite-0x102 = 老爷爷
# Old woman: grey hair in a bun, a lined forehead; "Time to lend a hand, Meowth!"
sprite-0x3e = 老奶奶
# Woman: long brown hair, pink; "Our family is growing!"
sprite-0x36 = 女性
# Woman: long brown hair, red and pink; "Welcome to the Alola tourist bureau!"
sprite-0x37 = 女性
# Woman: brown hair, a pink cardigan; "But, honey, I heard you can fish up..."
sprite-0x38 = 女性
# Woman: "My son had his hair styled like a Pikipek", "that hubby of mine!"
sprite-0x3b = 女性
# Young woman: dark hair, a skirt; "It takes an adult to appreciate both the bitter and the sweet"
sprite-0x34 = 年轻女性
# Young man: a light blue shirt; "My little brother...", "We're going to get married!"
sprite-0x35 = 青年
# Young man: 0x35's clothes and a riding helmet, on Route 2
sprite-0x8a = 青年
# Young man: "Everyone who works on the ranch loves a good Pokémon battle!"
sprite-0x33 = 青年
# Man: a flowered shirt; "When I see her roughhousing with my little girl"
sprite-0x3a = 男性
# Man: 0x3a's clothes and a riding helmet, in Hau'oli Cemetery
sprite-0xc5 = 男性
# Man: a cap; "My house is overrun with kids and Pokémon"
sprite-0x39 = 男性
# Middle-aged man: "I'd love to perform my best song up on stage someday", "Welcome to Malie City!"
sprite-0x3c = 中年男性
# Man in sunglasses: sunglasses, a flowered shirt and a watch
sprite-0x23 = 戴墨镜的男人
# Little boy: a red cap; "Granny! Please! You've gotta buy me a Masked Royal mask!"
sprite-0x31 = 小男孩
# Little boy: 0x31's look; "I just got a Mewtwo through Wonder Trade!!!"
sprite-0x138 = 小男孩
# Little boy: spiky blond hair, sandals; "Wow! A Trainer!", "Pokémon are in space, too? Wow!"
sprite-0x30 = 小男孩
# Little boy: a cap; "I wanna get carried around in Pelipper's beak, too!"
sprite-0x32 = 小男孩
# Little boy: 0x32's look; "I really want your Psyduck! Come on, let's trade again!"
sprite-0x139 = 小男孩
# Little boy: 0x32's look and a riding helmet, in Heahea
sprite-0xb9 = 小男孩
# Others the ROM never names
# Wicke: purple hair; only at Aether Paradise; "My name is Wicke." (the story text's name in each language)
sprite-0xa = 碧珂
# Looker: a trench coat; Routes 2, 8 and 13, Seafolk Village, Aether Paradise, beside Anabel; "Mr. Looker?" (the story text's name in each language)
sprite-0x26 = 帅哥
# Café owner: a striped shirt and an apron, one in every Pokémon Center: "Welcome to the Pokémon Center Café!"; pa3ds's own words
sprite-0xb8 = 咖啡区老板
# car: tr1000, a red car, three in Iki Town; pa3ds's own words
sprite-0xc8 = 汽车
