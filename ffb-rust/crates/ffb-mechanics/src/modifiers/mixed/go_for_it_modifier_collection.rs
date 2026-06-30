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
        inner.add(GoForItModifier::new("Moles under the Pitch (Home)", 1)
            .with_predicate(|ctx| {
                ctx.teams_with_moles_under_pitch.contains(ctx.game.team_home.id.as_str())
            }));
        inner.add(GoForItModifier::new("Moles under the Pitch (Away)", 1)
            .with_predicate(|ctx| {
                ctx.teams_with_moles_under_pitch.contains(ctx.game.team_away.id.as_str())
            }));
        Self { inner }
    }

    pub fn get_modifiers(&self) -> &[GoForItModifier] { self.inner.get_modifiers() }
    pub fn find_applicable<'a>(&'a self, ctx: &GoForItContext<'_>) -> Vec<&'a GoForItModifier> { self.inner.find_applicable(ctx) }
}

impl Default for GoForItModifierCollection {
    fn default() -> Self { Self::new() }
}
