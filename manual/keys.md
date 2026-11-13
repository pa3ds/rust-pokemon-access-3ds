# Keys

pa3ds's keys work only while Azahar's game window is in front, and they never reach the game.
They do nothing while your screen reader's key (Insert or Caps Lock) or a Windows key is held.
When one of them is also bound to an Azahar button, pa3ds says so as it starts.

## The map's list

pa3ds lists what is on the map in categories: all, exits, interactables, obstacles, people, items,
berries, ash, bike, riding, climbing, currents, arrows and pokémon, and while soaring, towns,
routes, landmarks and mirage spots. A category with nothing in it is skipped. Each thing is said
with where it is from you: "door, 1 of 3, 3 east 2 north". A thing more than two squares' height
above or below you says how far, in the game's units, 18 to a square: "item ball, 1 north, 90
below". North is the way up walks you: where the camera is turned, as in Parfum Palace's garden in
X, round Prism Tower in Lumiose City or on the Battle Resort's beach, the directions turn with it.
Where the camera stays behind you and left, right and down turn you, as in X's Glittering Cave,
things are said from the way you face, "9 ahead 8 left", and `P` says the turns, "turn left, 5
forward". Each time the way up walks you turns a quarter, pa3ds plays a short tone: high for north,
low for south, in your right ear for east and your left for west.

Exits side by side into the same place are one exit. On Lumiose City's streets, where a street
ends on a plaza you walk freely, the exits list that end as "open ground": walk onto the plaza and
its own exits are listed. In X and Y, a gate is said with the place beyond it: "Lumiose Gate to
Route 4". Lumiose's exits say where they go even where the game puts no sign up: the alleys onto
its five plazas by the plaza, "Rouge Plaza"; the Looker Bureau and Restaurant Le Wow by their
signs; and the doors the game never names by what is behind them, "back alley", "office
building", "photo shoot". In Lilycove City, the way into the hideout is the one of your own story,
Team Magma's in Omega Ruby and Team Aqua's in Alpha Sapphire.

In Laverre Gym, in X and Y, the interactables list each teleporter with where it puts you, and `P`
leads you through them, saying the way as far as the first and how many more.

In the Poké Ball Factory, in X and Y, the interactables list each place you step onto a conveyor
belt as "conveyor belt", and the switch that turns a belt round as "switch".

In Lysandre Labs, in X and Y, the arrows list each arrow tile by the way it points, "arrow west",
with where it leaves you, and the exits list each round pad as "teleporter", with where it puts
you.

## Modes

The list has three **modes**, and the apostrophe steps through them. During a battle or a scene
the map's keys say "no map", and the thing you had chosen is kept for when the map comes back.
The battle's keys work throughout.

**In every mode**

| Key | What it does |
|---|---|
| `'` (apostrophe) | Next mode: list mode, direction mode, touch mode. |
| `[` and `]` | Previous and next category. |
| `P` | Find the way on foot to the chosen thing, and guide you along it turn by turn. Where only `Shift+P` reaches it, `P` takes you as far as the spot where a move is needed and says every move and bike the way needs, and anyone standing in it: "then can't continue, requires Acro Bike and Surf, Team Magma Grunt in the way". |
| `Shift+P` | The same, but also using Surf, Cut, Rock Smash, Strength, Waterfall and the bikes. It pushes a big boulder only where there is no way round it. |
| `Alt+P` | Stop guiding. |
| `Ctrl+P` | Leave out of the list everything `Shift+P` finds no way to from where you stand, or put it back. Everything is listed each time pa3ds starts. |
| `,` and `.` (comma and period) | Previous and next way to the chosen thing, where there is more than one, such as which way to push a big boulder. |
| `R` | Say your side of the battle again. In a contest, your place. In Pokémon-Amie, your Pokémon's numbers, and in Head It, the score. On Super Training's page, your Pokémon's base stats and the kind of shot it fires, and in a regimen, the score, the time left and how full the energy meter is. |
| `Shift+R` | Say the enemy's side of the battle again. In a contest, everyone else's place. |
| `Shift+K` | Give the chosen thing a name of your own. |
| `Shift+J` | Give whoever spoke last a name of your own. |
| `C` | Say the square you are standing on, as x and z: x grows going east, z going south. In a Super Training regimen, where your Pokémon stands, as below. |
| `Shift+C` | Say the chosen thing's squares the same way. |
| `Y` | Say again the last thing pa3ds said, whatever said it. |

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

## Touch mode

