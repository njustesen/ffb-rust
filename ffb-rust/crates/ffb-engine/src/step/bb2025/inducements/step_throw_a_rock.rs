use ffb_model::enums::ApothecaryMode;
use ffb_model::events::GameEvent;
use ffb_model::model::game::Game;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
use crate::step::framework::{Step, StepOutcome, StepParameter};
use crate::step::framework::StepId;
use crate::step::util_server_injury::handle_injury_by_name;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.inducements.StepThrowARock.
///
/// Throws a rock at a random standing opponent on the pitch.
/// On a hit (roll ≥ 4) performs an injury roll via InjuryTypeThrowARockStalling and
/// publishes a STEADY_FOOTING_CONTEXT wrapping a DropPlayerContext.
///
/// Init parameters: HOME_TEAM.
pub struct StepThrowARock {
    /// Java: homeTeam — which team threw the rock (targets the opponent).
    pub home_team: bool,
}

impl StepThrowARock {
    pub fn new(home_team: bool) -> Self {
        Self { home_team }
    }
}

impl Step for StepThrowARock {
    fn id(&self) -> StepId { StepId::ThrowARock }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        if let StepParameter::HomeTeam(v) = param {
            self.home_team = *v;
            return true;
        }
        false
    }
}

impl StepThrowARock {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: Team team = homeTeam ? game.getTeamAway() : game.getTeamHome()
        // (homeTeam = the team that THREW the rock, so the TARGET is the opponent)
        let eligible: Vec<String> = {
            let team = if self.home_team { &game.team_away } else { &game.team_home };
            team.players.iter()
                .filter(|p| {
                    game.field_model.player_coordinate(&p.id)
                        .map(|c| FieldCoordinateBounds::FIELD.is_in_bounds(c))
                        .unwrap_or(false)
                        && game.field_model.player_state(&p.id)
                            .map(|s| !s.is_prone_or_stunned())
                            .unwrap_or(false)
                })
                .map(|p| p.id.clone())
                .collect()
        };

        if eligible.is_empty() {
            return StepOutcome::next();
        }

        // Java: Collections.shuffle(players) — random selection
        let idx = (rng.die(eligible.len() as u32) - 1) as usize;
        let target_id = eligible[idx].clone();

        // Java: int roll = rollDice(6); boolean successful = roll >= 4
        let roll = rng.d6();
        let successful = roll >= 4;

        let throw_event = GameEvent::ThrowAtPlayer { player_id: target_id.clone(), roll, successful };

        // Java: FieldCoordinate startCoordinate (animation origin — not tracked in Rust)
        // Java: UtilServerGame.syncGameModel(this)

        if successful {
            let coord = game.field_model
                .player_coordinate(&target_id)
                .unwrap_or(FieldCoordinate::new(0, 0));

            // Java: UtilServerInjury.handleInjury(this, new InjuryTypeThrowARockStalling(),
            //         null, player, playerCoordinate, null, null, ApothecaryMode.DEFENDER)
            let injury_result = handle_injury_by_name(
                game, rng,
                "InjuryTypeThrowARockStalling",
                None,
                &target_id,
                coord,
                None,
                None,
                ApothecaryMode::Defender,
            );

            // Java: DropPlayerContextBuilder.builder()
            //         .injuryResult(injuryResult).playerId(player.getId())
            //         .apothecaryMode(DEFENDER).eligibleForSafePairOfHands(true).build()
            let dpc = DropPlayerContext::with_injury(
                injury_result,
                target_id,
                ApothecaryMode::Defender,
                true,
            );

            // Java: publishParameter(new StepParameter(STEADY_FOOTING_CONTEXT, new SteadyFootingContext(dropPlayerContext)))
            let ctx = SteadyFootingContext::from_drop_player(dpc);
            return StepOutcome::next()
                .with_event(throw_event)
                .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
        }

