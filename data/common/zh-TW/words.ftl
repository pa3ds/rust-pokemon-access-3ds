### Traditional Chinese machine translation of `../en/words.ftl` (2026-10-18), a baseline for a
### translator to correct. The English file says what each message is said of, and where; a
### message missing here is said in English.

## Ways and distances

way-north = 北
way-east = 東
way-south = 南
way-west = 西
way-north-east = 東北
way-south-east = 東南
way-south-west = 西南
way-north-west = 西北
way-ahead = 前
way-behind = 後
way-left = 左
way-right = 右
steps = { $way }{ $count }
leg-with = { $how } { $way }{ $steps }
here = 這裡
rail-forward = 向前{ $steps }
rail-back = 向後{ $steps }
rail-hold-right = 按住右
rail-hold-left = 按住左
rail-angled = 鏡頭傾斜，按住左，按住右

## The list of what is on the map, and the navigation keys' answers

category-all = 全部
category-exits = 出口
category-signs = 可互動物件
category-obstacles = 障礙物
category-people = 人物
category-items = 道具
category-berries = 樹果
category-ash = 火山灰
category-bike = 自行車
category-riding = 騎乘
category-currents = 海流
category-pokemon = 寶可夢
key-category = { $category }，{ $count }
position = 第{ $number }，共{ $count }
key-nothing-way = { $way }邊什麼都沒有
key-none-way = { $way }邊沒有{ $category }
key-nothing-here = 這裡什麼都沒有
key-no-map = 沒有地圖
key-cant-walk-now = 現在無法行走
key-list-mode = 清單模式
key-direction-mode = 方向模式
key-touch-mode = 觸控模式，{ $first }
key-unreachable-hidden = 已隱藏無法到達的
key-unreachable-shown = 已顯示無法到達的
key-path-off = 路線關閉
key-no-path = 沒有路線
key-no-battle = 不在對戰中
key-one-way = 只有一條路
out-at = 出口在{ $bearing }
leads-to = 通往{ $place }的{ $thing }
span = { $low }到{ $high }
either = { $these }或{ $last }

## Big boulders

key-push = 向{ $way }推
boulder-into-hole = 向{ $way }推進洞裡
boulder-moveable = 可向{ $ways }移動
boulder-immoveable = 無法移動
boulder-moved = 已移動{ $bearing }

## Renaming

rename-title = 重新命名
rename-label = { $thing }，{ $bearing }。{ $name }
rename-speaker-label = 最後說話的人。{ $name }
rename-called = 名字：{ $name }
rename-not-named = 尚未命名
rename-only-this-one = 僅限這一個
rename-now = { $thing }現在叫{ $name }
rename-no-name = { $thing }沒有名字
rename-the-last-speaker = 最後說話的人
rename-nothing = 沒有可重新命名的
rename-cannot = { $thing }無法重新命名
rename-already = 已在重新命名
rename-nobody-spoke = 還沒有人說話

## P: the way to the thing chosen, and guiding the player along it

how-water = 水面
how-bush = 草叢
how-boulder = 巨石
how-rock = 岩石
how-waterfall = 瀑布
how-slope = 斜坡
how-bumpy-slope = 崎嶇斜坡
how-push = 推
how-ride-the-current = 順著海流
how-get-off = 下來
how-smash = 擊碎
p-current = 順著海流{ $bearing }
p-to-push = 要向{ $way }推：
p-nudge = 稍微向{ $way }
p-nudge-then = { $nudge }，然後
p-lined-up = 已對齊
p-then-cant-continue = 然後無法繼續
p-cant-continue = 無法繼續
p-in-the-way = { $thing }擋路
p-surface = 浮上水面，前往{ $bearing }的潛水點
p-and-ice-cleared = 冰面全部碎裂
p-ice-cleared = 冰面已碎
p-no-path-to = 沒有通往{ $thing }的路線
p-no-route-in-game = 這個遊戲還沒有路線
p-no-route-on-streets = 這些街道上沒有路線
p-gone = { $thing }不見了

## Touch mode

touch-nothing = 沒有可觸碰的
touch-off = 觸控已關閉
touch-not-listening = Azahar不接受pa3ds的觸控

## What the field shows without words

dowsing = 探測器{ $light }
dowsing-blue = 藍
dowsing-green = 綠
dowsing-yellow = 黃
dowsing-red = 紅
dowsing-nothing = 無反應
decorating-placeable = 可放置
decorating-not-placeable = 不可放置

## The Pokédex's habitat map

habitat-found = 出現於{ $place }
habitat-grass = 草叢
habitat-water = 水面
habitat-fishing = 釣魚

## Super Training and Pokémon-Amie

training-goal = 球門
training-red-goal = 紅色球門
training-shot-coming = 來球了
training-left = 剩餘{ $time }
amie-stroke = 撫摸{ $pokemon }

## What pa3ds says of itself

app-waiting = 等待遊戲
app-ready = pa3ds已就緒
app-restarted = pa3ds已重新啟動
app-reader-error = 讀取錯誤
dialog-ok = 確定
dialog-cancel = 取消

## As pa3ds starts: its keys, its settings and Azahar's

both = { $these }和{ $last }
keys-taken = pa3ds佔用{ $controls }
keys-applied = 按鍵已套用到Azahar
keys-waiting = 關閉Azahar並重新啟動pa3ds以套用你的按鍵
keys-commented-out = { $file }中的新按鍵已註解掉：{ $action }的{ $key }，因為{ $key }已用於{ $taken }
keys-edit-to-choose = 編輯{ $file }來選擇
file-unreadable = 無法讀取{ $file }：{ $problem }。請修正它，或刪除它讓pa3ds寫一個新的
azahar-settings-applied = Azahar設定已套用
azahar-settings-waiting = 關閉Azahar並重新啟動pa3ds以套用其設定
azahar-settings-missing = 找不到Azahar的設定：先啟動一次Azahar，關閉它，再重新啟動pa3ds
azahar-folder-ask = 找不到Azahar的設定。請選擇azahar.exe所在的資料夾；如果Azahar是安裝版，請取消
azahar-folder-again = { $problem }，請選擇其他資料夾或取消
azahar-folder-no-exe = { $folder }中沒有azahar.exe
azahar-folder-not-portable = { $folder }的azahar.exe旁邊沒有user資料夾，所以這個Azahar把設定儲存在AppData而不是旁邊

## What a thing on the map is, where neither the game nor a name says

kind-door = 門
kind-way-out = 出口
kind-warp = 傳送點
kind-path = 小路
kind-edge = 邊緣
kind-stairs = 樓梯
kind-stairs-up = 上樓梯
kind-stairs-down = 下樓梯
kind-passage = 通道
kind-entrance = 入口
kind-teleporter = 傳送裝置
kind-elevator = 電梯
kind-sign = 告示牌
kind-flag = 旗子
kind-object = 物體
kind-person = 人
kind-item-ball = 掉落的道具
kind-barrier = 屏障
kind-hole = 洞
kind-dive-spot = 潛水點
kind-current = 海流
kind-flat-ground = 平地
kind-slope = 斜坡
kind-bumpy-slope = 崎嶇斜坡
kind-rail = 軌道
kind-ice = 冰面
kind-blank = 空白
kind-wall = 牆
kind-crumbled-cliff = 崩塌的懸崖

## Battle

battle-battler = { $name } 等級{ $level } { $hp }
battle-share = { $percent }%

## Scenes

narration = 旁白
