use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.ArmorModifierContext.
pub struct ArmorModifierContext<'a> {
    pub game: &'a Game,
    pub attacker: Option<&'a Player>,
    pub defender: &'a Player,
    pub is_stab: bool,
    pub is_foul: bool,
    pub foul_assists: i32,
    pub is_ttm: bool,
}

impl<'a> ArmorModifierContext<'a> {
    pub fn new(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, foul_assists: 0, is_ttm: false }
    }

    pub fn new_with_foul_assists(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool, foul_assists: i32) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, foul_assists, is_ttm: false }
    }

    pub fn new_full(game: &'a Game, attacker: Option<&'a Player>, defender: &'a Player, is_stab: bool, is_foul: bool, foul_assists: i32, is_ttm: bool) -> Self {
        Self { game, attacker, defender, is_stab, is_foul, foul_assists, is_ttm }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_attacker(&self) -> Option<&Player> { self.attacker }
    pub fn get_defender(&self) -> &Player { self.defender }
    pub fn is_stab(&self) -> bool { self.is_stab }
    pub fn is_foul(&self) -> bool { self.is_foul }
    pub fn get_foul_assists(&self) -> i32 { self.foul_assists }
    pub fn is_ttm(&self) -> bool { self.is_ttm }
}
