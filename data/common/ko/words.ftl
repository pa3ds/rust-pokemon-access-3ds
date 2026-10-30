### Korean machine translation of `../en/words.ftl` (2026-10-18), a baseline for a translator to
### correct. The English file says what each message is said of, and where; a message missing
### here is said in English. A particle that changes with the word before it (이/가, 을/를,
### 은/는, 와/과) never follows a value here: the value can be any name.

## Ways and distances

way-north = 북쪽
way-east = 동쪽
way-south = 남쪽
way-west = 서쪽
way-north-east = 북동쪽
way-south-east = 남동쪽
way-south-west = 남서쪽
way-north-west = 북서쪽
way-ahead = 앞
way-behind = 뒤
way-left = 왼쪽
way-right = 오른쪽
steps = { $way } { $count }
leg-with = { $how } { $way } { $steps }
here = 여기
above = 위 { $count }
below = 아래 { $count }
rail-forward = 앞으로 { $steps }
rail-back = 뒤로 { $steps }
rail-hold-right = 오른쪽 누르기
rail-hold-left = 왼쪽 누르기
rail-angled = 카메라 비스듬함, 왼쪽 누르기, 오른쪽 누르기
turn-right = 오른쪽으로 돌기
turn-left = 왼쪽으로 돌기
turn-around = 뒤로 돌기

## The list of what is on the map, and the navigation keys' answers

category-all = 전체
category-exits = 출구
category-signs = 상호작용 대상
category-obstacles = 장애물
category-people = 사람
category-items = 도구
category-berries = 나무열매
category-ash = 화산재
category-bike = 자전거
category-riding = 라이드
category-climbing = 클라이밍
category-currents = 해류
category-arrows = 화살표 발판
category-pokemon = 포켓몬
key-category = { $category }, { $count }
position = { $count }개 중 { $number }
key-nothing-way = { $way }에 아무것도 없음
key-none-way = { $way }에 { $category } 없음
key-nothing-here = 여기 아무것도 없음
key-no-map = 지도 없음
key-cant-walk-now = 지금은 걸을 수 없음
key-list-mode = 목록 모드
key-direction-mode = 방향 모드
key-touch-mode = 터치 모드, { $first }
key-unreachable-hidden = 갈 수 없는 곳 숨김
key-unreachable-shown = 갈 수 없는 곳 표시
key-path-off = 경로 끔
key-no-path = 경로 없음
key-no-battle = 배틀 중 아님
key-one-way = 가는 길이 하나뿐
out-at = 나오는 곳 { $bearing }
past-teleporter = 텔레포터 너머
leads-to = { $place } 방면 { $thing }
span = { $low }에서 { $high }까지
either = { $these } 또는 { $last }

## Big boulders

key-push = { $way } 방향으로 밀기
boulder-into-hole = { $way } 방향 구멍으로
boulder-moveable = { $ways } 방향으로 이동 가능
boulder-immoveable = 이동 불가
boulder-moved = { $bearing } 이동함

## Renaming

rename-title = 이름 바꾸기
rename-label = { $thing }, { $bearing }. { $name }
rename-speaker-label = 마지막으로 말한 사람. { $name }
rename-called = 이름: { $name }
rename-not-named = 아직 이름 없음
rename-only-this-one = 이것만
rename-now = { $thing }, 이제 { $name }
rename-no-name = { $thing }, 이름 없음
rename-the-last-speaker = 마지막으로 말한 사람
rename-nothing = 이름을 바꿀 것이 없음
rename-cannot = { $thing }, 이름을 바꿀 수 없음
rename-already = 이미 이름을 바꾸는 중
rename-nobody-spoke = 아무도 말하지 않았음

## P: the way to the thing chosen, and guiding the player along it

