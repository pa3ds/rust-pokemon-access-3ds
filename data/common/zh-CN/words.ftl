### Simplified Chinese machine translation of `../en/words.ftl` (2026-10-18), a baseline for a
### translator to correct. The English file says what each message is said of, and where; a
### message missing here is said in English.

## Ways and distances

way-north = 北
way-east = 东
way-south = 南
way-west = 西
way-north-east = 东北
way-south-east = 东南
way-south-west = 西南
way-north-west = 西北
way-ahead = 前
way-behind = 后
way-left = 左
way-right = 右
way-up = 上
way-down = 下
steps = { $way }{ $count }
leg-with = { $how } { $way }{ $steps }
here = 这里
above = 上方{ $count }
below = 下方{ $count }
rail-forward = 向前{ $steps }
rail-back = 向后{ $steps }
rail-hold-right = 按住右
rail-hold-left = 按住左
rail-angled = 镜头倾斜，按住左，按住右
turn-right = 向右转
turn-left = 向左转
turn-around = 向后转

## The list of what is on the map, and the navigation keys' answers

category-all = 全部
category-exits = 出口
category-signs = 可互动物件
category-obstacles = 障碍物
category-people = 人物
category-items = 道具
category-berries = 树果
category-ash = 火山灰
category-bike = 自行车
category-riding = 骑乘
category-climbing = 攀爬
category-currents = 海流
category-arrows = 箭头地板
category-pokemon = 宝可梦
key-category = { $category }，{ $count }
position = 第{ $number }，共{ $count }
key-nothing-way = { $way }边什么都没有
key-none-way = { $way }边没有{ $category }
key-nothing-here = 这里什么都没有
key-no-map = 没有地图
key-cant-walk-now = 现在无法行走
key-list-mode = 列表模式
key-direction-mode = 方向模式
key-touch-mode = 触摸模式，{ $first }
key-unreachable-hidden = 已隐藏无法到达的
key-unreachable-shown = 已显示无法到达的
key-path-off = 路线关闭
key-no-path = 没有路线
key-no-battle = 不在对战中
key-battle-not-shown = 还没有显示什么
key-nothing-said = 还没有说过什么
key-one-way = 只有一条路
out-at = 出口在{ $bearing }
past-teleporter = 传送装置另一边
leads-to = 通往{ $place }的{ $thing }
sign-saying = { $text }告示牌
span = { $low }到{ $high }
either = { $these }或{ $last }

## Big boulders

key-push = 向{ $way }推
boulder-into-hole = 向{ $way }推进洞里
boulder-moveable = 可向{ $ways }移动
boulder-immoveable = 无法移动
boulder-moved = 已移动{ $bearing }

## Renaming

rename-title = 重命名
rename-label = { $thing }，{ $bearing }。{ $name }
rename-speaker-label = 最后说话的人。{ $name }
rename-called = 名字：{ $name }
rename-not-named = 尚未命名
rename-only-this-one = 仅限这一个
rename-now = { $thing }现在叫{ $name }
rename-no-name = { $thing }没有名字
rename-the-last-speaker = 最后说话的人
rename-nothing = 没有可重命名的
rename-cannot = { $thing }无法重命名
rename-already = 已在重命名
rename-nobody-spoke = 还没有人说话

## P: the way to the thing chosen, and guiding the player along it

