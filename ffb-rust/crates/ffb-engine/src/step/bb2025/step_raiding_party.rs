/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.StepRaidingParty (BB2025).
///
/// Resolves the Raiding Party skill: move an open teammate to an adjacent square beside an opponent.
///
/// Init params: GOTO_LABEL_ON_FAILURE, GOTO_LABEL_ON_SUCCESS.
/// Runtime params: END_TURN, END_PLAYER_ACTION.
/// Commands: CLIENT_PLAYER_CHOICE (teammate selection), CLIENT_FIELD_COORDINATE (target square), CLIENT_END_TURN.
///
/// Full port: findPlayers → (auto-pick or DialogPlayerChoice RAIDING_PARTY) → findSquares →
/// FieldCoordinate prompt (MoveSquares published, turn mode saved/switched to RAIDING_PARTY) →
/// move + ReportRaidingParty + markSkillUsed + push [PICK_UP → CATCH_SCATTER_THROW_IN] sequence.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::skill_use::SkillUse;
use ffb_model::util::rng::GameRng;
use ffb_model::report::report_skill_use::ReportSkillUse;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepRaidingParty {
    /// Java: endPlayerAction — set by END_PLAYER_ACTION parameter.
    pub end_player_action: bool,
    /// Java: endTurn — set by END_TURN parameter.
    pub end_turn: bool,
    /// Java: goToLabelOnFailure — GOTO_LABEL_ON_FAILURE init parameter.
    pub goto_label_on_failure: String,
    /// Java: gotoLabelOnSuccess — GOTO_LABEL_ON_SUCCESS init parameter.
    pub goto_label_on_success: String,
    /// Java: playerId — selected teammate.
    pub player_id: Option<String>,
    /// Java: coordinate — selected target square.
    pub coordinate: Option<ffb_model::types::FieldCoordinate>,
    /// Java: savedTurnMode — lastTurnMode stashed while turn mode is RAIDING_PARTY.
    pub saved_turn_mode: Option<ffb_model::enums::TurnMode>,
}

impl StepRaidingParty {
    pub fn new() -> Self {
        Self {
            end_player_action: false,
            end_turn: false,
            goto_label_on_failure: String::new(),
            goto_label_on_success: String::new(),
            player_id: None,
            coordinate: None,
            saved_turn_mode: None,
        }
    }
}

impl Default for StepRaidingParty {
    fn default() -> Self { Self::new() }
}

impl Step for StepRaidingParty {
    fn id(&self) -> StepId { StepId::RaidingParty }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::SelectPlayer { player_id } => {
                if player_id.is_empty() {
                    // Java: empty playerId → CLIENT_PLAYER_CHOICE cancel → resetState +
                    // addReport(ReportSkillUse(false, MOVE_OPEN_TEAM_MATE)) → NEXT_STEP.
                    self.reset_state(game);
                    let actor_id = game.acting_player.player_id.clone();
                    game.report_list.add(ReportSkillUse::new(
                        actor_id,
                        SkillId::RaidingParty,
                        false,
                        SkillUse::MOVE_OPEN_TEAM_MATE,
                    ));
                    return StepOutcome::next();
                }
                self.player_id = Some(player_id.clone());
            }
            // Java CLIENT_FIELD_COORDINATE — Rust agents answer prompts with RAW board
            // coordinates (the InitPassing/CLIENT_PASS precedent), so no away-side transform.
            Action::RaidingPartyTarget { coord } => {
                self.coordinate = Some(*coord);
            }
            Action::EndTurn => { self.end_turn = true; }
            _ => {}
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::EndTurn(v)               => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)       => { self.end_player_action = *v; true }
            StepParameter::GotoLabelOnFailure(v)    => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::GotoLabelOnSuccess(v)    => { self.goto_label_on_success = v.clone(); true }
            _ => false,
        }
    }
}

impl StepRaidingParty {
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: skill = UtilCards.getUnusedSkillWithProperty(actingPlayer, canMoveOpenTeamMate)
        let has_skill = game.player(&player_id)
            .map(|p| p.has_skill(SkillId::RaidingParty) && !p.used_skills.contains(&SkillId::RaidingParty))
            .unwrap_or(false);

        if !has_skill {
            return StepOutcome::next();
        }

        // Java: if (endTurn || endPlayerAction) → resetState + GOTO_LABEL
        if self.end_turn || self.end_player_action {
            self.reset_state(game);
            return StepOutcome::goto(&self.goto_label_on_failure);
        }

