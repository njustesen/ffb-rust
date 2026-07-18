/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeTrapDoorFall.
use ffb_model::enums::{ApothecaryMode, SendToBoxReason, PS_RESERVE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer};
use crate::injury::injuryType::injury_type_crowd::crowd_handle_injury;

pub struct InjuryTypeTrapDoorFall { ctx: InjuryContext }
impl InjuryTypeTrapDoorFall { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeTrapDoorFall { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeTrapDoorFall {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        crowd_handle_injury(&mut self.ctx, game, rng, attacker_id, defender_id, coord, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    /// Java: `TrapDoorFall.fallingDownCausesTurnover()` → false.
    fn falling_down_causes_turnover(&self) -> bool { false }
    /// Java: `TrapDoorFall` constructed with `super("trapdoorFall", false, SendToBoxReason.TRAP_DOOR_FALL)`.
    /// Was previously missing (defaulted to `None`).
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::TrapDoorFall) }
    /// Java class simple name — was previously missing, which broke
    /// `can_apo_ko_into_stun()`'s name-based lookup: with `injury_type_name` left `None`
    /// (`java_class_name()` defaulting to `""`), the apothecary was incorrectly allowed to
    /// revive a KO'd trap-door victim into stun (Java: `TrapDoorFall.canApoKoIntoStun()` → false).
    fn java_class_name(&self) -> &'static str { "InjuryTypeTrapDoorFall" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_always_broken() {
        let mut t = InjuryTypeTrapDoorFall::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn injury_is_reserve_or_ko_or_casualty() {
        for seed in 1..=10u64 {
            let mut t = InjuryTypeTrapDoorFall::new(); let mut rng = GameRng::new(seed);
            t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
            let b = t.ctx.injury.expect("injury set").base();
            assert!(b == PS_RESERVE || t.ctx.is_knocked_out() || t.ctx.is_casualty(), "seed={seed}");
        }
    }
    #[test]
    fn does_not_cause_turnover() { assert!(!InjuryTypeTrapDoorFall::new().falling_down_causes_turnover()); }
    #[test]
    fn send_to_box_reason_is_trap_door_fall() {
        assert_eq!(InjuryTypeTrapDoorFall::new().send_to_box_reason(), Some(SendToBoxReason::TrapDoorFall));
    }
    #[test]
    fn java_class_name_matches_can_apo_ko_into_stun_lookup() {
        // Regression test: `can_apo_ko_into_stun` keys off this exact string to disallow
        // reviving a KO'd trap-door victim into stun.
        let t = InjuryTypeTrapDoorFall::new();
        assert_eq!(t.java_class_name(), "InjuryTypeTrapDoorFall");
        assert!(!crate::injury::can_apo_ko_into_stun(Some(t.java_class_name())));
    }
    #[test]
    fn context_stores_defender_id() {
        let mut t = InjuryTypeTrapDoorFall::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "trap_victim", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("trap_victim"));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeTrapDoorFall::new();
        let t2 = InjuryTypeTrapDoorFall::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }
}
