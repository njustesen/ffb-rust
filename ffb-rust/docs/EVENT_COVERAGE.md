# Event coverage — what the parity suite actually exercises

Generated 2026-08-17. Regenerate with the commands in *Method* below.

## Method

Two populations, both driven by the **Rust engine only** — no Java, no parity comparison.
Coverage is a property of what the agents make the engine do, so the JVM adds nothing here
(and costs ~98% of the wall clock; see `docs/PARITY_*` for the parity gate itself).

| | population | agent | command | games |
|---|---|---|---|---:|
| **A** | what the parity suite *verifies* | `RandomAgent`, tier 3 — the same agent whose per-step state hashes match stock Java on every one of these matchups | `ffb-parity --coverage --all-rosters --all-editions --seeds 1-100` | 8,700 |
| **B** | what any agent can *reach* | `UniformAgent` — samples uniformly over every legal action, full pregame, inducements enabled | `ffb-parity --uniform --all-rosters --all-editions --seeds 1-25` | 2,175 |

A is the number that matters: every mechanic exercised in A is diffed against Java step-by-step,
so it is *proven*. B is the upper bound of what the current engine + a maximally curious agent can
touch at all. Anything in neither is invisible to us today.

The catalogue is all **128 `GameEvent` variants** parsed straight from
`ffb-model/src/events/game_event.rs`, so nothing can be missing by omission. Counts come from an
`event_type_counts` map keyed on the serde tag rather than from the hand-written `tally()` arms —
without it, a variant no arm covers reads as zero and is indistinguishable from one that never fires.

## Headline

| | count | share |
|---|---:|---:|
| Emitted in parity-verified games (**A**) | **57** | 45% |
| Only under the uniform agent (**B**) | 1 | 1% |
| Never emitted by either | **70** | 55% |

The 70 that never fire split four ways, and telling them apart is the whole point — a zero means
something completely different in each:

| class | n | meaning |
|---|---:|---|
| **No emitter** | 33 | nothing anywhere constructs the event |
| **Dead path** | 15 | an emitter exists, but its step is *never dispatched* |
| **Helper** | 7 | emitted from a helper, not a step — three from the superseded `step/engine.rs` |
| **Gated** | 15 | the step runs; a condition or an edition twin blocks the event |

Only the **gated** class is about agents. The other three are instrumentation: the mechanic runs and
nothing reports it, so **those zeros are not coverage facts**. An earlier version of this report
bucketed by "is `GameEvent::X` constructed anywhere in the engine", which conflated the classes —
a constructor sitting in an edition twin the driver never routes to reads exactly like a mechanic
that never happens.

### Closed in the 2026-08-17 pass

Twelve emission sites, no logic changes, parity re-gated at 30/30 on all three editions after each batch:

| was | now | cause |
|---|---|---|
| bb2016: 0 activations | 795,493 | `PlayerAction` emitted only by the bb2025 `step_init_selecting` |
| bb2016: 0 fouls, 0 ejections | 7,598 / 1,520 | emitted only by the shared `mixed/foul/*` steps |
| bb2016: 0 kickoff results | 5,797 (11 kinds) | emitted only by the bb2020/bb2025 steps |
| **0 deaths in 7,071 casualties** | **990** | `casualty_tier_*` sets PS_RIP, `si_sub_type_*` returns None for 15-16, so the kind was never filled in |
| 0 Foul Appearance rolls | 3,770 | no construction site anywhere in the engine |
| 0 stand-up rolls | 8,323 | same |
| 0 throw-team-mate rolls | 3,087 | same |
| 0 pass deviations | 444 | emitter only in the bb2020 twin the driver never routes to |
| 0 MVP awards | 17,392 | emitter only in the bb2020 twin; the step runs every game |
| 0 winnings rolls | 17,392 | emitter only in the bb2016 twin; same |