how-water = 물
how-bush = 수풀
how-boulder = 큰 바위
how-rock = 바위
how-waterfall = 폭포
how-slope = 비탈
how-bumpy-slope = 울퉁불퉁한 비탈
how-push = 밀기
how-ride-the-current = 해류 타기
how-get-off = 내리기
how-smash = 부수기
how-climb = 오르기
p-climb-up = 위로 { $holds } 오르기
p-climb-down = 아래로 { $holds } 내려가기
how-slide = 미끄러지기
how-grind-the-rail = 레일 타기
p-grind = 레일 타기 { $bearing }
how-ride-the-belt = 컨베이어 벨트 타기
p-belt = 컨베이어 벨트 타기 { $bearing }
how-arrow-pushes = 화살표 발판에 밀림
p-arrow = 화살표 발판에 밀려 { $bearing }
p-switch-first = 먼저 { $thing }, { $way }
how-climb-the-vine-up = 덩굴 타고 오르기
how-climb-the-vine-down = 덩굴 타고 내려가기
how-swing-on-the-vine = 덩굴 타고 건너기
p-vine = { $way }을 보고, { $how }
p-more-vines = { $count ->
   *[other] 그다음 덩굴 { $count }개 더
}
p-more-teleporters = { $count ->
   *[other] 그다음 텔레포터 { $count }개 더
}
how-up-the-ramp = 경사로 오르기
how-down-the-ramp = 경사로 내려가기
p-current = 해류 타기 { $bearing }
p-to-push = { $way } 방향으로 밀려면:
p-nudge = 조금 { $way }
p-nudge-then = { $nudge }, 그다음
p-lined-up = 정렬됨
p-then-cant-continue = 그다음 더 갈 수 없음
p-cant-continue = 더 갈 수 없음
p-in-the-way = 길을 막는 { $thing }
p-requires = { $move } 필요
p-surface = 수면으로 올라가 { $bearing } 다이빙 포인트로
p-into = { $leg }, { $place }(으)로
p-and-ice-cleared = 그리고 얼음이 모두 깨짐
p-ice-cleared = 얼음 깨짐
p-no-path-to = { $thing }까지 경로 없음
p-no-route-in-game = 이 게임에는 아직 경로 없음
p-no-route-on-streets = 이 거리에는 경로 없음
p-gone = { $thing } 사라짐

## Touch mode

touch-nothing = 터치할 것 없음
touch-no-drag = 드래그할 것 없음
touch-continue = 계속
touch-off = 터치 꺼짐
touch-not-listening = Azahar가 pa3ds의 터치를 받지 않음
touch-holding = { $target } 잡고 있음
touch-dropped = { $target }에 놓음

## What the field shows without words

dowsing = 다우징머신 { $light }
dowsing-blue = 파랑
dowsing-green = 초록
dowsing-yellow = 노랑
dowsing-red = 빨강
dowsing-nothing = 반응 없음
decorating-placeable = 놓을 수 있음
decorating-not-placeable = 놓을 수 없음

## The Pokédex's habitat map

habitat-found = { $place }에서 발견
habitat-grass = 풀숲
habitat-water = 수면
habitat-fishing = 낚시

## Super Training and Pokémon-Amie

training-goal = 골
training-red-goal = 빨간 골
training-shot-coming = 슛이 온다
training-left = 남은 시간 { $time }
amie-stroke = { $pokemon } 쓰다듬기
puzzle-cell = { $column }{ $row }
puzzle-home = { $tile }, 제자리
puzzle-goes = { $tile }, 갈 곳 { $home }
puzzle-picture-left = 왼쪽에 { $pokemon }
puzzle-picture-front = 앞에 { $pokemon }
puzzle-picture-right = 오른쪽에 { $pokemon }
puzzle-picked = { $tile }, 선택함
berry-picker-wants = { $pokemon }, { $berry } 원함
berry-picker-for = { $berry }, { $pokemon }에게
berry-picker-wanted = { $berry } 원함
berry-picker-lineup = { $first }, { $second }, { $third }
berry-picker-not-up = { $pokemon } 나와 있지 않음
berry-picker-no-berry = 나무열매 { $number } 없음
berry-picker-no-pokemon = 포켓몬 { $number } 없음

## What pa3ds says of itself