        StepOutcome::next().with_event(throw_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING, PlayerType, PlayerGender};
    use ffb_model::model::player::Player;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    fn make_player(id: &str, nr: i32) -> Player {
        Player {
            id: id.into(), name: id.into(), nr,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    #[test]
    fn no_eligible_players_returns_next() {
        let mut game = make_game();
        let mut step = StepThrowARock::new(true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.is_empty());
    }

    #[test]
    fn home_team_targets_away_players() {
        let mut game = make_game();
        let p = make_player("away1", 1);
        game.team_away.players.push(p);
        game.field_model.set_player_coordinate("away1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("away1", ffb_model::enums::PlayerState::new(PS_STANDING));

        let mut step = StepThrowARock::new(true);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn away_team_targets_home_players() {
        let mut game = make_game();
        let p = make_player("home1", 1);
        game.team_home.players.push(p);
        game.field_model.set_player_coordinate("home1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("home1", ffb_model::enums::PlayerState::new(PS_STANDING));

        let mut step = StepThrowARock::new(false);
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next() {
        let mut game = make_game();
        let mut step = StepThrowARock::new(true);
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    /// A successful hit (roll ≥ 4) publishes STEADY_FOOTING_CONTEXT.
    #[test]
    fn successful_hit_publishes_steady_footing_context() {
        // Find a seed where both the player-selection roll AND the hit roll produce a hit.
        // The step rolls: die(1) (always 1 with 1 eligible player) then d6.
        // So we just need a seed where d6 >= 4.
        let mut game = make_game();
        let p = make_player("away1", 1);
        game.team_away.players.push(p);
        game.field_model.set_player_coordinate("away1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("away1", ffb_model::enums::PlayerState::new(PS_STANDING));

        for seed in 0..200u64 {
            let mut rng = GameRng::new(seed);
            rng.die(1); // player selection (always 1)
            let roll = rng.d6();
            if roll >= 4 {
                let mut game2 = make_game();
                let p2 = make_player("away1", 1);
                game2.team_away.players.push(p2);
                game2.field_model.set_player_coordinate("away1", FieldCoordinate::new(10, 7));
                game2.field_model.set_player_state("away1", ffb_model::enums::PlayerState::new(PS_STANDING));

                let mut step = StepThrowARock::new(true);
                let out = step.start(&mut game2, &mut GameRng::new(seed));
                assert_eq!(out.action, StepAction::NextStep);
                assert!(
                    out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
                    "seed={seed} roll={roll}: expected SteadyFootingContext"
                );
                return;
            }
        }
        panic!("no seed found with d6>=4");
    }

    /// A missed rock (roll < 4) does NOT publish STEADY_FOOTING_CONTEXT.
    #[test]
    fn missed_rock_does_not_publish_context() {
        let mut game = make_game();
        let p = make_player("away1", 1);
        game.team_away.players.push(p);
        game.field_model.set_player_coordinate("away1", FieldCoordinate::new(10, 7));
        game.field_model.set_player_state("away1", ffb_model::enums::PlayerState::new(PS_STANDING));

        for seed in 0..200u64 {
            let mut rng = GameRng::new(seed);
            rng.die(1); // player selection
            let roll = rng.d6();
            if roll < 4 {
                let mut game2 = make_game();
                let p2 = make_player("away1", 1);
                game2.team_away.players.push(p2);
                game2.field_model.set_player_coordinate("away1", FieldCoordinate::new(10, 7));
                game2.field_model.set_player_state("away1", ffb_model::enums::PlayerState::new(PS_STANDING));

                let mut step = StepThrowARock::new(true);
                let out = step.start(&mut game2, &mut GameRng::new(seed));
                assert!(
                    !out.published.iter().any(|p| matches!(p, StepParameter::SteadyFootingContext(_))),
                    "seed={seed} roll={roll}: should NOT publish SteadyFootingContext"
                );
                return;
            }
        }
        panic!("no seed found with d6<4");
    }

    /// set_parameter: HomeTeam accepted.
    #[test]
    fn set_parameter_home_team_accepted() {
        let mut step = StepThrowARock::new(false);
        assert!(step.set_parameter(&StepParameter::HomeTeam(true)));
        assert!(step.home_team);
    }
}
