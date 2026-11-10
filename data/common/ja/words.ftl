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
way-up = 上
way-down = 下
steps = { $way }{ $count }
leg-with = { $how } { $way }{ $steps }
here = ここ
above = 上{ $count }
below = 下{ $count }
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
category-climbing = クライミング
category-currents = 海流
category-arrows = 矢印床
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
key-nothing-said = まだ何も話していない
key-one-way = 行き方は一つだけ
out-at = 出る先は{ $bearing }
past-teleporter = ワープパネルの先
leads-to = { $place }への{ $thing }
sign-saying = { $text }の看板
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
how-rail = レール
how-hop = ジャンプ
how-bunny-hop = バニーホップ
how-push = 押す
how-ride-the-current = 海流に乗る
how-get-off = 降りる
how-smash = 砕く
how-climb = 登る
p-climb-up = 上へ{ $holds }登る
p-climb-down = 下へ{ $holds }降りる
how-slide = 滑り降りる
how-grind-the-rail = レールを滑る
p-grind = レールを滑って{ $bearing }
how-ride-the-belt = ベルトコンベアに乗る
p-belt = ベルトコンベアに乗って{ $bearing }
how-arrow-pushes = 矢印床に押される
p-arrow = 矢印床に押されて{ $bearing }
p-switch-first = 先に{ $thing }、{ $way }
how-climb-the-vine-up = つるを登る
how-climb-the-vine-down = つるを降りる
how-swing-on-the-vine = つるでスイングする
p-vine = { $way }を向いて、{ $how }
p-more-vines = { $count ->
   *[other] あと{ $count }本のつる
}
p-more-teleporters = { $count ->
   *[other] あと{ $count }枚のワープパネル
}
how-up-the-ramp = 坂を上る
how-down-the-ramp = 坂を下る
p-current = 海流に乗って{ $bearing }
p-to-push = { $way }に押すには：
p-nudge = 少し{ $way }
p-nudge-then = { $nudge }、それから
p-lined-up = 位置が合った
p-then-cant-continue = その先は進めない
p-cant-continue = 進めない
p-in-the-way = { $thing }がじゃまをしている
p-requires = { $move }がひつよう
p-surface = 浮上して{ $bearing }のダイビングポイントへ
p-into = { $leg }、{ $place }へ
p-and-ice-cleared = そして氷はすべて割れる
p-ice-cleared = 氷が割れた
p-no-path-to = { $thing }への経路なし
p-no-route-in-game = このゲームではまだ経路がない
p-no-route-on-streets = この通りでは経路がない
p-gone = { $thing }がいなくなった

## Touch mode

