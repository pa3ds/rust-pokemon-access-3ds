# Known issues

What pa3ds gets wrong or leaves out for now. To report one that is not here, see
[When something goes wrong](../README.md#when-something-goes-wrong).

## Every game

- **Some touch-screen apps are only partly read**: the DexNav, Super Training and the PSS.
  Turning a PlayNav page with its arrows says nothing of the new page.
- **X, Y, Sun and Moon** are read less completely than Omega Ruby and Alpha Sapphire.

## Pokémon-Amie

- **The names of a Pokémon's own parts** have not been checked by a sighted player yet, so some
  may name the wrong part. In every language but English they are a machine's translation.
- **The descriptions of your Pokémon's animations** were written from still pictures and have
  not been checked by a sighted player either. In every language but English they are a machine's
  translation.
- **With an Azahar older than 2126.2** the stylus says no parts, and a pet goes where the stylus
  is rather than to a part.
- **With full Affection** a pet says no hearts, as the number can rise no further.
- **Poké Puffs**: only the five the bar shows at once are offered.
- **Head It** says when a ball reaches its Pokémon only with `head_it_cue` on.
- **Tile Puzzle**: "Yes" is said as a puzzle begins, the last swap's points are said as it is
  solved, and a tile says where its piece of the picture belongs, not what is on it.

## Super Training

- **Tapping a training bag faster than about twice a second**, some of your Pokémon's hits run
  together and go unheard: about a third of them, at a tap every 0.4 seconds.

## The map

- **How far above or below** a thing is, is in the game's units, 18 to a square, as `C` and
  `Shift+C` say heights: a thing 54 below is three squares down, and a slope or stairs can take
  you there in three steps.
- **A part of a town or route joined to the rest only through the next one** can list two or
  more exits to the same place, and `P` walks you out and back in. Step off the way while you are
  out, and pa3ds says the thing you chose is gone, since it can read only the place you are in:
  walk in through the entrance `P` directed you to earlier, then press `P` again.

## The PC Box

- **An empty Battle Box** reads as the box behind it.
- **Carrying a Pokémon**, the slot it is over says who is in it, except in a box that has the
  same name as another box.
- **Opening the Box** the first time after starting Azahar may say "Normal Pokérus Paralysis"
  before the first Pokémon: the empty summary the game shows for a moment, held on screen while
  Azahar loads what the Box needs for the first time.

## Omega Ruby and Alpha Sapphire

- **Sorting the Bag with Y**: picking an item up is said only at your first step, as "moving"
  and the item.
- **In the Y button's menu**, what Select and Start do is not said.
- **A scene you skip** goes undescribed from there, and a description waits while the emulator is
  paused.
- **A Pokémon met in seaweed under water** is said as one met walking, and as met in grass in the
  Pokédex, because the game itself shows it that way.
- **Shoal Cave's ice** has been tried in Omega Ruby only, and `P`'s way across it has not been
  heard in play yet.
- **Team Magma's and Team Aqua's Hideouts**: `P` now leads you through a teleporter pad into the
  same hideout as one step of the way, as in Lysandre Labs, and nobody has tried it there.
- **The Acro Bike's rails** work with `P` on Route 119's top. In the Safari Zone `P` led you to
  the end of a rail standing over low ground, where no way goes on; that is fixed, and the
  Safari Zone's rails have not been tried with `P` since. The rail at Route 119's foot has not
  been tried.
- **The moves `P` says a way needs** are those of `Shift+P`'s shortest way, so one of them may only
  be a shortcut, such as a slope the Mach Bike rides up where a longer way goes round on foot.
- **The Safari Zone's other quarters**: someone in another quarter is listed where they start,
  and may have wandered off until you walk into their quarter. Until you have been in a quarter
  since starting pa3ds, it may also list someone the game has taken away.
- **Parts of a place**: Mauville City's floors, a cave's floors and a hideout's rooms are not
  named as parts yet, so their exits still say only what they are.
- **Your house and Professor Birch's in Littleroot Town** are named by their signs, "Leos's
  House", in every language but French, where both are still "door".
- **Secret Bases**: `P` walks you over mats, Pitfall Mats, Stands, Stairs and Solid Boards, but
  goes round the mats and panels that move you, and round Slides, tents and Paradoxical Poppers,
  so it may find no way where a base's owner made the only one across them.

## X and Y

- **A scene you skip** goes undescribed from there, and a description waits while the emulator is
  paused.
- **Cyllage Gym's climbs** have been tried on one wall, the one nearest the entrance. On the walls
  facing east or west, "north" and "south" along the wall may not be the keys that climb it.
- **Slides** have been tried in Cyllage Gym only. `P` also slides down a few short drops outside
  it that look the same, in Cyllage City and on Routes 8 and 10 among them, which nobody has tried.
- **Shalour Gym's rails**: `P` takes the rail out of the Gym only from the centre down to the
  entrance, never back up it. Skating onto a rail with the D-pad has not been tried.
- **The Tower of Mastery's ramp**: the camera turns as you climb, and `P` says the way in the
  keys of the camera you are under, so the steps past the next turn of the camera change once
  you reach it. Listen for the next step at each landing. One place in Coumarine City has ways
  high over the ground like it, walked the same way, and nobody has tried it.
- **Anistar Gym** has been tried from the entrance up to the first Psychic and through the first
  teleporter and back. The rest of the way to Olympia has not been walked with `P`.
- **Coumarine Gym's vines**: `P` never reaches the vine on the west platforms from the fourth
  level up to the fifth, and goes round it. The way down from the top level is listed, but `P`
  does not take it on the way to anything, so `P` to the way out from the top climbs back down
  the vines.
- **Lumiose City's plazas**: walking off a street onto a plaza at its "open ground" has not been
  tried. Round Prism Tower the directions have been tried on the tower's west side only, and `P`
  to a street off it from the Pokémon Center's street and from Autumnal Avenue; where it used to
  leave you a step short of the street, saying a direction that walked you into a wall, it now
  takes you to where that direction steps onto the street, which has not been tried in the game
  yet.
- **The Poké Ball Factory's conveyor belts** were tried while Team Flare holds the factory. Of the
  belts the switch could turn, two have not been ridden turned; `P` takes them to run as before.
- **Lysandre Labs** has been tried from the entrance west over the first arrow tile. Riding a
  round pad, a push that crosses a second arrow, and the rooms behind the doors have not been
  tried; the doors are said as "passage", and the books and the report in the room with the
  bookshelves are not named yet.
- **Snowbelle Gym**: its four switches are listed by colour, and `P` goes round the gaps in the
  coloured floors as the switches have turned them. Where the gaps cut the way, `P` says "can't
  continue" rather than which switch to step on. Only the pink switch has been tried. Step on a
  switch within half a minute of loading a save in the Gym and `P` may not know it turned yet.
  After Wulfric is beaten, the ice slide up the middle has not been tried with `P`.
- **Riding on Routes 9 and 17**: getting off and smashing a rock have not been tried with `P`.
- **Rails outside Shalour Gym**, on Routes 5, 13 and 18 and in Dendemille and Couriway Towns, are
  found the same way and nobody has tried them. A rail broken in the middle, as Route 5's over the
  water is, is not taken by `P`.
- **The Pokémon League's Radiant Chamber**: the way up to the Champion is listed as an elevator,
  the round glass platform, and `P` walks you onto it. The door it takes you to is not listed,
  since it stands in the wall. `P` onto the platform has not been tried in the game yet.
- **The Elite Four's rooms**: `P` now walks you straight north from the platform you arrive on,
  over the way back down, which only takes you down walking south. A way out of a building
  across its mat going north is the same rule, and has been tried in the Elite Four's rooms
  only.

## Sun and Moon

- **Your save's language** cannot be read: set it yourself (see [Languages](languages.md)).
- **How many Pokémon a trainer has** is not told as their battle opens: `trainer_party` works in
  Omega Ruby, Alpha Sapphire, X and Y only.
- **Townspeople are named by how they look**, "Old woman", "Clerk", "Little boy": the game has
  no word for them, so pa3ds chose one from their clothes and what they say, and one can be
  wrong. A few people, about 15 in the whole game, are still "person".
- **Some doors are only "door"**: every plain house, which the game never names, and a few
  buildings its map does not place, such as the Trainers' School and the Thrifty Megamart.
- **A board can run its two lines together** where its second line has no quotation marks:
  "Hau'oli Police Station Keep our roads safe!".