`ReRoll` was deliberately left out: emitting it means threading a return value through
`use_reroll`, and the parity agent declines team re-rolls by contract, so it would still read ~0.

### Surfaced by the new instrumentation, not yet explained

**BB2020 declares ThrowTeamMate 7,235 times and never resolves a single throw.**
`StepThrowTeamMate` is dispatched 0 times in a full ogre-vs-ogre BB2020 game, against 13 in the
same BB2025 matchup. BB2020 is 30/30 green against Java, so either Java declines these throws too
(faithful — but the mechanic is untested in that edition) or both engines share a gap the state
hash cannot see. Worth a direct check.

## 1. Never emitted, by class

### No emitter — nothing constructs the event (33)

| event | detail |
|---|---|
| `balefulHexRoll` | no construction site anywhere |
| `ballPickedUp` | no construction site anywhere |
| `ballScattered` | no construction site anywhere |
| `bombExplodesAfterCatch` | no construction site anywhere |
| `breatheFireRoll` | no construction site anywhere |
| `cardEffectRoll` | no construction site anywhere |
| `cardsAndInducementsBought` | no construction site anywhere |
| `coachBanned` | no construction site anywhere |
| `defectingPlayers` | no construction site anywhere |
| `doubleHiredStarPlayer` | no construction site anywhere |
| `gameOptions` | no construction site anywhere |
| `heatExhaustion` | no construction site anywhere |
| `hypnoticGazeRoll` | no construction site anywhere |
| `jumpRoll` | no construction site anywhere |
| `kickoffPitchInvasionStun` | no construction site anywhere |
| `kickoffThrowARock` | no construction site anywhere |
| `lonerRoll` | no construction site anywhere |
| `lookIntoMyEyesRoll` | no construction site anywhere |
| `passBlockEligible` | no construction site anywhere |
| `pilingOn` | no construction site anywhere |
| `playCard` | no construction site anywhere |
| `prayerRoll` | no construction site anywhere |
| `proRoll` | no construction site anywhere |
| `reRoll` | no construction site anywhere |
| `regenerationRoll` | no construction site anywhere |
| `rightStuffRoll` | no construction site anywhere |
| `riotousRookies` | no construction site anywhere |
| `secretWeaponBan` | no construction site anywhere |
| `specialEffectRoll` | no construction site anywhere |
| `teamCaptainRoll` | no construction site anywhere |
| `timeoutEnforced` | no construction site anywhere |
| `turnEnd` | no construction site anywhere |
| `weepingDaggerRoll` | no construction site anywhere |

### Dead path — emitter exists, step never dispatched (15)

| event | detail |
|---|---|
| `allYouCanEatRoll` | only `AllYouCanEat`, which is never dispatched |
| `bombOutOfBounds` | only `InitBomb`, which is never dispatched |
| `buyInducement` | only `BuyCardsAndInducements/BuyInducements`, which is never dispatched |
| `hitAndRun` | only `HitAndRun`, which is never dispatched |
| `kegThrow` | only `ThrowKeg`, which is never dispatched |
| `masterChefRoll` | only `MasterChef`, which is never dispatched |
| `pettyCash` | only `PettyCash`, which is never dispatched |
| `prayerAmount` | only `Prayers`, which is never dispatched |
| `safeThrowRoll` | only `SafeThrow`, which is never dispatched |
| `selectBlitzTarget` | only `SelectBlitzTarget`, which is never dispatched |
| `selectGazeTarget` | only `SelectGazeTarget`, which is never dispatched |
| `spellEffectRoll` | only `SpecialEffect`, which is never dispatched |
| `thenIStartedBlastin` | only `ThenIStartedBlastin`, which is never dispatched |
| `throwAtPlayer` | only `ThrowARock`, which is never dispatched |
| `wizardUse` | only `Wizard`, which is never dispatched |

### Helper — emitted outside a step (7)

