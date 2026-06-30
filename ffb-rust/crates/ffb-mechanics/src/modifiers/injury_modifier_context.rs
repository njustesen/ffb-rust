use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.InjuryModifierContext.
pub struct InjuryModifierContext<'a> {
    pub game: &'a Game,
    pub attacker: Option<&'a Player>,
    pub defender: &'a Player,
    pub is_stab: bool,
    pub is_foul: bool,
    pub is_vomit_like: bool,
    pub is_chainsaw: bool,
    pub is_ttm: bool,
    attacker_mode: bool,
}

impl<'a> InjuryModifierContext<'a> {
    pub fn new(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool, is_vomit_like: bool, is_chainsaw: bool) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, is_vomit_like, is_chainsaw, is_ttm: false, attacker_mode: true }
    }

    pub fn new_with_ttm(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool, is_vomit_like: bool, is_chainsaw: bool, is_ttm: bool) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, is_vomit_like, is_chainsaw, is_ttm, attacker_mode: true }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_attacker(&self) -> Option<&Player> { self.attacker }
    pub fn get_defender(&self) -> &Player { self.defender }
    pub fn is_stab(&self) -> bool { self.is_stab }
    pub fn is_foul(&self) -> bool { self.is_foul }
    pub fn is_vomit_like(&self) -> bool { self.is_vomit_like }
    pub fn is_chainsaw(&self) -> bool { self.is_chainsaw }
    pub fn is_ttm(&self) -> bool { self.is_ttm }
    pub fn set_defender_mode(&mut self) { self.attacker_mode = false; }
    pub fn is_attacker_mode(&self) -> bool { self.attacker_mode }
    pub fn is_defender_mode(&self) -> bool { !self.attacker_mode }
}
