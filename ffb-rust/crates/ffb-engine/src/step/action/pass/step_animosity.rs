/// 1:1 translation of com.fumbbl.ffb.server.step.action.pass.StepAnimosity (COMMON) plus its
/// three per-edition hooks: com.fumbbl.ffb.server.skillbehaviour.{bb2016,bb2020,bb2025}
/// .AnimosityBehaviour.handleExecuteStepHook.
///
/// Mandatory init param: GOTO_LABEL_ON_FAILURE.
/// Expected preceding param: CATCHER_ID.
///
/// Base `StepAnimosity.executeStep()` (shared by all editions): bomb-turn → NEXT_STEP; otherwise
/// dispatch to the rules-specific hook.
///
/// BB2016's hook is structurally different from BB2020/2025's: it starts with an
/// `isSufferingAnimosity()` check (the actor already failed an Animosity roll earlier this same
/// player-action) which, for HAND_OVER, searches for another same-race adjacent blockable target
/// before deciding NEXT_STEP vs GOTO_LABEL (+END_PLAYER_ACTION), and for PASS checks whether the
/// stored catcher's race still differs from the thrower's. Only when NOT already suffering does it
/// compute `doRoll` — via a direct `hasSkill(Animosity) && race mismatch && same team` check (bb2016
/// has no `SkillMechanic.animosityExists`; that mechanic hardcodes `false` for bb2016, matching the
/// Java behaviour of bb2016 Animosity never routing through it). After a roll, if still suffering,
/// bb2016 does one more team-wide search for a still-possible same-race animosity pass/hand-over
/// target before finally going to the failure label (resetting the in-flight pass coordinate/range
/// ruler only if such a target exists, since END_PASSING will push a fresh pass sequence).
///
/// BB2020/2025's hooks are identical apart from `SkillMechanic.animosityExists` (edition-specific
/// keyword/race parsing) and have no upfront "already suffering" branch and no tail team search —
/// they just recompute `doRoll` for the current catcher on every entry and GOTO_LABEL directly when
/// still suffering after the roll.
use ffb_model::enums::{PlayerAction, ReRollSource, Rules, SkillId};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::Player;
use ffb_model::report::report_animosity_roll::ReportAnimosityRoll;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use ffb_mechanics::bb2016::pass_mechanic::PassMechanic as Bb2016PassMechanic;
use ffb_mechanics::bb2020::skill_mechanic::SkillMechanic as Bb2020SkillMechanic;
use ffb_mechanics::bb2025::skill_mechanic::SkillMechanic as Bb2025SkillMechanic;
use ffb_mechanics::mechanics::{is_skill_roll_successful, minimum_roll_animosity};
use ffb_mechanics::pass_mechanic::PassMechanic as PassMechanicTrait;
use ffb_mechanics::skill_mechanic::SkillMechanic as SkillMechanicTrait;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

pub struct StepAnimosity {
    /// Java: state.gotoLabelOnFailure — mandatory.
    pub goto_label_on_failure: String,
    /// Java: state.catcherId — set by preceding step parameter.
    pub catcher_id: Option<String>,
    /// Java: AbstractStepWithReRoll.reRolledAction
    pub re_rolled_action: Option<String>,
    /// Java: AbstractStepWithReRoll.reRollSource
    pub re_roll_source: Option<String>,
}

