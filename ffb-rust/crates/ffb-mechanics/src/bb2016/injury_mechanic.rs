use ffb_model::enums::{PlayerState, PlayerType, Rules, SendToBoxReason};
use ffb_model::model::{Game, Player, RosterPosition, Team, TeamResult};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::raise_type::RaiseType;
use crate::mechanic::{Mechanic, MechanicType};
use crate::injury_mechanic::InjuryMechanic as InjuryMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.InjuryMechanic.
pub struct InjuryMechanic;

impl Default for InjuryMechanic {
    fn default() -> Self { InjuryMechanic }
}

impl Mechanic for InjuryMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::INJURY }
}

impl InjuryMechanicTrait for InjuryMechanic {
    fn raised_by_nurgle_reason(&self) -> SendToBoxReason {
        SendToBoxReason::NurglesRot
    }

    fn raised_by_nurgle_message(&self) -> String {
        " has been infected with Nurgle's Rot and will join team ".into()
    }

    fn can_raise_infected_players(
        &self,
        _team: &Team,
        _team_result: &TeamResult,
        attacker: Option<&Player>,
        dead_player: &Player,
    ) -> bool {
        attacker.is_some()
            && attacker.unwrap().has_skill_property(NamedProperties::ALLOWS_RAISING_LINEMAN)
            && dead_player.strength <= 4
            && !dead_player.has_skill_property_in(ffb_model::enums::Rules::Bb2016, NamedProperties::PREVENT_RAISE_FROM_DEAD)
            && !dead_player.has_skill_property(NamedProperties::REQUIRES_SECOND_CASUALTY_ROLL)
    }

    fn infected_goes_to_reserves(&self) -> bool { false }

    fn can_raise_dead(
        &self,
        team: &Team,
        team_result: &TeamResult,
        dead_player: &Player,
    ) -> bool {
        // Java: (roster.hasNecromancer() || roster.hasVampireLord())
        // && raisedDead == 0 && strength <= 4 && !preventRaiseFromDead
        //
        // Both reads here are edition-sensitive and BOTH were wrong:
        //
        // 1. `strength`, not `strength_with_modifiers()` — Java calls `deadPlayer.getStrength()`,
        //    the base stat.
        // 2. `has_skill_property_in(Bb2016, ..)`, not `has_skill_property(..)`. The edition-less
        //    accessor follows the enum's "latest edition wins" convention, and BB2025's
        //    Regeneration deliberately drops `preventRaiseFromDead` (see the note on
        //    `SkillId::Regeneration`) — so every bb2016/bb2020 Regeneration player silently lost
        //    the property and became raisable. Java's bb2016 Regeneration registers it.
        //
        // necromantic bb2016 seed 47: a Werewolf (Claw/Frenzy/REGENERATION) is killed, Java
        // declines to raise it, Rust raised a Zombie, fielded it at the next setup, and offered
        // the agent 18 activation candidates against Java's 17 — the extra draw split the game.
        // Only a necromantic/vampire opponent can reach this, so it needed the roster to exist.
        (team.necromancer || team.vampire_lord)
            && team_result.raised_dead == 0
            && dead_player.strength <= 4
            && !dead_player.has_skill_property_in(ffb_model::enums::Rules::Bb2016, NamedProperties::PREVENT_RAISE_FROM_DEAD)
    }

    fn raised_nurgle_type(&self) -> PlayerType { PlayerType::RaisedFromDead }

    fn can_use_apo(&self, game: &Game, defender: &Player, _player_state: PlayerState) -> bool {
        defender.player_type != PlayerType::Star
            && ((game.team_home.has_player(&defender.id) && game.turn_data_home.apothecaries > 0)
                || (game.team_away.has_player(&defender.id) && game.turn_data_away.apothecaries > 0))
    }

    /// Java: `Collections.emptyList()` -- BB2016/BB2020 raise from the roster's single
    /// `raisedPositionId` instead, via `UtilServerInjury.handleRaiseDead`.
    fn raise_positions(&self, _team: &Team, _rules: Rules) -> Vec<RosterPosition> { vec![] }

    fn raise_type(&self, team: &Team) -> RaiseType {
        // Java: if has_necromancer → ZOMBIE; if has_vampire_lord → THRALL; else ROTTER
        if team.necromancer {
            return RaiseType::ZOMBIE;
        }
        if team.vampire_lord {
            return RaiseType::THRALL;
        }
        RaiseType::ROTTER
    }
}

impl InjuryMechanic {
    pub fn new() -> Self { InjuryMechanic }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::SendToBoxReason;
    use crate::injury_mechanic::InjuryMechanic as InjuryTrait;

    fn empty_team(necromancer: bool, vampire_lord: bool) -> Team {
        Team {
            id: "t".into(), name: "T".into(), race: "human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord, necromancer,
        }
    }

    fn empty_result() -> TeamResult { TeamResult::default() }
    fn bare_player() -> Player { Player::default() }

