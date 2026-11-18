use ffb_model::enums::{PassingDistance, TurnMode};
use ffb_model::model::{Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
use crate::mechanic::{Mechanic, MechanicType};
use crate::modifiers::{PassModifier, StatBasedRollModifier};
use crate::pass_result::PassResult;
use crate::pass_mechanic::PassMechanic as PassMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.PassMechanic.
pub struct PassMechanic;

impl Default for PassMechanic {
    fn default() -> Self { PassMechanic }
}

impl Mechanic for PassMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::PASS }
}

impl PassMechanicTrait for PassMechanic {
    fn throwing_range_table(&self) -> Vec<String> {
        vec![
            "T Q Q Q S S S L L L L B B B".into(),
            "Q Q Q Q S S S L L L L B B B".into(),
            "Q Q Q S S S S L L L L B B B".into(),
            "Q Q S S S S S L L L B B B  ".into(),
            "S S S S S S L L L L B B B  ".into(),
            "S S S S S L L L L B B B    ".into(),
            "S S S S L L L L L B B B    ".into(),
            "L L L L L L L L B B B      ".into(),
            "L L L L L L L B B B B      ".into(),
            "L L L L L B B B B B        ".into(),
            "L L L B B B B B B          ".into(),
            "B B B B B B B              ".into(),
            "B B B B B                  ".into(),
            "B B B                      ".into(),
        ]
    }

    fn minimum_roll(
        &self,
        thrower: &Player,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        stat_based_roll_modifier: Option<&StatBasedRollModifier>,
    ) -> Option<i32> {
        let pa = thrower.passing_with_modifiers();
        if pa > 0 {
            let mut roll = pa + distance.modifier_2020() + modifiers.iter().map(|m| m.get_modifier()).sum::<i32>();
            if let Some(sbm) = stat_based_roll_modifier {
                roll += sbm.get_modifier();
            }
            Some(roll.max(2))
        } else {
            None
        }
    }

    fn minimum_roll_simple(
        &self,
        thrower: &Player,
        distance: PassingDistance,
        modifiers: &[PassModifier],
    ) -> Option<i32> {
        self.minimum_roll(thrower, distance, modifiers, None)
    }

    fn evaluate_pass(
        &self,
        thrower: &Player,
        roll: i32,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        _bomb_action: bool,
        stat_based_roll_modifier: Option<&StatBasedRollModifier>,
    ) -> PassResult {
        let pa = thrower.passing_with_modifiers();
        if pa <= 0 {
            return PassResult::FUMBLE;
        }
        let mut result_after_modifiers = roll
            - self.calculate_modifiers(modifiers)
            - distance.modifier_2020();
        if let Some(sbm) = stat_based_roll_modifier {
            result_after_modifiers += sbm.get_modifier();
        }
        if roll == 1 {
            return if thrower.has_skill_property(NamedProperties::DONT_DROP_FUMBLES) {
                PassResult::SAVED_FUMBLE
            } else {
                PassResult::FUMBLE
            };
        } else if roll == 6 || result_after_modifiers >= pa {
            PassResult::ACCURATE
        } else if result_after_modifiers <= 1 {
            PassResult::FUMBLE
        } else {
            PassResult::INACCURATE
        }
    }

    fn evaluate_pass_simple(
        &self,
        thrower: &Player,
        roll: i32,
        distance: PassingDistance,
        modifiers: &[PassModifier],
        bomb_action: bool,
    ) -> PassResult {
        self.evaluate_pass(thrower, roll, distance, modifiers, bomb_action, None)
    }

    fn format_report_roll(&self, roll: i32, thrower: &Player) -> String {
        if thrower.passing_with_modifiers() > 0 {
            format!("Pass Roll [ {} ]", roll)
        } else {
            format!("Pass fumbled automatically as {} has no Passing Ability", thrower.name)
        }
    }

    fn format_roll_requirement(&self, distance: PassingDistance, formatted_modifiers: &str, thrower: &Player) -> String {
        if thrower.passing_with_modifiers() <= 0 {
            return String::new();
        }
        format!(
            " (Roll - {} {} {} >= PA {}+).",
            distance.modifier_2020(),
            distance.name(),
            formatted_modifiers,
            thrower.passing_with_modifiers()
        )
    }

    fn eligible_to_re_roll(&self, re_rolled_action_name: &str, thrower: &Player) -> bool {
        re_rolled_action_name != "pass" && thrower.passing_with_modifiers() > 0
    }

    fn pass_modifiers(&self, game: &Game, player: &Player) -> i32 {
        let players = UtilPlayer::find_tacklezone_players(game, &player.id);
        let mut zones = players.len() as i32;
        let ap = &game.acting_player;
        if game.turn_mode == TurnMode::DumpOff {
            if let Some(ap_id) = ap.player_id.as_deref() {
                if ap.standing_up && players.iter().any(|id| id.as_str() == ap_id) {
                    zones -= 1;
                }
            }
        }
        zones
    }