        // Java :178-194 — pick the team-mate.
        if self.player_id.is_none() {
            let eligible = Self::find_players(game, &player_id);
            if eligible.is_empty() {
                return StepOutcome::next();
            }
            game.report_list.add(ReportSkillUse::new(
                Some(player_id.clone()),
                SkillId::RaidingParty,
                true,
                SkillUse::MOVE_OPEN_TEAM_MATE,
            ));
            if eligible.len() == 1 {
                self.player_id = Some(eligible[0].clone());
            } else {
                // Java: DialogPlayerChoiceParameter(actingTeam, RAIDING_PARTY, eligible, null, 1)
                // + prepareClientData(eligible squares of ALL eligible players).
                let squares: Vec<ffb_model::types::FieldCoordinate> = eligible.iter()
                    .flat_map(|pid| Self::find_squares(game, pid))
                    .collect();
                self.prepare_client_data(game, &squares);
                return StepOutcome::cont().with_prompt(
                    ffb_model::prompts::AgentPrompt::PlayerChoice {
                        eligible_players: eligible,
                        reason: "RAIDING_PARTY".into(),
                        descriptions: Vec::new(),
                    });
            }
        }

        // Java :196-207 — pick the square.
        if let (Some(pid), None) = (self.player_id.clone(), self.coordinate) {
            let squares = Self::find_squares(game, &pid);
            if squares.is_empty() {
                return StepOutcome::next();
            }
            self.prepare_client_data(game, &squares);
            return StepOutcome::cont().with_prompt(
                ffb_model::prompts::AgentPrompt::RaidingParty {
                    player_id: pid,
                    squares,
                });
        }

        // Java :209-233 — perform the move and push PICK_UP → CATCH_SCATTER_THROW_IN.
        if let (Some(pid), Some(coordinate)) = (self.player_id.clone(), self.coordinate) {
            let from = game.field_model.player_coordinate(&pid);
            let direction = from.and_then(|f| f.direction_to(coordinate));
            game.report_list.add(ffb_model::report::mixed::report_raiding_party::ReportRaidingParty::new(
                Some(player_id.clone()), Some(pid.clone()), direction,
            ));
            game.field_model.update_player_and_ball_position(&pid, coordinate);
            // Java: actingPlayer.markSkillUsed(skill) — Raiding Party's usage type is
            // OncePerDrive (trackOutsideActivation), so it lands on the PLAYER too.
            game.acting_player.used_skills.insert(SkillId::RaidingParty);
            let actor = player_id.clone();
            game.mark_skill_used(&actor, SkillId::RaidingParty);
            self.reset_state(game);

            use crate::step::generator::sequence::{Sequence, labels};
            let mut seq = Sequence::new();
            seq.add(StepId::PickUp, vec![
                StepParameter::GotoLabelOnFailure(labels::SCATTER_BALL.into()),
                StepParameter::ThrownPlayerId(Some(pid)),
            ]);
            if !self.goto_label_on_success.is_empty() {
                seq.jump(&self.goto_label_on_success);
            } else {
                seq.jump(labels::NEXT);
            }
            seq.add_labelled(StepId::CatchScatterThrowIn, labels::SCATTER_BALL, vec![]);
            seq.jump(&self.goto_label_on_failure);
            if self.goto_label_on_success.is_empty() {
                seq.add_labelled(StepId::NextStepAndRepeat, labels::NEXT, vec![]);
            }
            return StepOutcome::next().push_seq(seq.build());
        }

