# Changelog

## [0.9.0] - 2026-10-31

### Added

- Pokémon-Amie's touch screen: touch mode names its three buttons, Poké Puffs, games and return,
  and the Poké Puffs bar says each puff by its kind. To give one, hold Shift on it, carry it to
  your Pokémon, and let go once you hear it finish eating and cry.
- Drag in Pokémon-Amie: hold Shift on a button, choose where to with `J`, `L`, `I` and `O`, and
  let go of Shift to drop it, with the PC Box's sounds for picking up and putting down. The `drag`
  setting chooses holding or pressing twice, and a key of your own can drag too.
- Pokémon-Amie's games: the clock is said each half minute and in its last seconds, the `ticks`
  setting adds a tone each second, and the results say your rank and the stars it earned.
- Tile Puzzle: each tile is said as a spreadsheet names a cell, and where its piece belongs, "B1,
  goes to D3". The Normal and Hard boards are read, its difficulties say the best stars earned,
  and with `descriptions` on its picture and background are described, Play Again's too.
- Berry Picker: touch mode offers the berries on the tree and each Pokémon wanting one, "Pinap
  Berry for Treecko", said one at a time. Hold Shift on a berry, choose the Pokémon and let go to
  give it. `1` to `7` pick up a berry and `8`, `9` and `0` are the round's three Pokémon. The
  `berry_first` and `one_at_a_time` settings change how it is said.
- When a message on the bottom screen waits for a tap, as Super Training's welcome does, touch
  mode offers "continue".
- The `[pa3ds.touch]` settings `wrap` and `next_row` stop touch mode's keys at the ends and keep
  `J` and `L` in their row. The `feedback_sounds` setting sets the game's sounds for a drag and
  for a key with nothing to do, or says them instead at 0.
- In X and Y, `P` climbs and swings on Coumarine Gym's vines, and the climbing category lists each
  end of a vine.
- In X and Y, Prism Tower's Gym lists its lift and its quiz panels, and with `descriptions` on
  each question's picture is described.
- In X and Y, Laverre Gym's and Anistar Gym's teleporters are listed with where each puts you, and
  `P` leads you through them. Anistar Gym's walkways are said by the compass.
- In X and Y, `P` rides the Poké Ball Factory's conveyor belts, and goes by its switch where a belt
  needs turning round first.
- In X and Y, `P` takes Lysandre Labs' arrow tiles and round pads, the arrows category lists the
  tiles, and AZ's story there is described. X's masked heroes and Lysandre Labs' beds are named.
- In X and Y, `P` crosses Route 17's deep snow on the Mamoswine, as Route 9's spikes on the
  Rhyhorn.
- In X and Y, a gate is said with the place beyond it, "Lumiose Gate to Route 4", and Lysandre Café
  is named.
- Round Prism Tower in Lumiose City the directions turn with the camera, and a street that ends on
  a plaza lists that end as "open ground". A short tone plays each time the way up walks you
  turns.
- In Omega Ruby and Alpha Sapphire, Dewford Gym's switches and conveyor belts are listed, and
  Stern's Shipyard is named.
- Another player's Secret Base is named as the game names it, "Team Tarin's Base".
- Where only `Shift+P` reaches a thing, `P` takes you as far as the spot where a move is needed
  and says which, "then can't continue, requires Surf".

### Changed

- Touch mode works only where the game needs a touch: on the map with the PokéNav Plus, in the
  AreaNav, Pokémon-Amie and Super Training. Elsewhere it says "nothing to touch".
- In touch mode, `I` and `O` go to the nearest button in the row above and below.
- Of ways as short as each other, `P` says first the one with the fewest, longest steps.
- Every message pa3ds says in English is said in each of the other languages too.

### Fixed

- On Route 120's bridges `P` walks you over the deck, and while surfing under one you stay on the
  water.
- A warp across a diagonal street is listed once, and exits side by side into the same place are
  one exit.
- In X, `P` never steps off the side of a ramp, and nowhere climbs a step higher than a stair.
- Touch mode leaves out a list's buttons once the list has gone, and no longer names a button by
  the picture inside it.
- In the PC Box, each key waits half as long before saying the slot.

### Thanks

