# Team-Parity Matrix — BB2016 (hand-drafted teams)

Run 2026-08-08 — mirror matchups, tier 3, seeds 1-100,
teams from `data/teams/bb2016/` (see docs/TEAM_DRAFTS_BB2016.md), Java XMLs from scripts/gen_java_parity_data.py.
Reds are RECORDED, not fixed (scope of the 2026-08-08 team-creation task).

**UPDATE 2026-08-12 — bb2016 full-parity campaign (ongoing):** the "step 0 / seed 1"
divergences above are STALE (pre-campaign). `amazon` is now driven to **100/100** via
23 engine fixes (ITER20-37; recurring class = bb2016 flows routed through shared bb2025
steps/`make_injury_type`). Remaining rosters are being driven green next, in turn.

| Roster | Result | First divergence | Notes |
|---|---|---|---|
| `lineman` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 17bdbe0e289c3afd | FUMBBL-legacy roster |
| `amazon` | 🟢 100/100 | — | driven to full parity 2026-08-12 (23 engine fixes, ITER20-37; see parity_roster_progression memory) |
| `chaos` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust a646028152594c81 |  |
| `chaos_dwarf` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 5f87359ecbe8d622 |  |
| `chaos_pact` | 🔴 0/100 | seed 1, step 0, java c5d85980b8bb2c73 vs rust aa0748f996be2af7 |  |
| `dark_elf` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 6a50ebe1aae46be5 |  |
| `dark_elf_league_fumbbl` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust a00da3c56dc4990a | FUMBBL-legacy roster |
| `dwarf` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 4b038dcdfc33ba6e |  |
| `elf` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust a589c17c4bbe3bb2 |  |
| `goblin` | 🔴 0/100 | seed 1, step 0, java 384ccaed1d572749 vs rust 25663191654d6a64 |  |
| `halfling` | 🔴 0/100 | seed 1, step 0, java 384ccaed1d572749 vs rust 9849f4d5573e6c4c |  |
| `high_elf` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust a589c17c4bbe3bb2 |  |
| `human` | 🟢 100/100 | — | driven to full parity 2026-08-12 (Bone Head per-activation; bb2016 dodge & pickup old AG scale) |
| `khemri` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust d30a52f6c35d68d7 |  |
| `khemri_fumbbl` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust d30a52f6c35d68d7 | FUMBBL-legacy roster |
| `lizardman` | 🟢 100/100 | — | GREEN with no roster-specific fix — cleared by the shared amazon/human bb2016 fixes (AG-scale dodge/catch/pickup + Bone Head per-activation) |
| `necromantic` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 10bfe73abb533f6e |  |
| `nippon` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 96e1d188b6c11c4b | FUMBBL-legacy roster |
| `norse` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 71716983c04ccfdb |  |
| `nurgle` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 5a30154de20c0ab7 |  |
| `ogre` | 🔴 0/100 | seed 1, step 0, java f17d2e4bc0c146e5 vs rust d6ef3f4659806499 |  |
| `orc` | 🟢 100/100 | — | driven to full parity 2026-08-12 (Really Stupid per-activation + bb2016 RightStuff routing) |
| `renegades` | 🔴 0/100 | seed 1, step 0, java 36c2338c5a81ae43 vs rust 7de5fb2fda70c4fa | FUMBBL-legacy roster |
| `skaven` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust ed317bbeaaaabc9f |  |
| `slann` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 8b5bd002b4a05bb2 |  |
| `slann_fumbbl` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 019f905b194ab2dc | FUMBBL-legacy roster |
| `undead` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust 52ec2896b34b514f |  |
| `underworld` | 🔴 0/100 | seed 1, step 0, java 36c2338c5a81ae43 vs rust 74b156927e7fc691 |  |
| `vampire` | 🔴 0/100 | seed 1, step 0, java e752f3f1ca886d11 vs rust 475b6b600c40ec85 |  |
| `wood_elf` | 🔴 0/100 | seed 1, step 0, java 1afc6f15af26536c vs rust fecd448eb2f10080 |  |

**0 green / 30 red of 30.**

## Analysis (recorded, not fixed — task scope)

This is the FIRST-EVER bb2016 tier-3 parity run. All 30 matchups diverge at the very
first step with a common signature (verified on `human` seed 1):

- `game_start` state hashes MATCH — both engines load identical teams from the shared
  hand-drafted specs (`data/teams/bb2016/` ↔ `team_<race>_parity16_*.xml`).
- The first activation already differs: Java gives the first drive to **home**
  (`Activate(teamHumanParity16Home3,BLITZ)`), Rust to **away** (`Activate(away_03,Blitz)`),
  with different post-pregame state hashes.

So the shared dice stream splits during the PREGAME (weather / coin toss / fan-factor
rolls / kickoff): Java's BB2016 start sequence consumes the RNG differently from Rust's
bb2016 driver path. One root-cause likely gates the entire column — fixing the bb2016
pregame sequence should un-gate all 30 matchups to their first real in-drive frontier.
