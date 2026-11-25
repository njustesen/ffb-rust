//! 1:1 translation of `com.fumbbl.ffb.util.UtilRangeRuler` (ffb-common): the pass / throw
//! range ruler the server puts on the field model for the client to draw.

use ffb_mechanics::modifiers::pass_context::PassContext;
use ffb_mechanics::modifiers::pass_modifier::PassModifier;
use ffb_mechanics::modifiers::pass_modifier_factory::PassModifierFactory;
use ffb_mechanics::modifiers::modifier_type::ModifierType;
use ffb_model::model::game::Game;
use ffb_model::types::{FieldCoordinate, RangeRuler};

/// Java `UtilRangeRuler.createRangeRuler(game, thrower, targetCoordinate, throwTeamMate)`: null
/// without a thrower, a target or a passing distance; else the ruler with the pass (or TTM)
/// minimum roll, 0 when the mechanic has none.
pub fn create_range_ruler(game: &Game, thrower_id: &str, target: Option<FieldCoordinate>, throw_team_mate: bool)
    -> Option<RangeRuler>
{
    let target = target?;
    let thrower = game.player(thrower_id)?;
    let mechanic = crate::mechanic::pass_mechanic_for(game.rules);
    let thrower_coordinate = game.field_model.player_coordinate(thrower_id);
    let distance = mechanic.find_passing_distance(game, thrower_coordinate, Some(target), throw_team_mate)?;
    // Java PassModifierFactory.findModifiers(PassContext): the collection, skill and card
    // modifiers (Rust splits the factory's three sources; StepPass / StepThrowTeamMate sum them).
    let factory = PassModifierFactory::for_rules(game.rules);
    let ctx = PassContext::new(game, thrower, distance, throw_team_mate);
    let total: i32 = factory.find_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum::<i32>()
        + factory.find_skill_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum::<i32>()
        + factory.find_card_modifiers(&ctx).iter().map(|m| m.get_modifier()).sum::<i32>();
    let minimum_roll = if throw_team_mate {
        // as StepThrowTeamMate: the mechanic's distance term over an empty set, plus the factory sum
        let empty = std::collections::HashSet::new();
        crate::mechanic::ttm_mechanic_for(game.rules).minimum_roll(distance, &empty) + total
    } else {
        let summed: Vec<PassModifier> = if total != 0 {
            vec![PassModifier::new("pass_mods", total, ModifierType::REGULAR)]
        } else {
            Vec::new()
        };
        mechanic.minimum_roll_simple(thrower, distance, &summed).unwrap_or(0)
    };
    Some(RangeRuler::new(thrower_id.to_string(), Some(target), minimum_roll, throw_team_mate))
}
