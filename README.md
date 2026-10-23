# pa3ds

A screen reader accessibility layer for 3DS Pokémon games running in the
[Azahar](https://azahar-emu.org/) emulator.

pa3ds reads the game's memory while you play and sends what it finds to your screen reader
through [Prism](https://github.com/ethindp/prism): NVDA, another screen reader Prism supports, or
SAPI. The game's words are read from the game itself, in your game's language, and pa3ds's own
words, for what the game never says, are in your language too. pa3ds never presses a button for
you. The one exception is touch mode, where it taps the touch-screen button it has just named,
and only when you press its tap key.

Inspired by [pokemon-access](https://github.com/nuive/pokemon-access), which does the same job for
the Game Boy games. Nothing here is taken from its code.

**Status: pre-alpha.** pa3ds is one program, `pa3ds.exe`, with nothing to install.

## The manual

- This page: the games, what pa3ds says, setting it up, and updating.
- [Keys](manual/keys.md): pa3ds's keys, the map's list, finding the way, and naming things.
- [Settings](manual/settings.md): every setting, the Azahar settings pa3ds changes, and your files.
- [Languages](manual/languages.md): the language pa3ds speaks, and translating it.

## Games

- **Omega Ruby and Alpha Sapphire**: the most complete.
- **X and Y**: most of what Omega Ruby has, with some screens still silent.
- **Sun and Moon**: dialogue, menus and screens, the name entry keyboard, battles, and the map's
  doors, signs, people and items, with pathfinding.

## What it says

- **Dialogue**, a page at a time, with who is speaking where pa3ds knows.
- **Menus**: the item under the cursor, then its details, such as a description or a count.
- **Screen text**: labels, options and questions as they appear and when they change.
- **Name entry**: the key under the cursor.
- **Battles**: the battle text, and each side's Pokémon with its name, gender, level and HP. Your
  HP is said exactly, the enemy's as a percentage, because a bar is all the game shows. A move's
  category, power, accuracy and description on the move info screen, where the four moves are
  said by number, and when a move is to be forgotten.
- **The PC Box, the party, the Bag and the Pokédex**, the habitat map included.
- **Contests and the Pokéblock Kit**: each move with its appeal and jamming, each entrant's
  place, and your berries and blocks with how many you have.
- **Where you are**: the place name when you arrive somewhere new.
- **The map**: exits, interactables (signs and the like), people, items, berry spots, obstacles
  and more, each with its direction and distance, and the way to any of them on foot.
- **The touch screen**: the buttons on the bottom screen, which touch mode can tap for you, the
  PokéNav's and PlayNav's apps and the AreaNav's places among them.
- **The AreaNav**: each place, and the Pokémon seen there, as the DexNav says them.
- **The DexNav**: the area's Pokémon, which are met on the water or by fishing, and which you
  have caught, searching, and a hidden Pokémon's details. A Pokémon met in seaweed under water
  is said as one met walking, and as met in grass in the Pokédex, because the game itself shows
  it that way.
- **The Dowsing Machine**, **soaring** on Latios or Latias, and **Secret Bases**, decorating
  them included.
- **Pictures the game never names**: the boy and the girl to choose from as a game begins, and
  the Trainer Passport's photos in Sun and Moon, described in pa3ds's own words.
- **Scenes without words**, described when the `descriptions` setting is on: many in Omega Ruby
  and Alpha Sapphire, and one in Sun so far.
- **Footsteps** where Omega Ruby, Alpha Sapphire, X and Y play none, as on streets, floors and in
  caves: the game's own step sound, as loud as the game and Azahar would play it.

pa3ds says only what the game shows. Hidden information such as IVs or the enemy's exact HP is
not read.

## What you need

- Windows.
- Azahar, installed or portable.
- Your own Pokémon game, dumped from your own 3DS, as a **decrypted** ROM (`.3ds` or `.cci`). An
  encrypted one still plays and is still read, but pa3ds is missing the names it reads from the
  ROM: places, buildings, your rival, and the people and Pokémon on the map. ROMs are not
  distributed here and never will be.
- A screen reader: NVDA, or any other Prism supports. Without one, pa3ds speaks with SAPI.

## Setting up

1. Unpack pa3ds into a folder of its own that you can write to, such as one in Documents, not
   under Program Files: pa3ds keeps its settings beside itself. Before unpacking, open the zip's
   Properties and check Unblock, or Windows SmartScreen stops pa3ds when it starts. Unpack with
   Extract All rather than running `pa3ds.exe` from inside the zip, where the settings you
   change are lost on the next start.
2. Start Azahar once and close it, so that it writes its settings file.
3. With Azahar closed, run `pa3ds.exe`. pa3ds writes its own settings into the `user` folder
   beside it,
   changes the [Azahar settings it needs](manual/settings.md#azahar-settings-pa3ds-needs), and
   says "Azahar settings applied".
4. Start Azahar and your game. pa3ds says "Waiting for the game" until the game is running, then
   "pa3ds ready".

After that, start pa3ds and Azahar in either order. pa3ds closes when Azahar does. To restart
pa3ds, start it again: it takes the running one's place and says "pa3ds restarted".

pa3ds can change Azahar's settings only while Azahar is closed. If it says "close Azahar and start
pa3ds again to apply its settings", do exactly that. If it says "Azahar's settings were not
found", start Azahar once, close it, and start pa3ds again.

With a portable Azahar, one with a `user` folder beside `azahar.exe`, pa3ds finds it on its own
when Azahar is open or pa3ds is in the same folder. Otherwise pa3ds asks the first time for the
folder `azahar.exe` is in. Choose it, and pa3ds remembers it. Cancel instead if Azahar is
installed the usual way.

Started by double-clicking or from a shortcut, pa3ds hides its window while Azahar is open, so
it is not in Alt+Tab. While another window is in front of Azahar, pa3ds says nothing of the
game, but still answers its keys.

## Updating

When pa3ds starts, it checks for a newer version. If there is one, a window asks whether to
update. Yes downloads it, replaces `pa3ds.exe`, the `data` folder, the manual, `CHANGELOG.md` and
the licences, and starts the new pa3ds; your `user` folder is kept. No starts the version you
have, and pa3ds asks again on its next start.

What changed in each version is in `CHANGELOG.md` beside `pa3ds.exe`, and on the
[releases page](https://github.com/pa3ds/rust-pokemon-access-3ds/releases).

## When something goes wrong

- **pa3ds says "Waiting for the game" and nothing more while the game runs**: Azahar's RPC server
  is off. Close Azahar, start pa3ds, then start Azahar.
- **pa3ds says "Reader error"**: part of pa3ds failed on that screen. The log has the details.
- **pa3ds has gone quiet or stopped answering its keys**: start pa3ds again.

To report a problem, send `user\logs\pa3ds.log` from the run where it happened, and say what you
were doing.

## Known problems

- **Some touch-screen apps are only partly read**: the DexNav, Pokémon-Amie, Super Training and
  the PSS. Super Training's regimen descriptions read their two columns run together, and
  turning a PlayNav page with its arrows says nothing of the new page.
- **X, Y, Sun and Moon** are read less completely than Omega Ruby and Alpha Sapphire.

## For developers

Building pa3ds, and how this port follows the Python pa3ds:
[BUILDING.md](https://github.com/pa3ds/rust-pokemon-access-3ds/blob/main/BUILDING.md).
