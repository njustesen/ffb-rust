/// 1:1 translation of `Prayer.enhancements(StatsMechanic)` + `FieldModel.addPrayerEnhancements`.
///
/// Java: Each Prayer enum variant overrides `enhancements(mechanic)` to return a
/// `TemporaryEnhancements` object describing stat modifiers and/or skill grants.
/// `FieldModel.addPrayerEnhancements(player, prayer)` calls `player.addEnhancement(...)`.
///
/// Rust: Since `FieldModel` does not own players, the effect is applied via `Game` directly.
/// This module maps prayer names to their player-level effects and applies/removes them.
use ffb_model::enums::SkillId;
use ffb_model::model::game::Game;
use ffb_model::model::player::{STAT_MA, STAT_AV};

/// Java: Prayer.enhancements(mechanic) + Player.addEnhancement.
/// Applies the prayer's stat/skill effect to the given player.
pub fn apply_prayer_player_effect(game: &mut Game, player_id: &str, prayer_name: &str) {
    let Some(player) = game.player_mut(player_id) else { return };
    match prayer_name {
        // GREASY_CLEATS: TemporaryStatDecrementer(PlayerStatKey.MA)
        "GREASY_CLEATS" => {
            player.add_temporary_stat_mod(prayer_name, STAT_MA, -1);
        }
        // IRON_MAN: TemporaryStatIncrementer(PlayerStatKey.AV)
        "IRON_MAN" => {
            player.add_temporary_stat_mod(prayer_name, STAT_AV, 1);
        }
        // STILETTO: withSkills({Stab})
        "STILETTO" => {
            player.add_prayer_skill(prayer_name, SkillId::Stab, None);
        }
        // BAD_HABITS: withSkills({Loner, "2"}) → Loner (2+)
        "BAD_HABITS" => {
            player.add_prayer_skill(prayer_name, SkillId::Loner, Some("2".to_string()));
        }
        // KNUCKLE_DUSTERS: withSkills({MightyBlow})
        "KNUCKLE_DUSTERS" => {
            player.add_prayer_skill(prayer_name, SkillId::MightyBlow, Some("+1".to_string()));
        }
        // BLESSED_STATUE_OF_NUFFLE: withSkills({Pro}) — dialog-based, applied here post-selection
        "BLESSED_STATUE_OF_NUFFLE" => {
            player.add_prayer_skill(prayer_name, SkillId::Pro, None);
        }
        _ => {}
    }
}

/// Java: Player.removeEnhancements(prayerName) + FieldModel.removePrayerEnhancements.
/// Removes prayer-granted stat mods and skills from the player.
pub fn remove_prayer_player_effect(game: &mut Game, player_id: &str, prayer_name: &str) {
    if let Some(player) = game.player_mut(player_id) {
        player.remove_enhancements(prayer_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::player_status::PlayerStatus;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    fn add_player(game: &mut Game, team_id: &str, id: &str, movement: i32, armour: i32) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement, strength: 3, agility: 3, passing: 4, armour,
            player_status: PlayerStatus::ACTIVE,
            ..Default::default()
        };
        if team_id == "home" { game.team_home.players.push(p); }
        else { game.team_away.players.push(p); }
    }

    #[test]
    fn greasy_cleats_reduces_movement_by_one() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", 6, 8);
        apply_prayer_player_effect(&mut game, "h1", "GREASY_CLEATS");
        let p = game.player("h1").unwrap();
        assert_eq!(p.movement_with_modifiers(), 5);
        assert_eq!(p.armour_with_modifiers(), 8); // unaffected
    }

    #[test]
    fn iron_man_increases_armour_by_one() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", 6, 8);
        apply_prayer_player_effect(&mut game, "h1", "IRON_MAN");
        let p = game.player("h1").unwrap();
        assert_eq!(p.armour_with_modifiers(), 9);
        assert_eq!(p.movement_with_modifiers(), 6); // unaffected
    }

    #[test]
    fn stiletto_grants_stab_skill() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", 6, 8);
        apply_prayer_player_effect(&mut game, "h1", "STILETTO");
        let p = game.player("h1").unwrap();
        assert!(p.has_skill(SkillId::Stab));
    }

    #[test]
    fn bad_habits_grants_loner_with_2_value() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", 6, 8);
        apply_prayer_player_effect(&mut game, "h1", "BAD_HABITS");
        let p = game.player("h1").unwrap();
        assert!(p.has_skill(SkillId::Loner));
        let sw = p.temporary_skills.iter().find(|sw| sw.skill_id == SkillId::Loner).unwrap();
        assert_eq!(sw.value.as_deref(), Some("2"));
    }

    #[test]
    fn knuckle_dusters_grants_mighty_blow() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", 6, 8);
        apply_prayer_player_effect(&mut game, "h1", "KNUCKLE_DUSTERS");
        assert!(game.player("h1").unwrap().has_skill(SkillId::MightyBlow));
    }

    #[test]
    fn remove_prayer_player_effect_undoes_stat_mod() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", 6, 8);
        apply_prayer_player_effect(&mut game, "h1", "GREASY_CLEATS");
        assert_eq!(game.player("h1").unwrap().movement_with_modifiers(), 5);
        remove_prayer_player_effect(&mut game, "h1", "GREASY_CLEATS");
        assert_eq!(game.player("h1").unwrap().movement_with_modifiers(), 6);
    }

    #[test]
    fn remove_prayer_player_effect_undoes_skill_grant() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", 6, 8);
        apply_prayer_player_effect(&mut game, "h1", "STILETTO");
        assert!(game.player("h1").unwrap().has_skill(SkillId::Stab));
        remove_prayer_player_effect(&mut game, "h1", "STILETTO");
        assert!(!game.player("h1").unwrap().has_skill(SkillId::Stab));
    }

    #[test]
    fn unknown_prayer_is_noop() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", 6, 8);
        apply_prayer_player_effect(&mut game, "h1", "TREACHEROUS_TRAPDOOR");
        let p = game.player("h1").unwrap();
        assert_eq!(p.movement_with_modifiers(), 6);
        assert_eq!(p.armour_with_modifiers(), 8);
        assert!(p.temporary_skills.is_empty());
    }

    #[test]
    fn missing_player_is_noop() {
        let mut game = make_game();
        // player not added — should not panic
        apply_prayer_player_effect(&mut game, "nobody", "GREASY_CLEATS");
        remove_prayer_player_effect(&mut game, "nobody", "GREASY_CLEATS");
    }
}
