# 🏁 Milestone — skill coverage: every ruleset skill fires, on a green certification

**2026-11-22 · `78f317ab5` · tag `skill-coverage-2026-11-22`**

The user's goal for this tier (2026-11-17): *"make the agent use all skills in the game. Ideally to
play better but most importantly to reach better coverage."*

It is met in the form `scripts/skill_parity_check.py` defines: every skill that belongs to a
ruleset fires in games where the Rust engine and the Java engine agree, and the whole certification
ran on **one** frozen Rust build plus **one** frozen Java jar. The checker prints `DONE`.

| | |
|---|---|
| Ruleset skills firing (bb2016 + bb2020 + bb2025) | **411 / 411**, missing 0 |
| Mirror suite: 330 gates (100 games each) | **330 / 330** green |
| Cross suite: 4,012 gates (3 games each) | **4,012 / 4,012** green |
| Star suite: 4,012 gates (2 games each) | **4,012 / 4,012** green |
| Skill suite: 121 squads (20 games each; a few 1-game seed hunts) | **121 / 121** green |
| Games compared, all suites | roughly 55,000 |
| Red gates | **0** |

"Green" means steps **and** reports identical: the per-step state hashes match, and so does the
full report stream (which carries the roll-modifier names, the only place a passive skill shows up).

## The frozen pair

Recorded in [`cert_manifest.json`](cert_manifest.json), re-hashed after the run, unchanged:

- Rust: `D:/ffb_star/bin/ffb-parity-cert9.exe`, sha256 `f6a78d36f7fe0708…` (release build of `93bf58936`)
- Java: `D:/ffb_star/bin/ffb-ai-cert9.jar`, sha256 `71371e62af0b7039…`
- Skill ledger: `D:/ffb_ledger/cert9` (same exe)

Re-check at any time from `ffb-rust/`:

```
python -I scripts/cert_tally.py cert9        # per-suite verdict tally
python -I scripts/skill_parity_check.py      # the goal check; prints DONE
```

## What it does and does not claim

**Does:** every skill a ruleset can put on the pitch (roster positions, Infamous Staff, star
players, team special rules, advancements) is used by the agents in certified games, and both
engines resolve it identically.

**Counted as green without firing, on purpose:**
- bb2025 **TheBallista** and **Unsteady**: the Java engine never resolves them. They are fielded in
  the certified games and both engines agree; the user's ruling is "Java is the truth".
- bb2025 **Insignificant** is a draft rule, not a skill use: every bb2025 team obeys the draft
  limit (22 Insignificant players fielded).

**Not counted:** skills that no ruleset can field (no roster, star, staff or special rule carries
them, and they cannot be learned): bb2016 Disposable, FanFavourite, KickTeamMate, MonstrousMouth,
Swarming, Swoop, Timmmber, WeepingDagger; bb2020 PilingOn. The user ruled out an exempt list; these
are outside the ruleset, not exempted from it.

**Not done:** phase 4 of the original plan (a skill-aware value model so the agent plays *better*,
A/B-tested at ≥12,800 games). This milestone is coverage, not strength. Several coverage rules play
worse on purpose and sit behind `--coverage-bias` (0 = off), so the default agent is unchanged.

The standing limits from earlier milestones still apply: green is scoped to the drafted squads and
the agents' behaviour, and the state hash is not the whole game state.

## How it got there (2026-11-17 → 22, about 80 commits)

1. **Phase 0, ledger.** `scripts/skill_ledger.py` lists every skill × edition as not fielded,
   fielded but never fired, thin, or firing.
2. **Phase 1, unblock.** Answered the dialogs both harnesses had pinned to "decline" and offered the
   actions the agents never took (Chomp, Incorporeal, Flashing Blade, Vicious Vines, Kick 'em,
   Frenzied Rush, Shot to Nothing, Zoat gaze, Working in Tandem, …). Each one was gated, and every
   divergence was fixed in Rust 1:1 against Java.
3. **Phase 3, policy.** A shared skill-use policy table (`data/agent/skill_policy.json`) that both
   engines read.
4. **Phase 2, squads.** Rule-legal skill-advancement squads (`data/teams_skills`, `<race>_sk`) and
   the skill sweep.
5. **Phase 2b, designed coverage.** Engine ports and agent rules for what was still dark: Bounding
   Leap, Gored by the Bull, Two for One, stat increases at load, Team Captain, Josef Bugman's re-roll,
   Yoink!, Sneakiest of the Lot, Mesmerizing Dance, A Sneaky Pair, Violent Innovator, bb2020 team
   special rules, plus star hires (`data/teams_skills/star_hires.json`) and seed hunts.
6. **Phase 5, re-certification.** cert5 → cert9. Every red was fixed 1:1 in Rust and then
   re-frozen:
   - cert5: 34 reds, fixed in `8a08ef736` and `e0f63037e`.
   - cert6 and cert7: interrupted by the low-memory reaper and by mid-run fixes.
   - cert8: star 4,010/4,012. The crowd-landing injury was applied too early (`TtmToCrowdHandler`,
     fixed in `93bf58936`).
   - cert9: everything green.

The last stretch ran unattended through `scripts/skill_parity_loop.py`, with the work log in
[`BACKLOG.md`](BACKLOG.md) §M.

## Parked follow-ups

- Secure the Ball client-rule rework (`docs/secure_the_ball_client_rule_2026-11-21.patch`).
- `step/bb2020/ttm/step_init_scatter_player.rs:152`: the same early `apply_to` in a file that is
  never dispatched. It is dead code, but it should be fixed before anything wires it up.
- The known usage-type mismatches listed in BACKLOG §M phase 5. None has been observed in a game.
- Next tier candidate: §N, FUMBBL replay parity ([`REPLAY_PARITY_PLAN.md`](REPLAY_PARITY_PLAN.md)).
  2,178 replays have been downloaded and are kept outside git.
