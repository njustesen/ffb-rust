use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_cards::UtilCards;
use crate::mechanic::{Mechanic, MechanicType};
use crate::jump_mechanic::JumpMechanic as JumpMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.JumpMechanic.
pub struct JumpMechanic;

impl JumpMechanic {
    pub fn new() -> Self { Self }
}

impl Default for JumpMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for JumpMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::JUMP }
}

impl JumpMechanicTrait for JumpMechanic {
    fn is_available_as_next_move(&self, game: &Game, acting_player: &ActingPlayer, jumping: bool) -> bool {
        self.can_still_jump(game, acting_player) && {
            // TODO: UtilPlayer::is_next_move_possible(game, jumping)
            let _ = (game, jumping);
            false
        }
    }

    fn can_still_jump(&self, game: &Game, acting_player: &ActingPlayer) -> bool {
        acting_player.player_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| UtilCards::has_unused_skill_with_property(p, NamedProperties::CAN_LEAP))
            .unwrap_or(false)
    }

    fn can_jump(&self, _game: &Game, player: &Player, _coordinate: FieldCoordinate) -> bool {
        player.has_skill_property(NamedProperties::CAN_LEAP)
    }

    fn is_valid_jump(&self, _game: &Game, _player: &Player, from: FieldCoordinate, to: FieldCoordinate) -> bool {
        from != to && to.distance_in_steps(from) < 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, Rules, SkillId};
    use ffb_model::model::game::Game;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2016)
    }

    fn add_player_with_leap(game: &mut Game, id: &str) {
        let mut p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Leap)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        };
        game.team_home.players.push(p);
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn can_still_jump_true_with_leap_skill() {
        let mut game = make_game();
        add_player_with_leap(&mut game, "p1");
        let ap = game.acting_player.clone();
        assert!(JumpMechanic::new().can_still_jump(&game, &ap));
    }

    #[test]
    fn can_still_jump_false_without_leap_skill() {
        let mut game = make_game();
        game.team_home.players.push(Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        game.acting_player.player_id = Some("p1".into());
        let ap = game.acting_player.clone();
        assert!(!JumpMechanic::new().can_still_jump(&game, &ap));
    }

    #[test]
    fn can_still_jump_false_when_no_player() {
        let game = make_game();
        let ap = game.acting_player.clone();
        assert!(!JumpMechanic::new().can_still_jump(&game, &ap));
    }

    #[test]
    fn can_still_jump_false_when_leap_used() {
        let mut game = make_game();
        add_player_with_leap(&mut game, "p1");
        game.team_home.player_mut("p1").unwrap().used_skills.insert(SkillId::Leap);
        let ap = game.acting_player.clone();
        assert!(!JumpMechanic::new().can_still_jump(&game, &ap));
    }
}