| event | detail |
|---|---|
| `cardDeactivated` | emitted from a helper, not a step (crates/ffb-engine/src/util/util_server_cards.rs) |
| `coinThrow` | emitted from a helper, not a step (crates/ffb-engine/src/step/engine.rs) |
| `inducement` | emitted from a helper, not a step (crates/ffb-engine/src/mechanic/state_mechanic.rs) |
| `interceptionRoll` | emitted from a helper, not a step (crates/ffb-engine/src/step/engine.rs) |
| `leader` | emitted from a helper, not a step (crates/ffb-engine/src/mechanic/bb2025/state_mechanic.rs, crates/ffb-engine/src/mechanic/mixed/state_mechanic.rs) |
| `pumpUpTheCrowdReRoll` | emitted from a helper, not a step (crates/ffb-engine/src/mechanic/state_mechanic.rs, crates/ffb-engine/src/step/util_server_injury.rs) |
| `receiveChoice` | emitted from a helper, not a step (crates/ffb-engine/src/step/engine.rs) |

### Gated — step runs, event blocked by a condition or edition twin (15)

| event | detail |
|---|---|
| `apothecaryChoice` | `Apothecary` runs; the event is gated |
| `biasedRefRoll` | `Referee` runs; the event is gated |
| `bribesRoll` | `Bribes` runs; the event is gated |
| `chainsawRoll` | `BlockChainsaw/FoulChainsaw` runs; the event is gated |
| `fumblerooskie` | `ResetFumblerooskie` runs; the event is gated |
| `jumpUpRoll` | `JumpUp` runs; the event is gated |
| `kickTeamMateFumble` | `DispatchScatterPlayer` runs; the event is gated |
| `kickoffSequenceActivationsExhausted` | `ApplyKickoffResult` runs; the event is gated |
| `noPlayersToField` | `Setup` runs; the event is gated |
| `pickMeUpRoll` | `PickMeUp` runs; the event is gated |
| `playerNote` | `ForgoneStalling/StallingPlayer` runs; the event is gated |
| `projectileVomitRoll` | `ProjectileVomit` runs; the event is gated |
| `swarmingPlayersRoll` | `Swarming` runs; the event is gated |
| `swoopPlayer` | `InitScatterPlayer` runs; the event is gated |
| `throwAtStallingPlayer` | `StallingPlayer` runs; the event is gated |

## 3. Covered — emitted in parity-verified play