Touch mode walks the buttons on the bottom screen, in reading order. While a panel or a Yes
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
| `Shift` | In Pokémon-Amie, pick up the chosen button to drag it, with the PC Box's sound for picking up a Pokémon. Keep Shift down: `J`, `L`, `I` and `O` choose where to, and `K` says where it is. Let go of Shift to drop it there, with another sound. With the `drag` setting at `"toggle"`, press once to pick up and again to drop; with `feedback_sounds` at 0, "holding" and "let go on" are said instead of the sounds. In a Super Training regimen, hold it on a goal to charge a shot, as below. Nothing else drags. You can move drag to another key (Changing keys, below). |

In the AreaNav, touch mode offers each place on the map by name, and a tap chooses it.

When a message on the bottom screen waits for a tap, as Super Training's welcome does, touch
mode offers "continue", and a tap goes on with the message. The PlayNav's "previous" and "next"
turn the page instead, away from the message.

On Super Training's page, touch mode offers the training bag hanging beside your Pokémon, by its
name, or "empty bag" when none is in use. A tap on the bag or on your Pokémon makes it hit the
bag, and a hit on the empty bag can find a new one. Each hit of a tap makes a sound of the
game's, with `feedback_sounds` on, so a tap that misses is silent. Your Pokémon also hits the bag
on its own about once a minute, and that's silent. When a bag is finished pa3ds says so, "Ralts
finished Core Training", with the game's "!" sound: tap your Pokémon to get what the bag gave.
When the Pokémon you've chosen is resting, pa3ds says so, "Ralts is resting": it won't do a
regimen until you tap it.

`R` on that page says how much of each base stat your Pokémon has trained, in fifths of how full
the game's chart is of it, the total, and the kind of shot it fires: "HP 1 of 5, Attack 1 of 5,
Defense 1 of 5, Sp. Atk 0 of 5, Sp. Def 0 of 5, Speed 1 of 5, total 1 of 5, blue shot".
With the `base_stats` Extra on, it says the points instead: see [Settings](settings.md#extras).

In Pokémon-Amie touch mode beckons your Pokémon, pets it with a stylus and gives it Poké
Puffs, and Amie and its three games have keys of their own: see [Pokémon-Amie](amie.md).

In a Super Training regimen you shoot at the goals a Balloon Bot puts up, until you have the
points before the clock runs out. Touch mode offers each goal:

- `U` the first time says the goal, and each press after that shoots at it, wherever it has
  moved.
- **A charged shot:** hold `Shift` on the goal, the drag key. The press itself fires an ordinary
  shot. After about two seconds pa3ds says "charged", and if you let go in the second and a half
  after that, it is a Charged Shot, worth three times the points. Hold on too long and it says
  "fizzled", and nothing more is fired. Let go then, and your next press fires nothing: it's the
  press after that one that shoots. Touch mode has to be on, and the goal chosen, before you
  press it. Choose another goal while you hold, and the press moves over to it in a fifth of a
  second: let go before it gets there and pa3ds lets go once it has, so the shot still goes to
  the goal you chose. The first tutorial, against Scatterbug, cannot charge.
- **A barrier:** some Bots, such as Graveler's at Level 2, raise a barrier, and while it is up
  there are no goals. Touch mode offers "barrier" in their place. One shot that lands on it
  brings it down, a plain shot as well as a charged one. It circles quickly, so touch mode aims
  where it will be when your shot lands, by how fast your Pokémon's shots fly and how near it
  stands. The first shot of a round can land a little behind, as touch mode learns your
  Pokémon's speed from it. Left alone, a barrier goes by itself after 20 seconds.
- **The Bot's shots** cost you points. pa3ds plays the game's alert sound as one is fired, and
  the game's "!" sound when it's time to act: move out of the way with the Circle Pad, or press
  the game's `L` to raise a barrier. A barrier lasts under a second, so wait for the "!" rather
  than holding `L` from the start. pa3ds says "blocked" when it stops one. With `feedback_sounds`
  off, these are "shot coming" and "now" in words. Each can be changed under
  [`[pa3ds.training]`](settings.md).
- `C` says where your Pokémon stands, in steps from where it started, the way the map's list
  says where a thing is: "7 up 2 left", or "at the start". The Circle Pad moves it, and up takes
  it nearer the Balloon Bot, to about 8 up at the nearest. Near, your shots land sooner, so the
  goals have less time to drift away from them, and the Bot's shots reach you sooner too. A shot
  scores the same from anywhere. `C` works while you hold `Shift` too.
- `R` says the score, the time left and how full the energy meter is, even while you hold
  `Shift` for a charge. A shot with the meter at
  4 of 5 or more uses over half of it, about two and a half seconds' worth, and scores the
  regimen's full points. Below about 3 of 5 a shot uses none and scores far less (against
  Wailmer, 3 points to 21), so shooting steadily scores more than tapping as fast as you can.

