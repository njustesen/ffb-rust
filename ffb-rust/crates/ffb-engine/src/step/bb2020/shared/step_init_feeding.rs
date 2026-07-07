use ffb_model::enums::{ApothecaryMode, PlayerType, TurnMode};
use ffb_model::enums::{PlayerState, PS_KNOCKED_OUT, PS_RESERVE};
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_kickoff_sequence_activations_count::ReportKickoffSequenceActivationsCount;
use ffb_model::report::mixed::report_kickoff_sequence_activations_exhausted::ReportKickoffSequenceActivationsExhausted;
use ffb_model::report::mixed::report_player_event::ReportPlayerEvent;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::injury::injuryType::injury_type_bitten::InjuryTypeBitten;
use crate::step::framework::{CatchScatterThrowInMode, Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{drop_player, handle_injury};

/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.shared.StepInitFeeding` (BB2020).
///
/// Handles feeding for Blood Lust vampires. BB2020 adds:
/// - BlitzTurnState activation tracking for kickoff sequences.
/// - `canBiteOpponents` (Vargheist): may also bite weak opponents (ST ≤ 3, non-Star).
/// - Bite-spectator sets CONFUSED state (not RESERVE like bb2016).
///
/// Init: mandatory GOTO_LABEL_ON_END, FEEDING_ALLOWED.
///       optional END_PLAYER_ACTION, END_TURN.
pub struct StepInitFeeding {
    pub goto_label_on_end: Option<String>,
    pub feed_on_player_choice: Option<bool>,
    pub feeding_allowed: Option<bool>,
    pub end_player_action: bool,
    pub end_turn: bool,
}

impl StepInitFeeding {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: None,
            feed_on_player_choice: None,
            feeding_allowed: None,
            end_player_action: false,
            end_turn: false,
        }
    }
}

impl Default for StepInitFeeding {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitFeeding {
    fn id(&self) -> StepId { StepId::InitFeeding }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.FEED)
        if let Action::SelectPlayer { player_id } = action {
            let provided = !player_id.is_empty();
            self.feed_on_player_choice = Some(provided);
            if provided {
                // Java: check victim is on same team OR player has canBiteOpponents
                let acting_id = game.acting_player.player_id.clone().unwrap_or_default();
                let acting_is_home = game.team_home.has_player(&acting_id);
                let victim_is_home = game.team_home.has_player(player_id.as_str());
                let victim_on_same_team = acting_is_home == victim_is_home;
                let can_bite = victim_on_same_team || acting_player_has_unused_property(game, &acting_id, NamedProperties::CAN_BITE_OPPONENTS);
                if can_bite {
                    if !victim_on_same_team {
                        // Java: actingPlayer.markSkillUsed(NamedProperties.canBiteOpponents)
                        mark_property_used(game, &acting_id, NamedProperties::CAN_BITE_OPPONENTS);
                    }
                    game.defender_id = Some(player_id.clone());
                } else {
                    self.feed_on_player_choice = None;
                }
            }
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = Some(v.clone()); true }
            StepParameter::FeedOnPlayerChoice(v) => { self.feed_on_player_choice = Some(*v); true }
            StepParameter::FeedingAllowed(v) => { self.feeding_allowed = Some(*v); true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            _ => false,
        }
    }
}

impl StepInitFeeding {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // client-only: UtilServerDialog.hideDialog

        let goto_label = match &self.goto_label_on_end {
            Some(l) => l.clone(),
            None => return StepOutcome::next(),
        };

        // Java (BB2020 only): BlitzTurnState activation tracking
        if self.end_player_action
            && game.turn_mode == TurnMode::Blitz
            && self.feed_on_player_choice.is_none()
        {
            let activation_report: Option<(i32, i32, i32, bool, bool)> =
                if let Some(ref mut bts) = game.blitz_turn_state {
                    if game.acting_player.has_acted || bts.acting_player_was_changed() {
                        bts.add_activation();
                        let lr = bts.limit_reached();
                        let npl = !bts.available_players_left();
                        if lr || npl {
                            self.end_turn = true;
                        }
                        Some((bts.get_amount(), bts.get_available(), bts.get_limit(), lr, npl))
                    } else {
                        None
                    }
                } else {
                    None
                };
            if let Some((amount, available, limit, lr, npl)) = activation_report {
                game.report_list.add(ReportKickoffSequenceActivationsCount::new(amount, available, limit));
                if lr {
                    game.report_list.add(ReportKickoffSequenceActivationsExhausted::new(true));
                } else if npl {
                    game.report_list.add(ReportKickoffSequenceActivationsExhausted::new(false));
                }
            }
        }

        // Java: if (actingPlayer == null || !isSufferingBloodLust || hasFed) → goto label
        let player_exists = game.acting_player.player_id.is_some();
        let is_blood_lusting = game.acting_player.suffering_blood_lust;
        let has_fed = game.acting_player.has_fed;

        if !player_exists || !is_blood_lusting || has_fed {
            return StepOutcome::goto(&goto_label)
                .publish(StepParameter::EndPlayerAction(self.end_player_action))
                .publish(StepParameter::EndTurn(self.end_turn));
        }

        // Java: if (isSufferingBloodLust && !hasFed && !feedingAllowed) → fFeedOnPlayerChoice = false
        if self.feeding_allowed == Some(false) {
            self.feed_on_player_choice = Some(false);
        }

        let acting_id = game.acting_player.player_id.clone().unwrap_or_default();
        let player_state: Option<PlayerState> = game.field_model.player_state(&acting_id);
        let has_tackle_zones = player_state.map(|s| s.has_tacklezones()).unwrap_or(false);

        if has_tackle_zones && self.feed_on_player_choice.is_none() {
            let acting_coord = game.field_model.player_coordinate(&acting_id)
                .unwrap_or(FieldCoordinate::new(0, 0));
            let acting_team = if game.home_playing { game.team_home.clone() } else { game.team_away.clone() };

            // Java: findAdjacentPlayersToFeedOn (same-team thralls)
            let mut all_victims: Vec<String> = UtilPlayer::find_adjacent_players_to_feed_on(game, &acting_team, acting_coord)
                .into_iter()
                .map(|id| id.clone())
                .collect();

            // Java: if (hasUnusedSkillWithProperty(canBiteOpponents)) addAll(findOpponentsToFeedOn)
            if acting_player_has_unused_property(game, &acting_id, NamedProperties::CAN_BITE_OPPONENTS) {
                let opponents = find_opponents_to_feed_on(game, &acting_team, acting_coord);
                all_victims.extend(opponents);
            }

            if !all_victims.is_empty() {
                // client-only: DialogPlayerChoiceParameter(FEED) — headless skips
                return StepOutcome::cont();
            } else {
                self.feed_on_player_choice = Some(false);
            }
        }

        let mut outcome = StepOutcome::next();

        if self.feed_on_player_choice == Some(true) && game.defender_id.is_some() {
            // Java: handleInjury(InjuryTypeBitten, ...)
            let defender_id = game.defender_id.clone().unwrap_or_default();
            let defender_coord = game.field_model.player_coordinate(&defender_id)
                .unwrap_or(FieldCoordinate::new(0, 0));
            let acting_player_obj = game.player(&acting_id).map(|p| p.id.clone());

            let mut injury_type = InjuryTypeBitten::new();
            let injury_result = handle_injury(
                game, rng, &mut injury_type,
                acting_player_obj.as_deref(), &defender_id,
                defender_coord, None, None, ApothecaryMode::Feeding,
            );

            // Java: fEndTurn = (defender is on same team) && UtilPlayer.hasBall(game, defender)
            let acting_is_home = game.team_home.has_player(&acting_id);
            let defender_is_home = game.team_home.has_player(&defender_id);
            if acting_is_home == defender_is_home {
                self.end_turn |= UtilPlayer::has_ball(game, &defender_id);
            }

            outcome = outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)));
            let drop_params = drop_player(game, &defender_id, false);
            for p in drop_params { outcome = outcome.publish(p); }

            // client-only: SoundId.SLURP
            game.acting_player.suffering_blood_lust = false;
        } else {
            // Bite spectator
            self.end_turn = true;

            // Java: if (!isCasualty && base != KNOCKED_OUT && base != RESERVE)
            let is_eligible = player_state.map(|s| {
                !s.is_casualty() && s.base() != PS_KNOCKED_OUT && s.base() != PS_RESERVE
            }).unwrap_or(false);

            if is_eligible {
                let player_coord = game.field_model.player_coordinate(&acting_id);
                let ball_coord = game.field_model.ball_coordinate;

                if player_coord.is_some() && player_coord == ball_coord {
                    game.field_model.ball_moving = true;
                    outcome = outcome.publish(StepParameter::CatchScatterThrowInMode(
                        CatchScatterThrowInMode::ScatterBall,
                    ));
                }

                // BB2020: changeConfused(true) — not RESERVE like bb2016
                if let Some(s) = player_state {
                    game.field_model.set_player_state(&acting_id, s.change_confused(true));
                }
                game.report_list.add(ReportPlayerEvent::new(
                    Some(acting_id.clone()),
                    Some("failed to bite anyone causing a turnover".to_string()),
                ));
                // client-only: SoundId.ROAR
                outcome = outcome.with_event(GameEvent::BiteSpectator { player_id: acting_id.clone() });
            }
        }

        game.acting_player.has_fed = true;
        game.acting_player.has_acted = true;
        outcome
            .publish(StepParameter::EndPlayerAction(self.end_player_action))
            .publish(StepParameter::EndTurn(self.end_turn))
    }
}

/// Java: `StepInitFeeding.findOpponentsToFeedOn` — adjacent opponents with ST ≤ 3 and not STAR.
fn find_opponents_to_feed_on(game: &Game, acting_team: &ffb_model::model::team::Team, coord: FieldCoordinate) -> Vec<String> {
    let mut result = Vec::new();
    for adj in game.field_model.adjacent_on_pitch(coord) {
        if let Some(id) = game.field_model.player_at(adj) {
            if acting_team.has_player(id) { continue; }
            if let Some(player) = game.player(id) {
                if player.player_type != PlayerType::Star && player.strength_with_modifiers() <= 3 {
                    result.push(id.to_string());
                }
            }
        }
    }
    result
}

fn acting_player_has_unused_property(game: &Game, acting_id: &str, property: &str) -> bool {
    game.player(acting_id)
        .map(|p| UtilCards::has_unused_skill_with_property(p, property))
        .unwrap_or(false)
}

fn mark_property_used(game: &mut Game, acting_id: &str, property: &str) {
    use ffb_model::util::util_cards::UtilCards;
    if let Some(player) = game.player(acting_id) {
        if let Some(skill_id) = UtilCards::get_unused_skill_with_property(player, property) {
            game.mark_skill_used(acting_id, skill_id);
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PlayerGender, PS_STANDING, Rules};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, id: &str, coord: FieldCoordinate, is_home: bool) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        };
        if is_home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_init_feeding() {
        assert_eq!(StepInitFeeding::new().id(), StepId::InitFeeding);
    }

    #[test]
    fn set_parameter_feeding_allowed() {
        let mut step = StepInitFeeding::new();
        assert!(step.set_parameter(&StepParameter::FeedingAllowed(true)));
        assert_eq!(step.feeding_allowed, Some(true));
    }

    #[test]
    fn set_parameter_end_turn() {
        let mut step = StepInitFeeding::new();
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_feed_on_player_choice() {
        let mut step = StepInitFeeding::new();
        assert!(step.set_parameter(&StepParameter::FeedOnPlayerChoice(false)));
        assert_eq!(step.feed_on_player_choice, Some(false));
    }

    #[test]
    fn no_acting_player_goes_to_label() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        game.acting_player.player_id = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("lbl"));
    }

    #[test]
    fn not_blood_lusting_goes_to_label() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn already_fed_goes_to_label() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_fed = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn feeding_not_allowed_bites_spectator_and_ends_turn() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.suffering_blood_lust = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn bite_spectator_sets_confused_not_reserve() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "vamp", coord, true);
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.suffering_blood_lust = true;
        let _ = step.start(&mut game, &mut GameRng::new(0));
        let state = game.field_model.player_state("vamp");
        // BB2020: confused, not reserve
        assert!(state.map(|s| !s.has_tacklezones()).unwrap_or(false));
        assert_ne!(state.map(|s| s.base()), Some(PS_RESERVE));
    }

    #[test]
    fn bite_spectator_at_ball_coord_sets_ball_moving() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "vamp", coord, true);
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.suffering_blood_lust = true;
        game.field_model.ball_coordinate = Some(coord);
        game.field_model.ball_moving = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.field_model.ball_moving);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CatchScatterThrowInMode(_))));
    }

    #[test]
    fn feed_on_player_clears_blood_lust_and_publishes_injury() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(true);
        step.feed_on_player_choice = Some(true);
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        let vcoord = FieldCoordinate::new(6, 5);
        add_player(&mut game, "vamp", coord, true);
        add_player(&mut game, "thrall", vcoord, true);
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.suffering_blood_lust = true;
        game.defender_id = Some("thrall".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(!game.acting_player.suffering_blood_lust);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn blitz_turn_state_activation_exhausted_sets_end_turn() {
        use ffb_model::model::blitz_turn_state::BlitzTurnState;
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(true);
        step.end_player_action = true;
        let mut game = make_game();
        // BlitzTurnState with limit=1, available=0 → limit reached after 1 activation
        game.blitz_turn_state = Some(BlitzTurnState::new(1, 1));
        game.turn_mode = TurnMode::Blitz;
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_acted = true;
        // No victims → feed_on_player_choice = Some(false) → bites spectator → end_turn
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn bite_spectator_adds_player_event_report() {
        use ffb_model::report::report_id::ReportId;
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, "vamp", coord, true);
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.suffering_blood_lust = true;
        let _ = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::PLAYER_EVENT));
    }

    #[test]
    fn blitz_turn_activation_adds_activations_count_report() {
        use ffb_model::model::blitz_turn_state::BlitzTurnState;
        use ffb_model::report::report_id::ReportId;
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("lbl".into());
        step.feeding_allowed = Some(true);
        step.end_player_action = true;
        let mut game = make_game();
        game.blitz_turn_state = Some(BlitzTurnState::new(2, 2));
        game.turn_mode = TurnMode::Blitz;
        game.acting_player.player_id = Some("vamp".into());
        game.acting_player.suffering_blood_lust = true;
        game.acting_player.has_acted = true;
        let _ = step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.has_report(ReportId::KICKOFF_SEQUENCE_ACTIVATIONS_COUNT));
    }
}