    #[test]
    fn raised_by_nurgle_reason_is_nurgles_rot() {
        assert_eq!(InjuryMechanic.raised_by_nurgle_reason(), SendToBoxReason::NurglesRot);
    }

    #[test]
    fn infected_goes_to_reserves_is_false() {
        assert!(!InjuryMechanic.infected_goes_to_reserves());
    }

    #[test]
    fn raised_nurgle_type_is_raised_from_dead() {
        assert_eq!(InjuryMechanic.raised_nurgle_type(), PlayerType::RaisedFromDead);
    }

    #[test]
    fn can_raise_dead_false_without_necromancer_or_vampire_lord() {
        let team = empty_team(false, false);
        let dead = bare_player();
        assert!(!InjuryMechanic.can_raise_dead(&team, &empty_result(), &dead));
    }

    #[test]
    fn can_raise_dead_true_with_necromancer() {
        let team = empty_team(true, false);
        let dead = bare_player();
        assert!(InjuryMechanic.can_raise_dead(&team, &empty_result(), &dead));
    }

    #[test]
    fn can_raise_dead_true_with_vampire_lord() {
        let team = empty_team(false, true);
        let dead = bare_player();
        assert!(InjuryMechanic.can_raise_dead(&team, &empty_result(), &dead));
    }

    #[test]
    fn can_raise_dead_false_when_raised_dead_nonzero() {
        let team = empty_team(true, false);
        let mut tr = empty_result();
        tr.raised_dead = 1;
        let dead = bare_player();
        assert!(!InjuryMechanic.can_raise_dead(&team, &tr, &dead));
    }

    #[test]
    fn raise_type_rotter_without_special_roster() {
        let team = empty_team(false, false);
        assert_eq!(InjuryMechanic.raise_type(&team), RaiseType::ROTTER);
    }

    #[test]
    fn raise_type_zombie_for_necromancer() {
        let team = empty_team(true, false);
        assert_eq!(InjuryMechanic.raise_type(&team), RaiseType::ZOMBIE);
    }

    #[test]
    fn raise_type_thrall_for_vampire_lord() {
        let team = empty_team(false, true);
        assert_eq!(InjuryMechanic.raise_type(&team), RaiseType::THRALL);
    }

    #[test]
    fn raise_type_zombie_when_both_necromancer_and_vampire_lord() {
        // necromancer takes priority over vampire_lord
        let team = empty_team(true, true);
        assert_eq!(InjuryMechanic.raise_type(&team), RaiseType::ZOMBIE);
    }
}

#[cfg(test)]
mod raise_edition_tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::skill_def::SkillWithValue;

    fn necro_team() -> Team {
        Team {
            id: "t".into(), name: "T".into(), race: "necromantic".into(),
            roster_id: "necromantic.lrb6".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false, necromancer: true,
        }
    }

    fn regen_player(st: i32) -> Player {
        Player {
            id: "dead".into(), name: "dead".into(), nr: 1, position_id: "pos".into(),
            movement: 6, strength: st, agility: 3, passing: 3, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: SkillId::Regeneration, value: None }],
            ..Default::default()
        }
    }

    /// bb2016/bb2020 Regeneration registers `preventRaiseFromDead`; BB2025's deliberately does
    /// not, and the edition-LESS property accessor returns the BB2025 flavour. Reading it here
    /// let every bb2016 Regeneration player be raised (necromantic bb2016 seed 47: a dead
    /// Werewolf came back as a Zombie and added an activation candidate Java never had).
    #[test]
    fn bb2016_regeneration_still_prevents_being_raised() {
        assert!(!crate::bb2016::injury_mechanic::InjuryMechanic
            .can_raise_dead(&necro_team(), &TeamResult::default(), &regen_player(3)),
            "a bb2016 Regeneration player must not be raisable");
        // The edition-less accessor is what made this look raisable — pin the difference.
        assert!(!regen_player(3).has_skill_property(
            ffb_model::model::property::named_properties::NamedProperties::PREVENT_RAISE_FROM_DEAD),
            "edition-less lookup returns the BB2025 flavour");
        assert!(regen_player(3).has_skill_property_in(Rules::Bb2016,
            ffb_model::model::property::named_properties::NamedProperties::PREVENT_RAISE_FROM_DEAD),
            "the bb2016 lookup keeps the property");
    }

    /// Java gates on `deadPlayer.getStrength()`, the BASE stat, not the modified one.
    #[test]
    fn the_strength_gate_reads_the_base_stat() {
        let mut p = regen_player(5);
        p.starting_skills.clear();
        assert!(!crate::bb2016::injury_mechanic::InjuryMechanic
            .can_raise_dead(&necro_team(), &TeamResult::default(), &p),
            "base ST 5 is above the raise threshold");
        p.strength = 4;
        assert!(crate::bb2016::injury_mechanic::InjuryMechanic
            .can_raise_dead(&necro_team(), &TeamResult::default(), &p),
            "base ST 4 is raisable");
    }
}