touch-nothing = タッチするものがない
touch-no-drag = ドラッグするものがない
touch-continue = 続ける
touch-off = タッチはオフ
touch-not-listening = Azaharはpa3dsからのタッチを受け付けていない
touch-holding = { $target }をつかんでいる
touch-dropped = { $target }の上で離した

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
training-barrier = バリア
training-shot-coming = シュートが来る
training-left = 残り{ $time }
training-empty-bag = 空のサンドバッグ
training-bag-finished = { $pokemon } がベーストレーニングを終えた
training-resting = { $pokemon } は休んでいる
training-total = 合計
training-total-points = 合計 { $points }/{ $full }
training-charged = チャージ完了
training-fizzled = 消えた
training-blocked = 防いだ
training-at-start = スタート位置
training-block-now = 今
training-energy = エネルギー
training-no-numbers = 数値なし
training-medals = メダル { $count } 個
amie-beckon = { $pokemon }を呼ぶ
amie-visitor = 訪問中の{ $pokemon }
amie-stylus = タッチペン
amie-stylus-on = タッチペン、{ $part }
amie-stylus-edge = 端
amie-view-down = { $pokemon }を下ろす
amie-view-up = { $pokemon }を上に戻す
amie-view-came-down = { $pokemon }が下りてきた
amie-view-went-up = { $pokemon }が上に戻った
amie-mood-coming = { $pokemon }が近づいてくる
amie-mood-back-turned = { $pokemon }は背中を向けている
amie-mood-turned-round = { $pokemon }が振り向いた
amie-mood-dozing = { $pokemon }がうとうとしている
amie-mood-asleep = { $pokemon }は眠っている
amie-mood-awake = { $pokemon }が目を覚ました
amie-mood-too-full = { $pokemon }はおなかがいっぱいで食べられない
amie-mood-puff-seen = { $pokemon }が{ $puff }を見つめている
amie-mood-reaching = { $pokemon }が{ $puff }に手を伸ばしている
amie-mood-eating = { $pokemon }が食べている
amie-mood-eaten = { $pokemon }が{ $puff }を食べ終わった
amie-mood-puff-fell = { $puff }が落ちた
amie-mood-picky = { $pokemon }が顔をしかめてそっぽを向いた
amie-mood-watching = { $pokemon }があなたの手を見ている
amie-mood-looking-around = { $pokemon }があたりを見回している
amie-mood-hand-raised = { $pokemon }が手を上げている
amie-mood-your-face = { $pokemon }があなたの顔に近づいた
amie-high-five = ハイタッチ！
amie-mood-disliked = { $pokemon }はいやがっている…
amie-mood-closer = { $pokemon }は大喜びで、ハートがあふれ出した
amie-mood-best-friends = { $pokemon }はハートの雨の中で大はしゃぎしている
amie-meter = { $label } 5のうち{ $count }
amie-hearts = { $count ->
    [one] ハート1つ
   *[other] ハート{ $count }つ
}
amie-part-off = { $pokemon }の外
amie-part-head = 頭
amie-part-body = 体
amie-part-right-eye = 右目
amie-part-left-eye = 左目
amie-part-eye = 目
amie-part-right-ear = 右耳
amie-part-left-ear = 左耳
amie-part-hand = 手
amie-part-spikes = トゲ
amie-part-ghostly = ゴースト
amie-part-flame = 炎
amie-part-sparks = 電気
amie-part-slime = ネバネバ
amie-part-sludge = ヘドロ
amie-part-ice = 氷
amie-part = 部分
puzzle-cell = { $column }{ $row }
puzzle-home = { $tile }、正しい位置
puzzle-goes = { $tile }、{ $home }へ
puzzle-picture-left = 左に{ $pokemon }
puzzle-picture-front = 手前に{ $pokemon }
puzzle-picture-right = 右に{ $pokemon }
puzzle-picked = { $tile }、選んだ
berry-picker-wants = { $pokemon }は{ $berry }がほしい
berry-picker-for = { $berry }を{ $pokemon }に
berry-picker-wanted = { $berry }がほしい
berry-picker-lineup = { $first }、{ $second }と{ $third }
berry-picker-not-up = { $pokemon }は出ていない
berry-picker-no-berry = きのみ{ $number }はない
berry-picker-no-pokemon = ポケモン{ $number }はいない
head-it-ball = { $pokemon }
head-it-ball-nobody = 毛糸玉
head-it-no-pokemon = ポケモン{ $number }はいない
head-it-no-score = スコアなし
head-it-lineup = { $first }、{ $second }と{ $third }
head-it-estimate = pa3dsの推定では、合図の先行時間は{ $seconds }秒です（{ $taps }回のタップから）

## What pa3ds says of itself