impl StepAnimosity {
    pub fn new(goto_label_on_failure: impl Into<String>) -> Self {
        Self {
            goto_label_on_failure: goto_label_on_failure.into(),
            catcher_id: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }
}

impl Step for StepAnimosity {
    fn id(&self) -> StepId { StepId::Animosity }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if let Action::UseReRoll { use_reroll: false } = action {
            self.re_roll_source = None;
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::CatcherId(v) => { self.catcher_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepAnimosity {
    /// Java: `StepAnimosity.executeStep()` (base class, shared by all editions) — bomb-turn
    /// short-circuit, then dispatch to the rules-specific `AnimosityBehaviour` hook.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if game.turn_mode.is_bomb_turn() {
            return StepOutcome::next();
        }
        match game.rules {
            Rules::Bb2016 => self.execute_bb2016(game, rng),
            _ => self.execute_stat_edition(game, rng),
        }
    }

    /// Java: `AnimosityBehaviour(bb2016).handleExecuteStepHook`.
    fn execute_bb2016(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let thrower = game.thrower().cloned();
        let thrower_coordinate = thrower.as_ref().and_then(|t| game.field_model.player_coordinate(&t.id));
        let catcher = self.catcher_id.as_deref().and_then(|id| game.player(id).cloned());

        if game.acting_player.suffering_animosity {
            if game.acting_player.player_action == Some(PlayerAction::HandOver) {
                // Java: findAdjacentBlockablePlayers(otherTeam, throwerCoordinate), any same race.
                let target_available = match (&thrower, thrower_coordinate) {
                    (Some(t), Some(coord)) => {
                        let other_team = UtilPlayer::find_other_team(game, &t.id);
                        let targets = UtilPlayer::find_adjacent_blockable_players(game, other_team, coord);
                        targets.iter().any(|target_id| {
                            game.player(target_id)
                                .and_then(|tp| tp.race.as_deref())
                                .zip(t.race.as_deref())
                                .map(|(a, b)| a.eq_ignore_ascii_case(b))
                                .unwrap_or(false)
                        })
                    }
                    _ => false,
                };
                if target_available {
                    StepOutcome::next()
                } else {
                    StepOutcome::goto(&self.goto_label_on_failure).publish(StepParameter::EndPlayerAction(true))
                }
            } else if let (Some(t), Some(c)) = (&thrower, &catcher) {
                let races_differ = match (t.race.as_deref(), c.race.as_deref()) {
                    (Some(tr), Some(cr)) => !tr.eq_ignore_ascii_case(cr),
                    _ => false,
                };
                if races_differ {
                    game.pass_coordinate = None;
                    game.field_model.range_ruler = None;
                    StepOutcome::goto(&self.goto_label_on_failure)
                } else {
                    StepOutcome::next()
                }
            } else {
                StepOutcome::next()
            }
        } else {
            let re_rolled = self.re_rolled_action.as_deref() == Some("ANIMOSITY");
            let mut do_roll = false;
            if re_rolled {
                if let Some(ref source_str) = self.re_roll_source.clone() {
                    let source = ReRollSource::new(source_str.as_str());
                    let thrower_id = thrower.as_ref().map(|t| t.id.clone()).unwrap_or_default();
                    if use_reroll(game, &source, &thrower_id) {
                        do_roll = true;
                    } else {
                        game.acting_player.suffering_animosity = true;
                    }
                } else {
                    game.acting_player.suffering_animosity = true;
                }
            } else if let (Some(t), Some(c)) = (&thrower, &catcher) {
                // Java: bb2016 never uses `SkillMechanic.animosityExists` (that mechanic hardcodes
                // `false` for bb2016) — it checks `hasSkill(Animosity) && race mismatch && same team`
                // directly.
                if let (Some(tr), Some(cr)) = (t.race.as_deref(), c.race.as_deref()) {
                    let same_team = game.team_home.has_player(&t.id) == game.team_home.has_player(&c.id);
                    do_roll = t.has_skill(SkillId::Animosity) && !tr.eq_ignore_ascii_case(cr) && same_team;
                }
            }

            if do_roll {
                let roll = rng.d6();
                let min_roll = minimum_roll_animosity();
                let successful = is_skill_roll_successful(roll, min_roll);
                let thrower_id = thrower.as_ref().map(|t| t.id.clone()).unwrap_or_default();
                if !thrower_id.is_empty() {
                    game.mark_skill_used(&thrower_id, SkillId::Animosity);
                }
                let event = GameEvent::AnimosityRoll { player_id: thrower_id.clone(), roll, success: successful };

                // Java: `reRolled = (ANIMOSITY == getReRolledAction()) && (getReRollSource() != null)`
                // — evaluated with the fields as they stood at hook entry, i.e. BEFORE this call's own
                // ask_for_reroll_if_available offer mutates them for the next invocation.
                let reported_rerolled = re_rolled && self.re_roll_source.is_some();
                // Java: `step.getResult().addReport(new ReportAnimosityRoll(actingPlayer.getPlayerId(),
                //         successful, roll, minimumRoll, reRolled, null))`
                game.report_list.add(ReportAnimosityRoll::new(
                    Some(thrower_id),
                    successful,
                    roll,
                    min_roll,
                    reported_rerolled,
                    Vec::new(),
                ));

                if successful {
                    game.acting_player.suffering_animosity = false;
                } else if !re_rolled {
                    if let Some(prompt) = ask_for_reroll_if_available(game, "ANIMOSITY", min_roll, false) {
                        self.re_rolled_action = Some("ANIMOSITY".into());
                        self.re_roll_source = Some("TRR".into());
                        return StepOutcome::cont().with_event(event).with_prompt(prompt);
                    }
                    game.acting_player.suffering_animosity = true;
                } else {
                    game.acting_player.suffering_animosity = true;
                }

                if game.acting_player.suffering_animosity {
                    self.fail_bb2016_tail(game, thrower_coordinate).with_event(event)
                } else {
                    StepOutcome::next().with_event(event)
                }
            } else if game.acting_player.suffering_animosity {
                self.fail_bb2016_tail(game, thrower_coordinate)
            } else {
                StepOutcome::next()
            }
        }
    }

    /// Java: bb2016 hook's shared tail — `if (actingPlayer.isSufferingAnimosity()) { ... }` team
    /// search for a still-possible same-race animosity pass/hand-over target before the final
    /// `GOTO_LABEL`. Only reachable from the roll branch (`state.doRoll` true or false), never from
    /// the upfront "already suffering" branch (which has its own distinct target-search logic).
    fn fail_bb2016_tail(&mut self, game: &mut Game, thrower_coordinate: Option<FieldCoordinate>) -> StepOutcome {
        let acting_id = game.acting_player.player_id.clone();
        let is_home = acting_id.as_deref().map(|id| game.team_home.has_player(id)).unwrap_or(false);
        let acting_race = acting_id.as_deref().and_then(|id| game.player(id)).and_then(|p| p.race.clone());
        let player_action = game.acting_player.player_action;
        let team_players: Vec<Player> =
            if is_home { game.team_home.players.clone() } else { game.team_away.players.clone() };

        let mechanic = Bb2016PassMechanic;
        let mut animosity_pass_possible = false;
        for player in &team_players {
            let player_state = game.field_model.player_state(&player.id);
            let has_tz = player_state.map(|s| s.has_tacklezones()).unwrap_or(false);
            // Java: `StringTool.isEqual(actingPlayer.getRace(), player.getRace())` — null-safe exact
            // equality (not case-insensitive, unlike the other race comparisons in this file).
            if has_tz && acting_race == player.race {
                let player_coordinate = game.field_model.player_coordinate(&player.id);
                let possible = match player_action {
                    Some(PlayerAction::HandOver) => match (player_coordinate, thrower_coordinate) {
                        (Some(pc), Some(tc)) => pc.is_adjacent(tc),
                        _ => false,
                    },
                    Some(PlayerAction::Pass) => {
                        mechanic.find_passing_distance(game, thrower_coordinate, player_coordinate, false).is_some()
                    }
                    _ => false,
                };
                if possible {
                    animosity_pass_possible = true;
                    break;
                }
            }
        }

        if animosity_pass_possible {
            game.pass_coordinate = None;
            game.field_model.range_ruler = None;
        }
        StepOutcome::goto(&self.goto_label_on_failure)
    }

    /// Java: `AnimosityBehaviour(bb2020/bb2025).handleExecuteStepHook` (identical bodies apart from
    /// `SkillMechanic.animosityExists`).
    fn execute_stat_edition(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let re_rolled = self.re_rolled_action.as_deref() == Some("ANIMOSITY");

        let mut do_roll = false;
        if re_rolled {
            if let Some(ref source_str) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_str.as_str());
                let thrower_id = game.thrower().map(|t| t.id.clone()).unwrap_or_default();
                if use_reroll(game, &source, &thrower_id) {
                    do_roll = true;
                } else {
                    game.acting_player.suffering_animosity = true;
                }
            } else {
                game.acting_player.suffering_animosity = true;
            }
        } else {
            let thrower = game.thrower().cloned();
            let catcher = self.catcher_id.as_deref().and_then(|id| game.player(id).cloned());
            let rules = game.rules;
            do_roll = match (&thrower, &catcher) {
                (Some(t), Some(c)) => match rules {
                    Rules::Bb2020 => Bb2020SkillMechanic::new().animosity_exists(t, c),
                    _ => Bb2025SkillMechanic::new().animosity_exists(t, c),
                },
                _ => false,
            };
        }

