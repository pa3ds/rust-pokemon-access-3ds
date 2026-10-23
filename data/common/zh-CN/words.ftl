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
steps = { $way }{ $count }
leg-with = { $how } { $way }{ $steps }
here = 这里
rail-forward = 向前{ $steps }
rail-back = 向后{ $steps }
rail-hold-right = 按住右
rail-hold-left = 按住左
rail-angled = 镜头倾斜，按住左，按住右

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
category-currents = 海流
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
key-one-way = 只有一条路
out-at = 出口在{ $bearing }
leads-to = 通往{ $place }的{ $thing }
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
how-push = 推
how-ride-the-current = 顺着海流
how-get-off = 下来
how-smash = 击碎
p-current = 顺着海流{ $bearing }
p-to-push = 要向{ $way }推：
p-nudge = 稍微向{ $way }
p-nudge-then = { $nudge }，然后
p-lined-up = 已对齐
p-then-cant-continue = 然后无法继续
p-cant-continue = 无法继续
p-in-the-way = { $thing }挡路
p-surface = 浮上水面，前往{ $bearing }的潜水点
p-and-ice-cleared = 冰面全部碎裂
p-ice-cleared = 冰面已碎
p-no-path-to = 没有通往{ $thing }的路线
p-no-route-in-game = 这个游戏还没有路线
p-no-route-on-streets = 这些街道上没有路线
p-gone = { $thing }不见了

## Touch mode

touch-nothing = 没有可触摸的
touch-off = 触摸已关闭
touch-not-listening = Azahar不接受pa3ds的触摸

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
training-shot-coming = 来球了
training-left = 剩余{ $time }
amie-stroke = 抚摸{ $pokemon }

## What pa3ds says of itself

app-waiting = 等待游戏
app-ready = pa3ds已就绪
app-restarted = pa3ds已重启
app-reader-error = 读取错误
dialog-ok = 确定
dialog-cancel = 取消

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
kind-warp = 传送点
kind-path = 小路
kind-edge = 边缘
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
kind-barrier = 屏障
kind-hole = 洞
kind-dive-spot = 潜水点
kind-current = 海流
kind-flat-ground = 平地
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

## Scenes

narration = 旁白
