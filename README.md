# pa3ds

A screen reader accessibility layer for 3DS Pokémon games running in the
[Azahar](https://azahar-emu.org/) emulator.

pa3ds reads the game's own memory while you play and sends what it finds to your screen reader
through [Prism](https://github.com/ethindp/prism): NVDA, or another screen reader Prism supports,
or SAPI. Every word the game shows is read from the game itself, in your game's language. pa3ds
never presses a button for you. The one exception is touch mode, where it taps the touch-screen
button it has just named, and only when you press its tap key.

Inspired by [pokemon-access](https://github.com/nuive/pokemon-access), which does the same job for
the Game Boy games. Nothing here is taken from its code.

**Status: pre-alpha. There is no release yet.** pa3ds is one program, `pa3ds.exe`, with nothing
to install.

## Games

- **Omega Ruby and Alpha Sapphire**: the most complete.
- **X and Y**: read by the same code as Omega Ruby, with some screens still silent.
- **Sun and Moon**: dialogue with the speaker's name, the language choice, menus and screens,
  the name entry keyboard, battles, and the map's doors, signs, people and item balls, with
  pathfinding.

## What it says

- **Dialogue**, a page at a time as it starts to type, with who is speaking where pa3ds knows.
- **Menus**: the item under the cursor, then its details, such as a description or a count.
- **Screen text**: labels, options and questions as they appear and when they change. A screen
  that changes all at once, as the summary does turning to another Pokémon or the Pokédex to
  another entry, says its first line at once, the Pokémon's name where it can, and the rest as
  one line once it has finished changing.
- **Name entry**: the key under the cursor.
- **The Trainer Passport's photos** in Sun and Moon: each photo described in pa3ds's own words
  as the cursor moves to it ("Boy, fair skin, blond hair, blue eyes, ..."), since the game shows
  the photos and never names them.
- **Battles**: the battle text, and each side's Pokémon with its name, gender, level and HP. Your
  HP is said exactly; the enemy's as a percentage, because a bar is all the game shows.
- **The PC Box, the party, the Bag and the Pokédex**, including the habitat map.
- **The Pokéblock Kit**: each berry or block with how many you have ("Chesto Berry Blue 4").
- **Contests**: each move under the cursor, then its category, its appeal and jamming hearts
  and its description ("Hold Back", "Coolness, Appeal 4 hearts, Jamming 0 hearts"); the
  results say each entrant's place ("Raiden & Metagross third place"). A Pokémon's summary
  says each move with its type, PP and contest category ("Meteor Mash Steel PP 10/10
  Coolness"), on either page. Its conditions, drawn as a star, are an [Extra](#extras).
- **The Bag's party** for an evolution stone: whether each Pokémon can use it, in party order
  ("UNABLE!"), with its species where pa3ds has found your party ("Metagross UNABLE!").
- **Where you are**: the place name when you arrive somewhere new.
- **Scenes** the game shows without words, described in pa3ds's own words when
  [`descriptions`](#settings) is on: in Omega Ruby, Groudon waking, the ride down to it, the sky
  clearing; in Alpha Sapphire, Kyogre waking, the ride down to it, the sky clearing; in Sun, the
  chase after Kukui's call.
- **The map**: exits, signs, people, items, berry spots and more, each with its direction and
  distance ("door, 1 of 3, 3 east 2 north"), and the way to any of them on foot. An exit says
  what it is: a door, stairs up or down, a passage between rooms with no floor between them, a
  ladder up or down, the name of the building behind it, or in a hideout a teleporter and where
  it comes out ("teleporter, 3 east, out at 17 east 8 south"). Something the game builds out of
  several pieces is one thing at its nearest cell: Seafloor Cavern's submarine is one sign, and
  an elevator's two-cell door one, and a doll is one thing, not the doll and the sign in front
  of it. The game's invisible walls are never listed. A PC is said as
  a PC, and in Omega Ruby and Alpha Sapphire so is everything else you read by facing it that is
  the same wherever it stands, in the words of what it shows you: a TV, a trash can, Pokémon
  magazines, Pokémon picture books, a vase, a vending machine, a wardrobe, an elevator, the
  Pokémon Journal in each Pokémon Center, the hideouts' beds and flags, the Hoenn Rangers'
  posters and figures, and the dolls in Littleroot's bedrooms, until you name it something else.
  Townsfolk are called by how they look ("Old man", "Clerk", "Nurse") in every language the game
  carries. What stands in
  your way until a move clears it is under obstacles: bushes for Cut, cracked boulders for Rock
  Smash, Mauville Gym's barriers, and big boulders for Strength with the holes they can be pushed into ("hole, 2
  east 3 south"): push
  a boulder into a hole and both leave the list, and the filled hole is floor you can walk over.
  A boulder says the ways it can be pushed from where it stands now, and which push drops it
  into a hole ("big boulder, 2 east, moveable north into a hole or east"), or "immoveable" when
  a wall, another boulder or somebody is in the way of every push, or there is nowhere to push
  it from. A push moves it one cell, and only while you hold the way for a moment.
- **The touch screen**: the buttons on the bottom screen, which touch mode can tap for you,
  and the PokéNav's apps, the BuzzNav's shows among them. On the PlayNav's pages the arrows in
  the top corners are "previous" and "next", the round button that opens an app is "start", and
  on the Pokémon-Amie page the Pokémon you play with is said by its name ("Sceptile"). Tap it
  for its menu, PLAY, DECORATE and SWITCH; "close" shuts the menu again. In Pokémon-Amie
  itself touch mode offers "stroke Sceptile": the tap key strokes your Pokémon, back and forth
  across the middle of the screen. This is an experiment.
- **Super Training**: on its PlayNav page your party's Pokémon are said by name; tap one to
  train it. "Super Training" in the lower left opens the regimens, and "training bags" in the
  lower right opens your bags: each bag is said by its name ("Strength Bag"), and tapping one
  says what it does, with "Use" and "Back". A bag in use hangs beside your Pokémon and is said by
  its name too: tap it to hit it. On the regimen menu each regimen is said by its title ("Hone
  Sp. Atk with Magnemite!"): tap it once to hear what it trains and its target time, and again
  to start it; "left" and "right" turn the menu's pages. Tapping the screen moves the messages
  after a regimen on. In a regimen, touch mode offers each goal the Balloon Bot puts up, "goal",
  or "red goal" for a bonus goal that takes many hits, and the tap key shoots at it: the first
  press says the goal and plays its tone, each press after that shoots, wherever the goal has
  moved to. A goal you hit goes away, and the next press says the next one. A round says "start"
  and "clear" or "game over", your score as it changes ("177 of 300"), and the time left at each
  half minute and at 10 seconds ("2:00 left"). When the bot fires at you, pa3ds says "shot
  coming" at once, about a second before the ball arrives: move your Pokémon with the Circle Pad
  or raise a shield with L, as the game's own instructions say.
- **The DexNav**: the area's Pokémon in touch mode, "caught" after each one you have caught,
  and the crown for an area whose Pokémon you have caught ("gold crown");
  searching; and a hidden Pokémon's details, its Potential's stars and the marks for a higher
  level, an egg move or the hidden ability among them ("Potential 2 stars", "First Move Helping
  Hand egg move").
- **The Dowsing Machine's lights** ("dowsing green"), **soaring** on Latios or Latias, and
  decorating a Secret Base. A base with a second floor lists its "crumbled cliff" among the
  exits, the place the Stairs decoration goes to climb up, until the Stairs stand there, and the
  decorating cursor says "crumbled cliff" on it. A base with a hole in its floor lists the "hole"
  the same way, until a Solid Board covers it. Once the Stairs or a Solid Board are set down,
  P's path goes up the Stairs or across the board. While you hold a decoration, the cursor says
  first whether it can be set down there, as the game draws its squares green or red:
  "placeable, crumbled cliff, 4 east 3 north" or "not placeable, wall, 6 east 3 north". The
  decorations in your base are listed among the signs by their names, "Stairs", "Welcome Mat",
  all but the Invisible Doll, which the game hides. Decorations of the same name that touch, like
  the line of Welcome Mats, are one thing.

pa3ds says only what the game shows. Hidden information such as IVs or the enemy's exact HP is
not read.

## What you need

- Windows.
- Azahar, installed. A portable Azahar (with a `user` folder beside `azahar.exe`) is not set up
  by pa3ds yet: see [Known problems](#known-problems).
- Your own Pokémon game, dumped from your own 3DS. pa3ds reads place names, building names, the
  rival's name, and what to call the people and Pokémon on the map from the ROM file. This works only with a **decrypted** ROM (`.3ds` or `.cci`).
  With an encrypted one the game still plays and pa3ds still reads it, but some of those names
  are missing. pa3ds finds the ROM among the ones Azahar has opened.
- A screen reader: NVDA, or any other Prism supports. Without one, pa3ds speaks with SAPI.

ROMs are not distributed here and never will be.

## Setting up

1. Unpack pa3ds into a folder of its own that you can write to, such as one in your Documents
   folder. Not under Program Files: pa3ds keeps its settings beside itself, and there it cannot
   without running as administrator. Before unpacking, open the zip's Properties and check
   Unblock, or Windows SmartScreen stops pa3ds when it starts. Unpack with Extract All rather than
   running `pa3ds.exe` from inside the zip: run that way, Windows runs a copy in a temporary
   folder, and the settings, keys and names you change there are gone on the next start.
2. Start Azahar once and close it, so that it writes its settings file.
3. With Azahar closed, run `pa3ds.exe`. On its first start pa3ds writes its own settings files
   into the `user` folder beside it, changes the Azahar settings it needs (listed below), and
   says "Azahar settings applied".
4. Start Azahar and your game. pa3ds says "Waiting for the game" until the game is running, then
   "pa3ds ready".

After that, start pa3ds and Azahar in either order.

pa3ds changes Azahar's settings only while Azahar is closed, because Azahar writes its own
settings back over the file when it closes. If pa3ds says "close Azahar and start pa3ds again to
apply its settings", do exactly that. If it says "Azahar's settings were not found", start Azahar
once, close it, and start pa3ds again.

pa3ds is quiet while another window is in front of Azahar. It still says its own messages and
answers its own keys.

## Azahar settings pa3ds needs

pa3ds writes these into Azahar's `qt-config.ini` (in `%APPDATA%\Azahar\config`). They come from
the `[azahar]` table of `user\config.toml`, so you can change them there.

| Setting | Value | Why |
|---|---|---|
| `enable_rpc_server` | `true` | **Required.** pa3ds reads the game through it. Azahar ships it off. |
| `use_touchpad` | `true` | Required for touch mode: lets pa3ds tap the touch screen. |
| `controller_touch_device` | pa3ds's own touch device | Required for touch mode: where the taps come from. |
| `log_filter` | `*:Info RPC_Server:Error` | Stops Azahar's log from growing by a megabyte a second. |
| `graphics_api` | `2` (Vulkan) | With OpenGL, Azahar's window is slow to answer the screen reader. |
| `async_shader_compilation` | `true` | Fewer stutters. |
| `shaders_accurate_mul` | `false` | Faster, at the cost of a rare wrong pixel. |
| `use_vsync` | `false` | Faster. |

Delete a line from the table to leave that setting as Azahar has it. Any other Azahar setting can
be added by its name in `qt-config.ini`.

## Keys

pa3ds's keys work only while Azahar's game window is in front, and they never reach the game.
They do nothing while your screen reader's modifier key (Insert or Caps Lock) or a Windows key is
held.

The map's list has three **modes**. The apostrophe steps through them. Once a battle or a scene
has put the map away, the map's keys say "no map"; while another screen runs over a map still
there, the list still answers but `P` says "can't walk now". The thing you had chosen is kept for
when the map comes back. Touch mode and the battle's keys work throughout.

**In every mode**

| Key | What it does |
|---|---|
| `'` (apostrophe) | Next mode: list mode, direction mode, touch mode. |
| `[` and `]` | Previous and next category of the map's list. |
| `P` | Find the way on foot to the chosen thing, and guide you along it turn by turn. The way to a passage or stairs ends on the side you walk into it from. |
| `Shift+P` | The same, but also using Surf, Cut, Rock Smash, Strength, Waterfall and the bikes. It goes round a big boulder wherever there is a way round, however long, and pushes one only where there is none, since a boulder pushed the wrong way can block the way; and only a way the boulder can be pushed, never through one that is immoveable. |
| `Alt+P` | Stop guiding: the way is dropped and nothing more of it is said. "path off", or "no path" when there was none. |
| `Ctrl+P` | Leave out of the list everything `Shift+P` finds no way to from where you stand ("unreachable hidden"), or put it back ("unreachable shown"). Off each time pa3ds starts. Ctrl+P is Azahar's key to save a screenshot, which pa3ds takes while the game is in front. |
| `,` and `.` (comma and period) | Previous and next way to the chosen thing, where there is more than one: for a big boulder, which way to push it ("push east into a hole, 2 of 2"). `P` then goes to the side you push it from that way, round the boulder if need be: "to push east: 3 west, 2 north". Without a choice, `P` goes to the nearest side it can be pushed from. Anything else has "one way to it". |
| `R` | Say your side of the battle again. In a contest's appeals, your place ("Raiden & Pikachu first place"). |
| `Shift+R` | Say the enemy's side of the battle again. In a contest's appeals, everyone else's place, in order. Each entrant is named from the first round's appeals on, or once you have moved the cursor onto their portrait. |
| `Shift+K` | Give the chosen thing a name of your own. |
| `Shift+J` | Give whoever spoke last a name of your own. |
| `C` | Say the square you are standing on, as two numbers: "x 381, z 117". x grows going east and z going south. With `show_y` on, how high you stand as well (see [Settings](#settings)). |
| `Shift+C` | Say the squares of the chosen thing the same way: "x 62, z 29". Something more than one square wide or deep says its first and last: "x 80 to 119, z 159". |

**List mode** walks the whole list, nearest first.

| Key | What it does |
|---|---|
| `I` and `O` | Previous and next category. |
| `J` and `L` | Previous and next thing in the category. |
| `K` | The chosen thing again, from where you stand now. |

**Direction mode** looks one way at a time, up to 10 cells away.

| Key | What it does |
|---|---|
| `O` | The next thing to the north. |
| `;` (semicolon) | The next thing to the east. |
| `L` | The next thing to the south. |
| `K` | The next thing to the west. |

**Touch mode** walks the buttons on the bottom screen, in reading order.

| Key | What it does |
|---|---|
| `J` and `L` | Previous and next button. |
| `I` and `O` | Previous and next row. |
| `K` | The chosen button again. |
| `Enter` | Tap the chosen button. |

The categories are all, exits, signs, obstacles, people, items, berries, ash, bike, currents and
pokémon; while soaring, towns, routes, landmarks and mirage spots. A category with nothing in it
is skipped.

**Lining up.** The D-pad walks you from the centre of one cell to the centre of the next. The
circle pad lets you stop anywhere, partway between cells, and the game refuses a step past a
wall's corner to a player standing off the centre on the corner's side: a gap one cell wide
between two pillars can stay shut however often you walk into it. So where that would stop you,
`P` says which way to step first, "a little west, then 2 north", and while it guides you, stopping
off the line before such a step gets "a little west" on its own. Sneak that way, the circle pad
tilted gently, or your walking modifier held and the key tapped, and pa3ds says "lined up" the
moment you are. Off the line in open ground nothing is said, since nothing stops you there.

**Currents.** Some of the sea, as on Routes 132 to 134, has currents that carry you while you
surf, a square at a time, and you cannot turn until one lets you go, sometimes onto a sandbar.
When one catches you, pa3ds says where it will leave you: "ride the current 28 west". `P` and
`Shift+P` know the currents: a way that rides one says it as one step, "Surf 3 south, ride the
current 28 west 4 north, 5 east", and says nothing more until it lets you go. If a current
carries you off the way, the way is found again where it leaves you. The currents category lists
them by where they let you go, each at its square nearest you, with where that square leaves you:
"current, 2 south, out at 30 west 1 south".

On a fresh Azahar, `I`, `J`, `K` and `L` are the C-stick and `O` and `P` are the debug and GPIO
buttons, none of which the Pokémon games use. When one of pa3ds's keys is on an Azahar button,
pa3ds says so as it starts.

### Changing keys

Keys live in `user\keys.toml`, written with the defaults on the first start and never
overwritten.

- `[pa3ds.navigation]` holds keys that do the same thing in every mode.
  `[pa3ds.navigation.list]`, `[pa3ds.navigation.direction]` and `[pa3ds.navigation.touch]` hold
  the keys of one mode, and win over it. A key can have modifiers: `"shift+k"`.
- `[azahar]` rebinds Azahar's own buttons, and `[azahar.circle_pad]` and `[azahar.c_stick]` its
  two sticks. It is empty by default. The comments in the file list what Azahar's buttons are
  bound to now. Changes apply the next time pa3ds starts with Azahar closed.

A key added to pa3ds after your file was written is not added to your file, so it cannot take a
key you are using. pa3ds says so as it starts, and the log has the line to add.

## Settings

Settings live in `user\config.toml`, written with the defaults on the first start and never
overwritten. Each setting has a comment in the file saying what it does.

| Setting | Default | What it does |
|---|---|---|
| **`[pa3ds]`** | | |
| `keys` | `true` | Answer pa3ds's keys and rebind Azahar's buttons as `keys.toml` says. |
| `poll_rate` | `60` | How many times a second pa3ds reads the game. 30 for a slower machine. |
| **`[pa3ds.ui]`** | | |
| `pauses` | `false` | Pause between pieces of screen text said together: "OT: Raiden". |
| `interrupt` | `false` | Cut off speech when a menu's cursor moves, instead of queueing. pa3ds's own keys always cut off speech. |
| `unfocused` | `false` | Keep saying what the game shows while another window is in front. |
| `tones` | `true` | A short tone as the map's cursor or touch mode moves: higher the further up, panned the further right. |
| `descriptions` | `false` | Describe the scenes the game shows without words, after the line each scene follows, said as "Narration:" the way a speaker is named. So far in English, for Omega Ruby's scenes from Groudon waking in Seafloor Cavern to the sky clearing after it is caught, Alpha Sapphire's from Kyogre waking to the sky clearing after it is caught, and Sun's chase after Kukui's call, said as it plays. |
| **`[pa3ds.navigation]`** | | |
| `positions` | `true` | Say where a thing is in the list: "door, 1 of 3". |
| `position_first` | `true` | Say the place in the list before the way to it. `false` says "door, 1 north, 1 of 3". |
| `turn_by_turn` | `true` | Say each turn of the way P found as you reach it. `false` says the way once, and P again says it from where you stand. |
| `guide_to_hidden` | `true` | When a DexNav search finds the Pokémon you searched for, or the DexNav finds one by itself while it is open and you are in touch mode, find the way to it as P does, and keep P on it. One found while the AreaNav is open is left alone. |
| `track_boulders` | `false` | Say how far a big boulder has moved from where its room starts it, so you know which you have pushed: "big boulder, 2 east, moved 1 north". Nothing is added for one that has not moved. Off by default, since the game shows where each boulder is but never where it started. |
| `hide_door_signs` | `true` | Leave a sign out of the list when all it says is the name of a door beside it, which the door is already called: Mauville Hills' "Apt. 13 Wattson's Home". A sign that says more, as a Gym's does with its leader, stays. |
| `show_y` | `false` | `C` and `Shift+C` say how high as well, as y after x and z: "x 71, z 35, y 20". y is in the game's own units, 18 to a square's width, so two things 18 apart stand a square's width apart in height. It is left out where pa3ds does not know a thing's height, as for a sign, and in Sun and Moon. |
| **`[pa3ds.extras]`** | | Extras: see [below](#extras). |
| `conditions` | `false` | Say a Pokémon's contest conditions as numbers on its summary and in the Pokéblock Case: "Conditions Coolness 116 Beauty 70 Cuteness 70 Cleverness 70 Toughness 70". |
| **`[azahar]`** | | Azahar's own settings: see [above](#azahar-settings-pa3ds-needs). |

A setting added to pa3ds after your file was written is not added to your file. It works at its
default, and you can add the line yourself.

### Extras

pa3ds says what the game shows, and no more: a sighted player and you hear of the same things.
An Extra is something pa3ds can only give you more exactly than the game shows it. Each is off
until you turn it on under `[pa3ds.extras]`.

- **`conditions`**: the game shows a Pokémon's contest conditions as a star on its summary and
  in the Pokéblock Case, each point reaching further as that condition grows. pa3ds cannot read
  the star itself, so it says the numbers the star is drawn from. They are exact, where a
  sighted player sees only a shape: nobody sees "Coolness 116". It is an Extra because the
  numbers are all pa3ds can give you of something the game does show; the alternative is
  hearing nothing of it. Sheen is not said: the game never names it, and in Omega Ruby and
  Alpha Sapphire it changes nothing.

No Extra so far tells you something the game keeps from everyone, such as a Pokémon's hidden
potential or an item buried on the ground.

### Command-line options

Run `pa3ds.exe --help` for the full list. The ones a player may want:

- `--rom PATH`: your decrypted ROM, if pa3ds does not find it among the ones Azahar has opened.
- `--no-keys`: answer no keys and rebind no buttons.
- `--port PORT`: Azahar's RPC port, if you changed it from 45987.

## Names of your own

The game never names some things: most people on the map, signs, doors. pa3ds gives some of them
a name of its own. You can give anything a name with `Shift+K`, or name whoever spoke last with
`Shift+J`: a box opens, type the name and press Enter. pa3ds says the new name back.

Many people share a look: every Youngster is drawn alike, and so is every big boulder. When you
name one of them, the box has an **Only this one** checkbox, a Tab away from the name and ticked
as the box opens. Ticked, the name is that one's alone, the Youngster on Route 102 who is Calvin,
or one boulder of eight, and it stays with them wherever they walk or are pushed. Unticked, it
names everyone drawn alike, everywhere in the game. A name given to one alone wins over one given
to everyone, and clearing it gives that one back to everyone's.

Your names are saved in `user\names\<language>\`, one file per game. Your names always win over
pa3ds's. To share them, send the file. A place is written map, kind and number, as in
`"0x10:warp:3" = "Devon Corporation"`: Rustboro City's door number 3. A file written before
2026-10-11 is rewritten that way the next time its game is played.

## Your files

Everything that is yours is in the `user` folder beside `pa3ds.exe`. Copy that folder to keep it
all when you move or update pa3ds.

- `config.toml`: settings.
- `keys.toml`: keys.
- `names\`: your names.
- `logs\`: what pa3ds did and said in each of the last ten runs. `pa3ds.log` is the latest run,
  `pa3ds.1.log` the one before, up to `pa3ds.9.log`.
- `cache\`: what pa3ds worked out from your ROM. Safe to delete; it is built again.

## When something goes wrong

- **pa3ds says "Waiting for the game" and nothing more while the game runs**: Azahar's RPC server
  is off. Close Azahar, start pa3ds, then start Azahar.
- **pa3ds says "Reader error"**: part of pa3ds failed on that screen. The log has the details.

To report a problem, send `user\logs\pa3ds.log` from the run where it happened, and say what you
were doing.

## Known problems

- **A portable Azahar is not set up by pa3ds.** pa3ds changes only the settings in
  `%APPDATA%\Azahar`. With a portable Azahar, set the settings in
  [the table above](#azahar-settings-pa3ds-needs) yourself, in `user\config\qt-config.ini` beside
  `azahar.exe`, while Azahar is closed.
- **Some touch-screen apps are not fully read yet**: the PokéNav's DexNav, and the PlayNav's apps
  once they open: Pokémon-Amie, Super Training and the PSS. In Pokémon-Amie the Pokémon has
  not been seen to come when stroked, and its Poké Puffs, minigames and way out have not been
  found. In Super Training, a regimen's description reads its two columns run together, and
  a red bonus goal has not been met yet. Turning a PlayNav page with its
  arrows says nothing of the page it turns to.
- **X, Y, Sun and Moon** are read less completely than Omega Ruby and Alpha Sapphire.

## For developers

This is a port of [the Python pa3ds](https://github.com/pa3ds/pokemon-access-3ds) to Rust, so
that a player runs one `pa3ds.exe` with nothing to install. The Python repository is where
features are built first; this one follows it module for module (`port.toml` names the commit it
matches) and is checked against it line for line on recorded replays (`tools/tape/`). The
decisions, the evidence behind them and the vocabulary are the Python repository's, in its
`docs/` and `GLOSSARY.md`.

```
cargo build --release
```

Building needs Rust's MSVC toolchain (rustup's default on Windows), Visual Studio 2022 or newer
with its C and C++ compiler and ATL (the Build Tools are enough), and CMake 4.2 or newer on the
path, since Prism is compiled into pa3ds; the first build takes about a minute longer for it.
Visual Studio's own CMake is 3.31, which is too old.
`target\release\pa3ds.exe` is the whole program. To pack it with its licences into one zip a
player unpacks and runs:

```
powershell -ExecutionPolicy Bypass -File tools\package.ps1
```

(or `tools\package.bat`, which runs the same), which runs the tests, builds the release, checks it starts, and writes
`target\package\pa3ds-<version>-windows.zip`.