app-waiting = ゲームを待っています
app-ready = pa3ds 準備完了
app-restarted = pa3ds 再起動
app-reader-error = 読み取りエラー
dialog-ok = OK
dialog-cancel = キャンセル
window-title = Pokémon Access 3DS
window-menu-mod = MOD(&M)
window-updates = 更新を確認(&U)
window-exit = 終了(&X)
window-menu-azahar = Azahar(&A)
window-azahar-portable = Azaharをダウンロード（ポータブル版）(&P)
window-azahar-installer = Azaharをダウンロード（インストーラー版）(&I)
window-menu-help = ヘルプ(&H)
window-documentation = 説明書(&D)...
window-keys = キー操作(&K)...
window-amie = ポケパルレ(&A)...
window-known-issues = 既知の問題(&I)...
window-translating = 翻訳について(&T)...
window-changelog = 変更履歴(&C)...
window-settings-guide = 設定ガイド(&S)...
window-issues = GitHubで問題を報告(&R)
setting-pa3ds-keys = MODのキーを使う
setting-pa3ds-keys-about = keys.tomlの設定どおりに、pa3dsのキーを受け付け、Azaharのボタンをキーに割り当てる。
setting-pa3ds-poll_rate = 1秒あたりの読み取り回数
setting-pa3ds-poll_rate-about = ゲームを1秒に何回読み取るか。60はゲーム自体のフレームレート。遅いパソコンでは30。
setting-pa3ds-azahar_folder = ポータブル版Azaharのフォルダ
setting-pa3ds-azahar_folder-about = ポータブル版Azaharのあるフォルダ。azahar.exeとuserフォルダが入っているフォルダ。通常の方法でインストールしたAzaharで、設定がAppDataにある場合は空にする。pa3dsが尋ねるフォルダを選ぶと、pa3dsが入力する。
setting-pa3ds-hide_window = Azaharを開いている間MODのウィンドウを隠す
setting-pa3ds-hide_window-about = Azaharを開いている間、pa3dsをダブルクリックしたときに開くウィンドウを隠し、Alt+Tabであなたとゲームの間にpa3dsのものが入らないようにする。falseにすると表示したままになり、OBSでそのウィンドウを通してpa3dsの音を録音できる。
setting-pa3ds-ui-pauses = 画面の文字の間に区切りを入れる
setting-pa3ds-ui-pauses-about = 一緒に読み上げる画面の文字の間に区切りを入れる。「おや：Raiden」のようにラベルの後にはコロン、それ以外の間にはコンマ。
setting-pa3ds-ui-interrupt = メニューで読み上げを中断する
setting-pa3ds-ui-interrupt-about = メニューのカーソルが動いたとき、読み上げを順番待ちにせず中断する。pa3ds自身のキーは常に中断する。
setting-pa3ds-ui-unfocused = Azaharが背面にあるときも読み上げる
setting-pa3ds-ui-unfocused-about = ほかのウィンドウが前面にあるときも、ゲームの表示を読み上げ続ける。オフのときは、pa3ds自身のキーとメッセージ以外は何も言わない。
setting-pa3ds-ui-descriptions = 場面の説明
setting-pa3ds-ui-descriptions-about = グラードンが目覚める場面のように、ゲームが言葉なしで見せる場面を説明する。説明は、その場面の前に出る文章の後に読み上げる。説明があるのはまだ一部の場面だけ。
setting-pa3ds-ui-trainer_party = トレーナーの手持ちポケモンの数
setting-pa3ds-ui-trainer_party-about = トレーナーとのバトルが始まるとき、勝負をしかけてきた文の後に、手持ちのポケモンの数を読み上げる（"text"）、1匹ごとにビープ音を鳴らす（"beeps"）、どちらもしない（"off"）。ビープ音の大きさは[pa3ds.audio]のトーンと同じ。
setting-pa3ds-ui-trainer_party-text = 読み上げる
setting-pa3ds-ui-trainer_party-beeps = 1匹ごとにビープ音
setting-pa3ds-ui-trainer_party-off = どちらもしない
setting-pa3ds-ui-language = 言語
setting-pa3ds-ui-language-about = pa3dsが自分の言葉を話す言語。データのフォルダ名で指定する：en、fr、de、es、it、ja、ko、zh-CN、zh-TW。空にするとゲーム自体の言語に従い、pa3dsはそれをセーブデータから読み取る。それが正しくない場合、たとえば別の言語の上にファン翻訳を当てている場合はここで設定する。pa3dsがセーブデータを読めないサン・ムーンでもここで設定し、その場合はこれがゲームの言語にもなる。
setting-pa3ds-audio-tones = トーンの相対音量
setting-pa3ds-audio-tones-about = マップのカーソルやタッチモードが動くときに鳴る短いトーンの大きさ。トーンはカーソルが上にあるほど高く、右にあるほど右から聞こえ、カーソルの位置を伝える。1でゲームと同じ大きさ、0.5で半分、1.5でより大きく、0で鳴らない。設定ダイアログではパーセントで、100がゲームと同じ大きさ。
setting-pa3ds-audio-footsteps = 足音の相対音量
setting-pa3ds-audio-footsteps-about = 通りや床、洞くつなど、ゲームが足音を鳴らさない地面でpa3ds自身が鳴らす足音の大きさ。1でゲームが鳴らす場合と同じ大きさ、0.5で半分、1.5でより大きく、0で鳴らない。設定ダイアログではパーセントで、100がゲームと同じ大きさ。
setting-pa3ds-audio-ticks = 時計の刻み音の相対音量
setting-pa3ds-audio-ticks-about = ポケパルレのミニゲームのように、ゲームの時計が1秒進むごとに鳴る短いトーンの大きさ。時計そのものは30秒ごとと最後の数秒に読み上げる。1でゲームと同じ大きさ、0.5で半分、1.5でより大きく、0で鳴らない（初期設定）。設定ダイアログではパーセントで、100がゲームと同じ大きさ。
setting-pa3ds-audio-feedback_sounds = 操作音の相対音量
setting-pa3ds-audio-feedback_sounds-about = キーで何が起きたかを伝える、ゲーム自体の短い音の大きさ。タッチモードでドラッグをつかんだり離したりするときの、パソコンのボックスでポケモンを持ち上げる音と置く音。そして、もぎってナッツのラウンドにないきのみのキーのように、何もすることのないキーで鳴るゲームのエラー音。それと、ねらってヘッドの合図。0にすると合図は鳴らず、pa3dsが代わりに「つかんでいる」「の上で離した」とその理由を言う。1でゲームと同じ大きさ、0.5で半分、1.5でより大きく。設定ダイアログではパーセントで、100がゲームと同じ大きさ。
setting-pa3ds-navigation-positions =リスト内の位置を読み上げる
setting-pa3ds-navigation-positions-about = 「ドア、3中1」のように、リストの何番目にあるかを言う。
setting-pa3ds-navigation-position_first = 方向の前に位置を読み上げる
setting-pa3ds-navigation-position_first-about = リスト内の位置を、そこへの行き方より先に言う：「ドア、3中1、北1」。falseにすると「ドア、北1、3中1」と言う。
setting-pa3ds-navigation-turn_by_turn = 曲がり角ごとの案内
setting-pa3ds-navigation-turn_by_turn-about = Pで道が見つかったら、曲がり角に着くたびに曲がり方を言う。falseにすると道を一度だけ言い、もう一度Pを押すと今いる場所からの道を言う。
setting-pa3ds-navigation-guide_to_hidden = ずかんナビで見つけたポケモンへ案内する
setting-pa3ds-navigation-guide_to_hidden-about = ずかんナビの検索で探していたポケモンが見つかったとき、またはずかんナビを開いていてタッチモードのときにずかんナビが自分でポケモンを見つけたとき、Pと同じようにそこへの道を探し、リストが何を指していてもPをそのポケモンに合わせたままにする。エリアナビを開いているときに見つかったものはそのままにする。
setting-pa3ds-navigation-track_boulders = 大岩の移動を追う
setting-pa3ds-navigation-track_boulders-about = 大岩が部屋の最初の位置からどれだけ動いたかを「大岩、東2、北1動いた」のように言い、どれを押したかがわかるようにする。ゲームは大岩の今の位置を見せるが、最初の位置は見せない。
setting-pa3ds-navigation-hide_door_signs = ドアの看板を隠す
setting-pa3ds-navigation-hide_door_signs-about = キンセツヒルズの「13号室 テッセンの家」のように、横のドアの名前だけを書いた看板はリストから外す。ジムの看板のようにそれ以上のことを書いた看板は残す。
setting-pa3ds-navigation-show_y = 高さを読み上げる
setting-pa3ds-navigation-show_y-about = Cで今いる場所を、ShiftとCで選んだものの場所を言うとき、xとzの後にyとして高さも言う：「x 71、z 35、y 20」。単位はゲーム自体のもので、1マスの幅が18。サン・ムーンのようにpa3dsが高さを知らない場合は言わない。
setting-pa3ds-navigation-sort_by_distance = 距離順に並べる
setting-pa3ds-navigation-sort_by_distance-about = リストモードのリストを近い順に並べる。歩くと順番が変わる。falseにすると、ゲーム自体の表の順番でそれぞれの位置が固定され、リストのどこにあるかで、どの家に入ったことがあるかがわかる。方向モードはどちらでも近い順。
setting-pa3ds-touch-wrap = 端で最初に戻る
setting-pa3ds-touch-wrap-about = タッチモードで、最後のボタンから最初のボタンへ、一番下の行から一番上の行へ戻る。falseにすると端で止まるので、Oを何回か押せば必ず一番下の行で止まる。
setting-pa3ds-touch-next_row = 次の行へ進む
setting-pa3ds-touch-next_row-about = タッチモードで、行の端に来たらJとLで上または下の行へ進む。falseにすると行の中にとどまるので、Lを何回か押せば必ずその行の右端で止まる。
setting-pa3ds-touch-drag = ドラッグの方法
setting-pa3ds-touch-drag-about = Shift、またはkeys.tomlでドラッグに割り当てた自分のキーを押したままドラッグし、キーを離すとタッチも離す（"hold"）。または、キーを一度押してつかみ、もう一度押して離す（"toggle"）。ドラッグするのはポケパルレだけ。
setting-pa3ds-touch-drag-hold = Shiftを押したまま
setting-pa3ds-touch-drag-toggle = 押してつかみ、もう一度押して離す
setting-pa3ds-amie-descriptions = ポケモンのしていることを説明する
setting-pa3ds-amie-descriptions-about = ポケパルレで、ポケモンのしていることを始まりと終わりに伝える。あたりを見回す、あなたの手を見る、食べる、ハイタッチのために手を上げるなど。動きのひとつひとつも、そのポケモンだけの文で伝える。なでて出たハートの数と、ハイタッチが決まったことも伝える。
setting-pa3ds-amie-fidgets = ポケモンの何気ないしぐさを説明する
setting-pa3ds-amie-fidgets-about = ポケパルレの説明がオンのとき、ポケモンが数秒ごとに自分からする小さなしぐさも伝える。オフのときは、あなたへの反応だけを伝える。
setting-pa3ds-amie-berry_first = もぎってナッツできのみを先に言う
setting-pa3ds-amie-berry_first-about = きのみをほしがるポケモンを、きのみをあげる順に、きのみ、ポケモンの順で言う：「パイルのみをピカチュウに」。falseにするとポケモンを先に言う：「ピカチュウはパイルのみがほしい」。
setting-pa3ds-amie-one_at_a_time = もぎってナッツでポケモンを一匹ずつ言う
setting-pa3ds-amie-one_at_a_time-about = きのみをほしがるポケモンを一匹ずつ言い、きのみをあげるか、言ったポケモンがいなくなると次を言う。falseにすると、現れるたびにそれぞれを言う。
setting-pa3ds-amie-head_it_cue = ねらってヘッドでタップの合図を鳴らす
setting-pa3ds-amie-head_it_cue-about = ねらってヘッドで、毛糸玉をタップする瞬間が来たら、ゲーム自体の「！」の音を鳴らす。毛糸玉がポケモンに届く、下の先行時間だけ前に鳴り、そのポケモンの側の耳に、真ん中のポケモンなら両耳に聞こえる。操作音がオンである必要がある。
setting-pa3ds-amie-head_it_lead = ねらってヘッドの合図の先行時間
setting-pa3ds-amie-head_it_lead-about = 毛糸玉がポケモンに届く何秒前に合図が鳴るか。聞いてからポケモンのキーを押すまでにかかる時間くらい。ラウンドの結果でpa3dsがあなたの推定値を言うので、ここに入れる。タップが遅すぎるなら上げ、早すぎるなら下げる。
setting-pa3ds-amie-berry_picker_style = もぎってナッツでのポケモンの聞き分け方
setting-pa3ds-amie-berry_picker_style-about = もぎってナッツで、ポケモンを名前（"name"）、そのポケモン自身の鳴き声（"cry"）、または両方（"both"）で伝える。鳴き声は、どこに出てきても、そのラウンドの並び順での位置から聞こえる。1匹目は左耳、2匹目は両耳、3匹目は右耳。鳴き声には操作音がオンである必要がある。鳴らせないときは名前を言う。
setting-pa3ds-amie-berry_picker_style-name = 名前
setting-pa3ds-amie-berry_picker_style-cry = 鳴き声
setting-pa3ds-amie-berry_picker_style-both = 名前と鳴き声
setting-pa3ds-amie-head_it_style = ねらってヘッドでのポケモンの聞き分け方
setting-pa3ds-amie-head_it_style-about = ねらってヘッドで、毛糸玉が落ちていくポケモンを名前（"name"）、そのポケモン自身の鳴き声（"cry"）、両方（"both"）で伝えるか、どちらも伝えず（"none"）合図だけにする。鳴き声は、そのラウンドの並び順での位置から聞こえる。1匹目は左耳、2匹目は両耳、3匹目は右耳。鳴き声には操作音がオンである必要がある。鳴らせないときは名前を言う。
setting-pa3ds-amie-head_it_style-name = 名前
setting-pa3ds-amie-head_it_style-cry = 鳴き声
setting-pa3ds-amie-head_it_style-both = 名前と鳴き声
setting-pa3ds-amie-head_it_style-none = どちらもなし
setting-pa3ds-amie-head_it_score = ねらってヘッドでスコアの変化を伝える
setting-pa3ds-amie-head_it_score-about = ねらってヘッドで、スコア、コンボ、ミスをその都度伝える。スコアとコンボは上がり続けている間は一度だけ伝える。false にすると、ねらってヘッドのスコアキー（R）に任せる。押すたびに、残り時間か残りのチャンスと一緒に伝える。
setting-pa3ds-extras-conditions = コンテストのコンディションを数値で
setting-pa3ds-extras-conditions-about = ステータス画面やポロックケースでゲームが星で表示するポケモンのコンテストのコンディションを、数値で言う。星は誰にでも見えるがpa3dsは読み取れないので、伝えられるのは数値だけ。数値は星より正確。
setting-pa3ds-training-shot_cue = バルーンボットが撃ったとき
setting-pa3ds-training-shot_cue-about = スパトレ中、バルーンボットの発射をどう聞くか。ゲームの警告音、「シュートが来る」という言葉、またはなし。反応音がオフのときは言葉で読み上げます。
setting-pa3ds-training-shot_cue-sound = 音
setting-pa3ds-training-shot_cue-speech = 音声
setting-pa3ds-training-shot_cue-off = オフ
setting-pa3ds-training-block_cue = ガードやよけるタイミング
setting-pa3ds-training-block_cue-about = スパトレ中、シュートが届く直前をどう聞くか。ゲームの「!」の音、「今」という言葉、またはなし。反応音がオフのときは言葉で読み上げます。
setting-pa3ds-training-block_cue-sound = 音
setting-pa3ds-training-block_cue-speech = 音声
setting-pa3ds-training-block_cue-off = オフ
setting-pa3ds-training-block_lead = ガード合図のタイミング
setting-pa3ds-training-block_lead-about = シュートが届く何秒前に合図するか。バリアは L を押してから 1 秒足らずしか続きません。
setting-pa3ds-extras-base_stats = スパトレの能力を数値で
setting-pa3ds-extras-base_stats-about = スパトレのページで R を押すと、グラフで描かれている各能力のポイントを、おおよその割合の代わりに読み上げます。
setting-pa3ds-extras-hidden_items = 隠された道具
setting-pa3ds-extras-hidden_items-about = まだ見つかっていない隠された道具を、道具の中に「隠された道具」として並べる。海の底だけ（"underwater"）：ダウジングマシンが使えず、目の見えるプレイヤーなら岩や海草の間にきらめきが見えることがある場所。またはどこでも（"all"）：ゲームが誰にも見せないものまで伝える。"off"では何も並べない。
setting-pa3ds-extras-hidden_items-off = オフ
setting-pa3ds-extras-hidden_items-underwater = 海の底
setting-pa3ds-extras-hidden_items-all = どこでも
settings-title = MODの設定
settings-page-general = 全般
settings-page-speech = 読み上げ
settings-page-audio = オーディオ
settings-page-navigation = ナビゲーション
settings-page-touch = タッチモード
settings-page-amie = ポケパルレ
settings-page-training = スパトレ
settings-page-extras = 追加機能
settings-browse = 参照(&B)...
settings-language-game = ゲーム自体の言語
settings-restart = 設定を反映するために、今すぐpa3dsを再起動しますか？「いいえ」を選ぶと、次にpa3dsを起動したときに反映されます。
settings-unwritten = { $path }に書き込めなかったので、何も変わっていません。
dialog-apply = 適用(&A)
window-settings = 設定(&S)...

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
kind-way-down = 下へ降りる道
kind-conveyor-belt = ベルトコンベア
kind-switch = スイッチ
kind-pink-switch = ピンクのスイッチ
kind-blue-switch = 青いスイッチ
kind-yellow-switch = 黄色いスイッチ
kind-green-switch = 緑のスイッチ
kind-warp = ワープ
kind-path = 道
kind-edge = 道
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
kind-hidden-item = 隠された道具
kind-barrier = 柵
kind-hole = 穴
kind-dive-spot = ダイビングポイント
kind-current = 海流
kind-arrow-east = 東向きの矢印床
kind-arrow-west = 西向きの矢印床
kind-arrow-north = 北向きの矢印床
kind-arrow-south = 南向きの矢印床
kind-flat-ground = 平らな地面
kind-open-ground = 開けた場所
kind-quiz-panel = クイズのパネル
kind-blue-spot = 青い足場
kind-wall-ledge = 壁の岩棚
kind-vine-up = 上りのつる
kind-vine-down = 下りのつる
kind-swinging-vine = スイングするつる
kind-gym-leader-spot = ジムリーダーの場所
kind-machine = 装置
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
battle-party = ポケモン{ $count }匹

## Scenes

narration = ナレーション
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = 日本語

registered-slot = { $way } { $item }
registered-up = 上
registered-right = 右
registered-down = 下
registered-left = 左

bag-moving = { $item } を移動中
bag-between = { $above } と { $below } の間
bag-above = { $below } の上
bag-below = { $above } の下