- [@cat123go](https://github.com/cat123go) translated pa3ds's words into Simplified Chinese: the
  mod window and its settings, and the scene descriptions of Omega Ruby, Alpha Sapphire, X and Y
  (#28). The lines of theirs that read better than ours are in the Chinese this version ships.
- Luyi translated into Simplified Chinese the names pa3ds gives the people and things of Omega Ruby
  and Alpha Sapphire, Mr. Briney's 哈奇 among them.

## [0.8.0] - 2026-10-26

### Added

- Started by double-clicking or from a shortcut, pa3ds opens a window of its own, Pokémon Access
  3DS, which is only a menu bar: press Alt for its menus. **Mod** has Settings, Check for updates
  and Exit, **Azahar** downloads Azahar's portable zip or its installer, and **Help** opens a page
  of the manual or reports an issue. The window hides while Azahar is open; `hide_window` keeps it
  in sight, for OBS to record pa3ds's sounds through it.
- The mod settings: Settings in the **Mod** menu changes every setting, on a tab for each part of
  `config.toml`, each said by a short name and then what it does, in your language.
- The `hidden_items` setting lists the hidden items you have not found yet among the items, as
  "hidden item": `"underwater"` under the sea, where the Dowsing Machine cannot be used, with the
  Scanner under Sea Mauville until you take it; `"all"` everywhere. `P` takes you beside one,
  facing it. It is off unless you turn it on.
- A thing more than two squares' height above or below you says how far: "item ball, 1 north,
  90 below".
- A step of `P`'s way that takes you into another town or route names it: "5 north into Route
  101".
- In X, `P` climbs Cyllage Gym's rock walls and slides down the two you slide down, "slide 5
  south". The climbing category lists the walls' blue spots and ledges.
- In X, `P` grinds Shalour Gym's rails on roller skates, and the spot where Korrina comes out is
  listed as the gym leader's spot.
- In X and Y, `P` climbs the Tower of Mastery's ramp, each stretch of it one step of the way:
  "8 west, up the ramp 22 east".
- X's Dowsing Machine is said as Omega Ruby's is.
- X and Y's title screen and the film it plays are described.
- In X's boutiques the fitting room is listed, and its tabs are said by the game's own words,
  "hat", "top" and the rest.
- Choosing a partner for a Multi Battle in the Battle Maison says each partner's name.
- In the PC Box, a slot says who is in it while you carry a Pokémon. The Battle Box's slots are
  said, and touch mode leaves out the box drawn behind them.
- In Shoal Cave, the way in from Route 125 and the ways out are listed, the entrance room is
  described at high tide, and `P` slides across the ice room's ice.
- The rocks that evolve Eevee are called the ice rock and the moss rock.

### Changed

- Every known problem is on a page of its own in the manual, Known issues.

### Fixed

- The Contest Spectacular Halls' lobbies no longer list ten people who are not there.
- In Omega Ruby and Alpha Sapphire, directions follow the camera where it turns, as on the Battle
  Resort's beach.
- A name a button shows as a picture is said once, not "Torchic Torchic L14".
- Past the League, the Bag and the Battle Maison's screens name your party's Pokémon, and the
  Maison names the Battle Box's.
- The PC Box's summary reads its stats, then its moves, instead of across both.
- The fitting room's card is said once, when it is written, and not again as you move between
  Change and Back; moving onto the AreaNav's Pokémon no longer says the route again.
- During a contest's appeals, `R` and `Shift+R` say the places, not "no battle".
- On Mt. Chimney's summit, `P` goes round and down the path to an item ball far below instead of
  saying it is beside you, and never walks up to a thing from the top of a cliff above it.
- The Mega Stone's sparkle is listed under items.
- Sea Mauville's locked doors are listed as doors, at the water, and diving under Sea Mauville
  lists the way into its sunken control room.

## [0.7.0] - 2026-10-23

### Added

- Omega Ruby's and Alpha Sapphire's intros and title screens are described, with the words they
  show as pictures: "Presented by", "Game Freak", the game's name and "Press Start". A scene you
  skip goes undescribed from there, and a description waits while the emulator is paused.
- More of Omega Ruby and Alpha Sapphire is described: the Sky Pillar, from Zinnia's tale to
  Rayquaza, and the end of the Delta Episode, from Deoxys to home. Playing as a girl, Brendan's
  room next door and the Beautifly over Route 102 are described too.
- In X, `P` crosses a mud pit by its jumping stones, two squares a hop.
- On Route 9 in X and Y, the riding category lists the flat ground where you get on a Rhyhorn.
  `P` walks you to the Rhyhorn, across the spiky ground and off again, "get off 3 north", and on
  it says the rocks to smash, "smash 1 east".
- In X's Glittering Cave, where the camera stays behind you, things are said from the way you
  face, "9 ahead 8 left", and `P` says the turns, "turn left, 5 forward". The cave's lurking
  Pokémon are listed under pokémon, and `P` walks through them.
- The `sort_by_distance` setting: `false` keeps list mode's list in one order as you walk, so
  you can tell by its place which house you have been in.
- Simplified Chinese is complete: the names of things on the map in all four games, and the
  scenes and pictures that were still in English.

### Fixed

- pa3ds finds a portable Azahar on its own when Azahar is open or pa3ds is in its folder, even
  with an older Azahar's settings left in AppData.
- In X, `P` no longer leads through the cliff on Route 8, and keeps the way to Cyllage City
  instead of saying "no path to edge" as you walk.
- The sparkle where a Mega Stone lies in Omega Ruby and Alpha Sapphire is called a Mega Stone.

## [0.6.0] - 2026-10-21

### Added

- The AreaNav is read. Touch mode offers each place on the map by name, and a tap chooses it.
  Left and right say its tabs, Places to Berries. Once a place is named, the Pokémon seen there
  are said a line for each way they are met, those met walking first: "Swablu, Seedot caught",
  "water: Marill". The icons along the top of the PokéNav's AreaNav window are named by the view
  each opens.
- The DexNav says which of the area's Pokémon are met on the water or by fishing: "Magikarp
  fishing caught".
- Footsteps in X and Y where the game plays none, as in Omega Ruby and Alpha Sapphire.
- In X, the map's directions turn with the camera, as in Parfum Palace's garden: north is the way
  up walks you.

### Fixed

- Touch mode offers the DexNav's Pokémon you have not caught again, and more than the magnifier
  while a hidden Pokémon is up.
- Touch mode's taps are no longer dropped for a while after the game restarts, its language is
  read again or pa3ds is started again.
- The AreaNav keeps its buttons in touch mode each time it opens, not only the first, and its
  View Details is said again on coming back to the Trainer's Eye.
- In X, the map finds its streets and the ways between outdoor places, says each step of a street
  by the key that walks it, and names Route 6's rare item ball and shaking grass. The way to a
  thing goes round the people it can.
- The scene after Groudon or Kyogre is described when you knock it out, as when you catch it.

### Known issues

- In the AreaNav's Pokémon Distribution, touch mode names a Pokémon you have caught twice:
  "Surskit Surskit". A later release fixes it.

## [0.5.0] - 2026-10-20

### Added

- Touch mode reads Pokémon-Amie's decorate screen: its arrows, return and exit, each Poké Puff by
  its kind and each Interior Item by name.
- More scenes are described under `descriptions`: May's room before you meet her, the Beautifly
  on Route 102, and the mural in Granite Cave, each version's own, then Steven leaving.

### Fixed

- In X and Y, the Pokémon-Amie page names your Pokémon and its menu offers PLAY, DECORATE and
  SWITCH; the Super Training page names your party, so a tap changes Pokémon, and names its
  training bags, its regimens and its Super Training button.
- While a panel or a Yes and No is up in Super Training, touch mode leaves out the buttons behind
  it that a tap no longer reaches, and the PlayNav's arrows and the X menu's icons while a page
  has the input.
- The PC's menus say only the row the cursor moves to, not the other rows as new.
- X's language select says the language: "Play Pokémon X in English".
- The Bag says "Nothing" at an empty pocket again.
- A TM named for its move's type, as TM29 Psychic is, says its type among its details.
- The Bag no longer stops with a reader error as it opens.

## [0.4.0] - 2026-10-19

### Added

- pa3ds's own words, the names it gives and the scenes it describes are in your game's language:
  German, English, Spanish, French, Italian, Japanese, Korean and Chinese. Omega Ruby, Alpha
  Sapphire, X and Y take it from your save; in Sun and Moon, set `language` under `[pa3ds.ui]`.
  The translations began as a machine's. The Languages page of the manual says how to correct a
  word or translate pa3ds into another language.
- Footsteps where Omega Ruby and Alpha Sapphire play none, as on streets, floors and in caves: the
  game's own step sound, as loud as the game and Azahar would play it.
- `[pa3ds.audio]` sets how loud pa3ds's own sounds are beside the game's: `tones` for the map's
  cursor and touch mode, `footsteps` for the footsteps. 1 is as loud as the game, 0 none. If you
  had turned the tones off, they stay off.
- The battle's move info screen is read: each move's category, power, accuracy and description,
  with the four moves said as "move 1" to "move 4". So is the move to forget after a level up.

### Changed

- The map's signs category is called interactables, in every language.
- The tones follow Azahar's volume slider and its slider in Windows' mixer, as the footsteps do.
- The manual is a short README, with pages for keys, settings and languages in the `manual`
  folder.
- The `data` folder holds only pa3ds's words, the files you translate. The rest is built into
  pa3ds.

### Fixed

- The Bag says only what is new as you move: not the pocket and the item's name again before its
  count and description, not the party's ABLE!, UNABLE! and levels at every TM, and not the whole
  Bag as it comes back from teaching a move. The move to forget no longer reads the Pokémon's
  number, name and stats with every move.
- pa3ds no longer stops while you soar on Latios or Latias.
- On Sharpedo, on Kyogre and under the water, pa3ds no longer says you are on the bike, and says
  where a current that catches you will leave you.
- Battles are said on the Chinese Omega Ruby translated by fans.
- X and Y's Poké Mart says your money, each item's description and how many you are buying.
- The moving truck at the start of Omega Ruby and Alpha Sapphire lists its way out to Littleroot
  Town.
- A comment in `config.toml` or `keys.toml` holding an unusual line break no longer breaks the
  file when pa3ds adds a setting to it.
- The manual names `U` as touch mode's tap key, not Enter.

## [0.3.0] - 2026-10-17

### Added

- While Azahar is open, pa3ds hides its own window, so it is not in Alt+Tab between you and the
  game. When you close Azahar, pa3ds closes too.
- To restart pa3ds, start it again: the new one closes the one running, says "pa3ds restarted",
  and carries on the same log.
- With a portable Azahar, pa3ds asks for the folder `azahar.exe` is in the first time it finds no
  Azahar settings, records it as `azahar_folder` in `user\config.toml`, and changes that Azahar's
  settings in the same start.
- `keys.toml` and `config.toml` gain the keys and settings each new version of pa3ds adds, and
  never lose a line you have. A new key whose default you already use for something else is added
  with a `#` in front of it, and pa3ds says so as it starts.
- If `config.toml` or `keys.toml` has a mistake pa3ds cannot read past, pa3ds says so as it
  starts, with the line the mistake is on, and uses its defaults until you fix it.

## [0.2.1] - 2026-10-17

### Fixed

- `P` finds the way across Santalune Gym's spider web in X and Y, along its diagonal threads, and
  says a step along one as "1 north-east".
- In Santalune Gym, the pole down to the web and the one back up are exits, as are the stairs past
  Viola's tent, and the lobby's statues and photographs are said as statues and photographs.
- The gate behind Viola in Santalune Gym is no longer listed as five nameless people.

## [0.2.0] - 2026-10-17

### Added

- X and Y name what is on their maps the way Omega Ruby and Alpha Sapphire do: their people
  ("Dexio", "Hairdresser"), the Pokémon walking about ("Fletchling"), things like the bedroom's
  Kalos Pokédex awards, Berry trees, sparkles and cones, and what you face and read, a TV, a trash
  can, an elevator, a photo spot, a bed, a Gogoat Shuttle stop.
- X and Y's obstacles: bushes for Cut, cracked boulders and cracked walls for Rock Smash, and big
  boulders for Strength.
- More scenes described with the `descriptions` setting. In Omega Ruby: Wally on Victory Road, the
  Elite Four's bridges as you cross them and their rooms, Steven and the Hall of Fame, the
  credits, May's night on Route 103, and the Delta Episode's opening. In Alpha Sapphire: the Hall
  of Fame, the credits, and the Delta Episode's opening.
- The credits name the Pokémon your own save remembers from each Gym and great battle, by the
  nickname you gave it, or else by its species in the form it had then ("Primal Reversion Groudon").

### Changed

- In your house in X and Y, only the room you are in is listed, with its stairs.
- Sun and Moon's passport photos are described in every language, in English for now.
- The release zip keeps every licence in one `licenses` folder, one file each.

### Fixed

- Doors and door mats facing east, west or south are said as a door or a way out instead of a
  warp.
- Lumiose City's openings onto the next street are said as paths.
- Pokémon walking about in X and Y, like Mom's Rhyhorn, are named instead of said as a person.

## [0.1.0] - 2026-10-16

The first release. pa3ds is a screen reader accessibility layer for 3DS Pokémon games running in the Azahar emulator: it reads the game's own memory while you play and sends what it finds to your screen reader, every word in your game's language. It reads Omega Ruby and Alpha Sapphire most completely, X and Y with some screens still silent, and Sun and Moon's dialogue, menus and screens, name entry, battles and map.
