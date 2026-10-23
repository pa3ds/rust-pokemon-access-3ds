### What Omega Ruby and Alpha Sapphire show without words, in pa3ds's own words, a beat at a time,
### said under [pa3ds.ui] descriptions. When each is said is `../scenes.toml`'s, every language's;
### a beat is found here by its id there (ADR-0040). A beat this language has no words for is said
### in English's.
###
### A name the game shows is never written here: `{ POKEMON(383) }` is the game's own word out of
### the player's ROM, as are `{ TRAINER(235) }`, `{ ITEM(534) }`, `{ PLACE(198) }`, and a whole line
### of the game's text drawn as a picture, `{ TEXT(123, 91) }`, and the game's own name, `{ GAME() }`.
### Words drawn as a picture no line holds are the picture's words in `pictures.ftl`, by its
### texture, `{ PICTURE("push_start") }`. `{ MEMORY(24) }` is the Pokémon the
### player's own save keeps for the credits' 24th moment, in the form it keeps, and a beat naming one
### the save has not kept is left out. `{ YOURS(24) }` is the same Pokémon as the player's own: the
### nickname they gave it, or `scene-yours` below with its name in. Someone the game never names is
### `{ SPRITE(179) }`, called what a speaker drawn as sprite 179 is: `sprites.ftl`, behind the
### player's own name for them. A scene whose names the ROM does not have is left out, and logged.
###
### Each scene was described from screenshots of a replay; the comments say which, and what the
### game shows around it.

# The player's own Pokémon they gave no nickname, the game's name for it in: "your Mega Latios".
scene-yours = your { $pokemon }

# Omega Ruby's intro, from boot to the title screen, and the title screen
# (or_intro_and_title_screen, spike 236). The studio's credit and the title screen's prompt are
# pictures with no line in the game's text, said as `pictures.ftl` calls them, `{ PICTURE(...) }`;
# the logo is the game's name, as its ROM calls it, `{ GAME() }`.
#
# The intro's first part, on the field's engine.
or-intro-forest = A { POKEMON(267) } flutters among tall, dewy leaves. Below lies a small, dark pond in the forest, two small Pokémon peeking in at its edge.
or-intro-drops = The view sinks to the water. A drop of dew falls from a leaf tip, and rings spread across the pond.
or-intro-credit = Words fade in over the water: { PICTURE("presented_by") } { PICTURE("gf_logo") }.
or-intro-volcano = The view rises through the plants. Beyond the forest stand two mountains under a blue sky, one a smoking volcano.
or-intro-latios = { POKEMON(381) } swoops past low over the trees and climbs away into the sky. A flash of white.
or-intro-island = High over the sea, { POKEMON(381) } glides past an island: a brown volcano glowing red at its crater, a yellow field of ash, green forest. It dives away in a streak of mist.
or-intro-bike = On a dirt road, a red bike rolls along. Its rider is a boy in a white cap, red and black clothes and a green backpack.
or-intro-loop = He looks up. { POKEMON(381) } loops through the sky above him and streaks away. He closes his eyes and smiles.
or-intro-face = The view closes in on his face, and everything fades to white.

# Its second part, a movie of quick cuts.
or-intro-movie-1 = A { ITEM(4) }, drawn in strokes of paint, rushes at you. Then quick flashes: { POKEMON(323) } in a ring of fire, { POKEMON(319) } leaping from dark water, { POKEMON(383) } roaring under a blazing sun.
or-intro-movie-2 = { POKEMON(257) } in a desert, flames streaming from its wrists. { POKEMON(260) } by the sea, fists raised. { POKEMON(254) } in a dark forest.
or-intro-movie-3 = { TRAINER(235) } stands in a pillar of fire and pushes up his glasses. Then { TRAINER(557) }, in a cave of carved walls.
or-intro-movie-4 = Against an orange sky, the black shape of { POKEMON(383) }, a light burning in its chest. A sign like the letter omega forms in fire, flames swirl round it, and white light floods in.

# The title screen.
or-title-logo = The title screen. The Pokémon logo shines out of the dark among shards of red crystal, then bursts into white light.
or-title = The logo reads { GAME() }, burning in gold and red over a sunburst of flames. Along the bottom, blinking: { PICTURE("push_start") }. On the bottom screen, { POKEMON(383) } looms out of smoke and embers, then fades back.

# Alpha Sapphire's intro, the same as Omega Ruby's but for Latias in Latios's place, and the same
# boy on a girl's save (as_intro_and_title_screen); the rest of its beats are Omega Ruby's words.
as-intro-forest = { or-intro-forest }
as-intro-drops = { or-intro-drops }
as-intro-credit = { or-intro-credit }
as-intro-volcano = { or-intro-volcano }
as-intro-latias = { POKEMON(380) } swoops past low over the trees and climbs away into the sky. A flash of white.
as-intro-island = High over the sea, { POKEMON(380) } glides past an island: a brown volcano glowing red at its crater, a yellow field of ash, green forest. It dives away in a streak of mist.
as-intro-bike = { or-intro-bike }
as-intro-loop = He looks up. { POKEMON(380) } loops through the sky above him and streaks away. He closes his eyes and smiles.
as-intro-face = { or-intro-face }

# Its movie: Team Aqua's side where Omega Ruby's shows Team Magma's.
as-intro-movie-1 = A { ITEM(4) }, drawn in strokes of paint, rushes at you. Then quick flashes: { POKEMON(319) } leaping from dark water, { POKEMON(323) } in a ring of fire, { POKEMON(382) } floating on the sea as storm clouds gather, and rain lashing a dark sea.
as-intro-movie-2 = { or-intro-movie-2 }
as-intro-movie-3 = { TRAINER(178) }, a dark shape in the deep among rising bubbles, then laughing as he grips his collar. Then { TRAINER(557) }, in a cave of carved walls.
as-intro-movie-4 = Against deep blue, the black shape of { POKEMON(382) }, a light burning at its heart. A sign like the letter alpha forms in water, blue drops swirl round it, and white light floods in.

