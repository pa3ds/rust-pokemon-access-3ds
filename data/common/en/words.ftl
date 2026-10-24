### What pa3ds says in words of its own, every game alike: the ways and distances on a map, the
### navigation keys' answers, P's guidance, touch mode, and what pa3ds says of itself (ADR-0040).
### A game's own `words.ftl` is read before this one. The game's own words, a place's name, a
### Pokémon's, never come from here: they are read from the player's ROM.
###
### A message's values are inserted as `{ $name }`: a number, or words pa3ds or the game has
### already put together, "Route 101". Each message says what it is said of, and the code says
### where (`own("steps")`).

## Ways and distances

# The ways of the compass, as a leg of a way, a direction key's answer and a bearing say them.
# North is up on the pad (spike 119).
way-north = north
way-east = east
way-south = south
way-west = west
way-north-east = north-east
way-south-east = south-east
way-south-west = south-west
way-north-west = north-west
# The ways of the player's own frame, soaring and on Lumiose's streets: "9 ahead 8 left".
way-ahead = ahead
way-behind = behind
way-left = left
way-right = right
# So many steps one way: a leg of a way, "5 east", and half of a bearing, "3 east 2 north".
steps = { $count } { $way }
# A leg walked with a move, "Surf 3 south": the move's name is the game's.
leg-with = { $how } { $steps } { $way }
# Where something is when it is where the player stands.
here = here
# A leg on Lumiose's rails, where up walks the way the camera looks and left and right turn the
# player (spike 105): "hold right, 5 forward, 3 back".
rail-forward = { $steps } forward
rail-back = { $steps } back
rail-hold-right = hold right
rail-hold-left = hold left
# A camera a tap left pointing between two ways, which a hold either way settles.
rail-angled = camera angled, hold left, hold right
# A leg in Glittering Cave, where the camera is behind the player and left, right and
# down turn them with it (spike 239): "turn left, 5 forward".
turn-right = turn right
turn-left = turn left
turn-around = turn around

## The list of what is on the map, and the navigation keys' answers

# The categories the list is walked by (`things.CATEGORIES`), and soaring's (ORAS's
# `words.ftl`): "exits, 3".
category-all = all
category-exits = exits
category-signs = interactables
category-obstacles = obstacles
category-people = people
category-items = items
category-berries = berries
category-ash = ash
category-bike = bike
category-riding = riding
category-currents = currents
category-pokemon = pokémon
# A category chosen, with how many things are in it.
key-category = { $category }, { $count }
# Where a thing is in its category, "1 of 3", and in a list of ways a boulder can be pushed.
position = { $number } of { $count }
# Nothing at all on the map's list that way, or nothing of the category chosen: "no exits north".
key-nothing-way = nothing { $way }
key-none-way = no { $category } { $way }
key-nothing-here = nothing here
key-no-map = no map
key-cant-walk-now = can't walk now
# The ways a key steps through the list: list mode, direction mode and touch mode, which names
# the first thing to touch as it comes on.
key-list-mode = list mode
key-direction-mode = direction mode
key-touch-mode = touch mode, { $first }
# Ctrl and P, leaving out what Shift and P finds no way to, and putting it back.
key-unreachable-hidden = unreachable hidden
key-unreachable-shown = unreachable shown
# Shift and P with no way being followed, and with one, which it drops.
key-path-off = path off
key-no-path = no path
# The battle keys out of a battle.
key-no-battle = no battle
# Comma and period on something with only one way to it.
key-one-way = one way to it
# An exit's arrival somewhere else on the map: "teleporter, 3 east, out at 10 north".
out-at = out at { $bearing }
# What an exit is called, then where it leads, when that is another place, the game's own word
# for it: "edge to Route 101".
leads-to = { $thing } to { $place }
# A thing's coordinates spanning several cells: "x 80 to 119".
span = { $low } to { $high }
# One of two or more: "north, east or west".
either = { $these } or { $last }

## Big boulders

# The way a big boulder is pushed, chosen with comma and period: "push east, 2 of 2".
key-push = push { $way }
boulder-into-hole = { $way } into a hole
# What a big boulder can do, after where it is (Orion, 2026-10-11): "moveable north or east".
boulder-moveable = moveable { $ways }
boulder-immoveable = immoveable
# How far a big boulder has moved from where its room starts it, under `track_boulders`.
boulder-moved = moved { $bearing }

## Renaming

