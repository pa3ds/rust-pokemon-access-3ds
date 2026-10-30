### What pa3ds calls the pictures the games draw in place of words, and what it says of them.
###
### `texture-fight_button` is the picture drawn with `fight_button.bclim`, by its name less
### `.bclim` (spike 50, `games.common.pictures`). A texture with no entry in the player's
### language or in English is said in words made from its name, `fight_button` "fight", and the
### log says so ("no word in data for texture ..."), which is how this list grows. A game's own
### `pictures.ftl` is read before this one, so its entry for a texture wins.
###
### Each is the word for what the picture does, as a sighted player reads it, and in the
### player's language: the game names none of them.

## Pictures whose name says what they do, in English or close to it: each is its name's words,
## less the designer's "button", "btn" and "icon", a move's "waza", a contest's "con", and the
## size "mini" (the user, 2026-09-16 to 2026-10-09).

texture-bag_button = bag
texture-button = button
texture-camera_btn_mini = camera
texture-change_box = change box
texture-clear = clear
texture-cleared = cleared
texture-fight_button = fight
texture-icon = icon
texture-icon_return = return
texture-mega_button = mega
texture-ok_button = ok
texture-pokemon = pokémon
texture-pokemon_button = pokémon
texture-return_btn = return
texture-return_button = return
texture-return_icon = return
texture-run_button = run
texture-start = start
texture-waza_con_change = change
# Super Training's `gameover_c.bclim` runs two words together.
texture-gameover = game over

## Pictures whose name says what they look like, or says it in Japanese, by what they do
## (Orion, 2026-10-01).

# The AreaNav's map draws `map_big` on the button that makes the map bigger, and `map_small` on
# the one that takes it back (movie17).
texture-map_big = zoom in
texture-map_small = zoom out
# The Bag's `rireki_icon` ("rireki", history) lists the last items used (Orion, 2026-10-02,
# movie21).
texture-rireki_icon = history
# The PC Box's three modes, its red tab (a Poké Ball), its blue (two balls and an arrow both ways)
# and its green (a grid and an arrow): pick a Pokémon up as usual, move one with a press, and
# select a group (Orion, 2026-10-02, movie22).
texture-mode01 = normal mode
texture-mode02 = swap mode
texture-mode03 = multi-select mode
# The DexNav's crown for an area's Pokémon all caught, by its colour, decoded from the ROM:
# brown, grey, yellow and a paler grey (Orion, movie28).
texture-complete0 = bronze crown
texture-complete1 = silver crown
texture-complete2 = gold crown
texture-complete3 = platinum crown
# The round ▶ under each app's logo on the PlayNav, which starts that app, its tutorial the first
# time (spike 198).
texture-menu_bg_button = start
# A message window's backing, which the PlayNav draws behind the menu its Pokémon opens as a touch
# area, and a tap on it closes the menu (omega_ruby_bottom-screen2).
texture-msg_win_c = close
# Super Training's button in the lower right of its PlayNav page, which opens the list of training
# bags; the game names it only in a sentence (Khai, 2026-10-09, omega_ruby_bottom-screen5).
texture-list_button = training bags
# The button that starts a regimen of Super Training once it is cleared, `stage_icon00_rock` and
# every `stage_icon` after it: before then the game titles each (Khai's session of 2026-10-09).
texture-stage_icon = start
# The battle's move info screen, opened from its move list, draws each move's button as a
# `waza_pos` picture with no words: no name of its own, said by its place (`picture-move`)
# (or_move-learning, 2026-10-18).
texture-waza_pos = {""}

# Pokémon-Amie's decorate screen (`kawaigari_arrange`): `bgbutton_L` and `bgbutton_R` turn its
# wallpaper in the top bar and its items' page beside the grid, `button_back` closes the items,
# and `button_exit` leaves the screen (x_pokemon-me and or_pokemon-me, 2026-10-18).
texture-bgbutton_L = previous
texture-bgbutton_R = next
texture-button_back = return
texture-button_exit = exit
# Pokémon-Amie's own three buttons, beside the Pokémon PLAY opens it on: a Poké Puff in the top
# left, which opens the Poké Puffs to feed it, a music note in the top right, which opens the
# games, and a Poké Ball in the bottom right, which goes back to the PlayNav (as_play_nav, and
# the guides).
texture-button_food = Poké Puffs
texture-button_play = games
texture-button_ball = return

## The ◀ and ▶ in the top bar's corners, which turn the PlayNav's pages and the sub-screen
## tutorial's: both draw `button.bclim`, and only their side of the screen tells them apart
## (spike 198).

top-bar-previous = previous
top-bar-next = next

## What pa3ds says of a picture rather than what it is called.

# A button that is only the plate other buttons' words are written on, and a slot of the PC Box
# holding no Pokémon.
picture-empty = empty
# A battle's button drawn with no words, by its place among those like it in reading order: the
# move info's four bars, in the move list's order (Khai, 2026-10-18).
picture-move = move { $number }
# A Box mark set or not, which the game says only by drawing it solid: "●, on". The mark is its
# sign, which the screen reader names.
mark-on = { $mark }, on
mark-off = { $mark }, off
# A slot the PC Box's pointer rests on where no words are (spikes 129 and 143): a party's by its
# number, a box's by its row and column under the box's name, "BOX 1, row 2, column 3", the
# Battle Box's by its number after its name, "BATTLE BOX 2". While a Pokémon is carried, a slot
# holding another is said with its name: "BOX 1, row 1, column 2, erza".
slot-party = Party { $number }
slot-grid = row { $row }, column { $column }
slot-titled-grid = { $title }, row { $row }, column { $column }
slot-titled-list = { $title } { $number }
slot-empty = { $slot }, empty
slot-held = { $slot }, { $name }
# A contest's places, drawn as pictures beside each entrant, said after the entrant: "Raiden &
# Metagross third place" (Orion, 2026-10-09).
contest-place-one = first place
contest-place-two = second place
contest-place-three = third place
contest-place-four = fourth place
# What the DexNav's red mark says beside a level above the area's, a first move that is an egg
# move, and the hidden ability, as the guides read it (docs/plans/dexnav.md).
dexnav-higher-level = higher level
dexnav-egg-move = egg move
dexnav-hidden-ability = hidden ability
# Said after a Pokémon the DexNav shows in colour, caught, where the rest are its silhouettes:
# only the caught are said so (Orion, 2026-10-06).
dexnav-caught = caught
# Potential's stars as words, three in a row filled or not; and the stars Pokémon-Amie's games rate
# a result with, out of five.
dexnav-stars = { $count ->
    [one] { $count } star
   *[other] { $count } stars
}
# Those games' stars with half a star after the whole ones: "4 and a half stars".
stars-and-a-half = { $count ->
    [0] half a star
   *[other] { $count } and a half stars
}
# A contest move's hearts as words, those hidden counted as none: "Appeal 4 hearts".
contest-hearts = { $count ->
    [one] { $count } heart
   *[other] { $count } hearts
}