# Its title screen.
as-title-logo = The title screen. The Pokémon logo shines out of the dark among shards of blue crystal, then bursts into white light.
as-title = The logo reads { GAME() }, glowing in gold and blue over shimmering light. Along the bottom, blinking: { PICTURE("push_start") }. On the bottom screen, { POKEMON(382) } swims through bubbles in the deep, its markings glowing white.

# The opening, before the player is anywhere: the professor's lines are the rest of the game's
# text, file 118, and the stretches between them where he says nothing are described
# (OR_character-desc.ctm, 2026-10-15).
#
# "And this...is what we call a POKéMON.", then 12 s without a word.
opening-pokemon-1 = { POKEMON(298) } appears beside the professor: small, round and blue, with big round ears and a blue ball on the end of its tail.
opening-pokemon-2 = The picture pulls back. The professor is on the little screen of a yellow handheld with a carrying handle, lying in the dark back of a moving truck among bunk beds, cardboard boxes and a plush mouse doll.
opening-pokemon-3 = Outside, the truck's brown back doors rattle along a dirt road through a sunny forest.

# "...But that's enough about me!", then 21 s without a word.
opening-forest-1 = { POKEMON(357) } beats its broad leaf wings and flies off between the trees. The view sinks down into tall grass.
opening-forest-2 = A { POKEMON(278) } sweeps up out of the treetops on long, thin white wings, and a small dark bird climbs after it into the blue sky.
opening-forest-3 = They fly up into the sun, and its light floods everything white.
opening-forest-4 = Back in the dark truck, a bright line of light shines between the closed doors onto the handheld, where the professor waits on the screen.

# "...You're the one who's moving to Littleroot Town, where I live!", then 8 s without a word. The
# same words are line 56 too, and either matches.
opening-handheld-1 = On the handheld's screen you are shown from behind, then you turn round to face the front with a smile.
opening-handheld-2 = The screen changes to a map of the region: green islands in a blue sea, its towns marked in red.
opening-handheld-3 = The handheld is set down, and the dark truck rumbles on.

# "...Come see me in my Pokémon Lab!", then 11 s without a word until the player stands in the
# truck, zone 0x18f.
opening-truck-1 = The truck's back doors swing wide, and daylight pours in until everything is white.
opening-truck-2 = The light fades. You stand in the back of the parked truck among cardboard boxes and furniture, with sunlight spilling in through the open doors.

# May, before she says her name (movie1_opening, Omega Ruby): her mother's "She should be in her
# room upstairs", then a demo of 9 s of her room, and the player arrives in it.
may-room-1 = A bedroom upstairs: a map on the wall, a bed with red covers under the window, a desk by the wall. A girl stands at the desk with her back to you, her hands raised to the red and white bow in her hair.
may-room-2 = Up close, she leafs through a notebook on the desk, then checks the pouch on her belt.
may-room-3 = You stand at the top of the stairs, in a corner of her room.

# "Pokémon fully restored, items all ready, and..."
may-turns-1 = She turns round, starts at the sight of you, and walks over.

# Route 102 (movie4_petalburg): "Look! Up there! Check 'em out, [name]!", 15 s before she names
# what they saw.
may-beautifly-1 = You and { TRAINER(4) } stand side by side in the tall grass, facing a line of pines. Three butterfly Pokémon flutter out of the trees on bright yellow and black wings.
may-beautifly-2 = They flutter past the trees toward the red roofs of a town.
may-beautifly-3 = They climb into the sky and drift away among the clouds.

# Granite Cave's last room (scene_granite_cave_mural, in Alpha Sapphire and replayed in Omega
# Ruby): a demo of 22 s as the player walks in toward the steps, before Steven's first line, then
# the player is walked up to him. He is not named until he names himself, as a sighted player
# does not know him yet. Each version paints a wall of its own; the rest is the same.
or-granite-mural-1 = The screen goes dark. A circle of light moves slowly over pictures painted on rock: a great dark beast with a jagged, scaly back and its jaws open wide, then a row of smoking mountains.
or-granite-mural-2 = The view pulls back. The pictures cover a whole wall of the cave: the beast rearing up among rings and curling plumes on the left, tiny figures under a streak of light on the right, and the smoking mountains below. A man stands alone at the top of a flight of stone steps beneath it, looking up.
or-granite-mural-3 = Up close, the man has pale blue hair and a dark suit. He gazes up at the wall with a hand at his chin, then folds his arms, deep in thought.
or-granite-mural-4 = You stand at the foot of the steps, and walk up them to his side.

as-granite-mural-1 = The screen goes dark. A circle of light moves slowly over pictures painted on rock: jagged streaks, rows of curling spirals like waves, and a great dark shape marked with a ring.
as-granite-mural-2 = The view pulls back. The pictures cover a whole wall of the cave: tiny figures under streaks of light on the left, swirling clouds and waves on the right, and the great shape between them. A man stands alone at the top of a flight of stone steps beneath it, looking up.
as-granite-mural-3 = Up close, the man has pale blue hair and a dark suit. He gazes up at the wall with a hand at his chin, then folds his arms, deep in thought.
as-granite-mural-4 = You stand at the foot of the steps, and walk up them to his side.

# Steven's "Good luck to you. I must be going now."
granite-steven-leaves-1 = { TRAINER(557) } walks down the steps and out of the room, leaving you alone before the painted wall.

# Seafloor Cavern, zone 0x6f: Archie's "Augh... You...", after Maxie's battle and before Groudon
# wakes, a demo of 65 s.
or-groudon-wakes-1 = { TRAINER(235) } walks to the edge of the lava, where { POKEMON(383) } lies curled up, grey as stone.
or-groudon-wakes-2 = { TRAINER(235) } holds up the { ITEM(534) }. It blazes with red light, and cracks of light run across { POKEMON(383) }'s stone shell.
or-groudon-wakes-3 = { POKEMON(383) } rises out of the lava, red again. It opens its eyes and looks up at the cavern roof.
or-groudon-wakes-4 = It towers over the lava as { TRAINER(235) } and you watch from the shore.
or-groudon-wakes-5 = { POKEMON(383) } bursts out into the open sky over the sea in a flash of light.
or-groudon-wakes-6 = The sky turns a fierce burning yellow, and { POKEMON(383) } roars.

