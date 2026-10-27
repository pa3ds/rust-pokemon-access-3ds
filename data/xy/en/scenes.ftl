### What X and Y show without words, in pa3ds's own words, a beat at a time, said under [pa3ds.ui]
### descriptions. When each is said is `../scenes.toml`'s, every language's; a beat is found here
### by its id there (ADR-0040). A beat this language has no words for is said in English's.
###
### A name the game shows is never written here: `{ POKEMON(716) }` is the game's own word out of
### the player's ROM, as are `{ ITEM(4) }` and the game's own name, `{ GAME() }`. Words drawn as a
### picture no line holds are the picture's words in `pictures.ftl`, `{ PICTURE("push_start") }`.
###
### Each scene was described from pictures of a replay; the comments say which, and what the
### game shows around it.

# X's title screen, from boot (x_intro_and_title_screen). The prompt is a picture with no line in
# the game's text, said as `pictures.ftl` calls it; the logo is the game's name, as its ROM calls
# it. The bottom screen plays a reel of its own under the logo, Xerneas and then the journey.
x-title-light = The title screen. In the dark, a small blue light glows and grows, flares into a shining cross of light, and bursts into sparks.
x-title-emblem = A great letter X stands dark against a beam of white and green light, its upper arms branching like antlers, and turns deep blue.
x-title-logo = The view rushes into the light. The Pokémon logo shines out over swirling teal light, and the blue X sweeps in beside it.
x-title = The logo reads { GAME() }. Along the bottom, blinking: { PICTURE("push_start") }.
x-title-xerneas = On the bottom screen, { POKEMON(716) } stands in a misty forest in shafts of sunlight: a tall blue and black deer on long thin legs, its great antlers glowing in every colour of the rainbow. The view circles slowly round its head, its legs and its antlers spread wide.
x-title-riding = Then scenes of a journey. A girl in a red hat rides a { POKEMON(672) } along a fenced dirt path and across grassy ledges, and a boy in a red cap rides a { POKEMON(111) } over rocky ground among boulders.
x-title-city = A white archway with a golden emblem, and a tall white tower of steel lattice reaching into the sky. A girl in a red beret walks down a broad avenue of trees and tall stone buildings toward the tower. Then a long avenue of trees leads to a palace.
x-title-amie = { POKEMON(255) }, { POKEMON(25) } and { POKEMON(700) } smile out of a bright sky. On a sunny lawn { POKEMON(700) } dances, a { POKEMON(667) } munches a pink treat, a hand pets a { POKEMON(694) }, and hearts float up.
x-title-skates = The boy, on roller skates, weaves among orange cones on a court, grinds along a rail through the trees, and glides down a path beside the water.
x-title-hats = The girl twirls in a ring of golden music notes, then shows off hats: a white one with sunglasses on its brim, a white cap with a yellow flower, a pink and blue cap.
x-title-flight = The boy stands on sand under streaks of blue light as white feathers drift past. A dark winged shape swoops down, and he rides it up into the sky. A flash of white.
x-title-credit = { POKEMON(716) } stands in the misty forest again, and words fade in below it: { PICTURE("presented_by") } { PICTURE("gf_logo") }.

# Its film, when the title screen is left alone for a minute: quick cuts of battles.
x-movie-ball = A { ITEM(4) } arcs across a blue sky, and { POKEMON(650) } bursts out onto a grassy field.
x-movie-starters = { POKEMON(653) }, { POKEMON(650) } and { POKEMON(656) } appear side by side.
x-movie-fire = { POKEMON(650) } lashes out with a whirl of green leaves at { POKEMON(653) }, which answers with a blast of fire that fills the screen.
x-movie-water = { POKEMON(656) } hurls a swirling ball of water that crashes over { POKEMON(653) }.
x-movie-flashes = Quick flashes: { POKEMON(697) } roaring as a blast bursts against it, { POKEMON(668) } in a flood of white light, a { POKEMON(694) } sparking at a flock of white birds.
x-movie-charizard = In the dark, a black { POKEMON(6) } with blue flames at its mouth spreads its wings on a rocky stage under a spotlight. A pale, broad-shouldered { POKEMON(150) } gathers a glowing blue sphere, and { POKEMON(6) } streaks at it through yellow light. Everything goes dark.

# Y's title screen (y_intro_and_title_screen): X's to the frame, with Yveltal for Xerneas and a
# red Y for the blue X; the journey between them is the same.
y-title-light = The title screen. In the dark, a small red light glows, then bursts in a flash of pink and red light.
y-title-emblem = A great letter Y stands dark against a beam of white light, its arms spread like jagged wings, and turns red.
y-title-logo = The view rushes into the light. The Pokémon logo shines out over swirling clouds, and the red Y sweeps in beside it.
y-title = { x-title }
y-title-yveltal = On the bottom screen, { POKEMON(717) } glides among clouds in a purple sky: a huge red and black bird with wide jagged wings and a long tail. The view circles round its wings, its claws and its sharp head with one blue eye.
y-title-riding = { x-title-riding }
y-title-city = { x-title-city }
y-title-amie = { x-title-amie }
y-title-skates = { x-title-skates }
y-title-hats = { x-title-hats }
y-title-flight = { x-title-flight }
y-title-credit = { POKEMON(717) } wheels among the clouds again, and words fade in below it: { PICTURE("presented_by") } { PICTURE("gf_logo") }.

# Its film: X's, with Charizard and Mewtwo in other forms.
y-movie-ball = { x-movie-ball }
y-movie-starters = { x-movie-starters }
y-movie-fire = { x-movie-fire }
y-movie-water = { x-movie-water }
y-movie-flashes = { x-movie-flashes }
y-movie-charizard = In the dark, an orange { POKEMON(6) } with horns and wide wings stands on a rocky stage under a spotlight. A slender, pale { POKEMON(150) } with a long tail gathers a glowing blue sphere, and { POKEMON(6) } streaks at it through yellow light. Everything goes dark.

# Prism Tower's quiz rooms, from the rig's own run through the Gym (spike 252): the picture each
# question shows for a few seconds before it is asked, never named, and the booths at the back.
prism-quiz-1 = Before the question, a black shape in a white frame: a small creature standing on two legs, with two long pointed ears and a tail bent in zigzags like a lightning bolt.
prism-quiz-2 = Before the question, a black shape in a white frame: a small round bird on two thin legs, with a short pointed beak and a tail of long feathers fanned up behind it.
prism-quiz-3 = Before the question, a black shape in a white frame: a small monkey standing on two legs, with big round ears and a thin tail curling at its end. A smooth tuft sweeps up from its head and curls back in one long rounded wave.
prism-quiz-4 = Before the question, three butterflies in black, one after another. The first: a plump body, a round head with two long antennae curling at their tips, and rounded wings. The second, seen from the front: broad wings with wavy, scalloped edges, the upper pair pointed at their tips. The third: a moth with its wings spread wide and flat, notched at their tips, two stubby horns and a fringe under its body.
prism-booths-1 = At the back stand three booths, a trainer in each, with big numbers above them: 1 on the left, 2 in the middle and 3 on the right.
prism-booths-2 = { prism-booths-1 }
prism-booths-3 = { prism-booths-1 }
prism-booths-4 = { prism-booths-1 }
