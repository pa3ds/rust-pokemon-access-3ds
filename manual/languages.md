# Languages

In Omega Ruby, Alpha Sapphire, X and Y, pa3ds reads your game's language from your save: the one
you chose as your game began. Everything pa3ds reads from the game is in that language, and so
are pa3ds's own words. Until a save is loaded, pa3ds speaks English.

In Sun and Moon pa3ds cannot read your save's language. Set `language` under `[pa3ds.ui]` in
`user\config.toml` to your game's.

A game translated by fans may keep the language it replaced in its save: the Chinese Omega Ruby
says Japanese. pa3ds still reads the game's words right, but would say its own in Japanese. Set
`language = "zh-CN"` and pa3ds says its own words in Chinese.

## pa3ds's own words

They are in the `data` folder beside pa3ds: every game's in `data\common`, and each game's own in
`data\oras`, `data\xy` and `data\sm`, with a folder for each language inside. They are plain
text: each line is a word's name, `=`, and the words.

Every language the games come in has every file. The translations began as a machine's, so some
words may sound wrong; names are each language's official ones where the games have one. A word missing from a language is said in English.

## Translating

To correct a word, change it after the `=` in your language's file. To translate pa3ds into
another language, copy the files of the `en` folders into folders named for your language, such
as `data\common\pt`, and change the words after each `=`. Keep what is inside `{ }` as it is:
`{ $count }` is a number pa3ds puts in, and `{ POKEMON(383) }` a name it reads from your game.

Send your files to the developers so they ship with pa3ds. An update replaces the `data` folder,
so keep a copy of yours.