# Tabitha's "Child! You're coming, too! Hurry up!": the lava pool is empty, and everyone runs out
# of the cavern onto Route 128, zone 0x39.
or-cavern-empties-1 = The lava pool is empty. { POKEMON(383) } is gone.
or-cavern-empties-2 = The sky blazes white and gold. You stand on a sandbar in the sea with { TRAINER(178) }, { TRAINER(235) } and { TRAINER(691) }, under a burning sun.

# Route 128: Tabitha's "Let's go. Boss Maxie...".
or-route128-tabitha-1 = { TRAINER(235) } and { TRAINER(691) } leave.

# Archie's "So...I'll be waitin' for you in Sootopolis.", before Steven comes.
or-route128-archie-1 = { TRAINER(178) } leaves. You stand alone on the sandbar, looking into the glare.
or-route128-archie-2 = Someone flies down out of the glare and lands beside you.

# Steven's "Good luck to you."
or-route128-steven-1 = { TRAINER(557) } flies off into the glare.

# The Cave of Origin's deepest floor, zone 0x76, each time the player comes down into it: the
# only way on is north, where the floor narrows onto a ledge at the magma, and the scene where the
# Magma Suit goes on starts as the player walks into that narrowing (an invisible trigger seven
# cells wide at (17, 24), which fills it). Nothing stands there; a sighted player sees the floor
# narrow toward the glow.
or-cave-depths-1 = The stairs come down at the east end of a passage. It runs west, then north through a gap into a wide hall, which narrows at its north end toward a river of glowing magma.

# The Magma Suit goes on as the scene starts, and its line comes after.
or-magma-suit-1 = The suit is red, with heavy boots and a round helmet you look out through.

# "fitted the suit with the Blue Orb!"
or-blue-orb-1 = The { ITEM(535) } glows, and a bubble of blue light closes around you.
or-blue-orb-2 = Blue light washes over the river of magma beside you, then everything flashes white.

# Groudon's roar there.
or-groudon-roars-1 = { POKEMON(383) } rears up out of the magma beside you.

# "The sound cut out...", then a demo of 24 s, to zone 0x7a.
or-sound-cut-out-1 = You leap onto { POKEMON(383) }'s back.
or-sound-cut-out-2 = { POKEMON(383) } wades into the magma and carries you along a river of fire, deeper and deeper.
or-sound-cut-out-3 = A blinding light fills the tunnel ahead, and { POKEMON(383) } carries you into it.

# Zone 0x7a, Groudon's "Gurrrooouuuhhh!!!" as the player faces it, then a demo of 56 s to the
# battle.
or-groudon-faced-1 = Great crystals glow over the magma. { POKEMON(383) } stands before you.
or-groudon-faced-2 = Pink light pours over { POKEMON(383) } and seals it inside a giant crystal.
or-groudon-faced-3 = The crystal turns red and breaks, and a golden Omega symbol shines.
or-groudon-faced-4 = { POKEMON(383) } bursts out in its primal form, huge and blazing with lava, and towers over you.

# "Gotcha! Groudon was caught!", then a demo of 40 s and the field across Hoenn's towns, to
# Sootopolis.
or-groudon-caught-1 = The crystals overhead glow softly. You make your way back out through the cave.
or-groudon-caught-2 = Far above, light rises from the crater of { PLACE(198) } into the burning sky, and the glare bursts apart.
or-groudon-caught-3 = The sky turns deep blue, and green sparkles drift down like snow over a calm sea.
or-groudon-caught-4 = People come out and look up at the sparkling sky.
or-groudon-caught-5 = { TRAINER(518) } looks up at the sky.
or-groudon-caught-6 = A crowd looks up at the sky, and on the shore a group in red hoods watches it too.
or-groudon-caught-7 = By a rocket on its launch tower, { TRAINER(694) } looks up at the sky.
or-groudon-caught-8 = The glare is gone. Your friends wait on the pier before the cave.

# Alpha Sapphire, from movie1_kyogre_awakens. Seafloor Cavern, zone 0x6e: Maxie's "No... That...",
# after Archie's battle and before Kyogre wakes, a demo of 65 s.
as-kyogre-wakes-1 = { TRAINER(178) } walks to the edge of the water, where { POKEMON(382) } lies still, grey as stone.
as-kyogre-wakes-2 = { TRAINER(178) } holds up the { ITEM(535) }. It blazes with blue light, and light bursts out beneath { POKEMON(382) }'s stone shell.
as-kyogre-wakes-3 = { POKEMON(382) } glows blue again. It opens its mouth wide and rises in the water.
as-kyogre-wakes-4 = It swims around the pool as { TRAINER(178) } and you watch from the shore.
as-kyogre-wakes-5 = { POKEMON(382) } bursts out of the sea into the open sky in a spray of water.
as-kyogre-wakes-6 = Storm clouds close over the sky, heavy rain lashes the sea, and { POKEMON(382) } roars.

# Shelly's "You, too! Well, hurry up!": the pool is still, and everyone runs out of the cavern onto
# Route 128, zone 0x39.
as-cavern-empties-1 = The pool lies still. { POKEMON(382) } is gone.
as-cavern-empties-2 = Black storm clouds fill the sky and heavy rain pours down. You stand on a sandbar in the sea with { TRAINER(235) }, { TRAINER(178) } and { TRAINER(688) }.

# Route 128: Shelly's "Let's go, Archie!".
as-route128-shelly-1 = { TRAINER(178) } and { TRAINER(688) } leave.

# Maxie's "I will expect your arrival in Sootopolis.", before Steven comes.
as-route128-maxie-1 = { TRAINER(235) } leaves. You stand alone on the sandbar in the pouring rain.
as-route128-maxie-2 = Someone flies down out of the storm and lands beside you.