how-water = 水面
how-bush = 草丛
how-boulder = 巨石
how-rock = 岩石
how-waterfall = 瀑布
how-slope = 斜坡
how-bumpy-slope = 崎岖斜坡
how-rail = 轨道
how-hop = 跳
how-bunny-hop = 兔子跳
how-push = 推
how-ride-the-current = 顺着海流
how-get-off = 下来
how-smash = 击碎
how-climb = 攀爬
p-climb-up = 向上攀爬{ $holds }
p-climb-down = 向下攀爬{ $holds }
how-slide = 滑下
how-grind-the-rail = 滑轨道
p-grind = 顺着轨道滑{ $bearing }
how-ride-the-belt = 顺着传送带
p-belt = 顺着传送带{ $bearing }
how-arrow-pushes = 被箭头推动
p-arrow = 被箭头推动{ $bearing }
p-switch-first = 先去{ $thing }，{ $way }
how-climb-the-vine-up = 顺着藤蔓爬上去
how-climb-the-vine-down = 顺着藤蔓爬下去
how-swing-on-the-vine = 抓住藤蔓荡过去
p-vine = 面向{ $way }，{ $how }
p-more-vines = { $count ->
   *[other] 然后还有{ $count }根藤蔓
}
p-more-teleporters = { $count ->
   *[other] 然后还有{ $count }个传送装置
}
how-up-the-ramp = 上坡道
how-down-the-ramp = 下坡道
p-current = 顺着海流{ $bearing }
p-to-push = 要向{ $way }推：
p-nudge = 稍微向{ $way }
p-nudge-then = { $nudge }，然后
p-lined-up = 已对齐
p-then-cant-continue = 然后无法继续
p-cant-continue = 无法继续
p-in-the-way = { $thing }挡路
p-requires = 需要{ $move }
p-surface = 浮上水面，前往{ $bearing }的潜水点
p-into = { $leg }，进入{ $place }
p-and-ice-cleared = 冰面全部碎裂
p-ice-cleared = 冰面已碎
p-no-path-to = 没有通往{ $thing }的路线
p-no-route-in-game = 这个游戏还没有路线
p-no-route-on-streets = 这些街道上没有路线
p-gone = { $thing }不见了

## Touch mode

touch-nothing = 没有可触摸的
touch-no-drag = 没有可拖动的
touch-continue = 继续
touch-off = 触摸已关闭
touch-not-listening = Azahar不接受pa3ds的触摸
touch-holding = 按住{ $target }
touch-dropped = 在{ $target }上松开

## What the field shows without words

dowsing = 探测器{ $light }
dowsing-blue = 蓝
dowsing-green = 绿
dowsing-yellow = 黄
dowsing-red = 红
dowsing-nothing = 无反应
decorating-placeable = 可放置
decorating-not-placeable = 不可放置

## The Pokédex's habitat map

habitat-found = 出现于{ $place }
habitat-grass = 草丛
habitat-water = 水面
habitat-fishing = 钓鱼

## Super Training and Pokémon-Amie