app-waiting = 게임을 기다리는 중
app-ready = pa3ds 준비 완료
app-restarted = pa3ds 다시 시작됨
app-reader-error = 읽기 오류
dialog-ok = 확인
dialog-cancel = 취소
window-title = Pokémon Access 3DS
window-menu-mod = 모드(&M)
window-updates = 업데이트 확인(&U)
window-exit = 끝내기(&X)
window-menu-azahar = &Azahar
window-azahar-portable = Azahar 다운로드 (포터블)(&P)
window-azahar-installer = Azahar 다운로드 (설치 프로그램)(&I)
window-menu-help = 도움말(&H)
window-documentation = 설명서(&D)...
window-keys = 키 명령(&K)...
window-known-issues = 알려진 문제(&I)...
window-translating = 번역하기(&T)...
window-changelog = 변경 기록(&C)...
window-settings-guide = 설정 안내(&S)...
window-issues = GitHub에 문제 보고(&R)
setting-pa3ds-keys = 모드 키 사용
setting-pa3ds-keys-about = keys.toml에 적힌 대로 pa3ds의 키에 응답하고, Azahar의 버튼을 키에 할당합니다.
setting-pa3ds-poll_rate = 초당 읽기 횟수
setting-pa3ds-poll_rate-about = 1초에 게임을 몇 번 읽을지 정합니다. 60은 게임 자체의 프레임 속도이고, 느린 컴퓨터라면 30입니다.
setting-pa3ds-azahar_folder = 포터블 Azahar 폴더
setting-pa3ds-azahar_folder-about = 포터블 Azahar가 있는 폴더, 즉 azahar.exe와 user 폴더가 있는 폴더입니다. 보통 방법으로 설치한 Azahar라면 비워 두세요. 그 설정은 AppData에 있습니다. pa3ds가 폴더를 물을 때 고르면 pa3ds가 직접 채웁니다.
setting-pa3ds-hide_window = Azahar가 열려 있는 동안 모드 창 숨기기
setting-pa3ds-hide_window-about = Azahar가 열려 있는 동안 pa3ds 창, 즉 pa3ds를 더블클릭하면 열리는 창을 숨깁니다. 그러면 Alt+Tab에서 게임과의 사이에 pa3ds의 것이 끼지 않습니다. false이면 창이 보이는 채로 남아, OBS가 그 창으로 pa3ds의 소리를 녹음할 수 있습니다.
setting-pa3ds-ui-pauses = 화면 텍스트에 쉼 넣기
setting-pa3ds-ui-pauses-about = 함께 읽는 화면 텍스트 조각 사이에 쉼을 넣습니다. "어버이: Raiden"처럼 항목 이름 뒤에는 쌍점을, 나머지 사이에는 쉼표를 넣습니다.
setting-pa3ds-ui-interrupt = 메뉴 이동 시 음성 끊기
setting-pa3ds-ui-interrupt-about = 메뉴의 커서가 움직이면 음성을 대기열에 넣지 않고 끊습니다. pa3ds 자체의 키는 항상 끊습니다.
setting-pa3ds-ui-unfocused = Azahar가 백그라운드일 때도 말하기
setting-pa3ds-ui-unfocused-about = 다른 창이 앞에 있어도 게임 화면의 내용을 계속 말합니다. 그렇지 않으면 그동안 pa3ds는 자체 키와 메시지 말고는 조용합니다.
setting-pa3ds-ui-descriptions = 장면 설명
setting-pa3ds-ui-descriptions-about = 그란돈이 깨어나는 장면처럼, 게임이 글 없이 보여 주는 장면을 설명합니다. 각 설명은 그 장면 앞의 텍스트 줄 다음에 말합니다. 아직 일부 장면만 설명합니다.
setting-pa3ds-ui-language = 언어
setting-pa3ds-ui-language-about = pa3ds가 자체 문구를 말할 언어로, 데이터 폴더의 이름으로 씁니다: en, fr, de, es, it, ja, ko, zh-CN, zh-TW. 비워 두면 pa3ds가 세이브에서 읽은 게임의 언어를 따릅니다. 그것이 틀린 경우에 설정하세요. 예를 들어 다른 언어 위에 팬 번역을 입힌 경우입니다. pa3ds가 세이브를 읽을 수 없는 썬·문에서는 이 설정이 게임의 언어이기도 합니다.
setting-pa3ds-audio-tones = 신호음 상대 음량
setting-pa3ds-audio-tones-about = 지도의 커서나 터치 모드가 움직일 때 나는 짧은 신호음의 크기입니다. 신호음은 커서가 위에 있을수록 높고, 오른쪽에 있을수록 오른쪽에서 들려 커서의 위치를 알려 줍니다. 1은 게임과 같은 크기, 0.5는 절반, 1.5는 더 크게, 0은 소리 없음. 설정 창에서는 퍼센트로, 100이 게임과 같은 크기입니다.
setting-pa3ds-audio-footsteps = 발소리 상대 음량
setting-pa3ds-audio-footsteps-about = 거리, 바닥, 동굴처럼 게임이 발소리를 내지 않는 곳에서 pa3ds가 내는 발소리의 크기입니다. 1은 게임이 낼 크기, 0.5는 절반, 1.5는 더 크게, 0은 소리 없음. 설정 창에서는 퍼센트로, 100이 게임과 같은 크기입니다.
setting-pa3ds-audio-ticks = 시계 째깍 소리 상대 음량
setting-pa3ds-audio-ticks-about = 포켓파를레의 게임처럼 게임에 시계가 있을 때, 1초마다 나는 짧은 신호음의 크기입니다. 시계 자체는 30초마다, 그리고 마지막 몇 초 동안 말합니다. 1은 게임과 같은 크기, 0.5는 절반, 1.5는 더 크게, 0은 소리 없음으로 기본값입니다. 설정 창에서는 퍼센트로, 100이 게임과 같은 크기입니다.
setting-pa3ds-audio-feedback_sounds = 피드백 효과음 상대 음량
setting-pa3ds-audio-feedback_sounds-about = 키로 한 일을 알리는 게임 자체의 짧은 효과음의 크기입니다. 터치 모드가 드래그로 잡고 놓을 때 나는 박스의 포켓몬을 집고 내려놓는 소리, 그리고 너츠를 따자의 이번 판에 없는 나무열매처럼 할 일이 없는 키에 나는 게임의 오류음입니다. 0이면 pa3ds가 소리 대신 "잡고 있음", "놓음"과 그 이유를 말합니다. 1은 게임과 같은 크기, 0.5는 절반, 1.5는 더 크게. 설정 창에서는 퍼센트로, 100이 게임과 같은 크기입니다.
setting-pa3ds-navigation-positions = 목록 위치 알리기
setting-pa3ds-navigation-positions-about = "문, 3개 중 1"처럼, 대상이 목록에서 몇 번째인지 말합니다.
setting-pa3ds-navigation-position_first = 방향보다 목록 위치 먼저
setting-pa3ds-navigation-position_first-about = 가는 길보다 목록 위치를 먼저 말합니다: "문, 3개 중 1, 북쪽 1". false이면 "문, 북쪽 1, 3개 중 1"로 말합니다.
setting-pa3ds-navigation-turn_by_turn = 단계별 길 안내
setting-pa3ds-navigation-turn_by_turn-about = P로 길을 찾은 뒤, 꺾는 곳에 이를 때마다 말합니다. false이면 길을 한 번만 말하고, P를 다시 누르면 지금 선 곳부터 말합니다.
setting-pa3ds-navigation-guide_to_hidden = 도감내비로 찾은 포켓몬 안내
setting-pa3ds-navigation-guide_to_hidden-about = 도감내비로 찾던 포켓몬을 찾았을 때, 또는 도감내비가 열려 있고 터치 모드일 때 도감내비가 스스로 포켓몬을 찾았을 때, 목록이 무엇에 있었든 P처럼 그곳까지 길을 찾고 P를 그 포켓몬에 둡니다. 에리어내비가 열려 있을 때 찾은 포켓몬은 그대로 둡니다.
setting-pa3ds-navigation-track_boulders = 큰 바위 이동 추적
setting-pa3ds-navigation-track_boulders-about = "큰 바위, 동쪽 2, 북쪽 1 이동함"처럼, 큰 바위가 그 방의 처음 자리에서 얼마나 움직였는지 말합니다. 그래서 어느 바위를 밀었는지 알 수 있습니다. 게임은 각 바위가 지금 어디 있는지만 보여 주고, 처음에 어디 있었는지는 보여 주지 않습니다.
setting-pa3ds-navigation-hide_door_signs = 문 표지판 숨기기
setting-pa3ds-navigation-hide_door_signs-about = 보라힐즈의 "13호 암페어의 집"처럼, 옆에 있는 문의 이름만 적힌 표지판은 목록에서 뺍니다. 체육관 표지판처럼 그 이상을 말하는 표지판은 남습니다.
setting-pa3ds-navigation-show_y = 높이 알리기
setting-pa3ds-navigation-show_y-about = C로 지금 선 곳을, Shift와 C로 고른 대상의 위치를 말할 때, x와 z 다음에 y로 높이도 말합니다: "x 71, z 35, y 20". 단위는 게임 자체의 것으로 한 칸의 폭이 18이며, 썬·문처럼 pa3ds가 높이를 모르는 곳에서는 뺍니다.
setting-pa3ds-navigation-sort_by_distance = 거리순 정렬
setting-pa3ds-navigation-sort_by_distance-about = 목록 모드의 목록을 가까운 순으로 늘어놓습니다. 걸으면 순서가 바뀝니다. false이면 게임 자체의 표 순서대로 각 대상이 늘 같은 자리에 있어서, 어느 집에 들어가 봤는지 그 자리로 알 수 있습니다. 방향 모드는 어느 쪽이든 가까운 순입니다.
setting-pa3ds-touch-wrap = 끝에서 처음으로 돌아가기
setting-pa3ds-touch-wrap-about = 터치 모드에서 마지막 버튼에서 첫 버튼으로, 맨 아래 줄에서 맨 위 줄로 돌아갑니다. false이면 끝에서 멈추므로, O를 몇 번 누르면 언제나 맨 아래 줄에서 멈춥니다.
setting-pa3ds-touch-next_row = 다음 줄로 넘어가기
setting-pa3ds-touch-next_row-about = 터치 모드에서 J와 L이 줄 끝에 이르면 위나 아래 줄로 넘어갑니다. false이면 그 줄 안에 머무르므로, L을 몇 번 누르면 언제나 줄의 오른쪽 끝에서 멈춥니다.
setting-pa3ds-touch-drag = 드래그 방법
setting-pa3ds-touch-drag-about = Shift, 또는 keys.toml에서 드래그에 할당한 키를 누른 채로 드래그하고 키를 떼면 터치를 놓거나("hold"), 키를 한 번 눌러 집고 다시 눌러 놓습니다("toggle"). 드래그는 포켓파를레에서만 합니다.
setting-pa3ds-touch-drag-hold = Shift 누른 채로
setting-pa3ds-touch-drag-toggle = 한 번 눌러 집고, 다시 눌러 놓기
setting-pa3ds-amie-berry_first = 너츠를 따자에서 나무열매 먼저 말하기
setting-pa3ds-amie-berry_first-about = 나무열매를 원하는 포켓몬을, 나무열매를 줄 때의 순서대로 나무열매 다음 포켓몬으로 말합니다: "파인열매, 피카츄에게". false이면 포켓몬을 먼저 말합니다: "피카츄, 파인열매 원함".
setting-pa3ds-amie-one_at_a_time = 너츠를 따자에서 포켓몬을 하나씩 말하기
setting-pa3ds-amie-one_at_a_time-about = 나무열매를 원하는 포켓몬을 한 번에 하나씩 말하고, 나무열매를 주거나 말한 포켓몬이 사라지면 다음 포켓몬을 말합니다. false이면 나올 때마다 하나하나 말합니다.
setting-pa3ds-extras-conditions = 콘테스트 컨디션을 숫자로
setting-pa3ds-extras-conditions-about = 정보 화면과 포켓블록 케이스에서 게임이 별 모양으로 그리는 포켓몬의 콘테스트 컨디션을 숫자로 말합니다. 별은 누구에게나 보이지만 pa3ds는 별을 읽을 수 없어서 숫자만 알려 줄 수 있고, 숫자는 별보다 더 정확합니다.
setting-pa3ds-extras-hidden_items = 숨겨진 도구
setting-pa3ds-extras-hidden_items-about = 아직 찾지 않은 숨겨진 도구를 "숨겨진 도구"라는 이름으로 도구 목록에 넣습니다. 바닷속에서만("underwater"): 다우징머신을 쓸 수 없고, 앞을 보는 플레이어는 바위와 해초 사이의 반짝임을 볼 수 있는 곳입니다. 또는 모든 곳에서("all"): 게임이 누구에게도 알려 주지 않는 것까지 알려 줍니다. "off"는 하나도 넣지 않습니다.
setting-pa3ds-extras-hidden_items-off = 끄기
setting-pa3ds-extras-hidden_items-underwater = 바닷속
setting-pa3ds-extras-hidden_items-all = 모든 곳
settings-title = 모드 설정
settings-page-general = 일반
settings-page-speech = 음성
settings-page-audio = 오디오
settings-page-navigation = 내비게이션
settings-page-touch = 터치 모드
settings-page-amie = 포켓파를레
settings-page-extras = 추가 기능
settings-browse = 찾아보기(&B)...
settings-language-game = 게임 자체의 언어
settings-restart = 이 설정을 적용하려면 지금 pa3ds를 다시 시작할까요? 아니요를 고르면 다음에 pa3ds를 시작할 때 적용됩니다.
settings-unwritten = pa3ds가 { $path }에 쓸 수 없어서 아무것도 바뀌지 않았습니다.
dialog-apply = 적용(&A)
window-settings = 설정(&S)...