        if do_roll {
            let roll = rng.d6();
            let min_roll = minimum_roll_animosity();
            let successful = is_skill_roll_successful(roll, min_roll);
            let thrower_id = game.thrower().map(|t| t.id.clone()).unwrap_or_default();
            if !thrower_id.is_empty() {
                game.mark_skill_used(&thrower_id, SkillId::Animosity);
            }
            let event = GameEvent::AnimosityRoll { player_id: thrower_id.clone(), roll, success: successful };

            // Java: `reRolled = (ANIMOSITY == getReRolledAction()) && (getReRollSource() != null)`,
            // evaluated before this call's own reroll-offer mutates those fields for the next call.
            let reported_rerolled = re_rolled && self.re_roll_source.is_some();
            // Java: `step.getResult().addReport(new ReportAnimosityRoll(actingPlayer.getPlayerId(),
            //         successful, roll, minimumRoll, reRolled, null))`
            game.report_list.add(ReportAnimosityRoll::new(
                Some(thrower_id),
                successful,
                roll,
                min_roll,
                reported_rerolled,
                Vec::new(),
            ));

            if successful {
                game.acting_player.suffering_animosity = false;
            } else if !re_rolled {
                if let Some(prompt) = ask_for_reroll_if_available(game, "ANIMOSITY", min_roll, false) {
                    self.re_rolled_action = Some("ANIMOSITY".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_event(event).with_prompt(prompt);
                }
                game.acting_player.suffering_animosity = true;
            } else {
                game.acting_player.suffering_animosity = true;
            }

            if game.acting_player.suffering_animosity {
                StepOutcome::goto(&self.goto_label_on_failure).with_event(event)
            } else {
                StepOutcome::next().with_event(event)
            }
        } else if game.acting_player.suffering_animosity {
            StepOutcome::goto(&self.goto_label_on_failure)
        } else {
            StepOutcome::next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{Rules, TurnMode};

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game
    }

    #[test]
    fn already_suffering_animosity_goes_to_failure_label_bb2020_2025() {
        // Java bb2020/2025's AnimosityBehaviour has NO upfront "already suffering" short-circuit
        // (unlike bb2016) — it always recomputes `doRoll` for the current catcher, and only at the
        // very end does `if (actingPlayer.isSufferingAnimosity()) GOTO_LABEL`. With no thrower/
        // catcher configured, doRoll is false, so the pre-set suffering flag (untouched) drives the
        // outcome straight to the failure label rather than short-circuiting to NEXT_STEP.
        let mut game = make_game();
        game.acting_player.suffering_animosity = true;
        let out = StepAnimosity::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("fail"));
    }