training-goal = 球门
training-red-goal = 红色球门
training-barrier = 屏障
training-shot-coming = 来球了
training-left = 剩余{ $time }
training-empty-bag = 空沙袋
training-bag-finished = { $pokemon } 完成了基础训练
training-resting = { $pokemon } 正在休息
training-total = 总计
training-total-points = 总计 { $points }/{ $full }
training-charged = 蓄力完成
training-fizzled = 消失了
training-blocked = 挡住了
training-at-start = 起点
training-block-now = 现在
training-energy = 能量
training-no-numbers = 暂无数值
training-medals = { $count } 枚奖牌
amie-beckon = 招呼{ $pokemon }
amie-visitor = 来访的{ $pokemon }
amie-stylus = 触控笔
amie-stylus-on = 触控笔，{ $part }
amie-stylus-edge = 边缘
amie-view-down = 让{ $pokemon }下来
amie-view-up = 让{ $pokemon }回到上面
amie-view-came-down = { $pokemon }下来了
amie-view-went-up = { $pokemon }回到上面了
amie-mood-coming = { $pokemon }向你走来
amie-mood-back-turned = { $pokemon }背对着你
amie-mood-turned-round = { $pokemon }转过身来了
amie-mood-dozing = { $pokemon }在打瞌睡
amie-mood-asleep = { $pokemon }睡着了
amie-mood-awake = { $pokemon }醒了
amie-mood-too-full = { $pokemon }太饱了，吃不下
amie-mood-puff-seen = { $pokemon }盯着{ $puff }
amie-mood-reaching = { $pokemon }伸手去够{ $puff }
amie-mood-eating = { $pokemon }正在吃
amie-mood-eaten = { $pokemon }吃完了{ $puff }
amie-mood-puff-fell = { $puff }掉下去了
amie-mood-picky = { $pokemon }皱起眉头转过身去
amie-mood-watching = { $pokemon }正看着你的手
amie-mood-looking-around = { $pokemon }在东张西望
amie-mood-hand-raised = { $pokemon }举起了手
amie-mood-your-face = { $pokemon }凑近了你的脸
amie-high-five = 击掌！
amie-mood-disliked = { $pokemon }不喜欢这样……
amie-mood-closer = { $pokemon }欣喜若狂，爱心涌了出来
amie-mood-best-friends = { $pokemon }在爱心雨中开心得手舞足蹈
amie-meter = { $label } { $count }，共5个
amie-hearts = { $count ->
    [one] 1颗心
   *[other] { $count }颗心
}
amie-part-off = { $pokemon }外面
amie-part-head = 头
amie-part-body = 身体
amie-part-right-eye = 右眼
amie-part-left-eye = 左眼
amie-part-eye = 眼睛
amie-part-right-ear = 右耳
amie-part-left-ear = 左耳
amie-part-hand = 手
amie-part-spikes = 尖刺
amie-part-ghostly = 幽灵
amie-part-flame = 火焰
amie-part-sparks = 电火花
amie-part-slime = 黏液
amie-part-sludge = 污泥
amie-part-ice = 冰
amie-part = 部位
puzzle-cell = { $column }{ $row }
puzzle-home = { $tile }，已归位
puzzle-goes = { $tile }，应放在{ $home }
puzzle-picture-left = { $pokemon }在左边
puzzle-picture-front = { $pokemon }在前面
puzzle-picture-right = { $pokemon }在右边
puzzle-picked = { $tile }，已选中
berry-picker-wants = { $pokemon }想要{ $berry }
berry-picker-for = { $berry }给{ $pokemon }
berry-picker-wanted = 想要{ $berry }
berry-picker-lineup = { $first }、{ $second }和{ $third }
berry-picker-not-up = { $pokemon }没有出来
berry-picker-no-berry = 没有树果{ $number }
berry-picker-no-pokemon = 没有宝可梦{ $number }
head-it-ball = { $pokemon }
head-it-ball-nobody = 毛线球
head-it-no-pokemon = 没有宝可梦{ $number }
head-it-no-score = 没有得分
head-it-lineup = { $first }、{ $second }和{ $third }
head-it-estimate = pa3ds 估计你的提示提前量为 { $seconds } 秒，基于 { $taps } 次点击

## What pa3ds says of itself