rename-title = Rename
# The box's words, for a thing: "Sign, 1 north. Called: Signpost 1".
rename-label = { $thing }, { $bearing }. { $name }
# And for whoever last spoke.
rename-speaker-label = The last speaker. { $name }
rename-called = Called: { $name }
rename-not-named = Not named yet
# The box's checkbox, for somebody or something that can be named alone, apart from everyone
# drawn as it (Orion, 2026-10-11).
rename-only-this-one = Only this one
# Said back once the box closes: what was renamed, as it was called when the box opened.
rename-now = { $thing } is now { $name }
rename-no-name = { $thing } has no name
rename-the-last-speaker = the last speaker
rename-nothing = nothing to rename
rename-cannot = { $thing } cannot be renamed
rename-already = already renaming
rename-nobody-spoke = nobody has spoken

## P: the way to the thing chosen, and guiding the player along it

# A leg a move or a bike takes the player along, where the game's own name for it cannot be read:
# where it goes (`guide.OWN_MOVE_WORDS`). "Surf" and the bikes' names are the game's.
how-water = water
how-bush = bush
how-boulder = boulder
how-rock = rock
how-waterfall = waterfall
how-slope = slope
how-bumpy-slope = bumpy slope
# A step into one of Fortree Gym's rotating doors, which turns it: "push 1 west" (spike 161).
how-push = push
# A leg a current carries the player along (spike 208), and a run of them, by where it leaves
# them: "Surf 3 south, ride the current 28 west 4 north" (Orion, 2026-10-13).
how-ride-the-current = ride the current
p-current = ride the current { $bearing }
# The step off a ridden Pokémon, from flat ground onto plain ground, and the rest of its leg: "get
# off 3 north" (spike 238). Getting on is said with the Pokémon's own name: "Rhyhorn 1 east".
how-get-off = get off
# The step into a rock the ridden Pokémon smashes with A: "smash 1 east" (spike 238).
how-smash = smash
# The way chosen to push a big boulder, before the way to its side: "to push east: 3 west".
p-to-push = to push { $way }:
# A step to line up with the next leg, where a wall's corner would catch the player: "a little
# west, then 5 north" (spike 202), and once they have.
p-nudge = a little { $way }
p-nudge-then = { $nudge }, then
p-lined-up = lined up
# A way that goes as near as it can: "5 east, then can't continue, Team Magma Grunt in the way".
p-then-cant-continue = then can't continue
p-cant-continue = can't continue
p-in-the-way = { $thing } in the way
# Under the sea, where the way goes on from another dive spot (spike 165).
p-surface = surface and go to the dive spot { $bearing }
# A way over a room of thin ice, which cracks every tile on the way (spike 167).
p-and-ice-cleared = and the ice is cleared
p-ice-cleared = ice cleared
p-no-path-to = no path to { $thing }
# What P says where it finds no way at all: in a game whose ground is not read yet, and off the
# streets of Lumiose the game has loaded.
p-no-route-in-game = no route in this game yet
p-no-route-on-streets = no route on these streets
# What the way led to has gone: a person walked off, a ball was picked up.
p-gone = { $thing } gone

## Touch mode

touch-nothing = nothing to touch
# A tap asked for where pa3ds cannot tap, and one Azahar is not listening for.
touch-off = touch is off
touch-not-listening = Azahar is not taking touches from pa3ds

## What the field shows without words

# The Dowsing Machine's lights, said as they change: how near a hidden item is, never where
# (spike 145).
dowsing = dowsing { $light }
dowsing-blue = blue
dowsing-green = green
dowsing-yellow = yellow
dowsing-red = red
dowsing-nothing = nothing
# Whether the decoration held over a Secret Base's cell fits there, before the cell's words
# (spike 154).
decorating-placeable = placeable
decorating-not-placeable = not placeable

## The Pokédex's habitat map (spike 152)

# A place drawn as the Pokémon's, then how it is found there, by the icons the top screen draws
# in colour (Orion's words, 2026-10-01). The DexNav's icons say how they are met in the same
# words: "Marill water" (2026-10-20).
habitat-found = Found in { $place }
habitat-grass = grass
habitat-water = water
habitat-fishing = fishing

## Super Training and Pokémon-Amie (spikes 198 and 200)