    #[test]
    fn bomb_turn_skips_animosity_check() {
        let mut game = make_game();
        game.turn_mode = TurnMode::BombHome;
        let out = StepAnimosity::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn hand_over_action_without_catcher_returns_next() {
        // No catcher/thrower configured → do_roll is false regardless of player_action.
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::HandOver);
        let out = StepAnimosity::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn hand_over_action_does_not_bypass_animosity_roll() {
        // Bug fix regression: Java's AnimosityBehaviour hooks never skip the animosity check just
        // because the acting player's action is HAND_OVER — that action only matters inside the
        // bb2016 "already suffering" branch's retargeting search. A HAND_OVER with a genuine
        // animosity-triggering catcher must still roll.
        let mut game = make_game();
        add_thrower_and_catcher(&mut game, "Goblin", vec!["Goblin"]);
        game.acting_player.player_action = Some(PlayerAction::HandOver);
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.events.is_empty(), "a roll should occur even for a HAND_OVER action");
    }

    #[test]
    fn no_animosity_skill_returns_next() {
        let mut game = make_game();
        // No thrower set, no catcher → animosity_exists = false
        let out = StepAnimosity::new("fail").start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn catcher_id_parameter_accepted() {
        let mut step = StepAnimosity::new("fail");
        step.set_parameter(&StepParameter::CatcherId(Some("c1".into())));
        assert_eq!(step.catcher_id.as_deref(), Some("c1"));
    }

    #[test]
    fn goto_label_on_failure_param_accepted() {
        let mut step = StepAnimosity::new("fail");
        step.set_parameter(&StepParameter::GotoLabelOnFailure("other".into()));
        assert_eq!(step.goto_label_on_failure, "other");
    }

    #[test]
    fn regular_turn_no_animosity_returns_next() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Regular;
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("c2".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        // animosity_exists returns false without matching race → always NEXT_STEP
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn step_id_is_animosity() {
        assert_eq!(StepAnimosity::new("fail").id(), StepId::Animosity);
    }

    #[test]
    fn decline_reroll_sets_suffering_animosity() {
        let mut step = StepAnimosity::new("fail");
        step.re_rolled_action = Some("ANIMOSITY".into());
        step.re_roll_source = None; // declined → source is None
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.acting_player.suffering_animosity);
    }

    fn add_thrower_and_catcher(game: &mut Game, animosity_value: &str, catcher_keywords: Vec<&str>) {
        use ffb_model::model::skill_def::SkillId;
        use ffb_model::model::skill_def::SkillWithValue;
        game.team_home.players.push(ffb_model::model::Player {
            id: "thrower".into(),
            starting_skills: vec![SkillWithValue::with_value(SkillId::Animosity, animosity_value)],
            ..Default::default()
        });
        game.team_home.players.push(ffb_model::model::Player {
            id: "catcher".into(),
            keywords: catcher_keywords.into_iter().map(String::from).collect(),
            ..Default::default()
        });
        game.thrower_id = Some("thrower".into());
        game.acting_player.player_id = Some("thrower".into());
    }

    #[test]
    fn different_race_catcher_skips_roll_entirely() {
        // Thrower is only configured against "Goblin" catchers; a Troll catcher never matches,
        // so animosity_exists is false and no roll (no AnimosityRoll event) ever happens.
        let mut game = make_game();
        add_thrower_and_catcher(&mut game, "Goblin", vec!["Troll"]);
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.events.is_empty(), "no roll should occur when animosity_exists is false");
        assert!(!game.acting_player.suffering_animosity);
    }

