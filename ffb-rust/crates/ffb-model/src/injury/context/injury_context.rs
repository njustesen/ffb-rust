use crate::injury::context::modified_injury_context::ModifiedInjuryContext;

/// 1:1 translation of `com.fumbbl.ffb.injury.context.InjuryContext`.
///
/// Holds all state for a single injury resolution: dice rolls, modifiers, results,
/// apothecary status, and an optional alternate context (ModifiedInjuryContext).
#[derive(Debug, Default)]
pub struct InjuryContext {
    /// Java: fArmorRoll
    pub armor_roll: Option<[i32; 2]>,
    /// Java: fArmorBroken
    pub armor_broken: bool,
    /// Java: fInjuryRoll
    pub injury_roll: Option<[i32; 2]>,
    /// Java: fCasualtyRoll
    pub casualty_roll: Option<[i32; 2]>,
    /// Java: fCasualtyRollDecay
    pub casualty_roll_decay: Option<[i32; 2]>,
    /// Java: fSendToBoxTurn
    pub send_to_box_turn: i32,
    /// Java: fSendToBoxHalf
    pub send_to_box_half: i32,
    /// Java: fDefenderId
    pub defender_id: Option<String>,
    /// Java: fAttackerId
    pub attacker_id: Option<String>,
    /// Java: modifiedInjuryContext — the alternate injury outcome (e.g. after apothecary)
    pub modified_injury_context: Option<Box<ModifiedInjuryContext>>,
}

impl InjuryContext {
    /// Java: `new InjuryContext()`
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `setDefenderId(String)`
    pub fn set_defender_id(&mut self, id: impl Into<String>) {
        self.defender_id = Some(id.into());
    }

    /// Java: `getDefenderId()`
    pub fn get_defender_id(&self) -> Option<&str> {
        self.defender_id.as_deref()
    }

    /// Java: `setAttackerId(String)`
    pub fn set_attacker_id(&mut self, id: impl Into<String>) {
        self.attacker_id = Some(id.into());
    }

    /// Java: `getAttackerId()`
    pub fn get_attacker_id(&self) -> Option<&str> {
        self.attacker_id.as_deref()
    }

    /// Java: `setArmorRoll(int[])`
    pub fn set_armor_roll(&mut self, roll: [i32; 2]) {
        self.armor_roll = Some(roll);
    }

    /// Java: `getArmorRoll()`
    pub fn get_armor_roll(&self) -> Option<[i32; 2]> {
        self.armor_roll
    }

    /// Java: `setArmorBroken(boolean)`
    pub fn set_armor_broken(&mut self, broken: bool) {
        self.armor_broken = broken;
    }

    /// Java: `isArmorBroken()`
    pub fn is_armor_broken(&self) -> bool {
        self.armor_broken
    }

    /// Java: `setInjuryRoll(int[])`
    pub fn set_injury_roll(&mut self, roll: [i32; 2]) {
        self.injury_roll = Some(roll);
    }

    /// Java: `getInjuryRoll()`
    pub fn get_injury_roll(&self) -> Option<[i32; 2]> {
        self.injury_roll
    }

    /// Java: `setCasualtyRoll(int[])`
    pub fn set_casualty_roll(&mut self, roll: [i32; 2]) {
        self.casualty_roll = Some(roll);
    }

    /// Java: `getCasualtyRoll()`
    pub fn get_casualty_roll(&self) -> Option<[i32; 2]> {
        self.casualty_roll
    }

    /// Java: `setSendToBoxTurn(int)`
    pub fn set_send_to_box_turn(&mut self, turn: i32) {
        self.send_to_box_turn = turn;
    }

    /// Java: `getSendToBoxTurn()`
    pub fn get_send_to_box_turn(&self) -> i32 {
        self.send_to_box_turn
    }

    /// Java: `setSendToBoxHalf(int)`
    pub fn set_send_to_box_half(&mut self, half: i32) {
        self.send_to_box_half = half;
    }

    /// Java: `getSendToBoxHalf()`
    pub fn get_send_to_box_half(&self) -> i32 {
        self.send_to_box_half
    }

    /// Java: `getModifiedInjuryContext()`
    pub fn get_modified_injury_context(&self) -> Option<&ModifiedInjuryContext> {
        self.modified_injury_context.as_deref()
    }

    /// Java: `setModifiedInjuryContext(ModifiedInjuryContext)`
    pub fn set_modified_injury_context(&mut self, ctx: ModifiedInjuryContext) {
        self.modified_injury_context = Some(Box::new(ctx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_rolls() {
        let ctx = InjuryContext::new();
        assert!(ctx.get_armor_roll().is_none());
        assert!(ctx.get_injury_roll().is_none());
        assert!(!ctx.is_armor_broken());
    }

    #[test]
    fn set_armor_roll_and_retrieve() {
        let mut ctx = InjuryContext::new();
        ctx.set_armor_roll([3, 4]);
        assert_eq!(ctx.get_armor_roll(), Some([3, 4]));
    }

    #[test]
    fn set_defender_id() {
        let mut ctx = InjuryContext::new();
        ctx.set_defender_id("player-1");
        assert_eq!(ctx.get_defender_id(), Some("player-1"));
    }

    #[test]
    fn armor_broken_flag() {
        let mut ctx = InjuryContext::new();
        assert!(!ctx.is_armor_broken());
        ctx.set_armor_broken(true);
        assert!(ctx.is_armor_broken());
    }

    #[test]
    fn send_to_box_fields() {
        let mut ctx = InjuryContext::new();
        ctx.set_send_to_box_turn(3);
        ctx.set_send_to_box_half(2);
        assert_eq!(ctx.get_send_to_box_turn(), 3);
        assert_eq!(ctx.get_send_to_box_half(), 2);
    }
}