# A goal of a regimen's, which a tap shoots at: the game's dialogue speaks of "the white goals"
# and "red goals" and names none.
training-goal = goal
training-red-goal = red goal
# A ball of the Balloon Bot's in flight, said interrupting: the game says nothing.
training-shot-coming = shot coming
# The clock, every half minute and at ten seconds: "1:30 left".
training-left = { $time } left
# Touch mode's one gesture that is not a tap, a slide back and forth over the Pokémon, the
# manual's word: "stroke Sceptile".
amie-stroke = stroke { $pokemon }

## What pa3ds says of itself

app-waiting = Waiting for the game
app-ready = pa3ds ready
# Said where "pa3ds ready" would be, by a pa3ds started over one that was running (ADR-0039).
app-restarted = pa3ds restarted
# A reader that failed, logged and snapshotted (ADR-0019).
app-reader-error = Reader error
# The rename box's buttons.
dialog-ok = OK
dialog-cancel = Cancel
# pa3ds's own window, opened when pa3ds is double-clicked: its title and its buttons.
window-title = Pokémon Access 3DS
window-issues = Report an issue on GitHub
window-azahar-portable = Download Azahar (portable)
window-azahar-installer = Download Azahar (installer)
window-exit = Exit

## As pa3ds starts: its keys, its settings and Azahar's

# Two or more things, as a list is heard: "c, d and the C-stick".
both = { $these } and { $last }
# The emulator's buttons pa3ds's keys sit on, the buttons by Azahar's names for them.
keys-taken = pa3ds takes { $controls }
keys-applied = keys applied to Azahar
keys-waiting = close Azahar and start pa3ds again to apply your keys
# A key pa3ds gained that the player's file already uses for something else (ADR-0022).
keys-commented-out = new key in { $file } commented out: { $key } for { $action }, because { $key } is already { $taken }
keys-edit-to-choose = edit { $file } to choose
# A file of the player's that will not parse; the problem is the TOML reader's own words.
file-unreadable = { $file } could not be read: { $problem }. Fix it, or delete it for pa3ds to write a new one
# The Azahar settings pa3ds writes (ADR-0026).
azahar-settings-applied = Azahar settings applied
azahar-settings-waiting = close Azahar and start pa3ds again to apply its settings
azahar-settings-missing = Azahar's settings were not found: start Azahar once, close it, and start pa3ds again
# The folder picker's title, read out with it, asking for a portable Azahar's folder, and again
# after a folder that is not one.
azahar-folder-ask = Azahar's settings were not found. Choose the folder azahar.exe is in, or cancel if Azahar is installed
azahar-folder-again = { $problem }, choose another folder or cancel
azahar-folder-no-exe = { $folder } has no azahar.exe in it
azahar-folder-not-portable = { $folder } has no user folder beside its azahar.exe, so this Azahar keeps its settings in AppData and not beside it

## What a thing on the map is, where neither the game nor a name says

# Each kind is said as the thing's name where it has none: "door, 3 north". The game's own word
# for a kind comes first where it has one ("PC"), and so do the use tiles and furniture of each
# game's `sprites.ftl`.
kind-door = door
kind-way-out = way out
kind-warp = warp
kind-path = path
kind-edge = edge
kind-stairs = stairs
kind-stairs-up = stairs up
kind-stairs-down = stairs down
# A warp walked through on a level with where it leads (spike 159).
kind-passage = passage
kind-entrance = entrance
kind-teleporter = teleporter
kind-elevator = elevator
kind-sign = sign
kind-flag = flag
kind-object = object
kind-person = person
kind-item-ball = item ball
kind-barrier = barrier
kind-hole = hole
kind-dive-spot = dive spot
kind-current = current
kind-flat-ground = flat ground
kind-slope = slope
kind-bumpy-slope = bumpy slope
kind-rail = rail
# Sootopolis Gym's rooms of thin ice, by where the way over each comes out (spike 167).
kind-ice = ice
# What the Secret Base's decorating cursor is on, where no decoration is (spike 154).
kind-blank = blank
kind-wall = wall
kind-crumbled-cliff = crumbled cliff

## Battle

# Each side's Pokémon, its name and gender sign, its level, and its HP: yours as the game shows it,
# "20/20", the enemy's as a share of its bar, since a bar is all the game shows (ADR-0015).
battle-battler = { $name } L{ $level } { $hp }
battle-share = { $percent }%

## Scenes

# Who a scene's description is said as, named as a speaker is when it takes over from whoever
# spoke last: "Narration: Groudon rises." (Orion, 2026-10-12).
narration = Narration