    #[test]
    fn same_race_catcher_triggers_roll() {
        // Thrower is configured against "Goblin" catchers; a Goblin catcher matches, so
        // animosity_exists is true and a real d6 roll happens (an AnimosityRoll event fires).
        let mut game = make_game();
        add_thrower_and_catcher(&mut game, "Goblin", vec!["Goblin"]);
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.events.is_empty(), "a roll should occur when animosity_exists is true");
    }

    #[test]
    fn bb2016_uses_direct_skill_check_not_bb2025_stat_mechanic() {
        // Bug fix regression: the file previously hardcoded bb2025's `SkillMechanic` for every
        // rules edition. bb2025/bb2020's mechanic requires `Animosity` configured with a matching
        // keyword/race value; bb2016 instead just needs `hasSkill(Animosity)` plus a plain race
        // mismatch and same-team membership — no configured value at all. A bb2016 game must still
        // trigger a roll even though the skill has no value configured (which would make bb2025's
        // mechanic report `animosity_exists == false`).
        use ffb_model::model::skill_def::{SkillId, SkillWithValue};
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2016);
        game.home_playing = true;
        game.team_home.players.push(ffb_model::model::Player {
            id: "thrower".into(),
            race: Some("Goblin".into()),
            starting_skills: vec![SkillWithValue::new(SkillId::Animosity)],
            ..Default::default()
        });
        game.team_home.players.push(ffb_model::model::Player {
            id: "catcher".into(),
            race: Some("Troll".into()),
            ..Default::default()
        });
        game.thrower_id = Some("thrower".into());
        game.acting_player.player_id = Some("thrower".into());
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(!out.events.is_empty(), "bb2016 should roll on hasSkill + race mismatch + same team");
    }

    #[test]
    fn animosity_roll_adds_report() {
        // Bug fix regression: Java's AnimosityBehaviour hook always calls
        // `step.getResult().addReport(new ReportAnimosityRoll(...))` whenever a roll actually
        // happens, both in the stat-edition and bb2016 hooks. The report was previously never
        // added at all (only the internal GameEvent was emitted).
        let mut game = make_game();
        add_thrower_and_catcher(&mut game, "Goblin", vec!["Goblin"]);
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(
            game.report_list.has_report(ffb_model::report::report_id::ReportId::ANIMOSITY_ROLL),
            "a ReportAnimosityRoll must be added whenever an animosity roll occurs"
        );
    }

    #[test]
    fn full_roll_cycle_with_real_trigger_can_fail_and_offer_reroll() {
        let mut game = make_game();
        add_thrower_and_catcher(&mut game, "Goblin", vec!["Goblin"]);
        let mut step = StepAnimosity::new("fail");
        step.catcher_id = Some("catcher".into());
        // Find a seed producing a failed roll (roll == 1) to exercise the re-roll-offer path.
        let mut seed = 0u64;
        loop {
            let mut rng = GameRng::new(seed);
            if rng.d6() == 1 { break; }
            seed += 1;
            assert!(seed < 100, "expected to find a failing roll seed quickly");
        }
        let out = step.start(&mut game, &mut GameRng::new(seed));
        // On failure with no re-roll source available, the step goes straight to the failure label.
        assert_eq!(out.action, StepAction::GotoLabel);
        assert!(game.acting_player.suffering_animosity);
    }
}
