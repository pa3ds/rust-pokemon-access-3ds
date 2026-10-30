# Keys

pa3ds's keys work only while Azahar's game window is in front, and they never reach the game.
They do nothing while your screen reader's key (Insert or Caps Lock) or a Windows key is held.
When one of them is also bound to an Azahar button, pa3ds says so as it starts.

## The map's list

pa3ds lists what is on the map in categories: all, exits, interactables, obstacles, people, items,
berries, ash, bike, riding, climbing, currents and pokémon, and while soaring, towns, routes, landmarks and mirage
spots. A category with nothing in it is skipped. Each thing is said with where it is from you:
"door, 1 of 3, 3 east 2 north". A thing more than two squares' height above or below you says how
far, in the game's units, 18 to a square: "item ball, 1 north, 90 below". North is the way up
walks you: where the camera is turned, as in Parfum Palace's garden in X, round Prism Tower in
Lumiose City or on the Battle Resort's beach, the directions turn with it. Where the camera stays
behind you and left, right and down turn you, as in X's Glittering Cave, things are said from the
way you face, "9 ahead 8 left", and `P` says the turns, "turn left, 5 forward". Each time the way
up walks you turns a quarter, pa3ds plays a short tone: high for north, low for south, in your
right ear for east and your left for west.

Exits side by side into the same place are one exit. On Lumiose City's streets, where a street
ends on a plaza you walk freely, the exits list that end as "open ground": walk onto the plaza and
its own exits are listed. In X and Y, a gate is said with the place beyond it: "Lumiose Gate to
Route 4".

In Laverre Gym, in X and Y, the interactables list each teleporter with where it puts you, and `P`
leads you through them, saying the way as far as the first and how many more.

In the Poké Ball Factory, in X and Y, the interactables list each place you step onto a conveyor
belt as "conveyor belt", and the switch that turns a belt round as "switch".

The list has three **modes**, and the apostrophe steps through them. During a battle or a scene
the map's keys say "no map", and the thing you had chosen is kept for when the map comes back.
The battle's keys work throughout.

**In every mode**

| Key | What it does |
|---|---|
| `'` (apostrophe) | Next mode: list mode, direction mode, touch mode. |
| `[` and `]` | Previous and next category. |
| `P` | Find the way on foot to the chosen thing, and guide you along it turn by turn. |
| `Shift+P` | The same, but also using Surf, Cut, Rock Smash, Strength, Waterfall and the bikes. It pushes a big boulder only where there is no way round it. |
| `Alt+P` | Stop guiding. |
| `Ctrl+P` | Leave out of the list everything `Shift+P` finds no way to from where you stand, or put it back. Everything is listed each time pa3ds starts. |
| `,` and `.` (comma and period) | Previous and next way to the chosen thing, where there is more than one, such as which way to push a big boulder. |
| `R` | Say your side of the battle again. In a contest, your place. |
| `Shift+R` | Say the enemy's side of the battle again. In a contest, everyone else's place. |
| `Shift+K` | Give the chosen thing a name of your own. |
| `Shift+J` | Give whoever spoke last a name of your own. |
| `C` | Say the square you are standing on, as x and z: x grows going east, z going south. |
| `Shift+C` | Say the chosen thing's squares the same way. |

**List mode** walks the whole list, nearest first.

| Key | What it does |
|---|---|
| `I` and `O` | Previous and next category. |
| `J` and `L` | Previous and next thing in the category. |
| `K` | The chosen thing again, from where you stand now. |

**Direction mode** looks one way at a time, up to 10 squares away.

| Key | What it does |
|---|---|
| `O` | The next thing to the north. |
| `;` (semicolon) | The next thing to the east. |
| `L` | The next thing to the south. |
| `K` | The next thing to the west. |

**Touch mode** walks the buttons on the bottom screen, in reading order. While a panel or a Yes
and No is up, it leaves out the buttons behind it that a tap no longer reaches. It works where the
game needs a touch: on the map with the PokéNav Plus, in the AreaNav, Pokémon-Amie and Super
Training. Elsewhere, in the Bag, the PC or a battle, the game's own buttons do everything a touch
does, and touch mode says "nothing to touch".

| Key | What it does |
|---|---|
| `J` and `L` | Previous and next button. |
| `I` and `O` | The button above and below: the nearest in the previous and next row. |
| `K` | The chosen button again. |
| `U` | Tap the chosen button. |
| `Shift` | In Pokémon-Amie, pick up the chosen button to drag it, with a short touch sound. Keep Shift down: `J`, `L`, `I` and `O` choose where to, and `K` says where it is. Let go of Shift to drop it there, with another sound. With the `drag` setting at `"toggle"`, press once to pick up and again to drop; with `feedback_sounds` at 0, "holding" and "let go on" are said instead of the sounds. Nothing else drags. A key of your own can drag too (Changing keys, below). |

