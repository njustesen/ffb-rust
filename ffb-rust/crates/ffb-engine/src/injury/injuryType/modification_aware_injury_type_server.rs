/// Translation of com.fumbbl.ffb.server.injury.injuryType.ModificationAwareInjuryTypeServer (90 lines).
///
/// Java: abstract class, implements `handleInjury()` as a template method that calls
/// abstract `armourRoll()` + `injuryRoll()`. Protected `savedByArmour()` defaults to PRONE;
/// Chainsaw/Stab override to null (no injury on armor save), BlockStunned overrides to STUNNED.
///
/// Rust: `ModificationAwareInjuryType` supertrait with abstract `armour_roll()` /
/// `injury_roll()` methods. Free function `modification_aware_handle_injury()` implements
/// the template and each concrete type delegates its `handle_injury()` to it.
///
/// The InjuryContextModification path (alternate context for Claws/MB interactions) is not
/// yet ported — requires `Player::get_unused_injury_modification()` from the full modifier
/// factory stack.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE};
use ffb_model::model::player::Player;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier::InjuryModifier;
use ffb_mechanics::modifiers::Modifier;
use crate::injury::{InjuryContext, InjuryTypeServer};

/// Java: `InjuryModifier` instances are transient objects owned by their `Skill`; Rust's
/// `Modifier` (used by `InjuryContext`) requires a `&'static str` name for cheap `Copy`-like use
/// across the codebase. Bridging a `Box<dyn InjuryModifier>` (whose `get_name()` borrows from an
/// owned `String`) into a `Modifier` needs *some* `'static`-ifying step; leaking the name is the
/// chosen approach — same convention as `injury_type_block.rs`'s `leak_modifier` for
/// `ArmorModifierFactory` results, shared here since Block/Foul/Chainsaw all need it for
/// `InjuryModifierFactory` results.
pub fn leak_injury_modifier(m: &dyn InjuryModifier, attacker: Option<&Player>, defender: &Player, rules: ffb_model::enums::Rules) -> Modifier {
    let name: &'static str = Box::leak(m.get_name().to_owned().into_boxed_str());
    Modifier::new(name, m.get_modifier(attacker, defender), rules)
}

/// Abstract methods of ModificationAwareInjuryTypeServer.
/// Concrete types implement these; the free function calls them in the correct order.
pub trait ModificationAwareInjuryType: InjuryTypeServer {
    /// Java: abstract armourRoll(game, gameState, diceRoller, pAttacker, pDefender,
    ///                           diceInterpreter, injuryContext, roll)
    /// `roll=true` means actually roll new dice; `roll=false` means just re-evaluate
    /// existing dice with the current modifiers (alternate-context path).
    fn armour_roll(
        &mut self,
        game: &Game,
        rng: &mut GameRng,
        attacker_id: Option<&str>,
        defender_id: &str,
        roll: bool,
    );

    /// Java: abstract injuryRoll(game, gameState, diceRoller, pAttacker, pDefender, injuryContext)
    fn injury_roll(
        &mut self,
        game: &Game,
        rng: &mut GameRng,
        attacker_id: Option<&str>,
        defender_id: &str,
    );

    /// Java: protected void savedByArmour(InjuryContext). Default: set PRONE.
    /// Override for Chainsaw/Stab (no injury), BlockStunned (STUNNED).
    fn saved_by_armour(&mut self) {
        self.injury_context_mut().injury = Some(PlayerState::new(PS_PRONE));
    }
}

