/// PrayerState lives in ffb-model (on Game) so that steps can access it via `game.prayer_state`.
/// Re-exported here for backward compatibility with engine tests that import `crate::prayer_state::PrayerState`.
pub use ffb_model::model::prayer_state::PrayerState;