## As pa3ds starts: its keys, its settings and Azahar's

both = { $these } 및 { $last }
keys-taken = pa3ds가 { $controls } 사용
keys-applied = 키를 Azahar에 적용함
keys-waiting = 키를 적용하려면 Azahar를 닫고 pa3ds를 다시 시작하세요
keys-commented-out = { $file } 파일의 새 키를 주석 처리함: { $action }용 { $key }. { $key } 키는 이미 { $taken }에 쓰임
keys-edit-to-choose = { $file } 파일을 편집해서 선택하세요
file-unreadable = { $file } 파일을 읽을 수 없음: { $problem }. 고치거나, 삭제하면 pa3ds가 새로 씁니다
azahar-settings-applied = Azahar 설정 적용함
azahar-settings-waiting = 설정을 적용하려면 Azahar를 닫고 pa3ds를 다시 시작하세요
azahar-settings-missing = Azahar 설정을 찾을 수 없음: Azahar를 한 번 실행하고 닫은 뒤 pa3ds를 다시 시작하세요
azahar-folder-ask = Azahar 설정을 찾을 수 없습니다. azahar.exe가 있는 폴더를 고르거나, Azahar가 설치되어 있다면 취소하세요
azahar-folder-again = { $problem }, 다른 폴더를 고르거나 취소하세요
azahar-folder-no-exe = { $folder } 폴더에 azahar.exe가 없음
azahar-folder-not-portable = { $folder } 폴더의 azahar.exe 옆에 user 폴더가 없어서, 이 Azahar는 설정을 옆이 아니라 AppData에 저장함