# Steven's "Good luck to you."
as-route128-steven-1 = { TRAINER(557) } flies off into the storm.

# Alpha Sapphire, from movie2_cave_of_origin. The Cave of Origin's deepest floor is zone 0x75, Omega Ruby's 0x76 laid out the
# same: its warp and its trigger stand on the same cells, but the hall opens onto deep water.
as-cave-depths-1 = The stairs come down at the east end of a passage. It runs west, then north through a gap into a wide hall, which narrows at its north end toward a pool of deep blue water.

# The suit goes on as the scene starts, and its line comes after.
as-aqua-suit-1 = The suit is blue with white trim, with yellow lights on its chest and a round helmet you look out through.

# "fitted the suit with the Red Orb!"
as-red-orb-1 = The { ITEM(534) } glows, and a bubble of red light closes around you.
as-red-orb-2 = Red light washes over the deep water beside you, then everything flashes white.

# Kyogre's roar there.
as-kyogre-roars-1 = { POKEMON(382) } surges up out of the water beside you.

# "The sound cut out...", then a demo of 24 s, to zone 0x79.
as-sound-cut-out-1 = You ride on { POKEMON(382) }'s back.
as-sound-cut-out-2 = { POKEMON(382) } dives and carries you down through deep blue water, past great rocks, deeper and deeper.
as-sound-cut-out-3 = A blinding light fills the water ahead, and { POKEMON(382) } carries you into it.

# Zone 0x79, Kyogre's "Krrraaawwwrrr!!!" as the player faces it, then a demo of 57 s to the
# battle.
as-kyogre-faced-1 = Great crystals glow over the water, under falls of water from the cavern roof. { POKEMON(382) } floats before you.
as-kyogre-faced-2 = Blue light pours over { POKEMON(382) } and seals it inside a giant crystal.
as-kyogre-faced-3 = The crystal goes dark, and a glowing blue Alpha symbol shines.
as-kyogre-faced-4 = { POKEMON(382) } bursts out in its primal form, huge and blue with glowing markings, and towers over you.

# "Gotcha! Kyogre was caught!", then a demo of 40 s and the field across Hoenn's towns, to
# Sootopolis.
as-kyogre-caught-1 = The crystals overhead glow softly. You make your way back out through the cave.
as-kyogre-caught-2 = Far above, green light rises from the crater of { PLACE(198) } into the storm clouds, and the clouds burst apart.
as-kyogre-caught-3 = The sky turns deep blue, and green sparkles drift down like snow over a calm sea.
as-kyogre-caught-4 = People come out and look up at the sparkling sky.
as-kyogre-caught-5 = { TRAINER(518) } looks up at the sky.
as-kyogre-caught-6 = A crowd looks up at the sky, and on the shore a group in blue and white stripes watches it too.
as-kyogre-caught-7 = By a rocket on its launch tower, { TRAINER(683) } looks up at the sky.
as-kyogre-caught-8 = The rain is gone. Your friends wait on the pier before the cave.

# Victory Road's way out, zone 0x7f, from movie48_battle_with_wally: Wally's "Ah! [name]!" as the
# player walks onto the trigger across the path at (21, 50).
wally-waits-1 = The path runs north through a meadow of red flowers between two pools, to wooden steps. { TRAINER(518) } stands at the top of them, and comes down to meet you.

# "Standing here facing you again gives me chills..."
wally-chills-1 = { TRAINER(518) } closes his eyes and bows his head, then looks up at you, determined.

# "Arghhh!!!", after his battle.
wally-loses-1 = { TRAINER(518) } flings his arms down and shouts, eyes shut tight, then hangs his head.

# "When I do, promise we'll battle again! That's a promise I intend to live up to!"
wally-promises-1 = { TRAINER(518) } steps off the path into the flowers, and the way north to the steps is clear.

# The Pokémon League, from movies 49 to 52. Each of the Elite Four's corridors is a bridge north to
# a hall, the same in shape and dressed for the member beyond it, and plays its scene as the
# player walks across the trigger halfway along it (row 34, five cells wide) with no line before
# or after: Sidney's 0x16b, Phoebe's 0x167, Glacia's 0x168 and Drake's 0x169, about 17 s before
# the player walks on into the room. The first is entered after "You heard doors closing!".
corridor-one-1 = Behind you, the way back is shut. A long bridge of red lacquered wood runs north through a dark sky of drifting embers, to a great hall with curved roofs and glowing windows.

corridor-one-bridge-1 = Golden clouds billow up on both sides beneath the bridge, and embers swirl around you.
corridor-one-bridge-2 = The hall's doors swing open onto white light.

corridor-two-1 = A bridge of purple stone, hung with wisteria, crosses a dark pond of pale glowing lotus flowers to a shadowy hall.

corridor-two-bridge-1 = A ghostly purple flame rises before the hall's doors, and wisps of it drift away.
corridor-two-bridge-2 = Lanterns along the hall flicker alight one by one.
corridor-two-bridge-3 = The hall's doors slide open onto darkness.

corridor-three-1 = Snow drifts through a pale blue haze around a bridge of pink stone. Ahead stands a hall roofed with snow and hung with icicles.

corridor-three-bridge-1 = A blizzard whirls up and hides the hall in white.
corridor-three-bridge-2 = It clears. The hall's doors stand open onto darkness, and frost creeps over the bridge toward your feet.

corridor-four-1 = A red bridge crosses a sea of glowing lava under a smoky sky full of sparks, to a hall with a great dragon's head carved over its door.

corridor-four-bridge-1 = Streaks of golden fire sweep around the hall like something flying.
corridor-four-bridge-2 = Flames burst from the dragon's head, its eyes blaze, and the doors open onto bright light.

# Sidney's room, zone 0x162: as the player comes in he sits on a black chair, eyes closed, then
# opens them and gets up, before his first line.
sidney-rises-1 = A square room with a battle court marked out on its floor and a great Poké Ball door in its far wall. { TRAINER(553) } gets up from a black chair before it, grinning.

