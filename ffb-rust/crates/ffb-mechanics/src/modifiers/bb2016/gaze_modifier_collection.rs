use ffb_model::util::util_player::UtilPlayer;
use crate::modifiers::gaze_modifier::GazeModifier;
use crate::modifiers::gaze_modifier_context::GazeModifierContext;
use crate::modifiers::modifier_type::ModifierType;

/// Java: `com.fumbbl.ffb.modifiers.bb2016.GazeModifierCollection`.
///
/// One modifier per tacklezone count (1-8). `find_applicable` returns the modifier
/// whose `multiplier == number_of_tacklezones(gazer)` — matching Java's
/// `GenerifiedModifierFactory.getTacklezoneModifier`.
pub struct GazeModifierCollection {
    modifiers: Vec<GazeModifier>,
}

impl GazeModifierCollection {
    pub fn new() -> Self {
        let mut col = Self { modifiers: Vec::new() };
        // Java: new GazeModifier("N Tacklezone(s)", "...", modifier=N-1, multiplier=N, TACKLEZONE)
        // modifier here is penalty to the minimum roll (0 for 1 TZ, 1 for 2 TZs, etc.)
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Tacklezone".to_string() } else { format!("{} Tacklezones", i) };
            let report = if i == 1 {
                "0 for being in 1 tacklezone (including target)".to_string()
            } else {
                format!("{} for being in {} tacklezones (including target)", i - 1, i)
            };
            col.modifiers.push(GazeModifier::new_full(name, report, i - 1, i, ModifierType::TACKLEZONE));
        }
        col
    }

    pub fn get_modifiers(&self) -> &[GazeModifier] { &self.modifiers }

    /// Java: `GenerifiedModifierFactory.getTacklezoneModifier` — finds the single
    /// tacklezone modifier matching the actual count of opposing tackle zones the gazer
    /// is in (using `UtilPlayer::find_tacklezones`).
    pub fn find_applicable<'a>(&'a self, ctx: &GazeModifierContext<'_>) -> Vec<&'a GazeModifier> {
        let tz_count = UtilPlayer::find_tacklezones(ctx.game, &ctx.player.id) as i32;
        self.modifiers.iter()
            .filter(|m| m.get_multiplier() == tz_count)
            .collect()
    }
}

impl Default for GazeModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerState, PlayerType, Rules};
    use ffb_model::model::{Game, Player, Team};
    use ffb_model::types::FieldCoordinate;

    // PS_STANDING(0x1) | BIT_ACTIVE(0x100) = 0x101
    const ACTIVE_STANDING: PlayerState = PlayerState(0x101);

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
        }
    }

    fn minimal_player(id: &str, agility: i32) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, state: PlayerState) {
        let p = minimal_player(id, 3);
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, state);
    }

    #[test]
    fn new_creates_8_modifiers() {
        let col = GazeModifierCollection::new();
        assert_eq!(col.modifiers.len(), 8);
    }

    #[test]
    fn modifier_values_match_tacklezone_minus_one() {
        let col = GazeModifierCollection::new();
        for (i, m) in col.modifiers.iter().enumerate() {
            let tz_count = (i + 1) as i32;
            assert_eq!(m.get_modifier(), tz_count - 1, "tacklezone {tz_count} should give modifier {}", tz_count - 1);
            assert_eq!(m.get_multiplier(), tz_count);
        }
    }

    #[test]
    fn find_applicable_no_opposing_players_returns_empty() {
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016);
        add_player(&mut game, true, "gazer", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        let gazer = game.team_home.player("gazer").unwrap();
        let col = GazeModifierCollection::new();
        let ctx = GazeModifierContext::new(&game, gazer);
        assert!(col.find_applicable(&ctx).is_empty());
    }

    #[test]
    fn find_applicable_one_tacklezone_returns_modifier_zero() {
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016);
        add_player(&mut game, true, "gazer", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "opp", FieldCoordinate::new(6, 5), ACTIVE_STANDING);
        let gazer = game.team_home.player("gazer").unwrap();
        let col = GazeModifierCollection::new();
        let ctx = GazeModifierContext::new(&game, gazer);
        let applicable = col.find_applicable(&ctx);
        assert_eq!(applicable.len(), 1);
        assert_eq!(applicable[0].get_modifier(), 0);
    }

    #[test]
    fn find_applicable_two_tacklezones_returns_modifier_one() {
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016);
        add_player(&mut game, true, "gazer", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "opp1", FieldCoordinate::new(6, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "opp2", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        let gazer = game.team_home.player("gazer").unwrap();
        let col = GazeModifierCollection::new();
        let ctx = GazeModifierContext::new(&game, gazer);
        let applicable = col.find_applicable(&ctx);
        assert_eq!(applicable.len(), 1);
        assert_eq!(applicable[0].get_modifier(), 1);
    }
}