app-waiting = 等待游戏
app-ready = pa3ds已就绪
app-restarted = pa3ds已重启
app-reader-error = 读取错误
dialog-ok = 确定
dialog-cancel = 取消
window-title = Pokémon Access 3DS
window-menu-mod = 模组(&M)
window-updates = 检查更新(&U)
window-exit = 退出(&X)
window-menu-azahar = Azahar(&A)
window-azahar-portable = 下载Azahar便携版(&P)
window-azahar-installer = 下载Azahar安装版(&I)
window-menu-help = 帮助(&H)
window-documentation = 文档(&D)...
window-keys = 按键命令(&K)...
window-amie = 宝可拍乐乐(&A)...
window-known-issues = 已知问题(&I)...
window-translating = 翻译(&T)...
window-changelog = 更新日志(&C)...
window-settings-guide = 设置指南(&S)...
window-issues = 在GitHub上报告问题(&R)
setting-pa3ds-keys = 响应模组按键
setting-pa3ds-keys-about = 响应pa3ds的按键，并按keys.toml的设定把Azahar的按钮绑定到键盘按键上。
setting-pa3ds-poll_rate = 每秒读取次数
setting-pa3ds-poll_rate-about = 每秒读取游戏多少次：60与游戏本身的帧率相同；电脑较慢时用30。
setting-pa3ds-azahar_folder = 便携版Azahar文件夹
setting-pa3ds-azahar_folder-about = 便携版Azahar所在的文件夹，也就是包含azahar.exe和user文件夹的那个；以常规方式安装的Azahar把设置保存在AppData中，这里留空即可。pa3ds询问文件夹时，你选好后它会自动填入。
setting-pa3ds-hide_window = Azahar打开时隐藏模组窗口
setting-pa3ds-hide_window-about = Azahar打开时，隐藏pa3ds的窗口，也就是双击pa3ds时打开的那个，这样在Alt+Tab中，你和游戏之间不会夹着pa3ds的任何窗口；设为false则保持窗口可见，以便OBS通过它录制pa3ds的声音。
setting-pa3ds-ui-pauses = 屏幕文字中的停顿
setting-pa3ds-ui-pauses-about = 一起朗读的几段屏幕文字之间加入停顿：标签后加冒号，如“初训家：Brendan”，其余之间加逗号。
setting-pa3ds-ui-interrupt = 菜单打断语音
setting-pa3ds-ui-interrupt-about = 菜单光标移动时打断正在朗读的语音，而不是排队等候；pa3ds自己的按键总是会打断。
setting-pa3ds-ui-unfocused = Azahar在后台时也朗读
setting-pa3ds-ui-unfocused-about = 其他窗口在前台时，继续朗读游戏显示的内容；否则此时pa3ds保持安静，只朗读它自己的按键和消息。
setting-pa3ds-ui-descriptions = 场景描述
setting-pa3ds-ui-descriptions-about = 描述游戏中没有文字的场景，比如固拉多苏醒。每段描述在场景之前的那行文字之后朗读。目前只描述了部分场景。
setting-pa3ds-ui-trainer_party = 训练家有几只宝可梦
setting-pa3ds-ui-trainer_party-about = 与训练家的对战开始时，在发起挑战的那行文字之后，朗读训练家有几只宝可梦（"text"），每只播放一声哔声（"beeps"），或都不做（"off"）。哔声与[pa3ds.audio]下的提示音一样响。
setting-pa3ds-ui-trainer_party-text = 朗读
setting-pa3ds-ui-trainer_party-beeps = 每只一声哔
setting-pa3ds-ui-trainer_party-off = 都不做
setting-pa3ds-ui-language = 语言
setting-pa3ds-ui-language-about = pa3ds朗读它自己的文字时使用的语言，填写它的一个数据文件夹的名称：en、fr、de、es、it、ja、ko、zh-CN或zh-TW。留空则跟随游戏本身的语言，pa3ds从存档中读取它；读取结果不对时需要设置，比如在另一种语言版本上打了民间翻译补丁时；在《太阳／月亮》中，pa3ds无法读取存档，这里设置的也就是游戏的语言。
setting-pa3ds-audio-tones = 提示音相对音量
setting-pa3ds-audio-tones-about = 地图光标或触摸模式移动时短提示音的音量；光标越往上音调越高，越往右声音越偏右，以此表示光标的位置：1与游戏一样响，0.5为一半，1.5更响，0为无声；在设置对话框中以百分比表示，100与游戏一样响。
setting-pa3ds-audio-footsteps = 脚步声相对音量
setting-pa3ds-audio-footsteps-about = 在游戏不播放脚步声的地面上，比如街道、地板和洞窟，pa3ds自己的脚步声有多响：1与游戏播放时一样响，0.5为一半，1.5更响，0为无声；在设置对话框中以百分比表示，100与游戏一样响。
setting-pa3ds-audio-ticks = 时钟滴答声相对音量
setting-pa3ds-audio-ticks-about = 游戏计时器每过一秒时短提示音的音量，比如在宝可拍乐乐的小游戏中；剩余时间本身每半分钟和最后几秒会朗读出来：1与游戏一样响，0.5为一半，1.5更响，0为无声，也是默认值；在设置对话框中以百分比表示，100与游戏一样响。
setting-pa3ds-audio-feedback_sounds = 反馈音效相对音量
setting-pa3ds-audio-feedback_sounds-about = 游戏自己用来表示按键结果的短音效有多响：电脑盒子里拿起和放下宝可梦的音效，用于触摸模式拿起和松开拖动；还有游戏的错误音效，用于无事可做的按键，比如采摘坚果这一轮没有的树果；以及瞄准头球的提示。设为0时，不播放提示，pa3ds改为说出来：“按住”“在……上松开”以及原因：1与游戏一样响，0.5为一半，1.5更响；在设置对话框中以百分比表示，100与游戏一样响。
setting-pa3ds-navigation-positions = 朗读列表位置
setting-pa3ds-navigation-positions-about = 说出一样东西在列表中的位置，如“门，第1，共3”。
setting-pa3ds-navigation-position_first = 列表位置在方向之前
setting-pa3ds-navigation-position_first-about = 先说列表中的位置，再说怎么走，如“门，第1，共3，北1”；设为false则说“门，北1，第1，共3”。
setting-pa3ds-navigation-turn_by_turn = 逐段导航
setting-pa3ds-navigation-turn_by_turn-about = P找到路线后，每到一个转弯处就说出该怎么走；设为false则只说一次路线，再按P会从你所在的位置重新说。
setting-pa3ds-navigation-guide_to_hidden = 引导至图鉴导航发现的宝可梦
setting-pa3ds-navigation-guide_to_hidden-about = 当图鉴导航搜索找到了要找的宝可梦，或者图鉴导航打开、你处于触摸模式时它自己发现了一只，就像按P一样找到通往它的路线，并让P一直指向它，无论列表原先在哪一项；区域导航打开时发现的则不处理。
setting-pa3ds-navigation-track_boulders = 追踪巨石移动
setting-pa3ds-navigation-track_boulders-about = 说出一块巨石离它在房间里的起始位置移动了多远，如“巨石，东2，已移动北1”，这样你就知道推过了哪一块；游戏会显示每块巨石在哪里，但从不显示它原来在哪里。
setting-pa3ds-navigation-hide_door_signs = 隐藏门牌
setting-pa3ds-navigation-hide_door_signs-about = 如果一块告示牌写的只是旁边那扇门的名字，比如紫堇公寓的“13号房 铁旋的家”，就不列入列表；写了更多内容的告示牌，比如道馆的，仍会保留。
setting-pa3ds-navigation-show_y = 朗读高度
setting-pa3ds-navigation-show_y-about = 按C说出你所在的位置、按Shift加C说出所选东西的位置时，也说出高度，即x和z之后的y：“x 71，z 35，y 20”；使用游戏自己的单位，一格宽为18，pa3ds不知道高度时省略，比如在《太阳／月亮》中。
setting-pa3ds-navigation-sort_by_distance = 按距离排序
setting-pa3ds-navigation-sort_by_distance-about = 列表模式的列表按由近到远排列，会随着你走动而变化；设为false则每样东西在列表中的位置固定，按游戏自己数据表的顺序排列，这样你可以根据位置分辨出进过哪栋房子；方向模式无论如何都按由近到远排列。
setting-pa3ds-touch-wrap = 循环移动
setting-pa3ds-touch-wrap-about = 在触摸模式中，从最后一个按钮回到第一个，从最底下一行回到最上面一行；设为false则停在两端，这样按几次O总会停在最底下一行。
setting-pa3ds-touch-next_row = 继续到下一行
setting-pa3ds-touch-next_row-about = 在触摸模式中，J和L到达一行的尽头时继续进入上一行或下一行；设为false则让它们留在本行，这样按几次L总会停在这一行的最右边。
setting-pa3ds-touch-drag = 拖动方式
setting-pa3ds-touch-drag-about = 按住Shift拖动，或者用你在keys.toml中绑定为拖动的按键，松开按键时触摸也随之松开（"hold"）；或者按一下拿起，再按一下松开（"toggle"）。目前只有宝可拍乐乐需要拖动。
setting-pa3ds-touch-drag-hold = 按住Shift
setting-pa3ds-touch-drag-toggle = 按一下拿起，再按一下松开
setting-pa3ds-amie-descriptions = 描述宝可梦在做什么
setting-pa3ds-amie-descriptions-about = 在宝可拍乐乐中，在宝可梦开始和停止动作时说出能看到的动作：东张西望、看着你的手、吃东西、举手等着击掌。它的每个动作也会在播放时用它自己的一句话描述。也会说出抚摸带来的心，以及击掌成功。
setting-pa3ds-amie-fidgets = 描述宝可梦的小动作
setting-pa3ds-amie-fidgets-about = 开启宝可拍乐乐的描述时，也说出宝可梦每隔几秒自己做的小动作。关闭时只说它对你的反应。
setting-pa3ds-amie-berry_first = 采摘坚果先说树果
setting-pa3ds-amie-berry_first-about = 按你给树果的顺序说出每只想要树果的宝可梦，先说树果再说宝可梦，如“凰梨果给皮卡丘”；设为false则先说宝可梦，如“皮卡丘想要凰梨果”。
setting-pa3ds-amie-one_at_a_time = 采摘坚果一次说一只宝可梦
setting-pa3ds-amie-one_at_a_time-about = 一次只说一只想要树果的宝可梦，等你给了树果，或者说到的那只离开后，再说下一只；设为false则每只一出现就说。
setting-pa3ds-amie-head_it_cue = 瞄准头球提示何时点击
setting-pa3ds-amie-head_it_cue-about = 在瞄准头球中，到了该点击毛线球的时刻时，播放游戏自己的“！”音效：在毛线球到达宝可梦之前，提前下面设定的时间响起，从那只宝可梦那一侧的耳朵传来，中间的宝可梦则两只耳朵都有；需要开启反馈音效。
setting-pa3ds-amie-head_it_lead = 瞄准头球的提示提前量
setting-pa3ds-amie-head_it_lead-about = 提示在毛线球到达宝可梦之前多少秒响起：大约是你从听到提示到按下那只宝可梦的键所需的时间。每轮结果时，pa3ds会说出它对你的估计值，可以填在这里。点击太晚就调高，太早就调低。
setting-pa3ds-amie-berry_picker_style = 采摘坚果中如何区分宝可梦
setting-pa3ds-amie-berry_picker_style-about = 在采摘坚果中，用名字（"name"）、宝可梦自己的叫声（"cry"）或两者（"both"）来说出宝可梦。不论它从哪里冒出来，叫声都从它在本轮出场顺序中的位置传来：第一只在左耳，第二只两只耳朵都有，第三只在右耳。叫声需要开启反馈音效；无法播放时会说出名字。
setting-pa3ds-amie-berry_picker_style-name = 名字
setting-pa3ds-amie-berry_picker_style-cry = 叫声
setting-pa3ds-amie-berry_picker_style-both = 名字和叫声
setting-pa3ds-amie-head_it_style = 瞄准头球中如何区分宝可梦
setting-pa3ds-amie-head_it_style-about = 在瞄准头球中，用名字（"name"）、宝可梦自己的叫声（"cry"）、两者（"both"）来说出毛线球落向的宝可梦，或两者都不用（"none"），只听提示。叫声从它在本轮出场顺序中的位置传来：第一只在左耳，第二只两只耳朵都有，第三只在右耳。叫声需要开启反馈音效；无法播放时会说出名字。
setting-pa3ds-amie-head_it_style-name = 名字
setting-pa3ds-amie-head_it_style-cry = 叫声
setting-pa3ds-amie-head_it_style-both = 名字和叫声
setting-pa3ds-amie-head_it_style-none = 都不用
setting-pa3ds-amie-head_it_score = 瞄准头球在得分变化时说出
setting-pa3ds-amie-head_it_score-about = 在瞄准头球中，每当得分、连击和失误出现时说出，得分和连击在持续上升时只说一次；false 则交给瞄准头球的得分键（R），每次按下时连同剩余时间或剩余机会一起说出。
setting-pa3ds-extras-conditions = 以数值朗读华丽大赛状态
setting-pa3ds-extras-conditions-about = 在概况和宝可方块盒中，游戏把宝可梦的华丽大赛状态画成星形图，pa3ds则以数值说出：星形图人人都能看到，但pa3ds读不出来，所以只能给出数值，而数值比星形图更精确。
setting-pa3ds-training-shot_cue = 气球机器人射击时
setting-pa3ds-training-shot_cue-about = 在超级特训中，如何得知气球机器人射击：游戏的警报声、“来球了”这句话，或无提示。关闭反馈音时用语音读出。
setting-pa3ds-training-shot_cue-sound = 音效
setting-pa3ds-training-shot_cue-speech = 语音
setting-pa3ds-training-shot_cue-off = 关闭
setting-pa3ds-training-block_cue = 格挡或闪避时机
setting-pa3ds-training-block_cue-about = 在超级特训中，如何得知射击即将到达：游戏的“!”音效、“现在”这个词，或无提示。关闭反馈音时用语音读出。
setting-pa3ds-training-block_cue-sound = 音效
setting-pa3ds-training-block_cue-speech = 语音
setting-pa3ds-training-block_cue-off = 关闭
setting-pa3ds-training-block_lead = 格挡提示提前量
setting-pa3ds-training-block_lead-about = 射击到达前多少秒发出提示。按下 L 后屏障持续不到一秒。
setting-pa3ds-extras-base_stats = 超级特训能力显示为数字
setting-pa3ds-extras-base_stats-about = 在超级特训页面按 R 时，读出游戏以图表绘制的各项能力点数，而不是大致比例。
setting-pa3ds-extras-hidden_items = 隐藏道具
setting-pa3ds-extras-hidden_items-about = 在道具类中列出尚未找到的隐藏道具，称为“隐藏道具”：可以只列海底的（"underwater"），那里不能使用探测器，而有视力的玩家可能会在岩石和海草间看到一点闪光；也可以列出所有地方的（"all"），这会说出游戏对所有人隐瞒的东西；"off"则一个都不列。
setting-pa3ds-extras-hidden_items-off = 关闭
setting-pa3ds-extras-hidden_items-underwater = 海底
setting-pa3ds-extras-hidden_items-all = 所有地方
settings-title = 模组设置
settings-page-general = 常规
settings-page-speech = 语音
settings-page-audio = 音频
settings-page-navigation = 导航
settings-page-touch = 触摸模式
settings-page-amie = 宝可拍乐乐
settings-page-training = 超级特训
settings-page-extras = 附加功能
settings-browse = 浏览(&B)...
settings-language-game = 跟随游戏语言
settings-restart = 现在重启pa3ds，让这些设置生效吗？如果选择“否”，它们会在pa3ds下次启动时生效。
settings-unwritten = pa3ds无法写入{ $path }，所以没有做任何更改。
dialog-apply = 应用(&A)
window-settings = 设置(&S)...