# "Go on to the next room and challenge your next battle!"
sidney-beaten-1 = The Poké Ball door behind { TRAINER(553) } slides open.

# Phoebe's room, zone 0x163: her scene plays as the player walks up to her.
phoebe-waits-1 = A dim room of purple stone, lit by pale flames along its walls. { TRAINER(554) } laughs, a hand to her mouth.

phoebe-rises-1 = She walks a slow circle around you as she talks.

# "Come on! Just try and see if you can even inflict damage on my Pokémon!"
phoebe-challenges-1 = { TRAINER(554) } skips back before the door, flings her arms wide and laughs.

# "Go ahead. Move on to the next room."
phoebe-beaten-1 = The Poké Ball door behind { TRAINER(554) } slides open.

# Glacia's room, zone 0x164.
glacia-waits-1 = A room of pale blue ice, snow falling past its windows. { TRAINER(555) } stands before a Poké Ball door in a long lavender gown.

# "Advance to the next room. There you will learn how fearsome the Pokémon League can truly be!"
glacia-beaten-1 = The Poké Ball door behind { TRAINER(555) } slides open.

# Drake's room, zone 0x165.
drake-waits-1 = A room of warm amber, red lamps glowing on its wooden walls. The view rises from { TRAINER(556) }'s boots to his stern face; he stands with his arms crossed.

# "Go! Go onward! The Champion is waiting!"
drake-beaten-1 = { TRAINER(556) } steps aside, and the Poké Ball door behind him stands open.

# The Champion's corridor, zone 0x16a, from movie53_battle_with_steven: no trigger; the player
# walks up it to the light. Then the Champion's room, zone 0x166, whose own script starts as the
# player arrives: it walks them 12 rows up a staircase, whatever they hold, and keeps them there
# while the view goes to Steven, his first line 22.2 s after arriving in both of Orion's movies.
# So the room is described as the player arrives, a beat as each shot comes (Orion, 2026-10-16).
champion-corridor-1 = A long, narrow corridor of grey stone runs north to a doorway pouring white light.
champion-corridor-2 = You climb a wide staircase into a vast hall of pale stone.
champion-corridor-3 = At its far end { TRAINER(557) } stands on the steps below a dark archway, his back to you.
champion-corridor-4 = He turns his head and smiles.

# "You are rightfully the Hoenn region's new...", after his battle; the friend who runs in is the
# rival, named as the speaker after it.
steven-beaten-1 = Someone runs in behind you.

# May's "Huh? Wait—what? What? ... It's already over?"; Birch is drawn as HAKASE_RS, sprite 0xb3.
may-runs-in-1 = { SPRITE(179) } walks in after her.

# Brendan's "You didn't... No, you couldn't have... Did you already beat the Champion?!", where
# the player is a girl (movie53_battle_with_steven_as_a_girl); each of May's scenes here has a
# twin of his, on his line, for him alone.
brendan-runs-in-1 = { SPRITE(179) } walks in after him.

# "No, I should call you the Hoenn region's new Pokémon League Champion... Come with me, Champion."
steven-leads-on-1 = { TRAINER(557) } leads you up the steps to the archway, and the others stay below.

# May's "Raiden! Honestly, congratulations!"
may-congratulates-1 = { TRAINER(4) } waves from the foot of the steps as you follow { TRAINER(557) } through the archway.

# Brendan's "That's awesome, [name]. Congratulations...really!"
brendan-congratulates-1 = { TRAINER(1) } raises a hand from the foot of the steps as you follow { TRAINER(557) } through the archway.

# The Hall of Fame, zone 0x16d: "This room... This is where we maintain the records of Pokémon..."
hall-of-fame-room-1 = A long, dim hall lit by lamps along its walls. At its far end stands a round machine with a glowing blue top.

# "Come. We will use this machine to record your name...", then the Hall of Fame's demo of 37 s,
# and back in the Champion's room, zone 0x166, where the script holds the player at the top of
# the steps for 21.6 s before the friend's "Welcome back, [name]": the high windows, the player
# stepping out of the archway, a close-up of the friend with eyes shut.
hall-of-fame-1 = You set your Poké Balls into the hollows around the machine, and they glow.
hall-of-fame-2 = One by one your Pokémon appear in a burst of light against a bright backdrop, as confetti falls.
hall-of-fame-3 = You stand with your Pokémon around you. { TEXT(123, 111) }
hall-of-fame-4 = Light streams in through high windows.
hall-of-fame-5 = You come out at the top of the steps. Below, your friend waits alone, eyes closed.

# May's "Hee hee..."
may-giggles-1 = { TRAINER(4) } smiles, eyes shut, cheeks pink.

# Brendan's "D-don't go trying to read anything into it, though!"
brendan-blushes-1 = { TRAINER(1) } smiles, then knits his brows and glances away.

