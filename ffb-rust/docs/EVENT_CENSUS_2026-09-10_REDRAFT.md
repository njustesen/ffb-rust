# Event census of the re-draft sweep — 2026-09-10

**What this is.** The coverage half of the re-draft. `docs/SWEEP_2026-09-10_REDRAFT.txt` says whether
the two engines agreed; this says **which rules were on the pitch while they agreed**.

330 gates (110 cells × 3 scales), 33,000 games, **27,183,290 events**. Reproduced with
`scripts/sweep_census/` against the `parity_rd_*` log roots (`CENSUS_PREFIX=parity_rd_`).

Unlike the 2026-09-10 baseline, the comparison below is **measured, not inferred**: the previous
sweep's 333 `parity_sw_*` roots were still on disk, so both censuses were run with the same code and
diffed. Where this document gives a "before" number it came out of that run, not out of the earlier
write-up.

## Headline, against the baseline sweep

| | baseline (333 gates) | re-draft (330 gates) |
|---|---:|---:|
| games | 33,300 | 33,000 |
| events | 27,858,969 | 27,183,290 |
| `GameEvent` variants emitted | 77 | **75** |
| `PlayerAction` variants declared | 27 | **14** |
| **block re-rolls** | **0** | **90,062** |
| `blockRoll` | 660,522 | 641,347 |
| `lonerRoll` | **0** | **6,030** |
| KO / casualty / dead | 59,018 / 49,530 / 6,239 | **65,968 / 54,761 / 6,862** |
| touchdowns / hand-offs / pass rolls | 25,856 / 32,913 / 47,697 | 25,363 / 33,217 / 41,514 |
| fouls / argue / ejections | 56,214 / 11,560 / 10,934 | 54,346 / 11,658 / 11,003 |
| kick-offs | 89,345 | 88,401 |
| weather | all 5, all 3 editions | all 5, all 3 editions |
| positional coverage | every drafted number acted, every cell | every drafted number acted, every cell |

## The two things that got better

**Block re-rolls went from zero to 90,062 (14.0% of 641,347 blocks).** The baseline's first and
worst finding was that a block die was never re-rolled in 660,522 blocks, in every edition and every
roster, while dodge/rush/pickup/catch/pass re-rolls all happened. §H.14 phase 1 fixed the offer;
this sweep is the measurement at scale. Re-roll rates now read: dodge 15.1%, **block 14.0%**, pickup
9.8%, catch 8.4%, rush 5.1%, pass 6.6%.

**`lonerRoll` went from a dead event to 6,030 rolls** across 68 cells. It had no producer at all —
the roll was made and never evented, so the checklist could not see it. Loner is now the
fourth-most-exercised skill in the tree.

Injuries are up about 11% (KO +11.8%, casualties +10.6%, deaths +10.0%) on **fewer** games and
**fewer** blocks. A re-rolled block is another chance at a knock-down, so more violence per block is
what the block re-roll fix should produce.

## The cost of dropping the star players, measured exactly

`PlayerAction` variants declared fell from 27 to 14. **All thirteen lost actions are star specials,
and nothing else was lost** — the set difference is exactly:

| action | declarations in the baseline |
|---|---:|
| `ThrowKeg` | 1,088 |
| `RaidingParty` | 894 |
| `FuriousOutburst` | 654 |
| `AutoGazeZoat` | 540 |
| `BalefulHex` | 535 |
| `WisdomOfTheWhiteDwarf` | 511 |
| `ThenIStartedBlastin` | 503 |
| `MultipleBlock` | 429 |
| `BlackInk` | 425 |
| `AllYouCanEat` | 228 |
| `CatchOfTheDay` | 147 |
| `Treacherous` | 80 |
| `LookIntoMyEyes` | 13 |

Three `GameEvent` variants went with them: `kegThrow` (761), `thenIStartedBlastin` (513),
`allYouCanEatRoll` (40). The net event-variant count is 77 → 75 because `lonerRoll` is new.