| event | A (8,700 games) | B (2,175 games) | bb2016 | bb2020 | bb2025 |
|---|---:|---:|---:|---:|---:|
| `playerAction` | 2,407,020 | 227,639 | 795,493 | 809,035 | 802,492 |
| `playerMoved` | 1,466,828 | 920,887 | 0 | 737,217 | 729,611 |
| `confusionRoll` | 180,380 | 16,825 | 69,252 | 51,506 | 59,622 |
| `injury` | 139,459 | 71,064 | 47,158 | 46,373 | 45,928 |
| `block` | 108,151 | 16,066 | 36,192 | 35,925 | 36,034 |
| `blockRoll` | 71,959 | 10,734 | 0 | 35,925 | 36,034 |
| `playerFellDown` | 55,106 | 41,601 | 0 | 28,000 | 27,106 |
| `dodgeRoll` | 51,553 | 31,045 | 0 | 25,412 | 26,141 |
| `pushback` | 44,807 | 6,648 | 0 | 22,506 | 22,301 |
| `scatterBall` | 27,792 | 6,673 | 9,411 | 9,195 | 9,186 |
| `bloodLustRoll` | 24,844 | 2,523 | 6,115 | 8,283 | 10,446 |
| `refereeSpotsFoul` | 22,438 | 4,228 | 7,598 | 7,410 | 7,430 |
| `foul` | 22,438 | 4,228 | 7,598 | 7,410 | 7,430 |
| `kickoffScatter` | 17,397 | 4,341 | 5,797 | 5,800 | 5,800 |
| `kickoffResultEvent` | 17,397 | 4,309 | 5,797 | 5,800 | 5,800 |
| `startHalf` | 17,397 | 4,240 | 5,797 | 5,800 | 5,800 |
| `winningsRoll` | 17,392 | 4,108 | 5,792 | 5,800 | 5,800 |
| `mvpRoll` | 17,392 | 4,108 | 5,792 | 5,800 | 5,800 |
| `animalSavagery` | 12,487 | 1,172 | 0 | 8,473 | 4,014 |
| `catchRoll` | 10,340 | 2,891 | 4,020 | 2,962 | 3,358 |
| `apothecaryRoll` | 9,372 | 0 | 2,043 | 4,100 | 3,229 |
| `standUpRoll` | 8,323 | 1,133 | 2,791 | 2,745 | 2,787 |
| `skillUse` | 7,665 | 1,078 | 2,388 | 2,726 | 2,551 |
| `passRoll` | 7,054 | 1,990 | 2,577 | 2,130 | 2,347 |
| `biteSpectator` | 5,391 | 429 | 1,040 | 2,052 | 2,299 |
| `argueTheCall` | 4,428 | 851 | 1,479 | 1,478 | 1,471 |
| `pickupRoll` | 4,362 | 1,682 | 0 | 2,134 | 2,228 |
| `playerEjected` | 4,146 | 790 | 1,520 | 1,352 | 1,274 |
| `goForItRoll` | 4,090 | 214,536 | 0 | 1,667 | 2,423 |
| `foulAppearanceRoll` | 3,770 | 640 | 1,127 | 1,230 | 1,413 |
| `handOver` | 3,332 | 1,051 | 1,339 | 947 | 1,046 |
| `throwTeamMateRoll` | 3,087 | 0 | 1,593 | 0 | 1,494 |
| `scatterPlayer` | 2,552 | 878 | 657 | 503 | 1,392 |
| `weatherChange` | 2,350 | 580 | 960 | 701 | 689 |
| `cheeringFans` | 1,930 | 496 | 0 | 937 | 993 |
| `kickoffExtraReRoll` | 1,847 | 494 | 0 | 951 | 896 |
| `kickoffExtraReRollBb2016` | 1,632 | 451 | 1,632 | 0 | 0 |
| `quickSnapRoll` | 1,198 | 387 | 0 | 609 | 589 |
| `solidDefenceRoll` | 971 | 163 | 0 | 477 | 494 |
| `kickoffTimeout` | 555 | 99 | 0 | 273 | 282 |
| `blitzRoll` | 551 | 208 | 0 | 551 | 0 |
| `passDeviate` | 444 | 97 | 0 | 444 | 0 |
| `kickoffPitchInvasion` | 424 | 154 | 0 | 214 | 210 |
| `dauntlessRoll` | 423 | 73 | 222 | 75 | 126 |
| `kickoffThrowARockBb2016` | 326 | 109 | 326 | 0 | 0 |
| `kickoffRiot` | 321 | 80 | 321 | 0 | 0 |
| `passBlock` | 299 | 29 | 0 | 141 | 158 |
| `kickoffOfficiousRef` | 269 | 38 | 0 | 269 | 0 |
| `dodgySnackRoll` | 269 | 40 | 0 | 0 | 269 |
| `kickoffDodgySnack` | 240 | 35 | 0 | 0 | 240 |
| `alwaysHungry` | 228 | 0 | 0 | 0 | 228 |
| `animosityRoll` | 129 | 79 | 0 | 56 | 73 |
| `throwIn` | 124 | 64 | 20 | 87 | 17 |
| `kickoffPitchInvasionBb2016` | 112 | 32 | 112 | 0 | 0 |
| `playerAdded` | 59 | 41 | 37 | 6 | 16 |
| `escapeRoll` | 36 | 0 | 0 | 0 | 36 |
| `trapDoor` | 4 | 1 | 0 | 4 | 0 |