## Finding the way

**Into another place.** A step of the way that takes you into another town or route names it:
"5 north into Route 101".

**Parts of a place.** The game splits a few places into parts with no line you can see: the
Safari Zone's four quarters, Route 119's north and south, Route 120's plateau, and Route 123's
four berry plots. pa3ds names each part, "Safari Zone Northwest", says it as you walk into it, and
says it for an exit or a step of the way into it: "way out to Northern Route 119". The ways into
the next part are listed as one exit, "path to Safari Zone Northeast", and `P` goes to the
nearest. Each part has a list of its own, but in the Safari Zone, where some things are reached
only through another quarter, the list holds all four quarters whichever you stand in, and a thing
in another quarter says which after the way to it: "item ball, 12 west 30 north, Safari Zone
Northwest". `P` keeps leading you to it from one quarter into the next. Of two ways as short, `P`
takes the one through fewer parts.

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

**Arrow tiles.** In Lysandre Labs an arrow tile pushes you the way it points until something
stops you, and an arrow you cross turns you its way. `P` takes a push as one step of the way, said
by where it leaves you, "arrow pushes 3 west", and waits until it has. `P` also leads you through
the round pads, and goes on with the way once a pad has put you down.

**Berry spots.** In Omega Ruby and Alpha Sapphire the berries category lists the soft soil
Berries grow in, by what grows there: "soft soil" with nothing planted, then "planted berry",
"berry sprout", "growing berry tree" and "berry tree in bloom" as it grows, each with "dry" in
front while its soil needs water, "dry berry sprout", and once a tree has Berries to pick, its
Berry, "harvestable Nanab Berry tree".

**Ice.** In Shoal Cave at low tide you slide across the ice until a wall or the floor stops you.
`P` says each slide as one step of the way, "slide 4 north": step that way once and let it carry
you.

**The Acro Bike.** On Route 119 and in the Safari Zone the Acro Bike rides along narrow rails, and
the bike category lists each end of one. Ride onto a rail in line with it from its end. `P` says a
ride along a rail with the bike's name, "Acro Bike 9 east", and where a rail steps sideways over
short bars, a hop across onto each, "hop 3 south": press B and the way together, once a hop. `P`
also finds the way on from a rail you have stopped on. Up or down a bumpy slope `P` says "bunny
hop 2 north": hold B until the bike bounces in place, then press the way with B still held. On the
Mach Bike `P` takes no rail or bumpy slope, and on the Acro Bike no sandy slope up, so it says
which bike the way needs.

**Jumping stones.** In Pokémon X, `P` crosses a mud pit by its stones: each press of the way hops
you two squares, onto the next stone. In Victory Road, where the stones stand in a pit you can
also walk, a way along the pit's floor goes round them.

**Riding Rhyhorn and Mamoswine.** In X and Y, only a Rhyhorn crosses Route 9's spiky ground, and
only a Mamoswine Route 17's deep snow. The riding category lists each patch of flat ground, where
you get on by facing the Pokémon and pressing A, and get off with B. `P` walks you to it,
"Mamoswine 1 east", across and off again, "get off 3 north". The Pokémon is wider than you, and
`P` keeps it clear of the walls. Rocks are obstacles: riding, face one and press A to smash it,
which `P` says as "smash 1 east" where the Pokémon reaches it.

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

**The Team Flare Secret HQ's lift.** In Geosenge Town, in X and Y, the way into the HQ is a lift.
The interactables list its panel as "elevator": face it and press A to take the lift down or up.
The exits list the one way out the lift's door leads to where the lift is now, the HQ or Geosenge
Town.

**The Furfrou in Parfum Palace's garden.** In X and Y, you and Shauna corner a Furfrou in the
hedge maze at the back of the garden. It stands in the middle of one of the maze's four sides and
runs to another as you come within three steps of it; Shauna follows you, and waits at one of
those four places when you talk to her there and answer Yes. While the chase is on, the
interactables list the four as waiting spots, each with what it is by, and one "next step": where
Shauna should wait, a walk back to talk to her so she follows again, or where to walk to make the
Furfrou run the right way. Press `P` on the next step, walk the way, do what it says, and look for
the next one. A "keep off" entry is the line the Furfrou would escape from; `P` leads you round
it. The whole chase: Shauna waits at the south spot by the entrance, you send the Furfrou to the
north spot by the dead end, Shauna waits at the east spot, and you come at the Furfrou along the
north path from the west.