In the AreaNav, touch mode offers each place on the map by name, and a tap chooses it.

When a message on the bottom screen waits for a tap, as Super Training's welcome does, touch
mode offers "continue", and a tap goes on with the message. The PlayNav's "previous" and "next"
turn the page instead, away from the message.

In Pokémon-Amie, touch mode offers to stroke your Pokémon, and the tap key strokes it. This is an
experiment. Its Poké Puffs button opens a bar of puffs, each said by its kind. To give one, hold
Shift on it and carry it to your Pokémon, and keep Shift down while your Pokémon eats it: let go
once you hear it finish and cry, a few seconds, longer for some Pokémon than others. In its Tile
Puzzle, each tile is said as a spreadsheet names a cell, its column a letter from A and its row a
number from 1, and then where the piece of the picture on it belongs: "B1, goes to D3", or "in
place". Tap a tile, then the tile where its piece belongs, to swap them. Over the finished picture there
is nothing to touch: B goes back. In its Berry Picker, touch mode offers the berries on the tree
by name, and each Pokémon that pops up wanting one, "Treecko wants Pinap Berry", which pa3ds also
says as it comes. Hold Shift on that berry, choose the Pokémon with `J`, `L`, `I` and `O`, and let
go of Shift: the berry goes straight to it. Nothing moves until you let go, so choosing past
another Pokémon is safe, but a wrong berry sends a Pokémon away. A Pokémon waits only about two
seconds. A tap knocks a berry off the tree for nobody. For a higher score, lower Azahar's speed
limit in its general settings: at 50 percent each Pokémon waits twice as long, and the round
lasts two minutes. Turn on realtime audio in Azahar's audio settings, or the sound stutters.

Berry Picker also has keys of its own, in any mode. `1` to `7` pick up a berry, in the game's own
order: Razz, Bluk and Pinap on Easy, then Wepear and Magost on Normal, and Durin and Nomel on Hard;
on Easy and Normal that is left to right. `8`, `9` and `0` are the round's three Pokémon, each
keeping its key all round. With a berry picked up, a Pokémon's key gives it the berry; with none,
it says what that Pokémon wants, so pressing `8`, `9` and `0` as the round starts says which is
which. Another berry's key picks that one up instead. A key with nothing to do, as `7` on Easy,
plays the game's error sound. Outside Berry Picker these keys are Azahar's as ever.

In a Super Training regimen, touch mode offers each goal the
Balloon Bot puts up: the tap key's first press says the goal, and each press after that shoots at
it, wherever it has moved. pa3ds says "shot coming" about a second before the bot's ball arrives:
move with the Circle Pad, or raise a shield with L.

## Finding the way

**Into another place.** A step of the way that takes you into another town or route names it:
"5 north into Route 101".

**Lining up.** The circle pad can leave you between two squares, and the game then refuses some
steps past the corner of a wall. Where that would stop you, `P` first says which way to step,
"a little west, then 2 north". Tilt the circle pad gently that way, and pa3ds says "lined up".

**Currents.** When a current catches you while you surf, pa3ds says where it will leave you.
`P` and `Shift+P` ride a current as one step of the way, and find the way again if one carries
you off it.

**Conveyor belts.** In the Poké Ball Factory a belt carries you to its end, round its corners,
whatever you press. `P` rides a belt as one step of the way, said by where it sets you down,
"ride the belt 7 west 4 north", and waits until it has. Where the belts as they run go nowhere
near what you chose, but would once the switch turned one round, `P` says "switch first" and leads
you to it; press A there, and `P` says the way on.

**Ice.** In Shoal Cave at low tide you slide across the ice until a wall or the floor stops you.
`P` says each slide as one step of the way, "slide 4 north": step that way once and let it carry
you.

**Jumping stones.** In Pokémon X, `P` crosses a mud pit by its stones: each press of the way hops
you two squares, onto the next stone.

**Riding Rhyhorn.** On Route 9, in X and Y, only a Rhyhorn crosses the spiky ground. The riding
category lists each patch of flat ground, where you get on the Rhyhorn by facing it and pressing A, and get off
with B. `P` walks you to the Rhyhorn, "Rhyhorn 1 east", across the spikes and off again, "get off
3 north". Rocks are obstacles: on the Rhyhorn, face one and press A to smash it, which `P` says as
"smash 1 east".