# May's "So...come on! Let's go home! Back to Littleroot Town!", then the credits: they start
# three times, before May's battle, after it and after the game saves, and their pictures run at
# their own pace, so each beat is timed from the start of its part. The Pokémon in each of the
# journey's pictures are the player's own, `{yours:N}` by their nicknames, but for the rival's
# starter, `{memory:2}`; the rest are the story's, Wally's and
# Steven's in their Mega forms, named from the game's list of forms (text 5). Seen in Omega
# Ruby with May as the rival, and with a line of hers, so with her alone; Brendan's twin follows.
or-credits-may-1 = You walk down the steps to { TRAINER(4) }. Light floods in through the high windows until all is white.
or-credits-may-2 = Sunlight through tall trees. You ride your bike along a forest path.
or-credits-may-3 = { TRAINER(4) } rides up beside you on her own bike.
or-credits-may-4 = Now pictures of your journey come and go over an old map of Hoenn. A bag dropped on a path with Poké Balls spilling out; a { POKEMON(261) }; and you, with { YOURS(0) }.
or-credits-may-5 = { TRAINER(4) } with her { MEMORY(2) }, and you with { YOURS(1) }, by a picture of a little pond.
or-credits-may-6 = { TRAINER(518) } lunges to throw a Poké Ball at a { POKEMON(280) }, with you beside him.
or-credits-may-7 = { TRAINER(561) }'s portrait and her { POKEMON(299) }, and { YOURS(5) }.
or-credits-may-8 = { TRAINER(563) }'s portrait and his { POKEMON(296) }, and { YOURS(6) }.
or-credits-may-9 = You and { YOURS(7) } with { TRAINER(557) }.
or-credits-may-10 = { TRAINER(235) }, by a picture of a hall of exhibits.
or-credits-may-11 = { TRAINER(567) }'s portrait and his { POKEMON(82) }, and { YOURS(8) }.
or-credits-may-12 = You and { TRAINER(4) } look at a picture of a mountain, with { YOURS(9) }.
or-credits-may-13 = { TRAINER(178) } raises a fist.
or-credits-may-14 = You send out { YOURS(10) } against { TRAINER(235) } and his { POKEMON(323) }.
or-credits-may-15 = { TRAINER(569) }'s portrait and her { POKEMON(324) }, and { YOURS(12) }.
or-credits-may-16 = { TRAINER(570) }'s portrait and his { POKEMON(289) }, and { YOURS(13) }.
or-credits-may-17 = { POKEMON(381) } and { POKEMON(380) } fly side by side.
or-credits-may-18 = { TRAINER(571) }'s portrait and her { POKEMON(334) }, and { YOURS(16) }.
or-credits-may-19 = You and { YOURS(17) }, by a picture of a long corridor.
or-credits-may-20 = { TRAINER(552) }'s portrait, and { YOURS(18) } and { YOURS(19) }.
or-credits-may-21 = { POKEMON(383) }, by a picture of a cavern.
or-credits-may-22 = You and { YOURS(21) } before { POKEMON(383) } in its primal form.
or-credits-may-23 = { TRAINER(572) }'s portrait and his { POKEMON(350) }, and { YOURS(23) }.
or-credits-may-24 = { TRAINER(518) } and his { TEXT(5, 816) }, and you with { YOURS(24) }.
or-credits-may-25 = You and { YOURS(25) } face { TRAINER(557) } and his { TEXT(5, 796) }.
or-credits-may-26 = Night. You ride your bike past a sleeping city.
or-credits-may-27 = Stars fill the night sky.
or-credits-may-28 = Night on { PLACE(208) }. You stand with { TRAINER(4) } among the trees under the stars.
or-credits-may-29 = A small pond among the leaves.
or-credits-may-30 = { TRAINER(4) } turns to you and smiles.
or-credits-may-31 = She raises her fists, ready to battle.
or-credits-may-32 = { TRAINER(4) } throws her arms up in the air, then turns to you.
or-credits-may-33 = Seen from above, the two of you walk side by side along a dirt path through the night.
or-credits-may-34 = { TRAINER(4) } points up at the sky.
or-credits-may-35 = Shooting stars streak across the night sky, one after another.
or-credits-may-36 = { TEXT(123, 91) }
or-credits-may-37 = Evening light falls across your desk at home: an open notebook, two Poké Balls and a map of Hoenn.
or-credits-may-38 = The light fades, and the room grows dim.
or-credits-may-39 = A dark emblem fades in over the desk, and blazes with green light.

# The same credits in Alpha Sapphire (movie4_hall_of_fame), picture for picture and second for
# second, but five: Archie by the hall of exhibits and Maxie where Archie stood, Archie's Sharpedo
# where Maxie's Camerupt was, and Kyogre where Groudon was.
as-credits-may-1 = You walk down the steps to { TRAINER(4) }. Light floods in through the high windows until all is white.
as-credits-may-2 = Sunlight through tall trees. You ride your bike along a forest path.
as-credits-may-3 = { TRAINER(4) } rides up beside you on her own bike.
as-credits-may-4 = Now pictures of your journey come and go over an old map of Hoenn. A bag dropped on a path with Poké Balls spilling out; a { POKEMON(261) }; and you, with { YOURS(0) }.
as-credits-may-5 = { TRAINER(4) } with her { MEMORY(2) }, and you with { YOURS(1) }, by a picture of a little pond.
as-credits-may-6 = { TRAINER(518) } lunges to throw a Poké Ball at a { POKEMON(280) }, with you beside him.
as-credits-may-7 = { TRAINER(561) }'s portrait and her { POKEMON(299) }, and { YOURS(5) }.
as-credits-may-8 = { TRAINER(563) }'s portrait and his { POKEMON(296) }, and { YOURS(6) }.
as-credits-may-9 = You and { YOURS(7) } with { TRAINER(557) }.
as-credits-may-10 = { TRAINER(178) }, by a picture of a hall of exhibits.
as-credits-may-11 = { TRAINER(567) }'s portrait and his { POKEMON(82) }, and { YOURS(8) }.
as-credits-may-12 = You and { TRAINER(4) } look at a picture of a mountain, with { YOURS(9) }.
as-credits-may-13 = { TRAINER(235) } stands, hands at his sides.
as-credits-may-14 = You send out { YOURS(10) } against { TRAINER(178) } and his { POKEMON(319) }.
as-credits-may-15 = { TRAINER(569) }'s portrait and her { POKEMON(324) }, and { YOURS(12) }.
as-credits-may-16 = { TRAINER(570) }'s portrait and his { POKEMON(289) }, and { YOURS(13) }.
as-credits-may-17 = { POKEMON(381) } and { POKEMON(380) } fly side by side.
as-credits-may-18 = { TRAINER(571) }'s portrait and her { POKEMON(334) }, and { YOURS(16) }.
as-credits-may-19 = You and { YOURS(17) }, by a picture of a long corridor.
as-credits-may-20 = { TRAINER(552) }'s portrait, and { YOURS(18) } and { YOURS(19) }.
as-credits-may-21 = { POKEMON(382) }, by a picture of a cavern.
as-credits-may-22 = You and { YOURS(21) } before { POKEMON(382) } in its primal form.
as-credits-may-23 = { TRAINER(572) }'s portrait and his { POKEMON(350) }, and { YOURS(23) }.
as-credits-may-24 = { TRAINER(518) } and his { TEXT(5, 816) }, and you with { YOURS(24) }.
as-credits-may-25 = You and { YOURS(25) } face { TRAINER(557) } and his { TEXT(5, 796) }.
as-credits-may-26 = Night. You ride your bike past a sleeping city.
as-credits-may-27 = Stars fill the night sky.
as-credits-may-28 = Night on { PLACE(208) }. You stand with { TRAINER(4) } among the trees under the stars.
as-credits-may-29 = A small pond among the leaves.
as-credits-may-30 = { TRAINER(4) } turns to you and smiles.
as-credits-may-31 = She raises her fists, ready to battle.
as-credits-may-32 = { TRAINER(4) } throws her arms up in the air, then turns to you.
as-credits-may-33 = Seen from above, the two of you walk side by side along a dirt path through the night.
as-credits-may-34 = { TRAINER(4) } points up at the sky.
as-credits-may-35 = Shooting stars streak across the night sky, one after another.
as-credits-may-36 = { TEXT(123, 91) }
as-credits-may-37 = Evening light falls across your desk at home: an open notebook, two Poké Balls and a map of Hoenn.
as-credits-may-38 = The light fades, and the room grows dim.
as-credits-may-39 = A dark emblem fades in over the desk, and blazes with green light.