**Anistar Gym.** In Anistar City, in X and Y, the walkways in space are said by the compass, as
the keys walk them: "12 north, 3 west, 8 north". The interactables list each teleporter, the
carpet in the lobby among them: step onto the carpet's middle and the Gym appears round you.
Each teleporter has a partner where it puts you, which sends you back once you step off it and
on again. Anything you can reach only through one is "past a teleporter", and `P` leads you
there, saying the way as far as the first teleporter and how many more.

**Ramps.** In the Tower of Mastery, in X and Y, a ramp winds up round the tower twice, past the
stairs on each landing to the top. `P` says each stretch of it as one step of the way, "8 west,
up the ramp 22 east": hold that key and the ramp carries you round its curve to the next
landing. The way to someone on the ground goes back down it first, "down the ramp 16 west".

**Big boulders** say which ways you can push them, from the sides you can get to, or "immoveable".
Hold the way for a moment to push one a square. Push one into a hole and both leave the list.

**Diagonal steps.** In Santalune Gym, in X and Y, `P` says a step along a diagonal thread of the
web as "1 north-east": tilt the circle pad that way, or hold both keys.

## Naming things

The game never names most people on the map, signs and doors, and pa3ds gives some of them a name
of its own. `Shift+K` names the chosen thing, and `Shift+J` whoever spoke last: type the name and
press Enter.

In Sun and Moon much of the map is named from the game itself. A trainer is their class and name,
"Youngster Jimmy", and anyone who looks like a kind of trainer is that kind, "Lass". Someone you
battle more than once is named wherever they stand: "Pokémon Trainer Hau", "Hala", "Team Skull
Gladion". The rest of the story's people, your mom, Professor Kukui, Lillie and the captains,
are named by pa3ds, and so are townspeople, by how they look: "Old woman", "Clerk", "Little
boy". A Pokémon standing on the map is its species, "Meowth". A sign is its
first line, "Route 1 sign" or "Trainer Tips sign". Something you can check is the first thing it
says when you do, "It's a Wii U!". A building's door is the building: "Pokémon Center", "Apparel
Shop", "Malie Library".

People and things drawn alike, such as every Youngster or every big boulder, share a name
unless you tick **Only this one**, a Tab away from the name and ticked as the box opens. A name
given to one alone stays with them wherever they walk or are pushed. Your names always win over
pa3ds's. They are saved in `user\names\<language>\`, a file for each game: to share them, send
the file.

## Changing keys

Keys are in `user\keys.toml`, written with the defaults on the first start. pa3ds adds the keys
later versions bring, takes out a key bound to something it no longer does, and keeps its tables
in order with `[azahar]` last; it changes no other line you have. To unbind a key, put a `#` in
front of its line: a line you delete is put back the next time pa3ds starts.

- `[pa3ds.navigation]` holds the keys of every mode. `[pa3ds.navigation.list]`,
  `[pa3ds.navigation.direction]` and `[pa3ds.navigation.touch]` hold one mode's, and win over it.
  A key can have modifiers: `"shift+k"`.
- `"shift" = "drag"` under `[pa3ds.navigation.touch]` is the drag key, `"shift"` meaning Shift
  alone. Change it to another key, such as `"d" = "drag"`, and that key drags instead and Shift
  no longer does. It works as Shift did, held or pressed as the `drag` setting says, and outside
  Pokémon-Amie says "nothing to drag".
- `[pa3ds.amie.stylus]`, `[pa3ds.amie.berry_picker]` and `[pa3ds.amie.head_it]` hold the keys
  of Pokémon-Amie and of its games, taken only while each runs: see
  [Changing Amie's keys](amie.md#changing-amies-keys).
- `[pa3ds.training.page]` and `[pa3ds.training.round]` hold the keys of Super Training's page and
  of its regimens, taken only while each is up.
- `[azahar]` rebinds Azahar's own buttons, and `[azahar.circle_pad]` and `[azahar.c_stick]` its
  two sticks. Changes apply the next time pa3ds starts with Azahar closed.

A new key whose default you already use for something else is added with a `#` in front of it,
so your key keeps doing what it did, and pa3ds says so as it starts. Give one of the two another
key, then take the `#` off the new line.
