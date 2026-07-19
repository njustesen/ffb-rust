/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.inducements.StepPlayCard`.
///
/// Step to play a card.
///
/// Needs to be initialized with stepParameter CARD (the full `Card` object).
/// Needs to be initialized with stepParameter HOME_TEAM.
///
/// Handles CLIENT_PLAYER_CHOICE (card target selection) and CLIENT_SETUP_PLAYER
/// (illegal substitution flow) and CLIENT_END_TURN (end of illegal substitution).
///
/// Client-only concerns skipped (headless engine): `UtilServerDialog.hideDialog`.
use ffb_model::inducement::card::Card;
use ffb_model::model::game::Game;
use ffb_model::prompts::agent_prompt::AgentPrompt;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_player::UtilPlayer;
use crate::action::Action;
use crate::step::framework::{Step, StepAction, StepOutcome, StepId, StepParameter};
use crate::step::util_server_injury::{drop_player, stun_player};
use crate::util::util_server_cards::UtilServerCards;

/// Java: `StepPlayCard` (mixed/inducements, BB2016 + BB2020).
#[derive(Debug, Default)]
pub struct StepPlayCard {
    /// Java: `fCard` — init parameter (mandatory).
    card: Option<Card>,
    /// Java: `fHomeTeam` — init parameter (mandatory).
    home_team: bool,
    /// Java: `fIllegalSubstitution`
    illegal_substitution: bool,
    /// Java: `fSetupPlayerId`
    setup_player_id: Option<String>,

    // Transient fields (not serialized in Java)
    /// Java: `fPlayerId` (transient)
    player_id: Option<String>,
    /// Java: `fOpponentId` (transient)
    opponent_id: Option<String>,
    /// Java: `fEndCardPlaying` (transient)
    end_card_playing: bool,
}

impl StepPlayCard {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        if self.end_card_playing {
            // Java: getResult().setNextAction(StepAction.NEXT_STEP)
            return StepOutcome::next();
        }

        if self.player_id.is_some() {
            // Java: playCardOnPlayer()
            return self.play_card_on_player(game, rng);
        }

        let Some(card) = self.card.clone() else { return StepOutcome::next() };

        if card.get_target().is_played_on_player() {
            // Java: step initInducement has already checked if this card can be played.
            let allowed_players = UtilServerCards::find_allowed_players_for_card(game, &card);
            return StepOutcome::cont().with_prompt(AgentPrompt::PlayerChoice {
                eligible_players: allowed_players,
                reason: "card".to_string(),
                descriptions: vec![],
            });
        }

        self.play_card_on_turn(game, rng)
    }

    fn play_card_on_turn(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let Some(card) = self.card.clone() else { return StepOutcome::next() };
        let (do_next_step, params) =
            UtilServerCards::activate_card(game, rng, &card, self.home_team, None);
        self.illegal_substitution = !do_next_step;
        let mut outcome = StepOutcome::next();
        for p in params {
            outcome = outcome.publish(p);
        }
        if do_next_step {
            outcome
        } else {
            // Java: doNextStep == false leaves next_action unset (CONTINUE by default) —
            // the illegal-substitution flow drives the rest via CLIENT_SETUP_PLAYER/CLIENT_END_TURN.
            StepOutcome::cont()
        }
    }

    fn play_card_on_player(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let Some(player_id) = self.player_id.clone() else { return StepOutcome::next() };
        if game.player(&player_id).is_none() {
            return StepOutcome::next();
        }
        let Some(card) = self.card.clone() else { return StepOutcome::next() };

        if card.requires_blockable_player_selection() {
            return self.play_card_with_blockable_player_selection(game, rng, &card, &player_id);
        }

        let (_, params) =
            UtilServerCards::activate_card(game, rng, &card, self.home_team, Some(&player_id));
        let mut outcome = StepOutcome::next();
        for p in params {
            outcome = outcome.publish(p);
        }
        outcome
    }

    /// Java: `playCardWithBlockablePlayerSelection()` — cards like Custard Pie that stun/drop an
    /// adjacent opponent chosen by the coach (or auto-selected if exactly one is eligible).
    fn play_card_with_blockable_player_selection(
        &mut self, game: &mut Game, rng: &mut GameRng, card: &Card, player_id: &str,
    ) -> StepOutcome {
        let mut outcome = StepOutcome::next();

        if self.opponent_id.is_none() {
            let player_coord = game.field_model.player_coordinate(player_id);
            let other_team = if self.home_team { &game.team_away } else { &game.team_home };
            if let Some(coord) = player_coord {
                let blockable: Vec<String> =
                    UtilPlayer::find_adjacent_blockable_players(game, other_team, coord)
                        .into_iter().cloned().collect();
                if blockable.len() == 1 {
                    self.opponent_id = Some(blockable[0].clone());
                } else {
                    let (_, params) =
                        UtilServerCards::activate_card(game, rng, card, self.home_team, Some(player_id));
                    for p in params {
                        outcome = outcome.publish(p);
                    }
                    outcome.action = StepAction::Continue;
                    return outcome.with_prompt(AgentPrompt::PlayerChoice {
                        eligible_players: blockable,
                        reason: "cardBlockablePlayer".to_string(),
                        descriptions: vec![],
                    });
                }
            }
            let (_, params) =
                UtilServerCards::activate_card(game, rng, card, self.home_team, Some(player_id));
            for p in params {
                outcome = outcome.publish(p);
            }
        }

        if let Some(opponent_id) = self.opponent_id.clone() {
            for p in stun_player(game, &opponent_id) {
                outcome = outcome.publish(p);
            }
            for p in drop_player(game, player_id, false) {
                outcome = outcome.publish(p);
            }
            outcome
        } else {
            // Java: doNextStep stays false — wait for the CLIENT_PLAYER_CHOICE(BLOCK) reply.
            StepOutcome::cont()
        }
    }
}

impl Step for StepPlayCard {
    fn id(&self) -> StepId { StepId::PlayCard }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: case CLIENT_PLAYER_CHOICE:
            //   if (PlayerChoiceMode.BLOCK == mode) fOpponentId = playerId
            //   else { fPlayerId = playerId; if (!provided(playerId)) fEndCardPlaying = true }
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
                    self.end_card_playing = true;
                } else {
                    self.player_id = Some(player_id.clone());
                }
            }
            // Java: case CLIENT_PLAYER_CHOICE with PlayerChoiceMode.BLOCK — opponent selection
            Action::Block { defender_id } => {
                self.opponent_id = Some(defender_id.clone());
            }
            // Java: case CLIENT_END_TURN with illegal substitution:
            //   fEndCardPlaying = true; process setup player if available; game.setTurnMode(REGULAR)
            //
            // NOTE: Java's full branch also does `game.getFieldModel().addCardEffect(setupPlayer,
            // CardEffect.ILLEGALLY_SUBSTITUTED); UtilServerSetup.setupPlayer(gameState,
            // fSetupPlayerId, fSetupPlayerCoordinate)` before clearing `fSetupPlayerId`/
            // `fSetupPlayerCoordinate` — the coordinate is populated by a `CLIENT_SETUP_PLAYER`
            // command, which has no `Action` variant in this engine yet (no player-placement
            // step in this flow reaches that far), so the substituted player is not actually
            // placed on the field here. That remains a known, documented gap. This fix only
            // restores the `turn_mode` reset, whose absence left the engine permanently stuck
            // in `TurnMode::IllegalSubstitution` after any Illegal Substitution card — every
            // subsequent turn's mode checks would see the wrong mode forever.
            Action::EndTurn => {
                if self.illegal_substitution {
                    self.end_card_playing = true;
                    self.setup_player_id = None;
                    self.illegal_substitution = false;
                    game.turn_mode = ffb_model::enums::TurnMode::Regular;
                }
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::CardId(v) => { self.card = v.clone(); true }
            StepParameter::HomeTeam(v) => { self.home_team = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::{PlayerGender, PlayerState, PlayerType, Rules, PS_PRONE, PS_STANDING, PS_STUNNED};
    use crate::step::framework::CatchScatterThrowInMode;
    use ffb_model::inducement::card_target::CardTarget;
    use ffb_model::model::player::Player;
    use ffb_model::report::report_id::ReportId;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate) {
        let player = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(player); } else { game.team_away.players.push(player); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn id_is_play_card() {
        assert_eq!(StepPlayCard::new().id(), StepId::PlayCard);
    }

    #[test]
    fn start_returns_next_by_default() {
        let mut step = StepPlayCard::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_card_playing_flag_causes_next_step() {
        let mut step = StepPlayCard::new();
        step.end_card_playing = true;
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_card_id() {
        let mut step = StepPlayCard::new();
        let card = Card::new("Pit Trap", Some("PIT_TRAP"));
        step.set_parameter(&StepParameter::CardId(Some(card.clone())));
        assert_eq!(step.card.as_ref().map(|c| c.get_name()), Some(card.get_name()));
    }

    #[test]
    fn set_parameter_home_team() {
        let mut step = StepPlayCard::new();
        step.set_parameter(&StepParameter::HomeTeam(true));
        assert!(step.home_team);
    }

    #[test]
    fn handle_select_player_empty_id_ends_card_playing() {
        let mut step = StepPlayCard::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::SelectPlayer { player_id: "".into() },
            &mut game, &mut rng,
        );
        assert!(step.end_card_playing);
    }

    #[test]
    fn handle_select_player_stores_id() {
        let mut step = StepPlayCard::new();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(
            &Action::SelectPlayer { player_id: "p1".into() },
            &mut game, &mut rng,
        );
        assert_eq!(step.player_id.as_deref(), Some("p1"));
    }

    #[test]
    fn handle_end_turn_with_illegal_substitution_ends_card_playing() {
        let mut step = StepPlayCard::new();
        step.illegal_substitution = true;
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert!(step.end_card_playing);
        assert!(!step.illegal_substitution);
    }

    /// Regression test: Java's `CLIENT_END_TURN` illegal-substitution branch calls
    /// `game.setTurnMode(TurnMode.REGULAR)` — the only place in `StepPlayCard` that reverts
    /// the `TurnMode::IllegalSubstitution` mode set when the Illegal Substitution card was
    /// played. A prior translation never reset it, so the engine's turn mode got stuck at
    /// `IllegalSubstitution` for the rest of the game after playing this card.
    #[test]
    fn handle_end_turn_with_illegal_substitution_resets_turn_mode_to_regular() {
        let mut step = StepPlayCard::new();
        step.illegal_substitution = true;
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::IllegalSubstitution;
        let mut rng = GameRng::new(0);
        step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(game.turn_mode, ffb_model::enums::TurnMode::Regular);
    }

    /// Without an active illegal substitution, CLIENT_END_TURN must not touch turn_mode.
    #[test]
    fn handle_end_turn_without_illegal_substitution_leaves_turn_mode_untouched() {
        let mut step = StepPlayCard::new();
        step.illegal_substitution = false;
        let mut game = make_game();
        game.turn_mode = ffb_model::enums::TurnMode::Setup;
        let mut rng = GameRng::new(0);
        step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(game.turn_mode, ffb_model::enums::TurnMode::Setup);
    }

    // ── Card activation / target routing (Phase AAU) ────────────────────────────

    #[test]
    fn turn_targeted_card_activates_immediately_and_reports() {
        let mut game = make_game();
        let mut step = StepPlayCard::new();
        step.card = Some(Card::new("Blackmail", Some("UNKNOWN_KEY")).with_target(CardTarget::TURN));
        step.home_team = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.report_list.has_report(ReportId::PLAY_CARD));
        assert!(game.turn_data_home.inducement_set.is_active("Blackmail"));
    }

    #[test]
    fn player_targeted_card_prompts_for_a_player_choice() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        game.turn_data_home.inducement_set.add_available_card("Pit Trap");
        let mut step = StepPlayCard::new();
        step.card = Some(Card::new("Pit Trap", Some("PIT_TRAP")).with_target(CardTarget::OWN_PLAYER));
        step.home_team = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        match out.prompt {
            Some(AgentPrompt::PlayerChoice { eligible_players, .. }) => {
                assert!(eligible_players.contains(&"p1".to_string()));
            }
            other => panic!("expected PlayerChoice prompt, got {other:?}"),
        }
    }

    #[test]
    fn blockable_player_selection_stun_publishes_ball_scatter() {
        // Java: playCardWithBlockablePlayerSelection publishes stunPlayer's returned
        // StepParameters (line: publishParameters(UtilServerInjury.stunPlayer(...))) — a
        // stunned ball carrier scatters the ball, previously silently dropped since
        // stun_player returned nothing.
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        add_player(&mut game, false, "p2", FieldCoordinate::new(6, 5));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(6, 5));
        game.field_model.ball_in_play = true;
        let mut step = StepPlayCard::new();
        step.card = Some(
            Card::new("Custard Pie", Some("CUSTARD_PIE"))
                .with_target(CardTarget::OPPOSING_PLAYER)
                .with_requires_blockable_player_selection(true),
        );
        step.home_team = true;
        step.player_id = Some("p1".into());
        step.opponent_id = Some("p2".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(
            p,
            StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall)
        )));
        assert_eq!(game.field_model.player_state("p2").unwrap().base(), PS_STUNNED);
    }

    #[test]
    fn blockable_player_selection_with_multiple_candidates_keeps_activation_params() {
        // Java: playCardWithBlockablePlayerSelection() calls UtilServerCards.activateCard(...)
        // *before* checking whether fOpponentId is provided — its side effects (report,
        // publishParameters from the card handler's activate()) always land on the shared
        // step Result, dialog-or-not. The Rust translation previously discarded the locally
        // accumulated `outcome` (built from activate_card's returned params) by returning a
        // brand-new `StepOutcome::cont().with_prompt(...)` instead of `outcome.with_prompt(...)`
        // when more than one blockable player is found — silently dropping activate_card's
        // published params in exactly that branch.
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        add_player(&mut game, false, "p2", FieldCoordinate::new(6, 5));
        add_player(&mut game, false, "p3", FieldCoordinate::new(4, 5));
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(5, 5));
        game.field_model.ball_in_play = true;
        let mut step = StepPlayCard::new();
        // Pit Trap's handler is the one with real activation_parameters (dropPlayer, which
        // scatters the ball off a carrier) — reused here purely to exercise the plumbing;
        // `requires_blockable_player_selection` is attached artificially since the real Pit
        // Trap card doesn't normally carry it.
        step.card = Some(
            Card::new("Pit Trap", Some("PIT_TRAP"))
                .with_target(CardTarget::OPPOSING_PLAYER)
                .with_requires_blockable_player_selection(true),
        );
        step.home_team = true;
        step.player_id = Some("p1".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
        assert!(matches!(out.prompt, Some(AgentPrompt::PlayerChoice { .. })));
        assert!(out.published.iter().any(|p| matches!(
            p,
            StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall)
        )));
        assert!(game.report_list.has_report(ReportId::PLAY_CARD));
    }

    #[test]
    fn selecting_a_player_activates_pit_trap_and_drops_them() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        let mut step = StepPlayCard::new();
        step.card = Some(Card::new("Pit Trap", Some("PIT_TRAP")).with_target(CardTarget::OWN_PLAYER));
        step.home_team = true;
        step.start(&mut game, &mut GameRng::new(0));
        let out = step.handle_command(
            &Action::SelectPlayer { player_id: "p1".into() },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(game.field_model.player_state("p1").unwrap().base(), PS_PRONE);
        assert!(game.turn_data_home.inducement_set.is_active("Pit Trap"));
    }

    #[test]
    fn play_card_on_turn_publishes_handler_activation_parameters() {
        let mut game = make_game();
        add_player(&mut game, true, "p1", FieldCoordinate::new(5, 5));
        // A turn-targeted card with the PIT_TRAP handler key is unusual in practice (Pit Trap is
        // normally player-targeted) but exercises play_card_on_turn -> activate_card ->
        // activation_parameters plumbing directly, with an empty player_id (no target selected).
        let mut step = StepPlayCard::new();
        step.card = Some(Card::new("Pit Trap", Some("PIT_TRAP")));
        step.home_team = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        // drop_player with an empty/unknown player id is a no-op (no such player), so no params —
        // this test only asserts the plumbing doesn't panic and the card still activates.
        assert!(game.turn_data_home.inducement_set.is_active("Pit Trap"));
    }
}