/// Template-method implementation of InjuryTypeServer::handle_injury() for all
/// ModificationAwareInjuryType implementors.
///
/// Java: ModificationAwareInjuryTypeServer.handleInjury() lines 32–62 (minus
/// the InjuryContextModification alternate-context path, lines 47–57).
pub fn modification_aware_handle_injury<T: ModificationAwareInjuryType>(
    this: &mut T,
    game: &Game,
    rng: &mut GameRng,
    attacker_id: Option<&str>,
    defender_id: &str,
    coord: FieldCoordinate,
    _from_coord: Option<FieldCoordinate>,
    _old_ctx: Option<&InjuryContext>,
    apo_mode: ApothecaryMode,
) {
    // Set shared context fields (Java: handled inside InjuryTypeServer before delegating)
    {
        let ctx = this.injury_context_mut();
        ctx.defender_id = Some(defender_id.to_owned());
        ctx.attacker_id = attacker_id.map(str::to_owned);
        ctx.defender_coordinate = Some(coord);
        ctx.apothecary_mode = apo_mode;
    }

    // Java line 45: armourRoll(game, gameState, diceRoller, pAttacker, pDefender,
    //                          diceInterpreter, injuryContext, true)
    this.armour_roll(game, rng, attacker_id, defender_id, true);

    // Java lines 67–79: private injury() helper
    if this.injury_context().armor_broken {
        // Java line 68: injuryRoll(...)
        this.injury_roll(game, rng, attacker_id, defender_id);
    } else {
        // Java line 77–79: savedByArmour(currentInjuryContext)
        this.saved_by_armour();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STUNNED};
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use crate::injury::InjuryContext;
    use crate::step::framework::test_team;

    struct AlwaysBrokenType { ctx: InjuryContext }

    impl InjuryTypeServer for AlwaysBrokenType {
        fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>,
                         defender_id: &str, coord: FieldCoordinate, from_coord: Option<FieldCoordinate>,
                         old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
            modification_aware_handle_injury(self, game, rng, attacker_id, defender_id,
                                             coord, from_coord, old_ctx, apo_mode);
        }
        fn injury_context(&self) -> &InjuryContext { &self.ctx }
        fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    }
    impl ModificationAwareInjuryType for AlwaysBrokenType {
        fn armour_roll(&mut self, _g: &Game, _r: &mut GameRng, _a: Option<&str>, _d: &str, _roll: bool) {
            self.ctx.armor_broken = true;
        }
        fn injury_roll(&mut self, _g: &Game, r: &mut GameRng, _a: Option<&str>, _d: &str) {
            use ffb_model::enums::PS_STUNNED;
            self.ctx.injury = Some(PlayerState::new(PS_STUNNED));
            let _ = r.d6();
        }
    }

    struct NeverBrokenType { ctx: InjuryContext }
    impl InjuryTypeServer for NeverBrokenType {
        fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>,
                         defender_id: &str, coord: FieldCoordinate, from_coord: Option<FieldCoordinate>,
                         old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
            modification_aware_handle_injury(self, game, rng, attacker_id, defender_id,
                                             coord, from_coord, old_ctx, apo_mode);
        }
        fn injury_context(&self) -> &InjuryContext { &self.ctx }
        fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    }
    impl ModificationAwareInjuryType for NeverBrokenType {
        fn armour_roll(&mut self, _g: &Game, _r: &mut GameRng, _a: Option<&str>, _d: &str, _roll: bool) {
            self.ctx.armor_broken = false;
        }
        fn injury_roll(&mut self, _g: &Game, _r: &mut GameRng, _a: Option<&str>, _d: &str) {
            panic!("injury_roll should not be called when armor not broken");
        }
    }

    struct OverridesSavedByArmour { ctx: InjuryContext }
    impl InjuryTypeServer for OverridesSavedByArmour {
        fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>,
                         defender_id: &str, coord: FieldCoordinate, from_coord: Option<FieldCoordinate>,
                         old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
            modification_aware_handle_injury(self, game, rng, attacker_id, defender_id,
                                             coord, from_coord, old_ctx, apo_mode);
        }
        fn injury_context(&self) -> &InjuryContext { &self.ctx }
        fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    }
    impl ModificationAwareInjuryType for OverridesSavedByArmour {
        fn armour_roll(&mut self, _g: &Game, _r: &mut GameRng, _a: Option<&str>, _d: &str, _roll: bool) {
            self.ctx.armor_broken = false;
        }
        fn injury_roll(&mut self, _g: &Game, _r: &mut GameRng, _a: Option<&str>, _d: &str) {
            panic!("injury_roll should not be called");
        }
        fn saved_by_armour(&mut self) {
            // Chainsaw/Stab pattern: no injury on armor save
            self.ctx.injury = None;
        }
    }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }

    #[test]
    fn armor_broken_leads_to_injury_roll() {
        let mut t = AlwaysBrokenType { ctx: InjuryContext::new(ApothecaryMode::Defender) };
        let mut rng = GameRng::new(1);
        let game = make_game();
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury.is_some());
    }

    #[test]
    fn armor_not_broken_calls_saved_by_armour_default_prone() {
        let mut t = NeverBrokenType { ctx: InjuryContext::new(ApothecaryMode::Defender) };
        let mut rng = GameRng::new(1);
        let game = make_game();
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }

    #[test]
    fn saved_by_armour_override_sets_null_injury() {
        let mut t = OverridesSavedByArmour { ctx: InjuryContext::new(ApothecaryMode::Defender) };
        let mut rng = GameRng::new(1);
        let game = make_game();
        t.handle_injury(&game, &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken);
        assert!(t.ctx.injury.is_none());
    }
    #[test]
    fn sets_defender_and_attacker_ids() {
        let mut t = AlwaysBrokenType { ctx: InjuryContext::new(ApothecaryMode::Defender) };
        let mut rng = GameRng::new(1);
        let game = make_game();
        t.handle_injury(&game, &mut rng, Some("atk1"), "def1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("def1"));
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("atk1"));
    }
    #[test]
    fn sets_defender_coordinate() {
        let mut t = NeverBrokenType { ctx: InjuryContext::new(ApothecaryMode::Defender) };
        let mut rng = GameRng::new(1);
        let game = make_game();
        let c = FieldCoordinate::new(3, 4);
        t.handle_injury(&game, &mut rng, None, "p1", c, None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.defender_coordinate, Some(c));
    }
}