## 4. Mechanic detail (population A - parity-verified, 8,700 games)

### Activations declared

| action | A | bb2016 | bb2020 | bb2025 |
|---|---:|---:|---:|---:|
| Move | 2,238,449 | 737,971 | 754,047 | 746,431 |
| Blitz | 66,009 | 22,103 | 21,946 | 21,960 |
| Block | 43,587 | 14,563 | 14,294 | 14,730 |
| Foul | 24,373 | 8,159 | 8,132 | 8,082 |
| ThrowTeamMate | 23,184 | 8,422 | 7,235 | 7,527 |
| Pass | 7,533 | 2,766 | 2,259 | 2,508 |
| HandOver | 3,885 | 1,509 | 1,122 | 1,254 |

> **bb2016 emits no `playerAction` events at all** - its activation column is empty across 2,900
> games even though the same games produce 1,479 argue-the-call rolls, which can only follow a
> foul. Activation coverage for bb2016 is currently unmeasurable, not absent.

`PlayerAction` has **57** variants. A declares 7; B adds HypnoticGaze, KickTeamMate, Punt, ThrowBomb.
Never declared by any agent (excluding the `*Move`/`*Select` sub-phases, which are internal legs
of an activation rather than declarable actions):

```
StandUp, RemoveConfusion, Gaze, MultipleBlock, HailMaryPass, DumpOff
StandUpBlitz, HailMaryBomb, Swoop, Treacherous, WisdomOfTheWhiteDwarf, ThrowKeg
RaidingParty, MaximumCarnage, LookIntoMyEyes, BalefulHex, AllYouCanEat, PutridRegurgitationBlitz
PutridRegurgitationBlock, KickEmBlock, KickEmBlitz, BlackInk, CatchOfTheDay, ThenIStartedBlastin
TheFlashingBlade, ViciousVines, FuriousOutburst, SecureTheBall, BreatheFire, Chainsaw
Stab, ProjectileVomit, AutoGazeZoat, Forgo, Incorporeal, Chomp
```

These are the weapon / special-action surface - Stab, Chainsaw, Gaze, Breathe Fire, Projectile
Vomit, Multiple Block, Hail Mary, star-player specials. Rosters carrying them are in the suite;
the agent simply never declares the action.

### Dice

| roll | total | success | failure | re-rolled |
|---|---:|---:|---:|---:|
| dodge rolls | 51,553 | 27,239 | 24,314 | 2,599 |
| go for it rolls | 4,090 | 3,375 | 715 | 0 |
| catch rolls | 10,340 | 5,173 | 5,167 | 156 |
| pickup rolls | 4,362 | 2,694 | 1,668 | 59 |
| dauntless rolls | 423 | 208 | 215 | 0 |
| foul appearance rolls | 3,770 | 3,145 | 625 | 0 |
| always hungry rolls | 228 | 192 | 36 | 0 |
| blood lust rolls | 24,844 | 19,443 | 5,401 | 0 |
| animosity rolls | 129 | 102 | 27 | 0 |
| confusion rolls | 180,380 | 105,546 | 74,834 | 0 |
| escape rolls | 36 | 31 | 5 | 0 |
| stand up rolls | 8,323 | 4,272 | 4,051 | 0 |
| argue the call rolls | 4,428 | 719 | 3,709 | 0 |
| animal savagery rolls | 12,487 | 6,829 | 5,658 | 0 |
| referee spots foul rolls | 22,438 | 4,521 | 17,917 | 0 |

**23 roll counters read zero.** Several are instrumentation gaps from section 1 rather than
unrolled dice (`foul_appearance_rolls`, `stand_up_rolls`, `regeneration_rolls`, `loner_rolls`,
`pro_rolls`, `right_stuff_rolls`):

