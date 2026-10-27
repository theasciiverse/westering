# Westering: the overnight build plan

Night of 2026-10-27, built by Claude in a self-paced loop while David sleeps.
Each loop iteration reads this file, takes the first unticked milestone,
builds it, checks it (`cargo build`, `cargo clippy`, `cargo test`, and the
headless `--script` run below), commits locally, ticks it here and adds a
line to the log at the bottom.

Rules for the night:

- Local git only: commits in this folder's own repo, author email
  sandquist@gmail.com. No GitHub repo, no push, nothing published.
- Our own text, names and code. Nothing from MECC's program or any later
  Oregon Trail game: no "You have died of dysentery", no ford/caulk/ferry
  menu wording copied, no profession list (banker/carpenter/farmer), no
  tombstone screen copied. History (the real trail, its forts and rivers,
  emigrant diaries) is free to use.
- Plain text first, teletype style, green on black with amber accents
  (Asciiverse palette). Half-block art only once the game plays well.
- No em-dashes or en-dashes anywhere, in code, text or commits.
- Session logging (log + env_logger) from the start.
- A headless mode for testing without a terminal, like Moraen's `--frame`:
  `westering --script "keys" [--seed N]` plays the keys and prints the
  final screen as text. Use it to check every milestone.
- Commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Milestones

- [x] 1. Crate scaffold: Cargo.toml (edition 2024, ratatui 0.30, log,
  env_logger), main loop, title screen, `--script`/`--seed` headless mode,
  a seeded RNG of our own (no rand dependency needed), git init.
- [ ] 2. The party and outfitting: name the leader and four companions
  (defaults offered), a starting purse, the general store in Independence,
  Missouri, spring 1848: oxen (yokes), flour and bacon (lb), clothing
  sets, powder and shot, spare wheels, axles, tongues. Pick the departure
  month (March to July) with its trade-off.
- [ ] 3. The trail loop: days pass, miles covered by pace (steady,
  hard, grueling) and oxen health; rations (filling, meager, bare) use
  food; health per person; weather by month and region; a status panel
  (date, weather, health, food, next landmark, miles). Rest a day; trade.
- [ ] 4. The real route: landmarks with true mileages from Independence
  (about 2,000 miles to the Willamette): Kansas River crossing, Big Blue
  River, Fort Kearny, Ash Hollow, Courthouse and Chimney Rock, Scotts Bluff,
  Fort Laramie, Independence Rock, South Pass, Fort Bridger (or the
  Sublette Cutoff choice), Soda Springs, Fort Hall, Three Island Crossing
  of the Snake, Fort Boise, the Blue Mountains, the Whitman Mission, The
  Dalles, then the Barlow Road or rafting the Columbia. A short
  historical note at each, in our own words. Forts have stores at
  rising prices.
- [ ] 5. Events: illness (cholera, mountain fever, scurvy, measles,
  a broken arm), wagon breakdowns (spares fix them), lost or strayed oxen,
  storms, hail, a found cache, a trade offer, a lost trail, bad water,
  grass scarce. Weighted by region and season, written in our own words.
  Deaths are possible; the game goes on with the survivors.
- [ ] 6. River crossings: depth and width from season and the river;
  choices to wade, float the wagon, pay for a ferry where one historically
  ran, or wait for the water to drop; outcomes with real risk.
- [ ] 7. The end: arrival in the Willamette Valley (or every one lost),
  a score from survivors, goods and how early you arrived, and a short
  epitaph line for each who died on the trail, written by the player
  (default offered). Play again.
- [ ] 8. Hunting as text: spend a day and shot, success by skill of
  the dice and region (buffalo on the plains, antelope, deer, elk in the
  mountains), carry limit of 100 lb back to the wagon.
- [ ] 9. Balance: run 200 seeded headless playthroughs with a simple
  bot strategy and tune so a sensible party arrives about 60 to 70
  percent of the time and a careless one rarely does. Record the numbers
  here.
- [ ] 10. Catalog readiness: README.md (in the style of Moraen's), LICENSE
  (MIT, David Sandquist), CLAUDE.md status updated, .github/workflows/
  release.yml (adapted from terminal-pong, no nfpm packages unless trivial),
  hub manifest draft in this folder (not in hub, David reviews it first).
- [ ] 11. If time is left: half-block landmark vignettes (Chimney Rock,
  Independence Rock, the Blue Mountains), a wagon rolling across the
  travel screen, a map screen of the trail so far.

## Log

- M1 done: crate, title screen, `--script`/`--seed` headless mode, SplitMix64 RNG with a test.
