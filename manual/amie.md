# Pokémon-Amie

Pokémon-Amie is played on the touch screen. pa3ds lets you play it with touch mode
([Keys](keys.md#touch-mode)), with keys of its own for petting, and with keys of its own in each of
its three games. The settings for all of it are under `[pa3ds.amie]` ([Settings](settings.md)). What
does not work yet is in [Known issues](known-issues.md#pokémon-amie).

## Getting there

In Omega Ruby and Alpha Sapphire, Pokémon-Amie is a page of the PlayNav; in X and Y, the bottom
screen shows it on its own. Touch mode offers the Pokémon walking there, by name, and the menu:
PLAY opens Amie, DECORATE its decorate screen, and SWITCH its Switch screen. Another player's
Pokémon sometimes visits the page, "visiting Sceptile"; a tap says whose it is.

## Keys in Amie

While Amie is open, these keys are Amie's, in any mode:

| Key | What it does |
|---|---|
| Arrow keys | Move the stylus a step over your Pokémon, saying the part it comes onto, instead of the circle pad. |
| `Ctrl` and an arrow | Jump the stylus to the next part that way; while your Pokémon holds up its hand, to the hand. |
| `R` | Say your Pokémon's Affection, Fullness and Enjoyment: "Affection 2 of 5". |

And touch mode's keys, on what it offers in Amie:

| Key | On "beckon" | On the stylus | On a Poké Puff |
|---|---|---|---|
| `U` (tap) | Beckon your Pokémon. | Pet for about a second. | |
| `Shift` (drag) | | Pet for as long as you hold it. | Carry it, and give it to your Pokémon. |

## Beckoning

Touch mode offers to beckon your Pokémon, "beckon Ralts". Amie sometimes opens with your
Pokémon's back to you and its buttons hidden: tap beckon three or four times, about a second
apart, and it turns round and comes up to you, and the buttons appear.

## Petting

Touch mode offers the stylus, "stylus, head", which you pet your Pokémon with. The arrow keys move
it over your Pokémon, saying the part it comes onto by what it is on your Pokémon: Pikachu's
cheeks, Metagross's metal X. A part with no name of its own is said by its kind: head, body, an
eye, an ear, a hand, or a part with a touch of its own, flame, sparks, ice, spikes, slime, sludge
or ghostly. Off your Pokémon it says "off Ralts". The arrows keep it within a step of your
Pokémon.

To pet, choose the stylus in touch mode. `U` rubs it back and forth across the part for about a
second, and `U` again before the second is out keeps rubbing without lifting it. Or hold Shift to
rub for as long as you hold it: it starts a moment after you press it, so Shift pressed to quiet
your screen reader touches nothing. With the `drag` setting at `"toggle"`, press Shift once to
start and again to stop. The arrows move the stylus while it rubs. Your Pokémon counts a pet only
while the stylus keeps moving and stays down, and starts counting afresh each time it is lifted.

The game's own sounds tell you the rest. A part that burns, shocks or freezes your hand plays its
sound; the petting sound goes higher on a spot your Pokémon loves and lower on one it dislikes.

Your Pokémon moves about a lot, so the stylus moves over a still picture of it, taken as it
appears and again when it comes up to a beckon. Each part stays where you found it, and a pet goes
to that part wherever your Pokémon has taken it.

**A Pokémon too big for the bottom screen**, such as Wailord or Snorlax, stands on the top screen
with little of it below. Touch mode then offers "bring Wailord down": a tap presses for three
seconds, and "Wailord came down" says it is on the bottom screen, with the whole of it for the
stylus. "send Wailord back up" returns it.

## A high five

Move the stylus off your Pokémon and hold Shift, and keep holding: your Pokémon watches your hand,
and up to about eight seconds later holds up its own, sooner the fonder it is of you. A Pokémon you
have barely played with does not hold up its hand at all. Its hand waves for about two seconds:
press Ctrl and any arrow while it does, with Shift still down, and the stylus jumps to the hand.
Keep holding Shift, and the game's sound says when the hand comes back to it. With `descriptions`
on (below), pa3ds says when the hand goes up and "High five!" when one lands.

## Poké Puffs

Amie's Poké Puffs button opens a bar of puffs, each said by its kind. To give one, hold Shift on
it and carry it to your Pokémon with touch mode's keys, and keep Shift down while your Pokémon
eats it: let go once you hear it finish and cry, a few seconds, longer for some Pokémon than
others.

## Your Pokémon's numbers

`R` says your Pokémon's Affection, Fullness and Enjoyment, each as the row of five icons the
Switch screen shows: "Affection 2 of 5". The Switch screen says them too, as it opens and as you
choose another Pokémon, and touch mode offers your party there by name.

## What your Pokémon is doing

With `descriptions` under `[pa3ds.amie]` on, pa3ds says what your Pokémon is seen doing, in a
sentence: "Pikachu is looking around", "Pikachu is watching your hand". Each of its animations is
described as it plays, in a sentence of that Pokémon's own, and with `fidgets` on the small
movements it makes by itself every few seconds too. pa3ds says what a pet shows, "3 hearts", or
"Pikachu didn't like that...", when Affection fills another heart, and a Poké Puff by its kind
where you carried it with touch mode. Both settings are off as pa3ds ships.

## Decorating

On the decorate screen, touch mode names each Poké Puff by its kind and each interior item by its
name.

## Berry Picker

Pokémon pop up wanting a berry, and you give each the one it wants before it goes. pa3ds says each
as it comes, "Pinap Berry for Treecko", one at a time: the next once you have given a berry or the
one said has gone. A Pokémon waits only about two seconds, and a wrong berry sends it away.

| Key | What it does |
|---|---|
| `1` | Razz Berry |
| `2` | Bluk Berry |
| `3` | Pinap Berry |
| `4` | Wepear Berry, from Normal |
| `5` | Magost Berry, from Normal |
| `6` | Durin Berry, on Hard |
| `7` | Nomel Berry, on Hard |
| `8`, `9`, `0` | The round's first, second and third Pokémon, each keeping its key all round. |

A berry's key picks it up, and a Pokémon's key then gives it to that Pokémon; with no berry
picked up, a Pokémon's key says what it wants. As each round is about to start, pa3ds says the
three Pokémon in their keys' order. Another berry's key picks that one up instead. A key with
nothing to do, as `7` on Easy, plays the game's error sound. Outside Berry Picker these keys are
Azahar's as ever.

Touch mode works too: it offers the berries on the tree by name, left to right, and each Pokémon
wanting one. Hold Shift on a berry, choose the Pokémon with `J`, `L`, `I` and `O`, and let go:
the berry goes straight to it. Nothing moves until you let go, so choosing past another Pokémon is
safe. A tap knocks a berry off the tree for nobody.

For a higher score, lower Azahar's speed limit in its general settings: at 50 percent each Pokémon
waits twice as long, and the round lasts two minutes. Turn on realtime audio in Azahar's audio
settings, or the sound stutters.

How Berry Picker is said: `berry_first`, `one_at_a_time` and `berry_picker_style`, which can say
each Pokémon by its own cry.

## Head It

Balls of yarn fall at your Pokémon, and you tap a Pokémon as its ball reaches it to head it back.
pa3ds says which Pokémon each ball falls at as it comes: "Ralts". Tap it about a second after its
name; too early or too late is a miss. On Easy the Pokémon you play with stands alone in the
middle at first, and two more come up beside it as the round goes on.

| Key | What it does |
|---|---|
| `8` | Tap the Pokémon on the left. |
| `9` | Tap the Pokémon in the middle. |
| `0` | Tap the Pokémon on the right. |
| `R` | Say the score, the combo and the time or the misses left: "819, 98-HIT COMBO, Time Left: 23". |

As each round starts, pa3ds says its three Pokémon in that order, those not up yet too. A key
where no Pokémon stands, or before the balls start, plays the game's error sound. Outside Head It
these keys are Azahar's as ever. Touch mode offers each Pokémon by name too.

The score, the combo and each MISS! are left unsaid as they come, so the names keep up with the
balls; the game plays its own sound for a miss. The time left is said each half minute and over
the last ten seconds, the misses you have left as they change, and the results say the rest.
`head_it_score` says the score and the combo as they come instead.

With `head_it_cue` on, the game's "!" sound plays as the moment to tap a ball comes: in your left
ear for the left Pokémon, both ears for the middle, your right ear for the right. Press the
Pokémon's key as soon as you hear it: that is steadier than timing your tap from the sound of the
ball falling. At each round's results, pa3ds says its own estimate of how long you take to press
after the cue; put it in the `head_it_lead` setting, or nudge from it. `head_it_style` can say
each Pokémon by its own cry, or say nothing, for the cue alone.

## Tile Puzzle

Each tile is said as a spreadsheet names a cell, its column a letter from A and its row a number
from 1, and then where the piece of the picture on it belongs: "B1, goes to D3", or "in place".
Tap a tile, then the tile where its piece belongs, to swap them. Its difficulties say the best
stars earned on each, and while you play its points are said only as a puzzle starts and once it
is solved. With `descriptions` under `[pa3ds.ui]` on, the picture is described: which of your
Pokémon stand where, and where it is set. Over the finished picture there is nothing to touch: B
goes back.

## Changing Amie's keys

Amie's keys are in `user\keys.toml`, each game's in a table of its own, taken only while it runs
and winning there over pa3ds's other keys ([Changing keys](keys.md#changing-keys)):

- `[pa3ds.amie.stylus]`: `"stylus up"`, `"stylus down"`, `"stylus left"` and `"stylus right"` on
  the arrows, `"stylus jump up"` to `"stylus jump right"` on Ctrl and the arrows, and `"stylus
  numbers"` on `R`.
- `[pa3ds.amie.berry_picker]`: `"first berry"` to `"seventh berry"` and `"first pokemon"` to
  `"third pokemon"`.
- `[pa3ds.amie.head_it]`: `"left pokemon"`, `"middle pokemon"`, `"right pokemon"` and `"score"`.
