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
# How far a thing stands above or below the player, in the game's units, 18 to a cell, where
# it is more than two cells' height: "ladder, 2 west, 54 above".
above = { $count } above
below = { $count } below
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
category-climbing = climbing
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
# A leg along a climbing wall's face, and the handholds climbed in place before or after one:
# "climb 3 up, 6 east", "climb 6 west, climb 3 down, 1 west" (spike 245).
how-climb = climb
p-climb-up = climb { $holds } up
p-climb-down = climb { $holds } down
# A leg down a climbing wall's slide, which carries the player once they step onto its top:
# "slide 5 south" (spike 245).
how-slide = slide
# A leg along a grind rail, skated onto from the end cell before it, and a run of them, by
# where it lands the player: "3 east, grind the rail 17 east" (spike 247).
how-grind-the-rail = grind the rail
p-grind = grind the rail { $bearing }
# A leg a conveyor belt carries the player along, and a run of them, by where it leaves
# them: "3 north, ride the belt 7 west 4 north" (X's Poké Ball Factory, spike A).
how-ride-the-belt = ride the belt
p-belt = ride the belt { $bearing }
# The step that grabs a vine and its ride, as one leg, said with the way faced to grab it and
# what the ride does: "3 north, face north, climb the vine up, 2 east" (spike 250).
how-climb-the-vine-up = climb the vine up
how-climb-the-vine-down = climb the vine down
how-swing-on-the-vine = swing on the vine
p-vine = face { $way }, { $how }
# How many more vines a way rides past its first, said after it in place of the rest: "face
# north, climb the vine up, then 16 more vines" (spike 250).
p-more-vines = { $count ->
    [one] then 1 more vine
   *[other] then { $count } more vines
}
# How many more teleporters a way takes past its first, said after it in place of the rest: "6
# north, teleporter, then 3 more teleporters" (spike 258).
p-more-teleporters = { $count ->
    [one] then 1 more teleporter
   *[other] then { $count } more teleporters
}
# A run of steps up or down a ramp, said as one leg by the way it starts, the key held all along
# it: "up the ramp 25 south-east" (spike 248).
how-up-the-ramp = up the ramp
how-down-the-ramp = down the ramp
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
# A leg that crosses into another place, said with it: "10 east into Route 117" (Orion,
# 2026-10-24).
p-into = { $leg } into { $place }
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
# A message on the bottom screen waiting for a tap to go on, as Super Training's welcome: its ▼.
touch-continue = continue
# A tap asked for where pa3ds cannot tap, and one Azahar is not listening for.
touch-off = touch is off
touch-not-listening = Azahar is not taking touches from pa3ds
# Shift and U holding a touch on a button, and Shift let go of, with the touch where it was
# carried to.
touch-holding = holding { $target }
touch-dropped = let go on { $target }

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
# A tile of Pokémon-Amie's Tile Puzzle, by its row and column (slot-grid), and where the piece of
# the picture it shows belongs: "row 1, column 2, in place", "row 1, column 2, goes to row 3,
# column 4". The game says neither.
puzzle-home = { $tile }, in place
puzzle-goes = { $tile }, goes to { $home }
# Moving to the next tile says only the row or the column that changed, in place of slot-grid:
# "column 3, in place".
puzzle-row = row { $row }
puzzle-column = column { $column }
# Tile Puzzle's picture, described once a puzzle where descriptions are on: three of the player's
# Pokémon, said left to right, "Metagross on the left, Pikachu Rock Star in front, Blaze on the
# right". The game says nothing of it.
puzzle-picture-left = { $pokemon } on the left
puzzle-picture-front = { $pokemon } in front
puzzle-picture-right = { $pokemon } on the right
# The tile tapped first, which the next tap swaps with.
puzzle-picked = { $tile }, picked

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
# pa3ds's own window, opened when pa3ds is double-clicked: its title, and its menus and their
# items. The letter after & is the one that opens it, underlined, with Alt on a menu.
window-title = Pokémon Access 3DS
window-menu-mod = &Mod
# The Rust port's alone, which updates itself; the Python has no updater.
window-updates = Check for &updates
window-exit = E&xit
window-menu-azahar = &Azahar
window-azahar-portable = Download Azahar (&portable)
window-azahar-installer = Download Azahar (&installer)
window-menu-help = &Help
# Each opens a page of the manual in the browser.
window-documentation = &Documentation...
window-keys = &Key commands...
window-known-issues = Known &issues...
window-translating = &Translating...
window-changelog = &Changelog...
window-settings-guide = &Settings guide...
window-issues = &Report an issue on GitHub