## What a thing on the map is, where neither the game nor a name says

kind-door = 문
kind-way-out = 출구
kind-way-down = 내려가는 길
kind-conveyor-belt = 컨베이어 벨트
kind-switch = 스위치
kind-warp = 워프
kind-path = 길
kind-edge = 가장자리
kind-stairs = 계단
kind-stairs-up = 올라가는 계단
kind-stairs-down = 내려가는 계단
kind-passage = 통로
kind-entrance = 입구
kind-teleporter = 텔레포터
kind-elevator = 엘리베이터
kind-sign = 표지판
kind-flag = 깃발
kind-object = 물건
kind-person = 사람
kind-item-ball = 떨어진 도구
kind-hidden-item = 숨겨진 도구
kind-barrier = 장벽
kind-hole = 구멍
kind-dive-spot = 다이빙 포인트
kind-current = 해류
kind-arrow-east = 동쪽 화살표 발판
kind-arrow-west = 서쪽 화살표 발판
kind-arrow-north = 북쪽 화살표 발판
kind-arrow-south = 남쪽 화살표 발판
kind-flat-ground = 평평한 땅
kind-open-ground = 트인 곳
kind-quiz-panel = 퀴즈 패널
kind-blue-spot = 파란 발판
kind-wall-ledge = 벽의 턱
kind-vine-up = 오르는 덩굴
kind-vine-down = 내려가는 덩굴
kind-swinging-vine = 건너는 덩굴
kind-gym-leader-spot = 체육관 관장 자리
kind-slope = 비탈
kind-bumpy-slope = 울퉁불퉁한 비탈
kind-rail = 레일
kind-ice = 얼음
kind-blank = 빈칸
kind-wall = 벽
kind-crumbled-cliff = 무너진 절벽

## Battle

battle-battler = { $name } 레벨 { $level } { $hp }
battle-share = { $percent }%

## Scenes

narration = 내레이션
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = 한국어
