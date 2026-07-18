/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBitten.
/// Armor always broken. Injury roll with niggling modifier, edition-aware Stunty/Thick-Skull
/// interpretation — but unlike every other injury type, Java's Bitten *never* rolls a casualty:
/// `injuryContext.setInjury(interpretInjury(...)); if (injuryContext.getPlayerState() == null) {
/// injuryContext.setInjury(new PlayerState(BADLY_HURT)); }` — `interpretInjury` returns `null`
/// for any Casualty-range total, and Bitten caps that at BADLY_HURT directly instead of rolling
/// the casualty dice (`setInjury`'s normal fallback). No turnover.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE, PS_STUNNED, PS_KNOCKED_OUT, PS_BADLY_HURT, Rules, SkillId};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::mechanics::{interpret_injury_total_bb2016, interpret_injury_total_bb2020, InjuryOutcome};
use ffb_mechanics::modifiers::niggling_injury_modifier;
use crate::injury::{InjuryContext, InjuryTypeServer};

pub struct InjuryTypeBitten { ctx: InjuryContext }
impl InjuryTypeBitten { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeBitten { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeBitten {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        let defender = game.player(defender_id);
        if let Some(defender) = defender {
            if let Some(m) = niggling_injury_modifier(defender.niggling_injuries) {
                self.ctx.add_injury_modifier(m);
            }
        }

        let d1 = rng.d6();
        let d2 = rng.d6();
        self.ctx.injury_roll = Some([d1, d2]);
        let modifier_sum: i32 = self.ctx.injury_modifiers.iter().map(|m| m.value).sum();
        let total = d1 + d2 + modifier_sum;

        let outcome = match defender {
            Some(defender) => {
                let is_stunty = defender.has_skill(SkillId::Stunty);
                let has_thick_skull = defender.has_skill(SkillId::ThickSkull);
                match game.rules {
                    Rules::Bb2016 => interpret_injury_total_bb2016(total, is_stunty, has_thick_skull),
                    _ => interpret_injury_total_bb2020(total, is_stunty, has_thick_skull),
                }
            }
            None => None,
        };
        // Java: `interpretInjury` returning null (Casualty-range total) is capped at BADLY_HURT
        // here instead of falling through to `setInjury`'s casualty-roll fallback.
        self.ctx.injury = Some(match outcome {
            Some(InjuryOutcome::Stunned) => PlayerState::new(PS_STUNNED),
            Some(InjuryOutcome::KnockedOut) => PlayerState::new(PS_KNOCKED_OUT),
            Some(InjuryOutcome::BadlyHurt) | Some(InjuryOutcome::Casualty) | None => PlayerState::new(PS_BADLY_HURT),
        });
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    /// Java: `new Bitten()` constructor passes `SendToBoxReason.BITTEN`.
    fn send_to_box_reason(&self) -> Option<ffb_model::enums::SendToBoxReason> {
        Some(ffb_model::enums::SendToBoxReason::Bitten)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_always_broken() {
        let mut t = InjuryTypeBitten::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn injury_is_set_and_not_prone() {
        let mut t = InjuryTypeBitten::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury.is_some());
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn does_not_cause_turnover() { assert!(!InjuryTypeBitten::new().falling_down_causes_turnover()); }
    #[test]
    fn send_to_box_reason_is_bitten() {
        use ffb_model::enums::SendToBoxReason;
        assert_eq!(InjuryTypeBitten::new().send_to_box_reason(), Some(SendToBoxReason::Bitten));
    }
    #[test]
    fn defender_id_stored_in_context() {
        let mut t = InjuryTypeBitten::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, Some("att"), "def", coord(), None, None, ApothecaryMode::Attacker);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("def"));
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("att"));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeBitten::new();
        let t2 = InjuryTypeBitten::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    fn game_with_niggling_defender(niggling_injuries: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
    ..Default::default() });
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn niggling_injured_defender_gets_niggling_injury_modifier() {
        // Java: InjuryTypeBitten.handleInjury calls only
        // factory.getNigglingInjuryModifier(pDefender), not the full findInjuryModifiers.
        let mut t = InjuryTypeBitten::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_niggling_defender(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "2 Niggling Injuries"));
    }

    #[test]
    fn casualty_range_total_caps_at_badly_hurt_without_casualty_roll() {
        // Java: `injuryContext.setInjury(interpretInjury(...)); if (injuryContext.getPlayerState()
        // == null) { injuryContext.setInjury(new PlayerState(BADLY_HURT)); }` — Bitten never rolls
        // casualty dice; any Casualty-range total (interpretInjury returns null) is capped
        // directly at BADLY_HURT instead.
        let mut t = InjuryTypeBitten::new();
        let mut rng = GameRng::new(1); // first d6 pair = (2, 5) -> base total 7
        let game = game_with_niggling_defender(3); // +3 niggling modifier -> total 10 (Casualty range)
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_BADLY_HURT));
        assert!(t.ctx.casualty_roll.is_none(), "Bitten must never roll casualty dice");
    }
    #[test]
    fn non_niggling_defender_gets_no_niggling_injury_modifier() {
        let mut t = InjuryTypeBitten::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_niggling_defender(0), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }
}