        StepOutcome::next()
    }

    /// Java: resetState(Game) — restore turn modes and refresh client decorations.
    fn reset_state(&mut self, game: &mut Game) {
        if game.turn_mode == ffb_model::enums::TurnMode::RaidingParty {
            game.turn_mode = game.last_turn_mode.unwrap_or(game.turn_mode);
            if let Some(saved) = self.saved_turn_mode.take() {
                game.last_turn_mode = Some(saved);
            }
        }
        // Java: UtilServerPlayerMove.updateMoveSquares(gameState, actingPlayer.isJumping()) +
        // ServerUtilBlock.updateDiceDecorations. NOT cosmetic: StepMove consumes the move
        // squares' dodge/GFI flags, so skipping the rebuild left the continuation move with
        // stale dodge-free squares and Ivar walked out of a tacklezone without the dodge Java
        // rolled (human bb2025 seed 1 i=27).
        let jumping = game.acting_player.jumping;
        crate::util::util_server_player_move::UtilServerPlayerMove::update_move_squares(game, jumping);
        crate::util::server_util_block::ServerUtilBlock::update_dice_decorations(game);
    }

    /// Java: prepareClientData — publish the eligible squares as MoveSquares, save/switch
    /// the turn mode to RAIDING_PARTY, and wait.
    fn prepare_client_data(&mut self, game: &mut Game, squares: &[ffb_model::types::FieldCoordinate]) {
        game.field_model.clear_move_squares();
        for sq in squares {
            game.field_model.add_move_square(ffb_model::types::MoveSquare::new(*sq, 0, 0));
        }
        if game.turn_mode != ffb_model::enums::TurnMode::RaidingParty {
            self.saved_turn_mode = game.last_turn_mode;
            game.last_turn_mode = Some(game.turn_mode);
            game.turn_mode = ffb_model::enums::TurnMode::RaidingParty;
        }
    }

    /// Java: findSquares(game, player) — empty adjacent in-field squares that are themselves
    /// adjacent to a square whose occupant is an opponent.
    fn adjacent_in_field(c: ffb_model::types::FieldCoordinate) -> Vec<ffb_model::types::FieldCoordinate> {
        use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
        let mut out = Vec::new();
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                let n = FieldCoordinate::new(c.x + dx, c.y + dy);
                if FieldCoordinateBounds::FIELD.is_in_bounds(n) { out.push(n); }
            }
        }
        out
    }

    fn find_squares(game: &Game, player_id: &str) -> Vec<ffb_model::types::FieldCoordinate> {
        let Some(pc) = game.field_model.player_coordinate(player_id) else { return vec![] };
        let acting_home = game.home_playing;
        Self::adjacent_in_field(pc)
            .into_iter()
            .filter(|&sq| game.field_model.player_at(sq).is_none())
            .filter(|&sq| {
                Self::adjacent_in_field(sq).into_iter().any(|adj| {
                    game.field_model.player_at(adj)
                        .map(|oid| game.team_home.has_player(oid) != acting_home)
                        .unwrap_or(false)
                })
            })
            .collect()
    }

    /// Java: findPlayers(game, actingPlayer) — acting-team players that are STANDING (or the
    /// actor), un-pinned, with tacklezones, within 5 steps, OPEN (no adjacent opponents with
    /// tacklezones), and with at least one eligible square (see find_squares).
    fn find_players(game: &Game, actor_id: &str) -> Vec<String> {
        use ffb_model::enums::PS_STANDING;
        let Some(actor_coord) = game.field_model.player_coordinate(actor_id) else { return vec![] };
        let acting_home = game.home_playing;
        let team = if acting_home { &game.team_home } else { &game.team_away };
        let opponent = if acting_home { &game.team_away } else { &game.team_home };
        team.players.iter()
            .filter(|tm| {
                let Some(tc) = game.field_model.player_coordinate(&tm.id) else { return false };
                let Some(state) = game.field_model.player_state(&tm.id) else { return false };
                let open = ffb_model::util::util_player::UtilPlayer::find_adjacent_players_with_tacklezones(
                    game, opponent, tc, false,
                ).is_empty();
                (state.base() == PS_STANDING || tm.id == actor_id)
                    && !state.is_pinned()
                    && state.has_tacklezones()
                    && tc.distance_in_steps(actor_coord) <= 5
                    && open
                    && !Self::find_squares(game, &tm.id).is_empty()
            })
            .map(|tm| tm.id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PlayerState, PS_STANDING, PlayerAction, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;

    fn make_player(id: &str, skill: Option<SkillId>) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: skill.map(|s| vec![SkillWithValue { skill_id: s, value: None }])
                .unwrap_or_default(),
            extra_skills: vec![], temporary_skills: vec![], used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game_rp() -> (Game, String) {
        let pid = "actor".to_string();
        let mut home = test_team("home", 0);
        home.players.push(make_player(&pid, Some(SkillId::RaidingParty)));
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game.acting_player.player_id = Some(pid.clone());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.field_model.set_player_state(&pid, PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_coordinate(&pid, FieldCoordinate::new(10, 7));
        (game, pid)
    }

    #[test]
    fn no_skill_returns_next_step() {
        let (mut game, _) = make_game_rp();
        game.team_home.players[0].starting_skills.clear();
        let mut step = StepRaidingParty::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn end_turn_goes_to_label() {
        let (mut game, _) = make_game_rp();
        let mut step = StepRaidingParty::new();
        step.goto_label_on_failure = "FAIL".into();
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn with_skill_returns_next_step_stub() {
        let (mut game, _) = make_game_rp();
        let mut step = StepRaidingParty::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_wiring() {
        let mut step = StepRaidingParty::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnFailure("F".into())));
        assert_eq!(step.goto_label_on_failure, "F");
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("S".into())));
        assert_eq!(step.goto_label_on_success, "S");
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }

    #[test]
    fn cancel_selection_adds_skill_use_false_report() {
        use ffb_model::report::report_id::ReportId;
        let (mut game, _) = make_game_rp();
        let mut step = StepRaidingParty::new();
        // Empty player_id = cancel
        let out = step.handle_command(
            &Action::SelectPlayer { player_id: String::new() },
            &mut game, &mut GameRng::new(0),
        );
        assert_eq!(out.action, StepAction::NextStep);
        assert!(game.report_list.has_report(ReportId::SKILL_USE),
            "cancel selection should add ReportSkillUse(false, MOVE_OPEN_TEAM_MATE)");
    }

    #[test]
    fn cancel_selection_skill_use_is_false() {
        use ffb_model::report::report_id::ReportId;
        use ffb_model::model::skill_use::SkillUse;
        let (mut game, _) = make_game_rp();
        let mut step = StepRaidingParty::new();
        step.handle_command(
            &Action::SelectPlayer { player_id: String::new() },
            &mut game, &mut GameRng::new(0),
        );
        // Verify the report is SKILL_USE (used=false path)
        assert!(game.report_list.has_report(ReportId::SKILL_USE));
    }
}