```
interception_rolls, jump_rolls, jump_up_rolls, loner_rolls
pro_rolls, hypnotic_gaze_rolls, right_stuff_rolls, safe_throw_rolls
pick_me_up_rolls, breathe_fire_rolls, projectile_vomit_rolls, baleful_hex_rolls
look_into_my_eyes_rolls, weeping_dagger_rolls, bribes_rolls, biased_ref_rolls
then_i_started_blastin_rolls, keg_throws, throw_at_stalling_player_rolls, throw_at_player_rolls
fumblerooskie_uses, all_you_can_eat_rolls, regeneration_rolls
```

### Block dice

71,959 block rolls, 0 re-rolled.

| dice | count | result | count |
|---|---:|---|---:|
| 1 dice | 38,276 | Pushback | 24,099 |
| 2 dice | 17,226 | BothDown | 12,313 |
| -2 dice | 13,276 | Skull | 11,957 |
| 3 dice | 1,717 | PowPushback | 11,882 |
| -3 dice | 1,464 | Pow | 11,708 |

All five block results and every dice count from 3-against to 3-for are covered. Re-rolled
block dice are **never** exercised (the parity agent declines team re-rolls by contract).

### Passing

7,054 pass rolls, 314 re-rolled. All 4 real distances, 4 of 6 `PassOutcome` values:

| | complete | fumble | inaccurate | wildlyInaccurate |
|---|---:|---:|---:|---:|
| Long Bomb | 53 | 170 | 36 | 37 |
| Long Pass | 569 | 1,031 | 461 | 259 |
| Quick Pass | 999 | 401 | 456 | 19 |
| Short Pass | 958 | 789 | 687 | 129 |

Never seen: `Caught`, `MissedCatch`, and the `PassToPartner` distance.

### Injuries

| | A | bb2016 | bb2020 | bb2025 |
|---|---:|---:|---:|---:|
| total | 139,459 | 47,158 | 46,373 | 45,928 |
| armor_only | 100,622 | 34,470 | 33,521 | 32,631 |
| ko | 9,054 | 2,962 | 3,053 | 3,039 |
| cas | 7,071 | 2,370 | 2,211 | 2,490 |
| dead | 990 | 451 | 248 | 291 |

| lasting injury | count |
|---|---:|
| Dead | 990 |
| DislocatedHipAg | 295 |
| HeadInjuryAv | 295 |
| NeckInjuryAg | 248 |
| DislocatedShoulderSt | 243 |
| BrokenArmPa | 224 |
| SmashedKneeMa | 222 |

> **Lead worth chasing.** 7,071 casualties produced 2,517 lasting injuries, all six of them
> in the -1-stat band. `SeriouslyHurt`, `SeriousInjuryNi` (niggling) and `Dead` never appear, and
> **zero deaths in 7,071 casualties** is hard to square with a d16 table where 15-16 is Dead:
> `bb2020/roll_mechanic.rs:121` rolls a real `rng.die(16)`, so the band is reachable. Either the
> casualty interpretation never reaches its top and bottom bands, or those results are relabelled
> before the event is built. Worth a direct check against Java - **6 of 32 `SeriousInjuryKind`**
> values are all we have ever produced.

### Kickoff

| result | A | bb2016 | bb2020 | bb2025 |
|---|---:|---:|---:|---:|
| Cheering Fans | 2,874 | 944 | 937 | 993 |
| Weather Change | 2,350 | 960 | 701 | 689 |
| Brilliant Coaching | 2,295 | 688 | 822 | 785 |
| High Kick | 2,222 | 667 | 777 | 778 |
| Quick Snap | 1,946 | 748 | 609 | 589 |
| Solid Defence | 971 | 0 | 477 | 494 |
| Blitz | 908 | 357 | 551 | 0 |
| Charge | 569 | 0 | 0 | 569 |
| Time-out | 555 | 0 | 273 | 282 |
| Perfect Defence | 542 | 542 | 0 | 0 |
| Pitch Invasion | 536 | 112 | 214 | 210 |
| Get the Ref | 473 | 132 | 170 | 171 |
| Throw a Rock | 326 | 326 | 0 | 0 |
| Riot | 321 | 321 | 0 | 0 |
| Officious Ref | 269 | 0 | 269 | 0 |
| Dodgy Snack | 240 | 0 | 0 | 240 |