# Brendan's "Now, come on! We're going home! Back to Littleroot Town!", then the credits where the
# player is a girl (movie53_battle_with_steven_as_a_girl): May's credits, picture for picture and
# second for second, with Brendan where she was.
or-credits-brendan-1 = You walk down the steps to { TRAINER(1) }. Light floods in through the high windows until all is white.
or-credits-brendan-2 = Sunlight through tall trees. You ride your bike along a forest path.
or-credits-brendan-3 = { TRAINER(1) } rides up beside you on his own bike.
or-credits-brendan-4 = Now pictures of your journey come and go over an old map of Hoenn. A bag dropped on a path with Poké Balls spilling out; a { POKEMON(261) }; and you, with { YOURS(0) }.
or-credits-brendan-5 = { TRAINER(1) } with his { MEMORY(2) }, and you with { YOURS(1) }, by a picture of a little pond.
or-credits-brendan-6 = { TRAINER(518) } lunges to throw a Poké Ball at a { POKEMON(280) }, with you beside him.
or-credits-brendan-7 = { TRAINER(561) }'s portrait and her { POKEMON(299) }, and { YOURS(5) }.
or-credits-brendan-8 = { TRAINER(563) }'s portrait and his { POKEMON(296) }, and { YOURS(6) }.
or-credits-brendan-9 = You and { YOURS(7) } with { TRAINER(557) }.
or-credits-brendan-10 = { TRAINER(235) }, by a picture of a hall of exhibits.
or-credits-brendan-11 = { TRAINER(567) }'s portrait and his { POKEMON(82) }, and { YOURS(8) }.
or-credits-brendan-12 = You and { TRAINER(1) } look at a picture of a mountain, with { YOURS(9) }.
or-credits-brendan-13 = { TRAINER(178) } raises a fist.
or-credits-brendan-14 = You send out { YOURS(10) } against { TRAINER(235) } and his { POKEMON(323) }.
or-credits-brendan-15 = { TRAINER(569) }'s portrait and her { POKEMON(324) }, and { YOURS(12) }.
or-credits-brendan-16 = { TRAINER(570) }'s portrait and his { POKEMON(289) }, and { YOURS(13) }.
or-credits-brendan-17 = { POKEMON(381) } and { POKEMON(380) } fly side by side.
or-credits-brendan-18 = { TRAINER(571) }'s portrait and her { POKEMON(334) }, and { YOURS(16) }.
or-credits-brendan-19 = You and { YOURS(17) }, by a picture of a long corridor.
or-credits-brendan-20 = { TRAINER(552) }'s portrait, and { YOURS(18) } and { YOURS(19) }.
or-credits-brendan-21 = { POKEMON(383) }, by a picture of a cavern.
or-credits-brendan-22 = You and { YOURS(21) } before { POKEMON(383) } in its primal form.
or-credits-brendan-23 = { TRAINER(572) }'s portrait and his { POKEMON(350) }, and { YOURS(23) }.
or-credits-brendan-24 = { TRAINER(518) } and his { TEXT(5, 816) }, and you with { YOURS(24) }.
or-credits-brendan-25 = You and { YOURS(25) } face { TRAINER(557) } and his { TEXT(5, 796) }.
or-credits-brendan-26 = Night. You ride your bike past a sleeping city.
or-credits-brendan-27 = Stars fill the night sky.
or-credits-brendan-28 = Night on { PLACE(208) }. You stand with { TRAINER(1) } among the trees under the stars.
or-credits-brendan-29 = A small pond among the leaves.
or-credits-brendan-30 = { TRAINER(1) } opens his eyes and smiles at you.
or-credits-brendan-31 = He raises his fists, ready to battle.
or-credits-brendan-32 = { TRAINER(1) } throws his arms up in the air, puts his hands on his hips, then turns to you.
or-credits-brendan-33 = Seen from above, the two of you walk side by side along a dirt path through the night.
or-credits-brendan-34 = { TRAINER(1) } points up at the sky.
or-credits-brendan-35 = Shooting stars streak across the night sky, one after another.
or-credits-brendan-36 = { TEXT(123, 91) }
or-credits-brendan-37 = Evening light falls across your desk at home: an open notebook, two Poké Balls and a map of Hoenn.
or-credits-brendan-38 = The light fades, and the room grows dim.
or-credits-brendan-39 = A dark emblem fades in over the desk, and blazes with green light.

