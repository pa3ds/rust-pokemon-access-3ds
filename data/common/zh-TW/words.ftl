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
way-up = 上
way-down = 下
steps = { $way }{ $count }
leg-with = { $how } { $way }{ $steps }
here = 這裡
above = 上方{ $count }
below = 下方{ $count }
rail-forward = 向前{ $steps }
rail-back = 向後{ $steps }
rail-hold-right = 按住右
rail-hold-left = 按住左
rail-angled = 鏡頭傾斜，按住左，按住右
turn-right = 向右轉
turn-left = 向左轉
turn-around = 向後轉

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
category-climbing = 攀爬
category-currents = 海流
category-arrows = 箭頭地板
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
key-battle-not-shown = 還沒有顯示什麼
key-nothing-said = 還沒有說過什麼
key-one-way = 只有一條路
out-at = 出口在{ $bearing }
past-teleporter = 傳送裝置另一邊
leads-to = 通往{ $place }的{ $thing }
sign-saying = { $text }告示牌
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
how-rail = 軌道
how-hop = 跳
how-bunny-hop = 兔子跳
how-push = 推
how-ride-the-current = 順著海流
how-get-off = 下來
how-smash = 擊碎
how-climb = 攀爬
p-climb-up = 向上攀爬{ $holds }
p-climb-down = 向下攀爬{ $holds }
how-slide = 滑下
how-grind-the-rail = 滑軌道
p-grind = 順著軌道滑{ $bearing }
how-ride-the-belt = 順著輸送帶
p-belt = 順著輸送帶{ $bearing }
how-arrow-pushes = 被箭頭推動
p-arrow = 被箭頭推動{ $bearing }
p-switch-first = 先去{ $thing }，{ $way }
how-climb-the-vine-up = 順著藤蔓爬上去
how-climb-the-vine-down = 順著藤蔓爬下去
how-swing-on-the-vine = 抓住藤蔓盪過去
p-vine = 面向{ $way }，{ $how }
p-more-vines = { $count ->
   *[other] 然後還有{ $count }根藤蔓
}
p-more-teleporters = { $count ->
   *[other] 然後還有{ $count }個傳送裝置
}
how-up-the-ramp = 上坡道
how-down-the-ramp = 下坡道
p-current = 順著海流{ $bearing }
p-to-push = 要向{ $way }推：
p-nudge = 稍微向{ $way }
p-nudge-then = { $nudge }，然後
p-lined-up = 已對齊
p-then-cant-continue = 然後無法繼續
p-cant-continue = 無法繼續
p-in-the-way = { $thing }擋路
p-requires = 需要{ $move }
p-surface = 浮上水面，前往{ $bearing }的潛水點
p-into = { $leg }，進入{ $place }
p-and-ice-cleared = 冰面全部碎裂
p-ice-cleared = 冰面已碎
p-no-path-to = 沒有通往{ $thing }的路線
p-no-route-in-game = 這個遊戲還沒有路線
p-no-route-on-streets = 這些街道上沒有路線
p-gone = { $thing }不見了

## Touch mode

