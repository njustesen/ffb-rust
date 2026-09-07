# Parity campaign — undead (Shambling Undead, heuristic agent)

Mirror matchup `--home undead --away undead`, tier 3, seeds 1-100, `--heur-classes all`,
all three editions × scales `1.0` / `0` / `1e6`.

## 🏁 CLOSED 2026-09-07 — GREEN AT BASELINE, no engine change

`undead` was one of two races (with `vampire`) skipped by the alphabetical heuristic sweep: it has
rosters and drafted parity teams but had never had a heuristic ledger, and its only prior evidence
was the old **random**-agent matrix. The task brief carried one measured data point (bb2016 @1.0 read
100/100 in an ad-hoc check) and the expectation that the injury/Regeneration chain hardened for
`khemri` and `necromantic` would be the risk surface.

Baseline was taken on `cb4a3a9c5` (the commit that closed BACKLOG E12 / the Leap arm). **All nine
parity gates and all three random controls were 100/100 on the first measurement.** No Rust engine
file was edited for this race; there is therefore no fix, no new regression test, and no Java
harness change in this entry. This is the third race in the sweep (after `slann_fumbbl` and
`underworld`) to need zero engine work.

### Gate numbers actually run

| Edition | @1.0 | @0 | @1e6 | random control |
|---|---|---|---|---|
| bb2016 | 100/100 | 100/100 | 100/100 | 100/100 |
| bb2020 | 100/100 | 100/100 | 100/100 | 100/100 |
| bb2025 | 100/100 | 100/100 | 100/100 | 100/100 * |

\* the bb2025 random control prints the `…, but required coverage items are MISSING` trailer and
exits 1. That trailer is the tier-3 coverage checklist, not parity; its parity line is
`PARITY: 100/100 games match`, which is a PASS for the parity half. All three **heuristic** harvest
runs printed `Result: ALL REQUIRED ITEMS PRESENT`.

`TIMING … rust_total=`: @1.0 **29.3 / 30.9 / 29.8 s** (bb2016 / bb2020 / bb2025) against
`java_total=` 68.2 / 72.0 / 71.1 s; @0 **37.0 / 40.7 / 41.0 s**; @1e6 **58.2 / 58.4 / 56.2 s**.

`cargo test -p ffb-engine`: **7440 passed, 0 failed**, 15 ignored.

Closed-roster regressions, bb2025 @1.0, seeds 1-100, heuristic, `--heur-classes all` — all
**100/100** (confirmatory only; nothing in the engine changed):
nurgle, necromantic, nippon, lizardman, khemri, human, goblin, amazon, chaos, dwarf, norse, ogre,
orc, renegades, skaven, slann, underworld, wood_elf. (`lizardman` prints the coverage-checklist
trailer; its parity line is `100/100`.)

Coverage harvested ×3, each run alone: `docs/EVENT_COVERAGE_undead_bb2016.md`, `_bb2020.md`,
`_bb2025.md`.

## The surface, per edition, read off the roster JSON

`data/rosters/<edition>/roster_undead.json` — the skill sets differ per edition, and so does the
drafted squad (`data/teams/<edition>/team_undead.json`), which is what actually decides coverage:

| position | bb2016 | bb2020 | bb2025 |
|---|---|---|---|
| Skeleton | Regeneration | Regeneration, Thick Skull | Regeneration, Thick Skull |
| Zombie | Regeneration | Regeneration | **Eye Gouge**, Regeneration, **Unsteady** |
| Ghoul | Dodge | Dodge | Dodge, Regeneration |
| Wight | Block, Regeneration | Block, Regeneration | Block, Regeneration, **Tackle**, Thick Skull |
| Mummy | Mighty Blow, Regeneration | Mighty Blow(1), Regeneration | Mighty Blow, Regeneration |

Drafted squads:

- **bb2016** (14): 2 Mummy, 2 Wight, 4 Ghoul, 1 Zombie, 5 Skeleton. No Thick Skull in this edition.
- **bb2020** (13): 1 Mummy, 2 Wight, 4 Ghoul, 6 Zombie. **No Skeleton** so no Thick Skull carrier.
- **bb2025** (13): 2 Mummy, 2 Wight, 2 Ghoul, 6 Skeleton, **1** Zombie. `special_rules:
  ["Masters of Undeath"]`.

## Per-skill coverage verdicts

