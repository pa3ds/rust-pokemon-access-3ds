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
key-one-way = 只有一條路
out-at = 出口在{ $bearing }
past-teleporter = 傳送裝置另一邊
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
training-shot-coming = 來球了
training-left = 剩餘{ $time }
amie-stroke = 撫摸{ $pokemon }
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
setting-pa3ds-ui-language = 語言
setting-pa3ds-ui-language-about = pa3ds朗讀自身文字所用的語言，以它的資料夾名稱表示：en、fr、de、es、it、ja、ko、zh-CN或zh-TW。留空則跟隨遊戲本身的語言，由pa3ds從存檔讀取；讀到的語言不對時，例如在其他語言上套用了玩家自製的翻譯，就在這裡設定；在《太陽／月亮》中，pa3ds無法讀取存檔，也要在這裡設定遊戲的語言。
setting-pa3ds-audio-tones = 提示音相對音量
setting-pa3ds-audio-tones-about = 地圖游標或觸控模式移動時那聲短提示音的音量；音調越高表示越往上，聲音越偏右表示越往右，用來告訴你游標在哪裡：1與遊戲一樣大聲，0.5為一半，1.5更大聲，0為無聲；在設定對話方塊中以百分比表示，100與遊戲一樣大聲。
setting-pa3ds-audio-footsteps = 腳步聲相對音量
setting-pa3ds-audio-footsteps-about = 在遊戲本身不播放腳步聲的地面上，例如街道、地板和洞窟，pa3ds自己的腳步聲有多大聲：1與遊戲播放時一樣大聲，0.5為一半，1.5更大聲，0為無聲；在設定對話方塊中以百分比表示，100與遊戲一樣大聲。
setting-pa3ds-audio-ticks = 時鐘滴答聲相對音量
setting-pa3ds-audio-ticks-about = 遊戲計時器每過一秒時那聲短提示音的音量，例如在寶可拍樂樂的小遊戲中；剩餘時間本身會在每半分鐘和最後幾秒朗讀出來：1與遊戲一樣大聲，0.5為一半，1.5更大聲，0為無聲，也是預設值；在設定對話方塊中以百分比表示，100與遊戲一樣大聲。
setting-pa3ds-audio-feedback_sounds = 回饋音效相對音量
setting-pa3ds-audio-feedback_sounds-about = 遊戲本身用來表示按鍵結果的短音效有多大聲：電腦盒子裡拿起和放下寶可夢的音效，用於觸控模式拿起和放開拖曳；還有遊戲的錯誤音效，用於無事可做的按鍵，例如採果樂這一輪沒有的樹果。設為0時，pa3ds改為說出來：「按住」、「在……上放開」以及原因：1與遊戲一樣大聲，0.5為一半，1.5更大聲；在設定對話方塊中以百分比表示，100與遊戲一樣大聲。
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
setting-pa3ds-amie-berry_first = 採果樂先說樹果
setting-pa3ds-amie-berry_first-about = 依照你給樹果的順序說出每隻想要樹果的寶可夢，先說樹果再說寶可夢，例如「凰梨果給皮卡丘」；設為false則先說寶可夢，例如「皮卡丘想要凰梨果」。
setting-pa3ds-amie-one_at_a_time = 採果樂一次說一隻寶可夢
setting-pa3ds-amie-one_at_a_time-about = 一次只說一隻想要樹果的寶可夢，等你給了樹果，或說到的那隻離開後，再說下一隻；設為false則每隻一出現就說。
setting-pa3ds-extras-conditions = 以數字說出華麗大賽狀態
setting-pa3ds-extras-conditions-about = 在寶可夢資訊畫面和寶可方塊盒中，遊戲以星星表示寶可夢的華麗大賽狀態，這裡改以數字說出：星星人人都看得到，但pa3ds無法讀取，所以只能提供數字，而數字比星星更精確。
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
kind-hidden-item = 隱藏道具
kind-barrier = 屏障
kind-hole = 洞
kind-dive-spot = 潛水點
kind-current = 海流
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
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = 繁體中文