touch-nothing = 沒有可觸碰的
touch-no-drag = 沒有可拖曳的
touch-continue = 繼續
touch-off = 觸控已關閉
touch-not-listening = Azahar不接受pa3ds的觸控
touch-holding = 按住{ $target }
touch-dropped = 在{ $target }上放開

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
training-barrier = 屏障
training-shot-coming = 來球了
training-left = 剩餘{ $time }
training-empty-bag = 空沙袋
training-bag-finished = { $pokemon } 完成了基礎訓練
training-resting = { $pokemon } 正在休息
training-total = 總計
training-total-points = 總計 { $points }/{ $full }
training-charged = 蓄力完成
training-fizzled = 消失了
training-blocked = 擋住了
training-at-start = 起點
training-block-now = 現在
training-energy = 能量
training-no-numbers = 暫無數值
training-medals = { $count } 枚獎牌
amie-beckon = 招呼{ $pokemon }
amie-visitor = 來訪的{ $pokemon }
amie-stylus = 觸控筆
amie-stylus-on = 觸控筆，{ $part }
amie-stylus-edge = 邊緣
amie-view-down = 讓{ $pokemon }下來
amie-view-up = 讓{ $pokemon }回到上面
amie-view-came-down = { $pokemon }下來了
amie-view-went-up = { $pokemon }回到上面了
amie-mood-coming = { $pokemon }向你走來
amie-mood-back-turned = { $pokemon }背對著你
amie-mood-turned-round = { $pokemon }轉過身來了
amie-mood-dozing = { $pokemon }在打瞌睡
amie-mood-asleep = { $pokemon }睡著了
amie-mood-awake = { $pokemon }醒了
amie-mood-too-full = { $pokemon }太飽了，吃不下
amie-mood-puff-seen = { $pokemon }盯著{ $puff }
amie-mood-reaching = { $pokemon }伸手去搆{ $puff }
amie-mood-eating = { $pokemon }正在吃
amie-mood-eaten = { $pokemon }吃完了{ $puff }
amie-mood-puff-fell = { $puff }掉下去了
amie-mood-picky = { $pokemon }皺起眉頭轉過身去
amie-mood-watching = { $pokemon }正看著你的手
amie-mood-looking-around = { $pokemon }在東張西望
amie-mood-hand-raised = { $pokemon }舉起了手
amie-mood-your-face = { $pokemon }湊近了你的臉
amie-high-five = 擊掌！
amie-mood-disliked = { $pokemon }不喜歡這樣……
amie-mood-closer = { $pokemon }欣喜若狂，愛心湧了出來
amie-mood-best-friends = { $pokemon }在愛心雨中開心得手舞足蹈
amie-meter = { $label } { $count }，共5個
amie-hearts = { $count ->
    [one] 1顆心
   *[other] { $count }顆心
}
amie-part-off = { $pokemon }外面
amie-part-head = 頭
amie-part-body = 身體
amie-part-right-eye = 右眼
amie-part-left-eye = 左眼
amie-part-eye = 眼睛
amie-part-right-ear = 右耳
amie-part-left-ear = 左耳
amie-part-hand = 手
amie-part-spikes = 尖刺
amie-part-ghostly = 幽靈
amie-part-flame = 火焰
amie-part-sparks = 電火花
amie-part-slime = 黏液
amie-part-sludge = 污泥
amie-part-ice = 冰
amie-part = 部位
puzzle-cell = { $column }{ $row }
puzzle-home = { $tile }，已歸位
puzzle-goes = { $tile }，應放在{ $home }
puzzle-picture-left = { $pokemon }在左邊
puzzle-picture-front = { $pokemon }在前面
puzzle-picture-right = { $pokemon }在右邊
puzzle-picked = { $tile }，已選取
berry-picker-wants = { $pokemon }想要{ $berry }
berry-picker-for = { $berry }給{ $pokemon }
berry-picker-wanted = 想要{ $berry }
berry-picker-lineup = { $first }、{ $second }和{ $third }
berry-picker-not-up = { $pokemon }沒有出來
berry-picker-no-berry = 沒有樹果{ $number }
berry-picker-no-pokemon = 沒有寶可夢{ $number }
head-it-ball = { $pokemon }
head-it-ball-nobody = 毛線球
head-it-no-pokemon = 沒有寶可夢{ $number }
head-it-no-score = 沒有得分
head-it-lineup = { $first }、{ $second }和{ $third }
head-it-estimate = pa3ds 估計你的提示提前量為 { $seconds } 秒，根據 { $taps } 次點擊

## What pa3ds says of itself

