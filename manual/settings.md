# Settings

Settings are in `user\config.toml`, written with the defaults on the first start. pa3ds adds the
settings later versions bring, at their defaults, and never changes a line you have. Each setting
has a comment in the file saying what it does.

| Setting | Default | What it does |
|---|---|---|
| **`[pa3ds]`** | | |
| `keys` | `true` | Answer pa3ds's keys, and rebind Azahar's buttons as `keys.toml` says. |
| `poll_rate` | `60` | How many times a second pa3ds reads the game. 30 for a slower machine. |
| `azahar_folder` | `""` | The folder of a portable Azahar. Empty, pa3ds uses the open Azahar's folder or its own where either is a portable Azahar. pa3ds fills it in when you choose the folder it asks for. |
| **`[pa3ds.ui]`** | | |
| `pauses` | `false` | Pause between pieces of screen text said together, such as a label and what follows it. |
| `interrupt` | `false` | Cut off speech when a menu's cursor moves, instead of queueing. pa3ds's own keys always cut off speech. |
| `unfocused` | `false` | Keep saying what the game shows while another window is in front. |
| `language` | `""` | The language of pa3ds's own words: `en`, `fr`, `de`, `es`, `it`, `ja`, `ko`, `zh-CN` or `zh-TW`. Empty follows your save. See [Languages](languages.md). |
| `descriptions` | `false` | Describe the scenes the game shows without words, said as "Narration:". |
| **`[pa3ds.audio]`** | | How loud pa3ds's own sounds are, beside the game's. Each follows Azahar's volume: 1 as loud as the game, 0.5 half as loud, 1.5 louder, 0 none. |
| `tones` | `1.0` | The short tone as the map's cursor or touch mode moves: higher the further up, panned the further right. |
| `footsteps` | `1.0` | pa3ds's footsteps, where the game plays none. |
| **`[pa3ds.navigation]`** | | |
| `positions` | `true` | Say where a thing is in the list: "door, 1 of 3". |
| `position_first` | `true` | Say the place in the list before the way to it. `false` says "door, 1 north, 1 of 3". |
| `turn_by_turn` | `true` | Say each turn of the way `P` found as you reach it. `false` says the whole way once. |
| `guide_to_hidden` | `true` | When the DexNav finds a hidden Pokémon, find the way to it as `P` does. |
| `track_boulders` | `false` | Say how far a big boulder has moved from where it started, so you know which you have pushed. Off by default, since the game never shows where a boulder started. |
| `hide_door_signs` | `true` | Leave a sign out of the list when all it says is the name of the door beside it. |
| `show_y` | `false` | `C` and `Shift+C` say how high as well, after x and z, in the game's units: 18 to a square. |
| **`[pa3ds.extras]`** | | See [Extras](#extras). |
| `conditions` | `false` | Say a Pokémon's contest conditions as numbers, on its summary and in the Pokéblock Case. |
| **`[azahar]`** | | See [Azahar settings pa3ds needs](#azahar-settings-pa3ds-needs). |

If `config.toml` or `keys.toml` has a mistake pa3ds cannot read past, pa3ds uses its defaults,
leaves the file as it is, and says as it starts which line the mistake is on.

## Extras

pa3ds says what the game shows, and no more. An Extra says more exactly something the game shows
in a way pa3ds cannot otherwise put into words. Each is off until you turn it on under
`[pa3ds.extras]`.

- **`conditions`**: the game draws a Pokémon's contest conditions as a star. pa3ds cannot read
  the star, so it says the numbers the star is drawn from.

## Command-line options

Run `pa3ds.exe --help` for the full list. The ones a player may want:

- `--rom PATH`: your decrypted ROM, if pa3ds does not find it among the ones Azahar has opened.
- `--no-keys`: answer no keys and rebind no buttons.
- `--port PORT`: Azahar's RPC port, if you changed it from 45987.

## Azahar settings pa3ds needs

pa3ds writes these into Azahar's `qt-config.ini`, in `%APPDATA%\Azahar\config`, or for a portable
Azahar in `user\config` beside `azahar.exe`. They come from the `[azahar]` table of
`user\config.toml`, so you can change them there.

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

Put a `#` in front of a line to leave that setting as Azahar has it. A line you delete is put
back the next time pa3ds starts. Any other Azahar setting can be added by its name in
`qt-config.ini`.

## Your files

Everything that is yours is in the `user` folder beside `pa3ds.exe`. Copy that folder to keep it
all when you move pa3ds. Updating keeps it where it is.

- `config.toml`: settings.
- `keys.toml`: keys.
- `names\`: your names.
- `logs\`: what pa3ds did and said in each of the last ten runs, `pa3ds.log` the latest, then
  `pa3ds.1.log` back to `pa3ds.9.log`.
- `pa3ds.lock`: there while pa3ds runs, so a pa3ds started again can take its place.
- `cache\`: what pa3ds worked out from your ROM. Safe to delete; it is built again.

The `data` folder beside `user` is pa3ds's, not yours: its words, the names it gives people and
things, and its descriptions of scenes. An update replaces it. Without it pa3ds says "pa3ds data
folder not found" as it starts, and has much less to say.