| skill | bucket | evidence |
|---|---|---|
| **Regeneration** (on 11-13 players every edition) | exercised **+ evented** | `regenerationRoll` **105 / 81 / 162** (bb2016/20/25), on top of `injury` 2321 / 2241 / 2282. This is the chain khemri and necromantic hardened, and it is the single most-exercised undead mechanism. |
| **Dodge** (Ghoul) | exercised **+ evented** | `skillUse skill_id=127` **50 / 72 / 27** — and in bb2025 with `used=false` 8 times, i.e. both branches of the offer. Always on `_05`/`_06`, the two Ghouls. |
| **Tackle** (bb2025 Wight) | exercised **+ evented** | 8 × `skillUse skill_id=18` on `home_03`/`away_03`, the Wights. bb2016/bb2020 correctly show none — neither edition's Wight carries it. |
| **Block** (Wight) | exercised, un-evented | 1611 / 1648 / 1558 `blockRoll` events; Block is a passive both-down suppressor with no event of its own. Parity is the proof. |
| **Mighty Blow** (Mummy) | exercised, un-evented | 2321 / 2241 / 2282 `injury` events with Mummies blocking throughout; the modifier changes the armour/injury total and emits nothing. |
| **Thick Skull** (bb2025 Skeleton ×6, bb2025 Wight) | exercised, un-evented | changes the KO branch of the injury roll only; bb2020 has **no carrier** in the drafted squad and bb2016 has none in the edition. |
| **Eye Gouge** (bb2025 Zombie) | **agent never creates it — UNTESTED** | see below |
| **Unsteady** (bb2025 Zombie) | **unreachable in this campaign** | see below |
| **Masters of Undeath** (bb2025 special rule) | no event, no step | a roster/inducement-eligibility tag; it gates nothing inside a tier-3 game. |

### Eye Gouge is unverified, and the reason is the squad, not the engine

`EyeGougeBehaviour` **is** registered (`skill_behaviour/registry.rs:132`, inside `build_bb2025`),
its step modifier is a faithful port of
`ffb-server/…/skillbehaviour/bb2025/EyeGougeBehaviour.handleExecuteStepHook`, and it carries a
colocated test that drives it directly
(`skill_behaviour/bb2025/eye_gouge_behaviour.rs:161/180`). Yet zero `skillUse` events with
`SkillUse::EYE_GOUGED` appear in 100 bb2025 games, despite **1116 `pushback` events**.

The measurement that explains it: the hook requires the *pusher* to be the Eye Gouge carrier, and
the bb2025 squad has exactly **one** Zombie (nr 13 of 13, so frequently a reserve). Its complete
action log across all 100 games is

```
1 Foul, 1 HandOverMove, 56 Move, 1 PassMove      (home_13 + away_13, 59 activations total)
```

— **zero Block and zero Blitz**. The one player who could gouge never declares a blocking action,
so the branch is unreachable for this matchup. Consistent on both sides, which is why parity here is
genuine rather than papered over, but it means **bb2025 Eye Gouge carries no parity evidence from
this campaign**. Same shape as the khemri `Brawler` carry-over and the pre-sweep TTM finding.

Player-id labels in the event stream are 1-based jersey numbers, verified against the squad:
`skillUse skill_id=127` (Dodge) lands only on `_05`/`_06`, the two Ghouls, and `skill_id=18`
(Tackle) only on `_03`, a Wight.

### Unsteady is a negative gate with nothing to suppress

`Unsteady` appears in exactly one live place, `legal_actions/mod.rs:601`, where it removes the
`SecureTheBall` activation from the legal set. No `SecureTheBall` action is declared anywhere in the
three harvests (the action histogram is Move / BlitzMove / Block / Foul / PassMove / HandOverMove /
Pass / HandOver only), so the suppression has nothing to suppress. Its colocated test
`secure_the_ball_omitted_for_unsteady_player` is the only coverage it has.

## A conclusion of mine that was WRONG — recorded on purpose

On reading `35 skillUse` in the bb2025 harvest against `1116 pushback`, my first conclusion was that
`EyeGougeStepModifier` was **ported but unreached** — the fault pattern the procedure warns about
first, and one that in this repo has usually meant a missing `registry.rs` entry or a dead
`step/bb20xx/*.rs` twin. That was wrong. `grep -n "EyeGouge" skill_behaviour/registry.rs` shows the
behaviour registered in `build_bb2025`, and the event histogram of the *carrier* — not of the
mechanism — is what actually settles it: the sole Zombie never blocks.

The generalisable form, and it is a new one for this sweep: **when a per-carrier skill shows no
events, count the CARRIER's declared actions before you go looking at the registry.** A one-player
skill on a 13-man squad whose carrier is the last man on the sheet can be perfectly wired and still
never fire. "Unreached" and "no carrier ever in the situation" produce identical event histograms.

## Honest gaps (none is a parity red; both engines agree)

- **bb2016 emits no `playerMoved` and no `goForItRoll`.** Pre-existing and edition-wide — the same
  absence is in `EVENT_COVERAGE_underworld_bb2016.md`, `_ogre_bb2016.md`, `_goblin_bb2016.md`.
  bb2016 counts are not comparable to bb2020/bb2025 ones.
- Eye Gouge and Unsteady, above.

## Frontier

Empty. Remaining in the two-race carve-out after this entry: **vampire**.