**Climbing walls.** In Cyllage Gym, in X and Y, the rock walls are climbed from the blue spots at
their foot. The climbing category lists the blue spots and each wall ledge, where you step off a
wall sideways. `P` climbs them: "climb 3 up, 6 east" is up the wall three handholds, then six east
along it, and "climb 3 down" is back down to the floor. Some walls are slid down instead: step onto
the top and the game carries you to the floor below, which `P` says as "slide 5 south". A slide
only goes down, so `P` takes the way round to go back up.

**Grind rails.** In Shalour Gym, in X and Y, you skate along the rails on roller skates. `P` says
the way to a rail's end, then the ride as one step of the way, "3 east, grind the rail 17 east":
skate onto the rail with the circle pad and it carries you to the far end. The interactables list
the Gym leader's spot, where Korrina comes out to meet you.

**Vines.** In Coumarine Gym, in X and Y, you climb the tree by its vines: face a vine and press A,
and it carries you up or down a level, or swings you across to the next platform. The climbing
category lists each end of a vine as "vine up", "vine down" or "swinging vine". `P` says the way to
a vine and the ride as one step of the way, "2 east, face north, climb the vine up": walk the
steps, press the way it says to face the vine, then press A. Where the way rides more vines, `P`
says it up to the first and how many more, "then 16 more vines"; press `P` again once you land.
From the top, the exits list the way down, which drops you to the ground.

**Prism Tower's Gym.** In Lumiose City, in X and Y, the exits list the lift in the lobby's
middle as "elevator": stand on it and walk north into its door. In each quiz room you arrive on
the elevator back down, and the interactables list the quiz panel three steps north of it: step
onto it to hear Bonnie's question, and again to hear it again. The three trainers at the back
stand in booths numbered 1, 2 and 3 from west to east. Talk to the trainer whose number is your
answer and battle them; a wrong answer means battling another one, and after the right one the
game takes you up to the next room. With the `descriptions` setting on, pa3ds describes the
picture each question shows.

**Ramps.** In the Tower of Mastery, in X and Y, a ramp winds up round the tower twice, past the
stairs on each landing to the top. `P` says each stretch of it as one step of the way, "8 west,
up the ramp 22 east": hold that key and the ramp carries you round its curve to the next
landing. The way to someone on the ground goes back down it first, "down the ramp 16 west".

**Big boulders** say which ways they can be pushed from where they stand, or "immoveable". Hold
the way for a moment to push one a square. Push one into a hole and both leave the list.

**Diagonal steps.** In Santalune Gym, in X and Y, `P` says a step along a diagonal thread of the
web as "1 north-east": tilt the circle pad that way, or hold both keys.

## Naming things

The game never names most people on the map, signs and doors, and pa3ds gives some of them a name
of its own. `Shift+K` names the chosen thing, and `Shift+J` whoever spoke last: type the name and
press Enter.

People and things drawn alike, such as every Youngster or every big boulder, share a name
unless you tick **Only this one**, a Tab away from the name and ticked as the box opens. A name
given to one alone stays with them wherever they walk or are pushed. Your names always win over
pa3ds's. They are saved in `user\names\<language>\`, a file for each game: to share them, send
the file.

## Changing keys

Keys are in `user\keys.toml`, written with the defaults on the first start. pa3ds adds the keys
later versions bring and never changes a line you have. To unbind a key, put a `#` in front of
its line: a line you delete is put back the next time pa3ds starts.

- `[pa3ds.navigation]` holds the keys of every mode. `[pa3ds.navigation.list]`,
  `[pa3ds.navigation.direction]` and `[pa3ds.navigation.touch]` hold one mode's, and win over it.
  A key can have modifiers: `"shift+k"`.
- `"drag"` has no key of its own: Shift drags. To drag with another key as well, add one under
  `[pa3ds.navigation.touch]`, such as `"d" = "drag"`. It works as Shift does, held or pressed
  as the `drag` setting says, and elsewhere says "nothing to drag".
- `[pa3ds.amie.berry_picker]` holds Berry Picker's own keys, `"first berry"` to `"seventh berry"`
  and `"first pokemon"` to `"third pokemon"`. pa3ds takes them only while Berry Picker runs,
  and there they win over the keys above.
- `[azahar]` rebinds Azahar's own buttons, and `[azahar.circle_pad]` and `[azahar.c_stick]` its
  two sticks. Changes apply the next time pa3ds starts with Azahar closed.

A new key whose default you already use for something else is added with a `#` in front of it,
so your key keeps doing what it did, and pa3ds says so as it starts. Give one of the two another
key, then take the `#` off the new line.