13 of the 16 `KickoffResult` variants fire. Never seen: **Riot**, **Perfect Defence**, **Throw a Rock**.
(`OficiousRef` is the enum's spelling of the observed *Officious Ref* - a typo in the variant name,
not a gap.)

### Skills

**5 of 200 skills** are ever recorded as used in 8,700 parity games:

| skill | used | declined |
|---|---:|---:|
| Horns | 4,018 | 0 |
| Dodge | 3,046 | 10 |
| Juggernaut | 292 | 0 |
| Wrestle | 101 | 0 |
| Tackle | 10 | 0 |

B adds only `DumpOff`. That is **2.5%** of the skill catalogue - the single largest
gap in this report, and partly an artefact of section 1: a skill that fires without emitting
`skillUse` is invisible here (Dodge and Horns appear precisely because they do emit).

## 5. Per-edition

| | bb2016 | bb2020 | bb2025 |
|---|---:|---:|---:|
| games | 2,900 | 2,900 | 2,900 |
| distinct event types | 33 | 48 | 49 |
| fouls | 7,598 | 7,410 | 7,430 |
| players ejected | 1,520 | 1,352 | 1,274 |
| weather changes | 960 | 701 | 689 |
| throw ins | 20 | 87 | 17 |
| pass blocks | 0 | 141 | 158 |
| hand overs | 1,339 | 947 | 1,046 |
| half starts | 5,797 | 5,800 | 5,800 |

Edition-exclusive events (a real ruleset difference, not a gap):

* **bb2016 only** - `kickoffRiot`, `kickoffExtraReRollBb2016`, `kickoffThrowARockBb2016`, `kickoffPitchInvasionBb2016`
* **bb2020 only** - `trapDoor`, `kickoffOfficiousRef`, `blitzRoll`
* **bb2025 only** - `alwaysHungry`, `escapeRoll`, `kickoffDodgySnack`, `dodgySnackRoll`

Only **19** event types fire in all three editions.

## 6. What is completely dark

Neither population touches any of this:

* **Scoring.** Zero touchdowns in 8,700 parity games. The uniform agent scores 103 in 2,175, so it
  is reachable - the parity agent just never does it. Everything downstream of a score (the
  post-TD kickoff, stalling, drive setup) is therefore unverified.
* **Pre-game and post-game.** No coin throw, no receive choice, no winnings, no MVP, no petty cash.
* **Inducements, cards, wizards, star players.** Zero in *both* populations - including the uniform
  run, which enables `INDUCEMENTS` and drives the full pregame specifically to reach them. Worth a
  look on its own: the mode built to cover this is not covering it.
* **Apothecary.** 0 uses, 0 rolls, against 9,054 KOs and 7,071 casualties.
* **Team re-rolls.** `rerolls_total` is 0 in both - partly section 1 (`reRoll` is never constructed),
  partly the parity agent's contract of declining them.

## 7. Suggested order of attack

1. **Close the instrumentation gaps (section 1).** Until the engine emits these, no agent work can
   be measured - and today's Foul Appearance bug lived in a mechanic whose counter reads 0. Cheap,
   mechanical, and it makes every later number trustworthy.
2. **Make the agent score.** One change unlocks the largest single block of unverified rules.
3. **Declare the weapon/special actions.** Stab, Chainsaw, Gaze, Bomb, Multiple Block - the rosters
   are already in the suite.
4. **Chase the casualty-band lead (section 4).** Possibly a real engine bug; cheap to check.
5. **Pre/post-game and inducements.** Largest surface, furthest from the current harness shape.