## The settings: each one's name in the settings dialog, and what it does, said after the
## name there and above the setting in config.toml
setting-pa3ds-keys = Answer mod keys
setting-pa3ds-keys-about = Answer pa3ds's keys and bind Azahar's buttons, as keys.toml says.
setting-pa3ds-poll_rate = Reads per second
setting-pa3ds-poll_rate-about = How many times a second to read the game: 60, the game's own frame rate; 30 for a slower machine.
setting-pa3ds-azahar_folder = Portable Azahar folder
setting-pa3ds-azahar_folder-about = The folder a portable Azahar is in, the one with azahar.exe and a user folder; empty for an Azahar installed in the usual way, whose settings are in AppData. pa3ds fills it in when you choose the folder it asks for.
setting-pa3ds-hide_window = Hide the mod window while Azahar is open
setting-pa3ds-hide_window-about = Hide pa3ds's window, the one a double-clicked pa3ds opens, while Azahar is open, so nothing of pa3ds's sits in Alt+Tab between you and the game; false keeps it in sight, for OBS to record pa3ds's sounds through it.
setting-pa3ds-ui-pauses = Pauses in screen text
setting-pa3ds-ui-pauses-about = Put pauses between pieces of screen text said together: a colon after a label, as in "OT: Raiden", and a comma between the rest.
setting-pa3ds-ui-interrupt = Menus interrupt speech
setting-pa3ds-ui-interrupt-about = Cut off speech when a menu's cursor moves, instead of queueing; pa3ds's own keys always do.
setting-pa3ds-ui-unfocused = Speak while Azahar is in the background
setting-pa3ds-ui-unfocused-about = Keep saying what the game shows while another window is in front; otherwise pa3ds is quiet then, but for its own keys and messages.
setting-pa3ds-ui-descriptions = Scene descriptions
setting-pa3ds-ui-descriptions-about = Describe the scenes the game shows without words, such as Groudon waking. Each description is said after the line of text the scene follows. Only some scenes are described so far.
setting-pa3ds-ui-language = Language
setting-pa3ds-ui-language-about = The language pa3ds says its own words in, as the name of a folder of its data: en, fr, de, es, it, ja, ko, zh-CN or zh-TW. Empty follows the game's own language, which pa3ds reads from the save; set it where that is wrong, as for a fan translation over another language, and in Sun and Moon, whose save pa3ds cannot read, where it is the game's language too.
setting-pa3ds-audio-tones = Relative volume for tones
setting-pa3ds-audio-tones-about = How loud the short tone is as a map's cursor or touch mode moves, higher the further up and panned the further right, to say where the cursor is: 1 as loud as the game, 0.5 half as loud, 1.5 louder, 0 none; in the settings dialog, in percent, 100 as loud as the game.
setting-pa3ds-audio-footsteps = Relative volume for footsteps
setting-pa3ds-audio-footsteps-about = How loud pa3ds's own footsteps are on the ground where the game plays none, as streets, floors and caves: 1 as loud as the game would play them, 0.5 half as loud, 1.5 louder, 0 none; in the settings dialog, in percent, 100 as loud as the game.
setting-pa3ds-audio-ticks = Relative volume for clock ticks
setting-pa3ds-audio-ticks-about = How loud a short tone is at each second of a game's clock, as in Pokémon-Amie's games; the clock itself is said each half minute and in its last seconds: 1 as loud as the game, 0.5 half as loud, 1.5 louder, 0 none, as it ships; in the settings dialog, in percent, 100 as loud as the game.
setting-pa3ds-navigation-positions = Announce list position
setting-pa3ds-navigation-positions-about = Say where a thing is in the list, as in "door, 1 of 3".
setting-pa3ds-navigation-position_first = List position before directions
setting-pa3ds-navigation-position_first-about = Say the place in the list before the way to it, "door, 1 of 3, 1 north"; false says "door, 1 north, 1 of 3".
setting-pa3ds-navigation-turn_by_turn = Turn-by-turn directions
setting-pa3ds-navigation-turn_by_turn-about = Once P has found the way, say each turn as it is reached; false says the way once, and P again says it from where you stand.
setting-pa3ds-navigation-guide_to_hidden = Guide to DexNav finds
setting-pa3ds-navigation-guide_to_hidden-about = When a DexNav search finds the Pokémon searched for, or the DexNav finds one by itself while it is open and you are in touch mode, find the way to it as P does and keep P on it, whatever the list was on; one found while the AreaNav is open is left alone.
setting-pa3ds-navigation-track_boulders = Track boulder movement
setting-pa3ds-navigation-track_boulders-about = Say how far a big boulder has moved from where its room starts it, as in "big boulder, 2 east, moved 1 north", so you know which you have pushed; the game shows where each boulder is, never where it started.
setting-pa3ds-navigation-hide_door_signs = Hide door signs
setting-pa3ds-navigation-hide_door_signs-about = Leave a sign out of the list when all it says is the name of a door beside it, as Mauville Hills' "Apt. 13 Wattson's Home"; a sign that says more, as a Gym's does, stays.
setting-pa3ds-navigation-show_y = Announce height
setting-pa3ds-navigation-show_y-about = Say how high as well, as y after x and z, when C says where you stand and shift and C where the thing chosen is: "x 71, z 35, y 20"; in the game's own units, 18 to a square's width, and left out where pa3ds does not know it, as in Sun and Moon.
setting-pa3ds-navigation-sort_by_distance = Sort by distance
setting-pa3ds-navigation-sort_by_distance-about = List mode's list nearest first, which changes as you walk; false keeps each thing in one place in it, the order the game's own tables have, so you can tell which house you have been in by where it is; direction mode is nearest first either way.
setting-pa3ds-extras-conditions = Contest conditions as numbers
setting-pa3ds-extras-conditions-about = Say a Pokémon's contest conditions as numbers where the game draws them as a star, on the summary and in the Pokéblock Case: the star is shown to everyone but pa3ds cannot read it, so its numbers are all it can give, and they are more exact than the star.
setting-pa3ds-extras-hidden_items = Hidden items
setting-pa3ds-extras-hidden_items-about = List the hidden items still to be found, as "hidden item", among the items: under the sea alone ("underwater"), where the Dowsing Machine cannot be used and a sighted player may see a glint among the rocks and the weed, or everywhere ("all"), which tells what the game keeps from everyone; "off" lists none.
# What each of the hidden items setting's words is called in the settings dialog.
setting-pa3ds-extras-hidden_items-off = Off
setting-pa3ds-extras-hidden_items-underwater = Under the sea
setting-pa3ds-extras-hidden_items-all = Everywhere
# The settings dialog: its title, its pages, and what is not a setting's.
settings-title = Mod settings
settings-page-general = General
settings-page-speech = Speech
settings-page-audio = Audio
settings-page-navigation = Navigation
settings-page-extras = Extras
settings-browse = &Browse...
# The first of the languages pa3ds can speak in: none chosen.
settings-language-game = The game's own language
# Asked once settings are applied.
settings-restart = Restart pa3ds now for these settings to take effect? If you choose No, they take effect the next time pa3ds starts.
settings-unwritten = pa3ds could not write { $path }, so nothing has changed.
dialog-apply = &Apply
window-settings = &Settings...
# This language's own name, in itself, as the settings dialog lists it among the others.
language-name = English

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
# The way off Coumarine Gym's top level, walked onto east, which drops the player to the
# ground (spike 250).
kind-way-down = way down
# Dewford Gym's belts (spike 132), and each way onto one of the Poké Ball Factory's (spike A).
kind-conveyor-belt = conveyor belt
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
# What the Dowsing Machine finds, listed under the hidden items setting.
kind-hidden-item = hidden item
kind-barrier = barrier
kind-hole = hole
kind-dive-spot = dive spot
kind-current = current
kind-flat-ground = flat ground
# Where one of Lumiose's streets ends on ground walked on the grid, as its plazas are (spike 251).
kind-open-ground = open ground
# The small square in each of Prism Tower's quiz rooms that asks Bonnie's question, and asks it again (spike 252).
kind-quiz-panel = quiz panel
# A run of the blue spots a climbing wall is climbed from, and one on a ledge at its side, where
# the climber steps off it: the Gym's guide says "those blue spots" (spike 245).
kind-blue-spot = blue spot
kind-wall-ledge = wall ledge
# An end of one of Coumarine Gym's vines, by what its ride does from there: up a level, down
# one, or across a gap on one (spike 250).
kind-vine-up = vine up
kind-vine-down = vine down
kind-swinging-vine = swinging vine
# Where Shalour Gym's leader comes out to meet the player, at the centre's edges (spike 247).
kind-gym-leader-spot = gym leader's spot
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
