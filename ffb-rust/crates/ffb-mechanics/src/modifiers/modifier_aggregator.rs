use ffb_model::model::{Game, Player};
use crate::modifiers::catch_modifier::CatchModifier;
use crate::modifiers::go_for_it_context::GoForItContext;
use crate::modifiers::interception_modifier::InterceptionModifier;
use crate::modifiers::pass_context::PassContext;
use crate::modifiers::pass_modifier::PassModifier;
use crate::modifiers::dodge_modifier::DodgeModifier;
use crate::modifiers::pickup_modifier::PickupModifier;
use crate::modifiers::jump_modifier::JumpModifier;
use crate::modifiers::jump_up_modifier::JumpUpModifier;
use crate::modifiers::gaze_modifier::GazeModifier;
use crate::modifiers::go_for_it_modifier::GoForItModifier;
use crate::modifiers::right_stuff_modifier::RightStuffModifier;
use crate::modifiers::armor_modifier::ArmorModifier;
use crate::modifiers::injury_modifier::InjuryModifier;

/// 1:1 translation of com.fumbbl.ffb.modifiers.ModifierAggregator.
///
/// Java's version is initialized once per `Game` and stores both a `SkillFactory` and a `Game`
/// reference so its no-arg getters can look both up internally. Rust has no `SkillFactory`
/// translation yet (a separate, undiscovered gap — not invented here), and holding a `&Game`
/// reference on this struct would tie its lifetime to a single game instance for no benefit, so
/// each getter takes the game/player/context it needs as a parameter instead.
///
/// Only the card half of the merge is implemented here (`UtilCards::find_all_cards` + the
/// per-card dispatch in `card_roll_modifiers`) — confirmed by reading every BB2016 card that
/// only 4 cards override `rollModifiers()`, all producing Interception/Pass/GoForIt modifiers.
/// Zero cards produce Catch/Dodge/Pickup/Jump/JumpUp/Gaze/RightStuff/Armour/Injury modifiers, so
/// those getters correctly stay empty. The skill half remains a documented gap.
pub struct ModifierAggregator {
    // TODO: skill_factory: SkillFactory (not yet translated)
}

impl ModifierAggregator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init(&mut self, _game: &Game) {
        // TODO: initialize skill_factory via game.get_factory(FactoryType::SKILL) once translated
    }

    pub fn get_catch_modifiers(&self) -> Vec<CatchModifier> {
        // No BB2016 card overrides CatchModifier rollModifiers(); TODO: skill_factory half.
        Vec::new()
    }

    /// Java: `ModifierAggregator.getInterceptionModifiers()`.
    pub fn get_interception_modifiers(&self, game: &Game, interceptor: &Player) -> Vec<InterceptionModifier> {
        crate::modifiers::card_roll_modifiers::find_interception_card_modifiers(game, interceptor)
    }

    /// Java: `ModifierAggregator.getPassModifiers()`.
    pub fn get_pass_modifiers(&self, context: &PassContext<'_>) -> Vec<PassModifier> {
        crate::modifiers::card_roll_modifiers::find_pass_card_modifiers(context)
    }

    pub fn get_dodge_modifiers(&self) -> Vec<DodgeModifier> {
        // No BB2016 card overrides DodgeModifier rollModifiers(); TODO: skill_factory half.
        Vec::new()
    }

    pub fn get_pickup_modifiers(&self) -> Vec<PickupModifier> {
        // No BB2016 card overrides PickupModifier rollModifiers(); TODO: skill_factory half.
        Vec::new()
    }

    pub fn get_jump_modifiers(&self) -> Vec<JumpModifier> {
        // No BB2016 card overrides JumpModifier rollModifiers(); TODO: skill_factory half.
        Vec::new()
    }

    pub fn get_jump_up_modifiers(&self) -> Vec<JumpUpModifier> {
        // No BB2016 card overrides JumpUpModifier rollModifiers(); TODO: skill_factory half.
        Vec::new()
    }

    pub fn get_gaze_modifiers(&self) -> Vec<GazeModifier> {
        // No BB2016 card overrides GazeModifier rollModifiers(); TODO: skill_factory half.
        Vec::new()
    }

    /// Java: `ModifierAggregator.getGoForItModifiers()`.
    pub fn get_go_for_it_modifiers(&self, context: &GoForItContext<'_>) -> Vec<GoForItModifier> {
        crate::modifiers::card_roll_modifiers::find_go_for_it_card_modifiers(context)
    }

    pub fn get_right_stuff_modifiers(&self) -> Vec<RightStuffModifier> {
        // No BB2016 card overrides RightStuffModifier rollModifiers(); TODO: skill_factory half.
        Vec::new()
    }

    pub fn get_armour_modifiers(&self) -> Vec<Box<dyn ArmorModifier>> {
        // No BB2016 card overrides armourModifiers(); TODO: skill_factory half.
        Vec::new()
    }

    pub fn get_injury_modifiers(&self) -> Vec<Box<dyn InjuryModifier>> {
        // No BB2016 card overrides injuryModifiers(); TODO: skill_factory half.
        Vec::new()
    }
}

impl Default for ModifierAggregator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PassingDistance, Rules};
    use ffb_model::model::Team;

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
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016)
    }

    #[test]
    fn get_catch_modifiers_empty_no_cards() {
        let agg = ModifierAggregator::new();
        assert!(agg.get_catch_modifiers().is_empty());
    }

    #[test]
    fn get_interception_modifiers_empty_when_no_cards_active() {
        let game = make_game();
        let interceptor = Player::default();
        let agg = ModifierAggregator::new();
        assert!(agg.get_interception_modifiers(&game, &interceptor).is_empty());
    }

    #[test]
    fn get_interception_modifiers_finds_magic_gloves() {
        let mut game = make_game();
        let interceptor = Player { id: "i1".into(), ..Default::default() };
        game.field_model.add_card("i1", ffb_model::inducement::card::Card::new("Magic Gloves of Jark Longarm", None::<&str>));
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Magic Gloves of Jark Longarm", None::<&str>));

        let agg = ModifierAggregator::new();
        let mods = agg.get_interception_modifiers(&game, &interceptor);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), -1);
    }

    #[test]
    fn get_pass_modifiers_finds_gromskull() {
        let mut game = make_game();
        let passer = Player { id: "p1".into(), ..Default::default() };
        game.field_model.add_card("p1", ffb_model::inducement::card::Card::new("Gromskull's Exploding Runes", None::<&str>));
        game.turn_data_home.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Gromskull's Exploding Runes", None::<&str>));

        let agg = ModifierAggregator::new();
        let ctx = PassContext::new(&game, &passer, PassingDistance::ShortPass, false);
        let mods = agg.get_pass_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), 1);
    }

    #[test]
    fn get_go_for_it_modifiers_finds_greased_shoes() {
        let mut game = make_game();
        game.home_playing = true;
        game.turn_data_away.inducement_set.activate_card_full(
            ffb_model::inducement::card::Card::new("Greased Shoes", None::<&str>));

        let player = Player::default();
        let agg = ModifierAggregator::new();
        let ctx = GoForItContext::new(&game, &player);
        let mods = agg.get_go_for_it_modifiers(&ctx);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].get_modifier(), 3);
    }

    #[test]
    fn get_armour_and_injury_modifiers_stay_empty() {
        let agg = ModifierAggregator::new();
        assert!(agg.get_armour_modifiers().is_empty());
        assert!(agg.get_injury_modifiers().is_empty());
    }
}