app-waiting = 等待遊戲
app-ready = pa3ds已就緒
app-restarted = pa3ds已重新啟動
app-reader-error = 讀取錯誤
dialog-ok = 確定
dialog-cancel = 取消
window-title = Pokémon Access 3DS
window-menu-mod = 模組(&M)
window-updates = 檢查更新(&U)
window-exit = 結束(&X)
window-menu-azahar = &Azahar
window-azahar-portable = 下載 Azahar 可攜版(&P)
window-azahar-installer = 下載 Azahar 安裝版(&I)
window-menu-help = 說明(&H)
window-documentation = 說明文件(&D)...
window-keys = 按鍵指令(&K)...
window-amie = 寶可拍樂樂(&A)...
window-known-issues = 已知問題(&I)...
window-translating = 翻譯(&T)...
window-changelog = 更新紀錄(&C)...
window-settings-guide = 設定指南(&S)...
window-issues = 在 GitHub 回報問題(&R)
setting-pa3ds-keys = 回應模組按鍵
setting-pa3ds-keys-about = 回應pa3ds的按鍵，並依照keys.toml的設定綁定Azahar的按鈕。
setting-pa3ds-poll_rate = 每秒讀取次數
setting-pa3ds-poll_rate-about = 每秒讀取遊戲的次數：60是遊戲本身的影格率；較慢的電腦可設為30。
setting-pa3ds-azahar_folder = 可攜版Azahar資料夾
setting-pa3ds-azahar_folder-about = 可攜版Azahar所在的資料夾，也就是有azahar.exe和user資料夾的那個；以一般方式安裝的Azahar會把設定存放在AppData中，這時留空。pa3ds詢問資料夾時，你選擇的資料夾會自動填入這裡。
setting-pa3ds-hide_window = Azahar開啟時隱藏模組視窗
setting-pa3ds-hide_window-about = Azahar開啟時隱藏pa3ds的視窗，也就是按兩下pa3ds時開啟的那個視窗，讓Alt+Tab中沒有pa3ds的東西夾在你和遊戲之間；設為false則讓它保持顯示，以便OBS透過它錄下pa3ds的聲音。
setting-pa3ds-ui-pauses = 畫面文字中的停頓
setting-pa3ds-ui-pauses-about = 在一起朗讀的幾段畫面文字之間加入停頓：標籤後加冒號，例如「初訓家：Raiden」，其餘之間加逗號。
setting-pa3ds-ui-interrupt = 選單打斷語音
setting-pa3ds-ui-interrupt-about = 選單游標移動時中斷語音，而不是排隊等候；pa3ds自己的按鍵一律會中斷語音。
setting-pa3ds-ui-unfocused = Azahar在背景時仍朗讀
setting-pa3ds-ui-unfocused-about = 其他視窗在前景時，仍繼續朗讀遊戲顯示的內容；否則這時pa3ds保持安靜，只朗讀它自己的按鍵和訊息。
setting-pa3ds-ui-descriptions = 場景描述
setting-pa3ds-ui-descriptions-about = 描述遊戲中沒有文字的場景，例如固拉多甦醒。每段描述會在場景所接續的那行文字之後朗讀。目前只描述了部分場景。
setting-pa3ds-ui-trainer_party = 訓練家有幾隻寶可夢
setting-pa3ds-ui-trainer_party-about = 與訓練家的對戰開始時，在發起挑戰的那行文字之後，朗讀訓練家有幾隻寶可夢（「text」），每隻播放一聲嗶聲（「beeps」），或都不做（「off」）。嗶聲與[pa3ds.audio]下的提示音一樣大聲。
setting-pa3ds-ui-trainer_party-text = 朗讀
setting-pa3ds-ui-trainer_party-beeps = 每隻一聲嗶
setting-pa3ds-ui-trainer_party-off = 都不做
setting-pa3ds-ui-language = 語言
setting-pa3ds-ui-language-about = pa3ds朗讀自身文字所用的語言，以它的資料夾名稱表示：en、fr、de、es、it、ja、ko、zh-CN或zh-TW。留空則跟隨遊戲本身的語言，由pa3ds從存檔讀取；讀到的語言不對時，例如在其他語言上套用了玩家自製的翻譯，就在這裡設定；在《太陽／月亮》中，pa3ds無法讀取存檔，也要在這裡設定遊戲的語言。
setting-pa3ds-audio-tones = 提示音相對音量
setting-pa3ds-audio-tones-about = 地圖游標或觸控模式移動時那聲短提示音的音量；音調越高表示越往上，聲音越偏右表示越往右，用來告訴你游標在哪裡：1與遊戲一樣大聲，0.5為一半，1.5更大聲，0為無聲；在設定對話方塊中以百分比表示，100與遊戲一樣大聲。
setting-pa3ds-audio-footsteps = 腳步聲相對音量
setting-pa3ds-audio-footsteps-about = 在遊戲本身不播放腳步聲的地面上，例如街道、地板和洞窟，pa3ds自己的腳步聲有多大聲：1與遊戲播放時一樣大聲，0.5為一半，1.5更大聲，0為無聲；在設定對話方塊中以百分比表示，100與遊戲一樣大聲。
setting-pa3ds-audio-ticks = 時鐘滴答聲相對音量
setting-pa3ds-audio-ticks-about = 遊戲計時器每過一秒時那聲短提示音的音量，例如在寶可拍樂樂的小遊戲中；剩餘時間本身會在每半分鐘和最後幾秒朗讀出來：1與遊戲一樣大聲，0.5為一半，1.5更大聲，0為無聲，也是預設值；在設定對話方塊中以百分比表示，100與遊戲一樣大聲。
setting-pa3ds-audio-feedback_sounds = 回饋音效相對音量
setting-pa3ds-audio-feedback_sounds-about = 遊戲本身用來表示按鍵結果的短音效有多大聲：電腦盒子裡拿起和放下寶可夢的音效，用於觸控模式拿起和放開拖曳；還有遊戲的錯誤音效，用於無事可做的按鍵，例如採果樂這一輪沒有的樹果；以及瞄準頭球的提示。設為0時，不播放提示，pa3ds改為說出來：「按住」、「在……上放開」以及原因：1與遊戲一樣大聲，0.5為一半，1.5更大聲；在設定對話方塊中以百分比表示，100與遊戲一樣大聲。
setting-pa3ds-navigation-positions = 報出清單位置
setting-pa3ds-navigation-positions-about = 說出物件在清單中的位置，例如「門，第1，共3」。
setting-pa3ds-navigation-position_first = 清單位置在方向之前
setting-pa3ds-navigation-position_first-about = 先說清單中的位置，再說怎麼走過去：「門，第1，共3，北1」；設為false則說「門，北1，第1，共3」。
setting-pa3ds-navigation-turn_by_turn = 逐步導航
setting-pa3ds-navigation-turn_by_turn-about = P找到路線後，每到一個轉彎就說出來；設為false則只說一次路線，再按P會從你所在的位置重新說。
setting-pa3ds-navigation-guide_to_hidden = 引導至圖鑑導航找到的寶可夢
setting-pa3ds-navigation-guide_to_hidden-about = 圖鑑導航搜尋找到要找的寶可夢時，或圖鑑導航開啟、你又在觸控模式中而它自己發現一隻時，像P一樣找出通往牠的路線，並讓P保持指向牠，不論清單原本在哪裡；區域導航開啟時發現的則不處理。
setting-pa3ds-navigation-track_boulders = 追蹤巨石移動
setting-pa3ds-navigation-track_boulders-about = 說出巨石離它在房間裡的起始位置移動了多遠，例如「巨石，東2，已移動北1」，讓你知道推過哪些；遊戲只顯示每塊巨石現在在哪裡，從不顯示它原本在哪裡。
setting-pa3ds-navigation-hide_door_signs = 隱藏門牌
setting-pa3ds-navigation-hide_door_signs-about = 告示牌只寫著旁邊那扇門的名稱時，例如紫堇山丘的「13號室 鐵旋的家」，就不列入清單；寫了更多內容的告示牌，例如道館的，仍會保留。
setting-pa3ds-navigation-show_y = 報出高度
setting-pa3ds-navigation-show_y-about = 按C說出你所在的位置、按Shift加C說出所選物件的位置時，也在x和z之後以y說出高度：「x 71, z 35, y 20」；使用遊戲本身的單位，一格寬為18；pa3ds不知道高度時則省略，例如在《太陽／月亮》中。
setting-pa3ds-navigation-sort_by_distance = 依距離排序
setting-pa3ds-navigation-sort_by_distance-about = 清單模式的清單由近到遠排列，會隨著你走動而改變；設為false則每個物件在清單中的位置固定，依照遊戲本身資料表的順序，讓你能從位置分辨進過哪間房子；方向模式不論如何都由近到遠排列。
setting-pa3ds-touch-wrap = 循環移動
setting-pa3ds-touch-wrap-about = 在觸控模式中，從最後一個按鈕回到第一個，從最下面一列回到最上面一列；設為false則停在兩端，這樣按幾次O一定會停在最下面一列。
setting-pa3ds-touch-next_row = 繼續到下一列
setting-pa3ds-touch-next_row-about = 在觸控模式中，J和L到了一列的盡頭時繼續進入上一列或下一列；設為false則讓它們停留在同一列，這樣按幾次L一定會停在該列的最右邊。
setting-pa3ds-touch-drag = 拖曳方式
setting-pa3ds-touch-drag-about = 按住Shift拖曳，或使用你在keys.toml中綁定為拖曳的按鍵，放開按鍵時觸控也跟著放開（「hold」）；或按一下拿起，再按一下放開（「toggle」）。目前只有寶可拍樂樂需要拖曳。
setting-pa3ds-touch-drag-hold = 按住Shift
setting-pa3ds-touch-drag-toggle = 按一下拿起，再按一下放開
setting-pa3ds-amie-descriptions = 描述寶可夢在做什麼
setting-pa3ds-amie-descriptions-about = 在寶可拍樂樂中，在寶可夢開始和停止動作時說出看得到的動作：東張西望、看著你的手、吃東西、舉手等著擊掌。牠的每個動作也會在播放時用牠自己的一句話描述。也會說出撫摸帶來的心，以及擊掌成功。
setting-pa3ds-amie-fidgets = 描述寶可夢的小動作
setting-pa3ds-amie-fidgets-about = 開啟寶可拍樂樂的描述時，也說出寶可夢每隔幾秒自己做的小動作。關閉時只說牠對你的反應。
setting-pa3ds-amie-berry_first = 採果樂先說樹果
setting-pa3ds-amie-berry_first-about = 依照你給樹果的順序說出每隻想要樹果的寶可夢，先說樹果再說寶可夢，例如「凰梨果給皮卡丘」；設為false則先說寶可夢，例如「皮卡丘想要凰梨果」。
setting-pa3ds-amie-one_at_a_time = 採果樂一次說一隻寶可夢
setting-pa3ds-amie-one_at_a_time-about = 一次只說一隻想要樹果的寶可夢，等你給了樹果，或說到的那隻離開後，再說下一隻；設為false則每隻一出現就說。
setting-pa3ds-amie-head_it_cue = 瞄準頭球提示何時點擊
setting-pa3ds-amie-head_it_cue-about = 在瞄準頭球中，到了該點擊毛線球的時刻時，播放遊戲本身的「！」音效：在毛線球到達寶可夢之前，提前下面設定的時間響起，從那隻寶可夢那一側的耳朵傳來，中間的寶可夢則兩隻耳朵都有；需要開啟回饋音效。
setting-pa3ds-amie-head_it_lead = 瞄準頭球的提示提前量
setting-pa3ds-amie-head_it_lead-about = 提示在毛線球到達寶可夢之前多少秒響起：大約是你從聽到提示到按下那隻寶可夢的鍵所需的時間。每輪結果時，pa3ds會說出它對你的估計值，可以填在這裡。點擊太晚就調高，太早就調低。
setting-pa3ds-amie-berry_picker_style = 採果樂中如何區分寶可夢
setting-pa3ds-amie-berry_picker_style-about = 在採果樂中，用名字（「name」）、寶可夢自己的叫聲（「cry」）或兩者（「both」）來說出寶可夢。不論牠從哪裡冒出來，叫聲都從牠在本輪出場順序中的位置傳來：第一隻在左耳，第二隻兩隻耳朵都有，第三隻在右耳。叫聲需要開啟回饋音效；無法播放時會說出名字。
setting-pa3ds-amie-berry_picker_style-name = 名字
setting-pa3ds-amie-berry_picker_style-cry = 叫聲
setting-pa3ds-amie-berry_picker_style-both = 名字和叫聲
setting-pa3ds-amie-head_it_style = 瞄準頭球中如何區分寶可夢
setting-pa3ds-amie-head_it_style-about = 在瞄準頭球中，用名字（「name」）、寶可夢自己的叫聲（「cry」）、兩者（「both」）來說出毛線球落向的寶可夢，或兩者都不用（「none」），只聽提示。叫聲從牠在本輪出場順序中的位置傳來：第一隻在左耳，第二隻兩隻耳朵都有，第三隻在右耳。叫聲需要開啟回饋音效；無法播放時會說出名字。
setting-pa3ds-amie-head_it_style-name = 名字
setting-pa3ds-amie-head_it_style-cry = 叫聲
setting-pa3ds-amie-head_it_style-both = 名字和叫聲
setting-pa3ds-amie-head_it_style-none = 都不用
setting-pa3ds-amie-head_it_score = 瞄準頭球在得分變化時說出
setting-pa3ds-amie-head_it_score-about = 在瞄準頭球中，每當得分、連擊和失誤出現時說出，得分和連擊在持續上升時只說一次；false 則交給瞄準頭球的得分鍵（R），每次按下時連同剩餘時間或剩餘機會一起說出。
setting-pa3ds-extras-conditions = 以數字說出華麗大賽狀態
setting-pa3ds-extras-conditions-about = 在寶可夢資訊畫面和寶可方塊盒中，遊戲以星星表示寶可夢的華麗大賽狀態，這裡改以數字說出：星星人人都看得到，但pa3ds無法讀取，所以只能提供數字，而數字比星星更精確。
setting-pa3ds-training-shot_cue = 氣球機器人射擊時
setting-pa3ds-training-shot_cue-about = 在超級特訓中，如何得知氣球機器人射擊：遊戲的警報聲、「來球了」這句話，或無提示。關閉回饋音時用語音讀出。
setting-pa3ds-training-shot_cue-sound = 音效
setting-pa3ds-training-shot_cue-speech = 語音
setting-pa3ds-training-shot_cue-off = 關閉
setting-pa3ds-training-block_cue = 格擋或閃避時機
setting-pa3ds-training-block_cue-about = 在超級特訓中，如何得知射擊即將到達：遊戲的「!」音效、「現在」這個詞，或無提示。關閉回饋音時用語音讀出。
setting-pa3ds-training-block_cue-sound = 音效
setting-pa3ds-training-block_cue-speech = 語音
setting-pa3ds-training-block_cue-off = 關閉
setting-pa3ds-training-block_lead = 格擋提示提前量
setting-pa3ds-training-block_lead-about = 射擊到達前多少秒發出提示。按下 L 後屏障持續不到一秒。
setting-pa3ds-extras-base_stats = 超級特訓能力顯示為數字
setting-pa3ds-extras-base_stats-about = 在超級特訓頁面按 R 時，讀出遊戲以圖表繪製的各項能力點數，而不是大致比例。
setting-pa3ds-extras-hidden_items = 隱藏道具
setting-pa3ds-extras-hidden_items-about = 在道具中列出尚未找到的隱藏道具，標為「隱藏道具」：只限海底（「underwater」），那裡無法使用探測器，看得見的玩家可能會在岩石和海草間看到閃光；或所有地方（「all」），這會說出遊戲對所有人隱藏的東西；「off」則不列出。
setting-pa3ds-extras-hidden_items-off = 關閉
setting-pa3ds-extras-hidden_items-underwater = 海底
setting-pa3ds-extras-hidden_items-all = 所有地方
settings-title = 模組設定
settings-page-general = 一般
settings-page-speech = 語音
settings-page-audio = 音訊
settings-page-navigation = 導覽
settings-page-touch = 觸控模式
settings-page-amie = 寶可拍樂樂
settings-page-training = 超級特訓
settings-page-extras = 額外功能
settings-browse = 瀏覽(&B)...
settings-language-game = 遊戲本身的語言
settings-restart = 要立即重新啟動pa3ds，讓這些設定生效嗎？如果選擇「否」，設定會在下次啟動pa3ds時生效。
settings-unwritten = pa3ds無法寫入{ $path }，所以沒有任何變更。
dialog-apply = 套用(&A)
window-settings = 設定(&S)...

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
kind-way-down = 下去的路
kind-conveyor-belt = 輸送帶
kind-switch = 開關
kind-pink-switch = 粉紅色開關
kind-blue-switch = 藍色開關
kind-yellow-switch = 黃色開關
kind-green-switch = 綠色開關
kind-warp = 傳送點
kind-path = 小路
kind-edge = 小路
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
kind-hidden-item = 隱藏道具
kind-barrier = 屏障
kind-hole = 洞
kind-dive-spot = 潛水點
kind-current = 海流
kind-water = 水面
kind-waterfall = 瀑布
kind-arrow-east = 向東箭頭
kind-arrow-west = 向西箭頭
kind-arrow-north = 向北箭頭
kind-arrow-south = 向南箭頭
kind-flat-ground = 平地
kind-open-ground = 空地
kind-quiz-panel = 答題面板
kind-blue-spot = 藍色落腳點
kind-wall-ledge = 岩壁平台
kind-vine-up = 向上的藤蔓
kind-vine-down = 向下的藤蔓
kind-swinging-vine = 可盪的藤蔓
kind-gym-leader-spot = 道館館主的位置
kind-north-waiting-spot = 北側等待點，靠近死路
kind-south-waiting-spot = 南側等待點，靠近迷宮入口
kind-east-waiting-spot = 東側等待點，中間通道的東端
kind-west-waiting-spot = 西側等待點，中間通道的西端
furfrou-wait = 下一步，{ $shauna }在這裡等
furfrou-fetch = 下一步，和{ $shauna }說話，讓她跟著你
furfrou-chase = 下一步，走到這裡，{ $furfrou }會逃跑
furfrou-keep-off = 不要踩，{ $furfrou }會從這裡逃走
furfrou-leave = 下一步，從這裡離開迷宮
furfrou-enter = 下一步，回到迷宮裡
kind-machine = 裝置
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
battle-party = { $count }隻寶可夢

## Scenes

narration = 旁白
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = 繁體中文

registered-slot = { $way } { $item }
registered-up = 上
registered-right = 右
registered-down = 下
registered-left = 左

bag-moving = 正在移動 { $item }
bag-between = { $above } 和 { $below } 之間
bag-above = { $below } 上方
bag-below = { $above } 下方
