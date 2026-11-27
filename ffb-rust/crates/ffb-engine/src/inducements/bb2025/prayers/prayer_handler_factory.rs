//! 1:1 translation of `com.fumbbl.ffb.server.factory.mixed.PrayerHandlerFactory` for BB2025.
//!
//! Java builds the handler set by classpath scanning for the game's rules and answers
//! `forPrayer(prayer)` with the first handler that `handles` it. Rust lists the BB2025 handlers
//! explicitly -- one per prayer of the bb2025 table (`inducement/bb2025/Prayers`, rolls 1..=16).
//! Before this factory the bb2025 `StepPrayer` was a stub, so a bought or underdog prayer reported
//! its roll and applied nothing (§Q2.8).

use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::inducements::bb2025::prayers::{
    bad_habits_handler::BadHabitsHandler,
    blessed_statue_of_nuffle_handler::BlessedStatueOfNuffleHandler,
    dazzling_catching_handler::DazzlingCatchingHandler,
    fan_interaction_handler::FanInteractionHandler,
    fouling_frenzy_handler::FoulingFrenzyHandler,
    friends_with_the_ref_handler::FriendsWithTheRefHandler,
    greasy_cleats_handler::GreasyCleatsHandler,
    intensive_training_handler::IntensiveTrainingHandler,
    iron_man_handler::IronManHandler,
    knuckle_dusters_handler::KnuckleDustersHandler,
    moles_under_the_pitch_handler::MolesUnderThePitchHandler,
    perfect_passing_handler::PerfectPassingHandler,
    stiletto_handler::StilettoHandler,
    throw_a_rock_handler::ThrowARockHandler,
    treacherous_trapdoor_handler::TreacherousTrapdoorHandler,
    under_scrutiny_handler::UnderScrutinyHandler,
};

/// Java: `PrayerHandlerFactory.initialize` -- every BB2025 `PrayerHandler`.
pub fn all_handlers() -> Vec<Box<dyn PrayerHandler>> {
    vec![
        Box::new(TreacherousTrapdoorHandler::new()),
        Box::new(FriendsWithTheRefHandler::new()),
        Box::new(StilettoHandler::new()),
        Box::new(IronManHandler::new()),
        Box::new(KnuckleDustersHandler::new()),
        Box::new(BadHabitsHandler::new()),
        Box::new(GreasyCleatsHandler::new()),
        Box::new(BlessedStatueOfNuffleHandler::new()),
        Box::new(MolesUnderThePitchHandler::new()),
        Box::new(PerfectPassingHandler::new()),
        Box::new(DazzlingCatchingHandler::new()),
        Box::new(FanInteractionHandler::new()),
        Box::new(FoulingFrenzyHandler::new()),
        Box::new(ThrowARockHandler::new()),
        Box::new(UnderScrutinyHandler::new()),
        Box::new(IntensiveTrainingHandler::new()),
    ]
}

/// Java: `forPrayer(Prayer)` -- the first handler that handles this prayer, else `None`.
pub fn for_prayer(prayer_name: &str) -> Option<Box<dyn PrayerHandler>> {
    all_handlers().into_iter().find(|h| h.handles_prayer(prayer_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::factory::bb2025::prayer_factory::PrayerFactory;
    use ffb_model::factory::prayer_factory::PrayerFactory as PrayerFactoryTrait;

    /// Every BB2025 prayer roll (1..=16) resolves to a handler; a missing one would take Java's
    /// "no handler found" path and skip the prayer's effect.
    #[test]
    fn every_bb2025_prayer_roll_has_a_handler() {
        let mut factory = PrayerFactory::new();
        factory.initialize();
        for roll in 1..=16 {
            let prayer = factory.for_roll(roll).unwrap_or_else(|| panic!("no prayer for roll {roll}"));
            assert!(for_prayer(&format!("{prayer:?}")).is_some(), "no handler for {prayer:?} (roll {roll})");
        }
    }
}
