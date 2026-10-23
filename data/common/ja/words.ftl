### Japanese machine translation of `../en/words.ftl` (2026-10-18), a baseline for a translator to
### correct. The English file says what each message is said of, and where; a message missing
### here is said in English.

## Ways and distances

way-north = 北
way-east = 東
way-south = 南
way-west = 西
way-north-east = 北東
way-south-east = 南東
way-south-west = 南西
way-north-west = 北西
way-ahead = 前
way-behind = 後ろ
way-left = 左
way-right = 右
steps = { $way }{ $count }
leg-with = { $how } { $way }{ $steps }
here = ここ
rail-forward = 前へ{ $steps }
rail-back = 後ろへ{ $steps }
rail-hold-right = 右を押し続ける
rail-hold-left = 左を押し続ける
rail-angled = カメラが斜め、左を押し続ける、右を押し続ける
turn-right = 右を向く
turn-left = 左を向く
turn-around = 後ろを向く

## The list of what is on the map, and the navigation keys' answers

category-all = すべて
category-exits = 出口
category-signs = 調べられるもの
category-obstacles = 障害物
category-people = 人
category-items = 道具
category-berries = きのみ
category-ash = 火山灰
category-bike = 自転車
category-riding = ライド
category-currents = 海流
category-pokemon = ポケモン
key-category = { $category }、{ $count }
position = { $count }中{ $number }
key-nothing-way = { $way }には何もない
key-none-way = { $way }に{ $category }なし
key-nothing-here = ここには何もない
key-no-map = マップなし
key-cant-walk-now = 今は歩けない
key-list-mode = リストモード
key-direction-mode = 方向モード
key-touch-mode = タッチモード、{ $first }
key-unreachable-hidden = 行けない場所を隠す
key-unreachable-shown = 行けない場所を表示
key-path-off = 経路オフ
key-no-path = 経路なし
key-no-battle = バトル中ではない
key-one-way = 行き方は一つだけ
out-at = 出る先は{ $bearing }
leads-to = { $place }への{ $thing }
span = { $low }から{ $high }
either = { $these }または{ $last }

## Big boulders

key-push = { $way }に押す
boulder-into-hole = { $way }の穴へ
boulder-moveable = { $ways }に動かせる
boulder-immoveable = 動かせない
boulder-moved = { $bearing }動いた

## Renaming

rename-title = 名前を変える
rename-label = { $thing }、{ $bearing }。{ $name }
rename-speaker-label = 最後に話した人。{ $name }
rename-called = 名前：{ $name }
rename-not-named = まだ名前がない
rename-only-this-one = これだけ
rename-now = { $thing }は{ $name }になった
rename-no-name = { $thing }には名前がない
rename-the-last-speaker = 最後に話した人
rename-nothing = 名前を変えるものがない
rename-cannot = { $thing }は名前を変えられない
rename-already = すでに名前を変えている
rename-nobody-spoke = まだ誰も話していない

## P: the way to the thing chosen, and guiding the player along it

how-water = 水
how-bush = 茂み
how-boulder = 大岩
how-rock = 岩
how-waterfall = 滝
how-slope = 坂
how-bumpy-slope = でこぼこの坂
how-push = 押す
how-ride-the-current = 海流に乗る
how-get-off = 降りる
how-smash = 砕く
p-current = 海流に乗って{ $bearing }
p-to-push = { $way }に押すには：
p-nudge = 少し{ $way }
p-nudge-then = { $nudge }、それから
p-lined-up = 位置が合った
p-then-cant-continue = その先は進めない
p-cant-continue = 進めない
p-in-the-way = { $thing }がじゃまをしている
p-surface = 浮上して{ $bearing }のダイビングポイントへ
p-and-ice-cleared = そして氷はすべて割れる
p-ice-cleared = 氷が割れた
p-no-path-to = { $thing }への経路なし
p-no-route-in-game = このゲームではまだ経路がない
p-no-route-on-streets = この通りでは経路がない
p-gone = { $thing }がいなくなった

## Touch mode

touch-nothing = タッチするものがない
touch-off = タッチはオフ
touch-not-listening = Azaharはpa3dsからのタッチを受け付けていない

## What the field shows without words

dowsing = ダウジングマシン { $light }
dowsing-blue = 青
dowsing-green = 緑
dowsing-yellow = 黄色
dowsing-red = 赤
dowsing-nothing = 反応なし
decorating-placeable = 置ける
decorating-not-placeable = 置けない

## The Pokédex's habitat map

habitat-found = { $place }で見つかる
habitat-grass = 草むら
habitat-water = 水上
habitat-fishing = 釣り

## Super Training and Pokémon-Amie

training-goal = ゴール
training-red-goal = 赤いゴール
training-shot-coming = シュートが来る
training-left = 残り{ $time }
amie-stroke = { $pokemon }をなでる

## What pa3ds says of itself

app-waiting = ゲームを待っています
app-ready = pa3ds 準備完了
app-restarted = pa3ds 再起動
app-reader-error = 読み取りエラー
dialog-ok = OK
dialog-cancel = キャンセル

## As pa3ds starts: its keys, its settings and Azahar's

both = { $these }と{ $last }
keys-taken = pa3dsは{ $controls }を使う
keys-applied = キーをAzaharに適用した
keys-waiting = キーを適用するには、Azaharを閉じてpa3dsを起動し直してください
keys-commented-out = { $file }の新しいキーをコメントアウトした：{ $action }の{ $key }。{ $key }はすでに{ $taken }に使われている
keys-edit-to-choose = 選ぶには{ $file }を編集してください
file-unreadable = { $file }を読めなかった：{ $problem }。直すか、削除すればpa3dsが新しく書きます
azahar-settings-applied = Azaharの設定を適用した
azahar-settings-waiting = 設定を適用するには、Azaharを閉じてpa3dsを起動し直してください
azahar-settings-missing = Azaharの設定が見つからない：Azaharを一度起動して閉じ、pa3dsを起動し直してください
azahar-folder-ask = Azaharの設定が見つかりません。azahar.exeのあるフォルダを選ぶか、Azaharがインストールされているならキャンセルしてください
azahar-folder-again = { $problem }。別のフォルダを選ぶかキャンセルしてください
azahar-folder-no-exe = { $folder }にazahar.exeがない
azahar-folder-not-portable = { $folder }のazahar.exeの横にuserフォルダがないので、このAzaharは設定を横ではなくAppDataに保存している

## What a thing on the map is, where neither the game nor a name says

kind-door = ドア
kind-way-out = 出口
kind-warp = ワープ
kind-path = 道
kind-edge = 端
kind-stairs = 階段
kind-stairs-up = 上り階段
kind-stairs-down = 下り階段
kind-passage = 通路
kind-entrance = 入口
kind-teleporter = ワープパネル
kind-elevator = エレベーター
kind-sign = 看板
kind-flag = 旗
kind-object = もの
kind-person = 人
kind-item-ball = 落ちている道具
kind-barrier = 柵
kind-hole = 穴
kind-dive-spot = ダイビングポイント
kind-current = 海流
kind-flat-ground = 平らな地面
kind-slope = 坂
kind-bumpy-slope = でこぼこの坂
kind-rail = レール
kind-ice = 氷
kind-blank = 空き
kind-wall = 壁
kind-crumbled-cliff = 崩れた崖

## Battle

battle-battler = { $name } レベル{ $level } { $hp }
battle-share = { $percent }%

## Scenes

narration = ナレーション
