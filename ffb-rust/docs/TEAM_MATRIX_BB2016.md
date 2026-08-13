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
| `lineman` | 🟢 100/100 | — | GREEN 2026-08-13, shared bb2016 fixes (synthetic FUMBBL-legacy roster) |
| `amazon` | 🟢 100/100 | — | driven to full parity 2026-08-12 (23 engine fixes, ITER20-37; see parity_roster_progression memory) |
| `chaos` | 🟢 100/100 | — | GREEN 2026-08-13, no roster-specific fix — cleared by the campaign's shared bb2016 fixes (esp. Claw armour cap bb2016=7; AG-scale; chain-push occupant) |
| `chaos_dwarf` | 🟢 100/100 | — | GREEN 2026-08-13, cleared by shared bb2016 fixes |
| `chaos_pact` | 🟢 100/100 | — | GREEN 2026-08-13, shared bb2016 fixes (FUMBBL-legacy roster) |
| `dark_elf` | 🟢 100/100 | — | GREEN 2026-08-13, cleared by shared bb2016 fixes (AG-scale) |
| `dark_elf_league_fumbbl` | 🟢 100/100 | — | GREEN 2026-08-13, shared bb2016 fixes (FUMBBL-legacy) |
| `dwarf` | 🟢 100/100 | — | GREEN 2026-08-13 (ITER59/61/63): Stand Firm follow-up suppression (Deathroller), bb2016 argues for a CASUALTY secret weapon, and bb2016 runs KO-recovery BEFORE the secret-weapon argue (bb2025 is the reverse) |
| `elf` | 🟢 100/100 | — | GREEN 2026-08-13 (ITER66): bb2016 Side Step auto-DECLINED where the parity harness auto-USES it, so an Elf Blitzer was pushed to a standard square instead of choosing its own |
| `goblin` | 🔴 0/100 | seed 1, step 0, java 384ccaed1d572749 vs rust 25663191654d6a64 |  |
| `halfling` | 🔴 0/100 | seed 1, step 0, java 384ccaed1d572749 vs rust 9849f4d5573e6c4c |  |
| `high_elf` | 🟢 100/100 | — | GREEN 2026-08-13, cleared by shared bb2016 fixes (AG-scale) |
| `human` | 🟢 100/100 | — | driven to full parity 2026-08-12 (Bone Head per-activation; bb2016 dodge & pickup old AG scale) |
| `khemri` | 🟢 100/100 | — | GREEN 2026-08-13, cleared by shared bb2016 fixes |
| `khemri_fumbbl` | 🟢 100/100 | — | GREEN 2026-08-13, shared bb2016 fixes (FUMBBL-legacy) |
| `lizardman` | 🟢 100/100 | — | GREEN with no roster-specific fix — cleared by the shared amazon/human bb2016 fixes (AG-scale dodge/catch/pickup + Bone Head per-activation) |
| `necromantic` | 🟢 100/100 | — | GREEN 2026-08-13 (ITER59-60): bb2016 Stand Firm must publish FOLLOWUP_CHOICE=false AND clear the pending pushback stack — without them the attacker followed up onto the un-pushed defender, stacking two players and silently disabling the Werewolf's Frenzy second block |
| `nippon` | 🟢 100/100 | — | GREEN 2026-08-13, shared bb2016 fixes (FUMBBL-legacy) |
| `norse` | 🟢 100/100 | — | driven to full parity 2026-08-12 (Snow Troll Wild Animal per-activation + prone-cancel via old_player_state; Claw armour cap bb2016=7 not 8; bb2016 chain-push moves the occupant not the block defender) |
| `nurgle` | 🟢 100/100 | — | GREEN 2026-08-13, cleared by shared bb2016 fixes (Claw/Decay/negatrait) |
| `ogre` | 🟢 100/100 | — | GREEN 2026-08-13 (ITER69): a bb2016 Throw Team-Mate spends the team's PASS, so a 2nd TTM in one turn is illegal — fixed in the Rust folded dispatch + BOTH agents' stale-action filters (ParityRunner change needed a jar rebuild) |
| `orc` | 🟢 100/100 | — | driven to full parity 2026-08-12 (Really Stupid per-activation + bb2016 RightStuff routing) |
| `renegades` | 🟢 100/100 | — | GREEN 2026-08-13 (FUMBBL-legacy roster). bb2016 TTM sub-project: step-set routing (ITER55) + declined-re-roll keeps re_rolled_action (ITER56) + StepRightStuff failed landing must dropPlayer → ball bounce + turnover (ITER57) |
| `skaven` | 🟢 100/100 | — | GREEN 2026-08-13, cleared by shared bb2016 fixes (AG-scale) |
| `slann` | 🟢 100/100 | — | GREEN 2026-08-13, shared bb2016 fixes |
| `slann_fumbbl` | 🟢 100/100 | — | GREEN 2026-08-13, shared bb2016 fixes (FUMBBL-legacy) |
| `undead` | 🟢 100/100 | — | GREEN 2026-08-13 (ITER64/65): a prone Blitz must set going-for-it (stand-up eats a Mummy's MA 3), and bb2016/bb2020 were wired to an EMPTY GoForIt modifier collection so the Blizzard +1 never applied |
| `underworld` | 🟢 100/100 | — | GREEN 2026-08-13. bb2016 TTM step-set routing + declined-re-roll (ITER55-56) then ITER58: route StepId::InitPassing to the bb2016 impl (bb2016 PassMechanic range table) so an out-of-range throw is refused as in stock Java |
| `vampire` | 🔴 0/100 | seed 1, step 0, java e752f3f1ca886d11 vs rust 475b6b600c40ec85 |  |
| `wood_elf` | 🟢 100/100 | — | GREEN 2026-08-13 (ITER71/73/77): bb2025-only startedStanding gate on Take Root, rooted players' pre-drawn move square reused, and bb2016's declined stand-up re-roll must not consume the action (edition-gated — bb2020/bb2025 DO consume it) |

**27 green / 3 red of 30** (2026-08-12→13: amazon, human, lizardman, orc, norse driven to full parity via engine fixes; chaos, chaos_dwarf, dark_elf, high_elf, khemri, nurgle, skaven then verified 100/100 with NO roster-specific fix — cleared by the shared campaign fixes. The remaining rows' "step 0 / seed 1" divergences are STALE pre-campaign snapshots and must be re-scouted before assuming red).

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

- ITER78 (goblin, still 100 fails): harness `PETTY_CASH` dialog routing (Java had been producing ZERO steps for goblin/halfling — treasury >= 50k) + `injury.rs` now takes the casualty roll AND its interpretation from the edition's `RollMechanic` (bb2016 = d6+d8 on roll[0], not d16+d6). Frontier: the bb2016 apothecary.
- ITER79 (goblin, still 100 fails): `UtilServerInjury.dropPlayer`'s `placedProneCausesInjuryRoll` branch ported (`drop_player_rng`) — a dropped Ball & Chain player rolls a full chain injury instead of going prone; 2 missing d6 per Fanatic. seed 1 i=1 -> i=2. Frontier: THROW_BOMB has no ParityRunner handler.
- ITER80 (goblin **100 -> 99**, FIRST drop): actions ParityRunner does not handle (THROW_BOMB, Stab, KickTeamMate, HypnoticGaze, Swoop, Punt, BreatheFire, ProjectileVomit, SecureTheBall) are DESELECTED, not carried out -- mirrored in random_agent; and ParityRunner no longer logs a step for the activation it abandons. 16 green rosters re-verified 0.
- ITER81 (**halfling 100/100 GREEN — 28/30**): bb2016 StepRightStuff applied the TTM landing injury BEFORE `dropPlayer`, so a KO'd/casualty'd thrown player was already off the pitch and the ball never bounced (one missing d8). Java rolls the injury, publishes it, THEN drops. goblin 99, vampire 100 remain.
- ITER82 (goblin **99 -> 5**): bb2016 `StepEndTurn` argues the call ONCE per team (the one player ParityRunner names); only bb2025 tracks `playerIdsArgued` and re-fires the dialog per weapon. Rust looped in both -> 4 argue d6 vs Java's 2 in a goblin mirror.