**Hail Mary Pass survives, reduced**: 54 declarations before, **16** now. It stays in the tree
because the BB2025 Elven Union Thrower has it printed, so it never needed a star; the lost two
thirds are the bb2016 dwarf star's. It is now gone from bb2016 entirely, which has no native carrier
in any roster.

Getting the rest back means hiring stars the way the rules do it: through inducements, paying gold
**and** Skill Points against a team's Matched Play Tier allowance. That is a mechanic neither engine
implements today, not a data change — see `PARITY_COVERAGE_REQUIREMENTS.md` §19.

## Skills

107 of the 200 skills in `SkillId` are fielded by the 110 squads (the baseline had 117 — the
difference is the stars' skills leaving with them). Of those 107:

- **15 provably exercised.** Dodge 21,047 · Horns 12,792 · Bombardier 8,124 · **Loner 6,030** ·
  Regeneration 5,118 · Wrestle 3,066 · Animosity 2,712 · Juggernaut 409 · Leap 407 · Pogo 407 ·
  Dauntless 331 · Tackle 183 · Swoop 112 · **Punt 105** · Chainsaw 48.
- **2 fielded with a dedicated event that never fired**: `Pro` (1 cell, `proRoll` = 0) and
  `Swarming` (1 cell, `swarmingPlayersRoll` = 0).
- **90 invisible to the event stream** — the engine emits nothing that names them, so the census
  cannot prove them either way. Mighty Blow (76 cells), Thick Skull (68), Block (61), Pass (50),
  Frenzy (42), Stunty (42) and so on. This is an instrumentation gap, not a coverage gap.

`Punt` at 105 declarations is worth noting: the campaign previously recorded its trigger as
unreachable.

**A caveat on the skills half.** It is not comparable to the baseline write-up's "45 provably
exercised / 11 dead / 61 no naming event". `skills.py` reads a `fielded.json` that **nothing in the
committed pipeline wrote** — it died on `FileNotFoundError`, so the earlier split cannot be
reproduced from the scripts. `inventory.py` now derives `fielded.json`, which is what makes the
numbers above reproducible; the earlier ones are taken on trust and not differenced here.

## Still not covered

Checklist items that are zero in at least one gate:

| item | gates with zero |
|---|---:|
| interceptions | **330 of 330** |
| crowd surfs | 321 |
| pass deviates | 239 |
| GFI rolls | **87** |
| block 3 dice | 42 |
| action HandOver | 23 |
| argue success | 20 |
| touchdowns | 6 |
| action Pass | 6 |
| pass rolls | 4 |
| throw-ins | 1 |

Two of those are structural rather than statistical:

**An interception never succeeds anywhere** — zero in all 330 gates, across 41,514 pass rolls. They
are *attempted*: §H.17 is a red caused by the interception dialog being answered twice. So the
mechanic is reached and never lands, which is the shape of a bug rather than of thin sampling.

**GFI rolls are zero in exactly 87 gates**, and 87 is exactly the bb2016 column (29 cells × 3). The
baseline's finding #2 — `GameEvent::PlayerMoved` and `GoForItRoll` are emitted only from
`step/bb2025/move_/*` while `driver.rs` routes bb2016 to `step/bb2016/move_/*`, which emits
neither — is **unchanged**. A third of the matrix still has no movement or rush telemetry, and
bb2016 rushing is asserted by nothing but the state hash.

## 30 gates short on the checklist, and why the sweep log says 29

The census finds 30 of 330 gates with a required item at zero; the sweep log lists 29. The extra one
is `goblin bb2016 @1.0`, and the difference is structural: a gate that FAILS parity prints
`PARITY: n/m passed, k FAILED` and never mentions coverage, so the log cannot report the coverage
status of a red gate. The census evaluates all 330 regardless. On the 29 the log *can* report, the
two agree item for item — which is what licenses the numbers above.

The dominant missing item is `action HandOver` (23 gates), concentrated at @1e6 and @0. That is
agent behaviour, not engine disagreement: the heuristic simply never hands off in 100 games with
those rosters at those sampling scales.
