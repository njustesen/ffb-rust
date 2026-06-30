/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeCrowd (abstract).
///
/// Provides shared crowd-push handleInjury logic for CrowdPush/TrapDoorFall variants.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_RESERVE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, do_injury_roll};

pub(crate) fn crowd_handle_injury(
    ctx: &mut InjuryContext, _game: &Game, rng: &mut GameRng,
    attacker_id: Option<&str>, defender_id: &str,
    coord: FieldCoordinate, apo_mode: ApothecaryMode,
) {
    ctx.defender_id = Some(defender_id.to_owned());
    ctx.attacker_id = attacker_id.map(str::to_owned);
    ctx.defender_coordinate = Some(coord);
    ctx.apothecary_mode = apo_mode;
    ctx.armor_broken = true;
    do_injury_roll(rng, ctx);
    if !ctx.is_casualty() && !ctx.is_knocked_out() {
        ctx.injury = Some(PlayerState::new(PS_RESERVE));
    }
}
