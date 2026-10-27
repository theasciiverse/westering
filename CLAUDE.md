# Westering

An Asciiverse journey game in the line of **The Oregon Trail** (Don
Rawitsch, Bill Heinemann and Paul Dillenberger, 1971; MECC): a wagon party
heading west, with supplies to buy, distances to cover, rivers to cross,
game to hunt, weather and illness to survive, and the odds against
arriving. Rust + Ratatui, our own code, writing and art.

**Status:** playable start to finish, built overnight 2026-10-27 by Claude
in a loop while David slept (`PLAN.md` has the milestones and the log). Put
on the roster by David, 2026-10-27, as the answer to "can we offer The
Oregon Trail in Asciiverse?" (see Rights). Local git repo only; no GitHub
repo, release or cast yet. A draft hub manifest and page are in
`asciiverse/` for David to review before they go to `hub/catalog/`.

## Code

Rust + Ratatui, edition 2024, no dependencies beyond ratatui, log and
env_logger.

- `main.rs`: terminal loop and the headless modes: `--script KEYS` plays
  keys and prints the screen (`>` Enter, `<` Backspace, `!` Esc, `^` `_`
  `{` `}` arrows), `--bot N` runs N journeys with a sensible and a careless
  rule player and prints the outcome, `--bot-show SEED [--careless]` prints
  one's last screen
- `game.rs`: the screens and every key; `ui.rs` draws them on a 72-column page
- `party.rs` (people, goods, store prices), `trail.rs` (a day: miles, food,
  weather, health, oxen), `route.rs` (the 25 landmarks), `events.rs`,
  `river.rs`, `hunt.rs`, `bot.rs`, `date.rs`, `rng.rs` (SplitMix64, so a
  seed replays a journey)
- `RUST_LOG=debug westering 2>westering.log` logs keys, screens, events and
  outcomes
- Balance is pinned by a test in `bot.rs`; rerun `--bot 400` after changing
  any odds and update the numbers in `PLAN.md`

## Name

**Westering** is a working name, not chosen yet: the 19th-century word for
the pull and the act of going west. Collision check, 2026-10-27: one web
search found no game called Westering; Steam, itch.io and crates.io not
checked yet.

## Rights: why our own game, not a port

Checked 2026-10-27:

- **The original is not open.** Rawitsch published the BASIC source in
  *Creative Computing* in 1978, and the 1975 and 1978 versions were
  reconstructed in 2011 (Wikipedia, "The Oregon Trail (series)"). Nothing
  found puts that code under an open licence or in the public domain; a
  forum post says it was registered with the US Copyright Office in 1979,
  not confirmed against the registry. The community ports on GitHub carry
  licences for their own code only.
- **The franchise is live.** Gameloft has published The Oregon Trail since
  2008 (2021 on Apple Arcade, 2022 on PC and consoles), and Apple Studios
  acquired the film rights in October 2024. So the name and the original
  code are someone's; a port of the 1975 BASIC would need the rights
  holder's permission.
- **The game's ideas are free:** the journey, the shopping, the pace and
  rations, hunting, river crossings, illness, the trail's landmarks as
  history. Others have built on them under their own names (Westward Ho!
  1847 on fuzearena.com).

So, as with Panzer, Moraen and the other spiritual successors: our own
name, code, text and art, and nothing from MECC's program or from later
Oregon Trail games. History itself (the real trail, its forts, rivers and
diaries) is open to us.

## Place in the catalog

On the website's history page, under 1971, as a descendant of The Oregon
Trail once it is playable.

## Open

The name (Steam, itch.io and crates.io not checked); the art (plain text
now, half-block landmark scenes considered); an action hunting scene; the
cast recording; the GitHub repo and first release.