# Brendan's credits in Alpha Sapphire: his Omega Ruby credits above with Alpha Sapphire's five
# pictures, composed before any replay showed them (ADR-0038, Orion, 2026-10-16), then seen so in
# movie4_hall_of_fame_as_a_girl.
as-credits-brendan-1 = You walk down the steps to { TRAINER(1) }. Light floods in through the high windows until all is white.
as-credits-brendan-2 = Sunlight through tall trees. You ride your bike along a forest path.
as-credits-brendan-3 = { TRAINER(1) } rides up beside you on his own bike.
as-credits-brendan-4 = Now pictures of your journey come and go over an old map of Hoenn. A bag dropped on a path with Poké Balls spilling out; a { POKEMON(261) }; and you, with { YOURS(0) }.
as-credits-brendan-5 = { TRAINER(1) } with his { MEMORY(2) }, and you with { YOURS(1) }, by a picture of a little pond.
as-credits-brendan-6 = { TRAINER(518) } lunges to throw a Poké Ball at a { POKEMON(280) }, with you beside him.
as-credits-brendan-7 = { TRAINER(561) }'s portrait and her { POKEMON(299) }, and { YOURS(5) }.
as-credits-brendan-8 = { TRAINER(563) }'s portrait and his { POKEMON(296) }, and { YOURS(6) }.
as-credits-brendan-9 = You and { YOURS(7) } with { TRAINER(557) }.
as-credits-brendan-10 = { TRAINER(178) }, by a picture of a hall of exhibits.
as-credits-brendan-11 = { TRAINER(567) }'s portrait and his { POKEMON(82) }, and { YOURS(8) }.
as-credits-brendan-12 = You and { TRAINER(1) } look at a picture of a mountain, with { YOURS(9) }.
as-credits-brendan-13 = { TRAINER(235) } stands, hands at his sides.
as-credits-brendan-14 = You send out { YOURS(10) } against { TRAINER(178) } and his { POKEMON(319) }.
as-credits-brendan-15 = { TRAINER(569) }'s portrait and her { POKEMON(324) }, and { YOURS(12) }.
as-credits-brendan-16 = { TRAINER(570) }'s portrait and his { POKEMON(289) }, and { YOURS(13) }.
as-credits-brendan-17 = { POKEMON(381) } and { POKEMON(380) } fly side by side.
as-credits-brendan-18 = { TRAINER(571) }'s portrait and her { POKEMON(334) }, and { YOURS(16) }.
as-credits-brendan-19 = You and { YOURS(17) }, by a picture of a long corridor.
as-credits-brendan-20 = { TRAINER(552) }'s portrait, and { YOURS(18) } and { YOURS(19) }.
as-credits-brendan-21 = { POKEMON(382) }, by a picture of a cavern.
as-credits-brendan-22 = You and { YOURS(21) } before { POKEMON(382) } in its primal form.
as-credits-brendan-23 = { TRAINER(572) }'s portrait and his { POKEMON(350) }, and { YOURS(23) }.
as-credits-brendan-24 = { TRAINER(518) } and his { TEXT(5, 816) }, and you with { YOURS(24) }.
as-credits-brendan-25 = You and { YOURS(25) } face { TRAINER(557) } and his { TEXT(5, 796) }.
as-credits-brendan-26 = Night. You ride your bike past a sleeping city.
as-credits-brendan-27 = Stars fill the night sky.
as-credits-brendan-28 = Night on { PLACE(208) }. You stand with { TRAINER(1) } among the trees under the stars.
as-credits-brendan-29 = A small pond among the leaves.
as-credits-brendan-30 = { TRAINER(1) } opens his eyes and smiles at you.
as-credits-brendan-31 = He raises his fists, ready to battle.
as-credits-brendan-32 = { TRAINER(1) } throws his arms up in the air, puts his hands on his hips, then turns to you.
as-credits-brendan-33 = Seen from above, the two of you walk side by side along a dirt path through the night.
as-credits-brendan-34 = { TRAINER(1) } points up at the sky.
as-credits-brendan-35 = Shooting stars streak across the night sky, one after another.
as-credits-brendan-36 = { TEXT(123, 91) }
as-credits-brendan-37 = Evening light falls across your desk at home: an open notebook, two Poké Balls and a map of Hoenn.
as-credits-brendan-38 = The light fades, and the room grows dim.
as-credits-brendan-39 = A dark emblem fades in over the desk, and blazes with green light.

# After the game restarts, Mossdeep City, zone 0x13: "XX:XX, the XXth day of the XXth month".
mossdeep-date-1 = A starry night sky.
mossdeep-date-2 = A tall building with round lit windows stands on a cliff above the dark sea.

# "Mossdeep Space Center Celestial Observatory", then inside, zone 0x152; Cozmo is drawn as
# SORAISI_RS, sprite 0x136.
space-center-1 = A control room: rows of desks with glowing monitors under a great screen. Scientists sit at them, and { SPRITE(310) }, an old man in a white coat, stands among them.

# "...Hmmn. So it's come to this.", then the Sky Pillar's top, zone 0xb5: at night in Omega Ruby,
# the disguise Magma's; by day in Alpha Sapphire (movie5_delta_episode_intro), the disguise Aqua's.
or-sky-pillar-1 = Night, at the top of a crumbling stone tower. Someone in a red hooded uniform stands before broken steps, a { POKEMON(293) } at their feet.

as-sky-pillar-1 = Day, at the top of a crumbling stone tower. Someone in a black bandana, a blue and white striped top and striped stockings stands before broken steps, a { POKEMON(293) } at their feet.

# "...Looks like I won't be needing THIS... Hiyyup!"
sky-pillar-hiyyup-1 = The uniform goes flying. Beneath it is a young woman with short black hair, a long ragged cloak and green boots.

# "I swear I'll protect you. ...Aster."
or-aster-1 = The view rises far above the tower, its top a small stone triangle in a sea of purple clouds.

as-aster-1 = The view rises far above the tower, its top a small stone triangle in a sea of white clouds.
