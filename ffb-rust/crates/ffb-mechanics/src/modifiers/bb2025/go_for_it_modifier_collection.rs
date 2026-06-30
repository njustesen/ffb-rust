use ffb_model::enums::Weather;
use crate::modifiers::go_for_it_modifier::GoForItModifier;
use crate::modifiers::go_for_it_context::GoForItContext;
use crate::modifiers::go_for_it_modifier_collection::GoForItModifierCollection as BaseGoForItModifierCollection;

pub struct GoForItModifierCollection {
    inner: BaseGoForItModifierCollection,
}

impl GoForItModifierCollection {
    pub fn new() -> Self {
        let mut inner = BaseGoForItModifierCollection::new();
        inner.add(GoForItModifier::new("Blizzard", 1)
            .with_predicate(|ctx| {
                !ctx.game.is_active("setGfiRollToFive")
                    && ctx.game.field_model.weather == Weather::Blizzard
            }));
        inner.add(GoForItModifier::new("Moles under the Pitch", 1)
            .with_predicate(|ctx| {
                ctx.teams_with_moles_under_pitch.iter()
                    .filter_map(|id| ctx.game.team_by_id(id))
                    .any(|t| !t.has_player(&ctx.player.id))
            }));
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[GoForItModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &GoForItContext<'_>) -> Vec<&'a GoForItModifier> { self.inner.find_applicable(ctx) }
}

impl Default for GoForItModifierCollection {
    fn default() -> Self { Self::new() }
}