## As pa3ds starts: its keys, its settings and Azahar's

both = { $these }和{ $last }
keys-taken = pa3ds占用{ $controls }
keys-applied = 按键已应用到Azahar
keys-waiting = 关闭Azahar并重新启动pa3ds以应用你的按键
keys-commented-out = { $file }中的新按键已注释掉：{ $action }的{ $key }，因为{ $key }已用于{ $taken }
keys-edit-to-choose = 编辑{ $file }进行选择
file-unreadable = 无法读取{ $file }：{ $problem }。请修正它，或删除它让pa3ds写一个新的
azahar-settings-applied = Azahar设置已应用
azahar-settings-waiting = 关闭Azahar并重新启动pa3ds以应用其设置
azahar-settings-missing = 找不到Azahar的设置：先启动一次Azahar，关闭它，再重新启动pa3ds
azahar-folder-ask = 找不到Azahar的设置。请选择azahar.exe所在的文件夹；如果Azahar是安装版，请取消
azahar-folder-again = { $problem }，请选择其他文件夹或取消
azahar-folder-no-exe = { $folder }中没有azahar.exe
azahar-folder-not-portable = { $folder }的azahar.exe旁边没有user文件夹，所以这个Azahar把设置保存在AppData而不是旁边

## What a thing on the map is, where neither the game nor a name says

