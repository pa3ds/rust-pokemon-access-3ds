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
category-currents = 해류
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
p-current = 해류 타기 { $bearing }
p-to-push = { $way } 방향으로 밀려면:
p-nudge = 조금 { $way }
p-nudge-then = { $nudge }, 그다음
p-lined-up = 정렬됨
p-then-cant-continue = 그다음 더 갈 수 없음
p-cant-continue = 더 갈 수 없음
p-in-the-way = 길을 막는 { $thing }
p-surface = 수면으로 올라가 { $bearing } 다이빙 포인트로
p-and-ice-cleared = 그리고 얼음이 모두 깨짐
p-ice-cleared = 얼음 깨짐
p-no-path-to = { $thing }까지 경로 없음
p-no-route-in-game = 이 게임에는 아직 경로 없음
p-no-route-on-streets = 이 거리에는 경로 없음
p-gone = { $thing } 사라짐

## Touch mode

touch-nothing = 터치할 것 없음
touch-off = 터치 꺼짐
touch-not-listening = Azahar가 pa3ds의 터치를 받지 않음

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

## What pa3ds says of itself

app-waiting = 게임을 기다리는 중
app-ready = pa3ds 준비 완료
app-restarted = pa3ds 다시 시작됨
app-reader-error = 읽기 오류
dialog-ok = 확인
dialog-cancel = 취소

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
kind-barrier = 장벽
kind-hole = 구멍
kind-dive-spot = 다이빙 포인트
kind-current = 해류
kind-flat-ground = 평평한 땅
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
