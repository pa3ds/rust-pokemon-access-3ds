# Known issues

What pa3ds gets wrong or leaves out for now. To report one that is not here, see
[When something goes wrong](../README.md#when-something-goes-wrong).

## Every game

- **Some touch-screen apps are only partly read**: the DexNav, Pokémon-Amie, Super Training and
  the PSS. Super Training's regimen descriptions read their two columns run together, and
  turning a PlayNav page with its arrows says nothing of the new page. Pokémon-Amie's Switch
  screen calls each party member "button pokémon" and says no hearts, puffs or notes after
  Affection, Fullness and Enjoyment. Of its games, Berry Picker's Unlimited says the misses you
  have left as a number alone, "2", and Berry Picker does not say when a Pokémon has eaten a
  berry or gone away; Head It says when a ball reaches its Pokémon only with `head_it_cue` on,
  past about 100 hits in a row its balls fall so fast that the cue comes early for about one in
  three, and it reads its results out of order, the total last; in Tile Puzzle "Yes" is said as a puzzle
  begins, the last
  swap's points are said as it is solved, and a tile says where its piece of the picture belongs,
  not what is on it. pa3ds does not say when your Pokémon has eaten a Poké Puff, and only the five
  puffs the bar shows at once are offered. It does not say when your Pokémon in Amie has its back
  to you, or when it turns round. With an Azahar older than 2126.2, Amie's stroke goes across the
  middle of the screen, and can miss a Pokémon standing to one side.
- **X, Y, Sun and Moon** are read less completely than Omega Ruby and Alpha Sapphire.

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
- **Opening the Box** sometimes says "Normal Pokérus Paralysis".

## Omega Ruby and Alpha Sapphire

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
- **Riding on Routes 9 and 17**: getting off and smashing a rock have not been tried with `P`.
- **Rails outside Shalour Gym**, on Routes 5, 13 and 18 and in Dendemille and Couriway Towns, are
  found the same way and nobody has tried them. A rail broken in the middle, as Route 5's over the
  water is, is not taken by `P`.

## Sun and Moon

- **Your save's language** cannot be read: set it yourself (see [Languages](languages.md)).
