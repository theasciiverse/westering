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
- [x] 2. The party and outfitting: name the leader and four companions
  (defaults offered), a starting purse, the general store in Independence,
  Missouri, spring 1848: oxen (yokes), flour and bacon (lb), clothing
  sets, powder and shot, spare wheels, axles, tongues. Pick the departure
  month (March to July) with its trade-off.
- [x] 3. The trail loop: days pass, miles covered by pace (steady,
  hard, grueling) and oxen health; rations (filling, meager, bare) use
  food; health per person; weather by month and region; a status panel
  (date, weather, health, food, next landmark, miles). Rest a day; trade.
- [x] 4. The real route: landmarks with true mileages from Independence
  (about 2,000 miles to the Willamette): Kansas River crossing, Big Blue
  River, Fort Kearny, Ash Hollow, Courthouse and Chimney Rock, Scotts Bluff,
  Fort Laramie, Independence Rock, South Pass, Fort Bridger (or the
  Sublette Cutoff choice), Soda Springs, Fort Hall, Three Island Crossing
  of the Snake, Fort Boise, the Blue Mountains, the Whitman Mission, The
  Dalles, then the Barlow Road or rafting the Columbia. A short
  historical note at each, in our own words. Forts have stores at
  rising prices.
- [x] 5. Events: illness (cholera, mountain fever, scurvy, measles,
  a broken arm), wagon breakdowns (spares fix them), lost or strayed oxen,
  storms, hail, a found cache, a trade offer, a lost trail, bad water,
  grass scarce. Weighted by region and season, written in our own words.
  Deaths are possible; the game goes on with the survivors.
- [x] 6. River crossings: depth and width from season and the river;
  choices to wade, float the wagon, pay for a ferry where one historically
  ran, or wait for the water to drop; outcomes with real risk.
- [x] 7. The end: arrival in the Willamette Valley (or every one lost),
  a score from survivors, goods and how early you arrived, and a short
  epitaph line for each who died on the trail, written by the player
  (default offered). Play again.
- [x] 8. Hunting as text: spend a day and shot, success by skill of
  the dice and region (buffalo on the plains, antelope, deer, elk in the
  mountains), carry limit of 100 lb back to the wagon.
- [x] 9. Balance: run 200 seeded headless playthroughs with a simple
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
- M2 done: naming the party (random 1840s defaults), departure month March to July, the Independence store with purse ($800) and a 2,000 lb wagon. Flour and bacon became one item, Provisions, to keep the store short.
- M3 and M4 done together (the trail needs the route): pace, rations, health, oxen, weather by month and region, travel a week at a time stopping at landmarks, rest, trade offers; 25 landmarks with both forks (Sublette Cutoff or Fort Bridger; Barlow Road or the Columbia), fort trade rooms at 1.4 to 2.2 times Independence prices. Mileages are rounded figures; checked against Wikipedia's Route of the Oregon Trail where it gives them (Fort Bridger to Fort Hall about 210, ours 230; Three Island Crossing to Fort Boise about 130, ours 125; Sublette Cutoff saves about 70, ours 70); totals 1,820 to 1,950 miles, matching "not quite 2,000" (historicoregoncity.org). Rivers are crossed without risk until M6. A bot that never restocks starves around mile 890 in mid-June: hunting and fort shopping are meant to fix that (M8, M9).
- M5 done: 20 events weighted by region, month, pace and time on the road (illness with a daily drain and a course, breakdowns that use spares or cost days, strays, hail, poor grass, bad water, lost trail, abandoned wagons, good camps, help from Lakota, Shoshone and Nez Perce families, fire, theft, mud). About one travelling day in ten. Cholera is rare on purpose: the trail's cholera years began in 1849. Deaths carry a cause; the status shows each person's health and illness.
- M6 done: seven rivers, depth from the river's usual depth times snowmelt by month (highest May and June) and a daily drift; drive across, float the sealed box (a day), ferry where one ran in 1848 (Kansas River, North Platte, Green River), or camp and wait. Losses of goods and oxen, and drowning at deep fords. Quicksand on the South Platte. Rafting the Columbia costs three days and runs the Cascades rapids. Cholera weighting checked: NPS says cholera reached St. Louis in early 1849.
- M7 done: graves marked with the date and place, an epitaph screen for each of the dead (words offered, the player may rewrite them), the end page with a scored reckoning on arrival (survivors and their health, oxen, goods, purse, days before October) or the graves if all were lost; Enter plays again. Also `src/bot.rs`: a sensible and a careless rule player, `--bot N` and `--bot-show SEED`. Before any balancing: sensible arrives 100 of 100 with 4.8 alive, careless 0 of 100. Too easy for the sensible: M9.
- M8 done: Go hunting on the trail menu (item 3): a day and 10 to 30 rounds, game by region and season (buffalo on the High Plains May to August, antelope, deer, elk, sage hens, rabbits, turkey), 100 lb carried back at most, and game grows wary with each day hunted on the same stretch.
- M9 done. The target is read as the whole party arriving, since a sensible party with someone left alive is nearly certain. Numbers, 400 seeded journeys each, two seed ranges:
  - Sensible (April, 8 oxen, 1,200 lb, clothing and spares, steady pace, rations by what is left, hunts when short, rests the weak, ferries, restocks at forts): all five arrive 65 to 68 percent, someone arrives 98 to 100 percent, 4.6 of 5 on arrival, about 183 days (around October 1).
  - Careless (May, 4 oxen, 900 lb, no spares, three sets of clothing, grueling pace, always fords, tops up food only to 700 lb, hunts only when out): someone arrives 8 percent, never all five.
  - Main killers for the sensible: typhoid, drowning, hunger, cholera.
  - Changes: illness severity varies per case (mild to grave); good food no longer heals the sick; events 11 to 15 percent of travelling days; a wagon accident event; grueling pace costs 2 a day, not 3; with fewer than 2 oxen the party leaves the wagon and walks on at 7 miles a day with what it can carry (before, the game could get stuck there).
  - A test pins it: 110 to 150 of 200 sensible parties arrive whole, at most 20 of 200 careless ones arrive.
