//! 1:1 translation of `com.fumbbl.ffb.server.factory.mixed.PrayerHandlerFactory` for BB2020.
//!
//! Java builds its handler set by classpath scanning
//! (`new Scanner<>(PrayerHandler.class).getSubclassInstances(game.getOptions())`) and then answers
//!
//! ```java
//! public Optional<PrayerHandler> forPrayer(Prayer prayer) {
//!     return handlers.stream().filter(handler -> handler.handles(prayer)).findFirst();
//! }
//! ```
//!
//! Rust has no classpath scanning, so the equivalent is an explicit list of the same handlers —
//! one per BB2020 prayer. `for_prayer` keeps Java's semantics: the first handler that `handles`
//! the prayer, or `None`.
//!
//! Without this factory both `StepPrayer` implementations were stubs ("PrayerHandlerFactory not
//! translated → treat as no handler found"), so a prayer awarded by the BB2020 Cheering Fans
//! kickoff was never applied and its effect dice were never rolled.

use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;
use crate::inducements::bb2020::prayers::{
    bad_habits_handler::BadHabitsHandler,
    blessed_statue_of_nuffle_handler::BlessedStatueOfNuffleHandler,
    fan_interaction_handler::FanInteractionHandler,
    fouling_frenzy_handler::FoulingFrenzyHandler,
    friends_with_the_ref_handler::FriendsWithTheRefHandler,
    greasy_cleats_handler::GreasyCleatsHandler,
    intensive_training_handler::IntensiveTrainingHandler,
    iron_man_handler::IronManHandler,
    knuckle_dusters_handler::KnuckleDustersHandler,
    moles_under_the_pitch_handler::MolesUnderThePitchHandler,
    necessary_violence_handler::NecessaryViolenceHandler,
    perfect_passing_handler::PerfectPassingHandler,
    stiletto_handler::StilettoHandler,
    throw_a_rock_handler::ThrowARockHandler,
    treacherous_trapdoor_handler::TreacherousTrapdoorHandler,
    under_scrutiny_handler::UnderScrutinyHandler,
};

/// Java: `PrayerHandlerFactory.initialize` — every BB2020 `PrayerHandler` subclass.
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
        Box::new(FanInteractionHandler::new()),
        Box::new(NecessaryViolenceHandler::new()),
        Box::new(FoulingFrenzyHandler::new()),
        Box::new(ThrowARockHandler::new()),
        Box::new(UnderScrutinyHandler::new()),
        Box::new(IntensiveTrainingHandler::new()),
    ]
}

/// Java: `forPrayer(Prayer)` — the first handler that handles this prayer, else `None`.
pub fn for_prayer(prayer_name: &str) -> Option<Box<dyn PrayerHandler>> {
    all_handlers().into_iter().find(|h| h.handles_prayer(prayer_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::factory::bb2020::prayer_factory::PrayerFactory;
    use ffb_model::factory::prayer_factory::PrayerFactory as PrayerFactoryTrait;

    /// Every BB2020 prayer roll (1..=16) must resolve to a handler. A missing one would silently
    /// take Java's "no handler found" path and skip the prayer's effect dice.
    #[test]
    fn every_bb2020_prayer_roll_has_a_handler() {
        let mut factory = PrayerFactory::new();
        factory.initialize(true);
        for roll in 1..=16 {
            let prayer = factory.for_roll(roll)
                .unwrap_or_else(|| panic!("no prayer for roll {roll}"));
            let name = format!("{prayer:?}");
            assert!(
                for_prayer(&name).is_some(),
                "roll {roll} → {name} has no PrayerHandler; it would silently do nothing"
            );
        }
    }

    /// Java's `forPrayer` returns the FIRST match; handler names must therefore be unique, or the
    /// list order would silently decide behaviour.
    #[test]
    fn handled_prayers_are_unique() {
        let handlers = all_handlers();
        let mut names: Vec<&str> = handlers.iter().map(|h| h.handled_prayer_name()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(names.len(), before, "two handlers claim the same prayer");
        assert_eq!(before, 16, "BB2020 has 16 prayers");
    }
}