    /// Java bb2025 `PassMechanic.findPassingDistance`: a non-TTM, non-bomb pass from the
    /// Working in Tandem PASS twin to the BLOCK twin is `PASS_TO_PARTNER` (no range modifier).
    fn find_passing_distance(
        &self, game: &Game, from_coordinate: Option<ffb_model::types::FieldCoordinate>,
        to_coordinate: Option<ffb_model::types::FieldCoordinate>, throw_team_mate: bool,
    ) -> Option<PassingDistance> {
        let distance = crate::pass_mechanic::base_find_passing_distance(
            self, game, from_coordinate, to_coordinate, throw_team_mate);
        let is_bomb = game.acting_player.player_action.map(|a| a.is_bomb()).unwrap_or(false);
        if distance.is_some() && !throw_team_mate && !is_bomb {
            let at = |c: Option<ffb_model::types::FieldCoordinate>| c
                .and_then(|c| game.field_model.player_at(c).cloned())
                .and_then(|id| game.player(&id).cloned());
            if let (Some(thrower), target) = (at(from_coordinate), at(to_coordinate)) {
                if is_passing_to_partner(&thrower, target.as_ref()) {
                    return Some(PassingDistance::PassToPartner);
                }
            }
        }
        distance
    }
}

/// Java `UtilPlayer.isPassingToPartner(thrower, target)`.
fn is_passing_to_partner(thrower: &Player, target: Option<&Player>) -> bool {
    let Some(target) = target else { return false };
    let Some(tandem) = thrower.skill_id_with_property(NamedProperties::CAN_PASS_TO_PARTNER_WITH_NO_MODIFIERS)
        else { return false };
    thrower.skill_value_excluding_temporary_ones(tandem).is_some_and(|v| v.eq_ignore_ascii_case("pass"))
        && target.skill_value_excluding_temporary_ones(tandem).is_some_and(|v| v.eq_ignore_ascii_case("block"))
}

impl PassMechanic {
    pub fn new() -> Self { PassMechanic }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerGender, PlayerType, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;

    fn make_thrower() -> Player {
        Player {
            id: "t".into(), name: "t".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 5, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    /// Java bb2025 `findPassingDistance` + `UtilPlayer.isPassingToPartner`: Valen (Working in
    /// Tandem "pass") throwing to Lucien ("block") is PASS_TO_PARTNER whatever the range; the
    /// reverse direction, or a bomb, keeps the ruler's distance.
    #[test]
    fn pass_twin_to_block_twin_is_pass_to_partner() {
        use ffb_model::enums::{PlayerAction, Rules};
        use ffb_model::types::FieldCoordinate;
        let tandem = |id: &str, v: &str| {
            let mut p = make_thrower();
            p.id = id.into();
            p.starting_skills = vec![SkillWithValue { skill_id: SkillId::WorkingInTandem, value: Some(v.into()) }];
            p
        };
        let bare = |id: &str| ffb_model::model::Team {
            id: id.into(), name: id.into(), race: "elf".into(), roster_id: "elf".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        };
        let mut game = Game::new(bare("home"), bare("away"), Rules::Bb2025);
        game.team_home.players.push(tandem("valen", "pass"));
        game.team_home.players.push(tandem("lucien", "block"));
        let (v, l) = (FieldCoordinate::new(5, 5), FieldCoordinate::new(15, 5));
        game.field_model.set_player_coordinate("valen", v);
        game.field_model.set_player_coordinate("lucien", l);
        game.acting_player.player_action = Some(PlayerAction::Pass);
        let m = PassMechanic::new();
        assert_eq!(m.find_passing_distance(&game, Some(v), Some(l), false), Some(PassingDistance::PassToPartner));
        assert_ne!(m.find_passing_distance(&game, Some(l), Some(v), false), Some(PassingDistance::PassToPartner),
            "only the pass twin throwing to the block twin");
        game.acting_player.player_action = Some(PlayerAction::ThrowBomb);
        assert_ne!(m.find_passing_distance(&game, Some(v), Some(l), false), Some(PassingDistance::PassToPartner));
    }

    #[test]
    fn roll_one_without_safe_pass_is_fumble() {
        let m = PassMechanic::new();
        let thrower = make_thrower();
        let result = m.evaluate_pass(&thrower, 1, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::FUMBLE);
    }

    #[test]
    fn roll_one_with_safe_pass_is_saved_fumble() {
        let m = PassMechanic::new();
        let mut thrower = make_thrower();
        thrower.starting_skills.push(SkillWithValue::new(SkillId::SafePass));
        let result = m.evaluate_pass(&thrower, 1, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::SAVED_FUMBLE);
    }

    #[test]
    fn no_passing_ability_is_always_fumble_regardless_of_safe_pass() {
        let m = PassMechanic::new();
        let mut thrower = make_thrower();
        thrower.passing = -1;
        thrower.starting_skills.push(SkillWithValue::new(SkillId::SafePass));
        // BB2025: pa<=0 returns FUMBLE unconditionally (before SafePass check)
        let result = m.evaluate_pass(&thrower, 3, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::FUMBLE);
    }

    #[test]
    fn normal_roll_above_pa_is_accurate() {
        let m = PassMechanic::new();
        let thrower = make_thrower();
        // pa=4, ShortPass modifier_2020=1; roll=5: result_after_modifiers=5-0-1=4 >= pa=4 → ACCURATE
        let result = m.evaluate_pass(&thrower, 5, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::ACCURATE);
    }

    #[test]
    fn roll_below_pa_is_inaccurate() {
        let m = PassMechanic::new();
        let thrower = make_thrower(); // passing=4
        // roll=3 < pa=4 → INACCURATE
        let result = m.evaluate_pass(&thrower, 3, PassingDistance::ShortPass, &[], false, None);
        assert_eq!(result, PassResult::INACCURATE);
    }
}