kind-door = 门
kind-way-out = 出口
kind-way-down = 下去的路
kind-conveyor-belt = 传送带
kind-switch = 开关
kind-pink-switch = 粉色开关
kind-blue-switch = 蓝色开关
kind-yellow-switch = 黄色开关
kind-green-switch = 绿色开关
kind-warp = 传送点
kind-path = 小路
kind-edge = 小路
kind-stairs = 楼梯
kind-stairs-up = 上楼梯
kind-stairs-down = 下楼梯
kind-passage = 通道
kind-entrance = 入口
kind-teleporter = 传送装置
kind-elevator = 电梯
kind-sign = 告示牌
kind-flag = 旗子
kind-object = 物体
kind-person = 人
kind-item-ball = 掉落的道具
kind-hidden-item = 隐藏道具
kind-barrier = 屏障
kind-hole = 洞
kind-dive-spot = 潜水点
kind-current = 海流
kind-water = 水面
kind-waterfall = 瀑布
kind-arrow-east = 向东箭头
kind-arrow-west = 向西箭头
kind-arrow-north = 向北箭头
kind-arrow-south = 向南箭头
kind-flat-ground = 平地
kind-open-ground = 空地
kind-quiz-panel = 答题面板
kind-blue-spot = 蓝色落脚点
kind-wall-ledge = 岩壁平台
kind-vine-up = 向上的藤蔓
kind-vine-down = 向下的藤蔓
kind-swinging-vine = 可荡的藤蔓
kind-gym-leader-spot = 道馆馆主的位置
kind-north-waiting-spot = 北侧等待点，靠近死路
kind-south-waiting-spot = 南侧等待点，靠近迷宫入口
kind-east-waiting-spot = 东侧等待点，中间通道的东端
kind-west-waiting-spot = 西侧等待点，中间通道的西端
furfrou-wait = 下一步，{ $shauna }在这里等
furfrou-fetch = 下一步，和{ $shauna }说话，让她跟着你
furfrou-chase = 下一步，走到这里，{ $furfrou }会逃跑
furfrou-keep-off = 不要踩，{ $furfrou }会从这里逃走
furfrou-leave = 下一步，从这里离开迷宫
furfrou-enter = 下一步，回到迷宫里
kind-machine = 装置
kind-slope = 斜坡
kind-bumpy-slope = 崎岖斜坡
kind-rail = 轨道
kind-ice = 冰面
kind-blank = 空白
kind-wall = 墙
kind-crumbled-cliff = 崩塌的悬崖

## Battle

battle-battler = { $name } 等级{ $level } { $hp }
battle-share = { $percent }%
battle-party = { $count }只宝可梦

## Scenes

narration = 旁白
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = 简体中文

registered-slot = { $way } { $item }
registered-up = 上
registered-right = 右
registered-down = 下
registered-left = 左

bag-moving = 正在移动 { $item }
bag-between = { $above } 和 { $below } 之间
bag-above = { $below } 上方
bag-below = { $above } 下方
