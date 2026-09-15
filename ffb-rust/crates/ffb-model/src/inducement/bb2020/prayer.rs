/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2020.Prayer`.
use crate::enums::InducementDuration;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prayer {
    TREACHEROUS_TRAPDOOR,
    FRIENDS_WITH_THE_REF,
    STILETTO,
    IRON_MAN,
    KNUCKLE_DUSTERS,
    BAD_HABITS,
    GREASY_CLEATS,
    BLESSED_STATUE_OF_NUFFLE,
    MOLES_UNDER_THE_PITCH,
    PERFECT_PASSING,
    FAN_INTERACTION,
    NECESSARY_VIOLENCE,
    FOULING_FRENZY,
    THROW_A_ROCK,
    UNDER_SCRUTINY,
    INTENSIVE_TRAINING,
}

impl Prayer {
    /// Java: getName()
    pub fn get_name(self) -> &'static str {
        match self {
            Prayer::TREACHEROUS_TRAPDOOR => "Treacherous Trapdoor",
            Prayer::FRIENDS_WITH_THE_REF => "Friends with the Ref",
            Prayer::STILETTO => "Stiletto",
            Prayer::IRON_MAN => "Iron Man",
            Prayer::KNUCKLE_DUSTERS => "Knuckle Dusters",
            Prayer::BAD_HABITS => "Bad Habits",
            Prayer::GREASY_CLEATS => "Greasy Cleats",
            Prayer::BLESSED_STATUE_OF_NUFFLE => "Blessed Statue of Nuffle",
            Prayer::MOLES_UNDER_THE_PITCH => "Moles under the Pitch",
            Prayer::PERFECT_PASSING => "Perfect Passing",
            Prayer::FAN_INTERACTION => "Fan Interaction",
            Prayer::NECESSARY_VIOLENCE => "Necessary Violence",
            Prayer::FOULING_FRENZY => "Fouling Frenzy",
            Prayer::THROW_A_ROCK => "Throw a Rock",
            Prayer::UNDER_SCRUTINY => "Under Scrutiny",
            Prayer::INTENSIVE_TRAINING => "Intensive Training",
        }
    }

    /// Java: affectsBothTeams()
    pub fn affects_both_teams(self) -> bool {
        matches!(self, Prayer::TREACHEROUS_TRAPDOOR)
    }

    /// Java: isChangingPlayer()
    pub fn is_changing_player(self) -> bool {
        matches!(
            self,
            Prayer::STILETTO
                | Prayer::IRON_MAN
                | Prayer::KNUCKLE_DUSTERS
                | Prayer::BAD_HABITS
                | Prayer::GREASY_CLEATS
                | Prayer::BLESSED_STATUE_OF_NUFFLE
                | Prayer::INTENSIVE_TRAINING
        )
    }

    /// Java: getDuration()
    pub fn get_duration(self) -> InducementDuration {
        match self {
            Prayer::TREACHEROUS_TRAPDOOR => InducementDuration::UntilEndOfHalf,
            Prayer::FRIENDS_WITH_THE_REF => InducementDuration::UntilEndOfDrive,
            Prayer::MOLES_UNDER_THE_PITCH => InducementDuration::UntilEndOfHalf,
            Prayer::UNDER_SCRUTINY => InducementDuration::UntilEndOfHalf,
            Prayer::IRON_MAN => InducementDuration::UntilEndOfGame,
            Prayer::BLESSED_STATUE_OF_NUFFLE => InducementDuration::UntilEndOfGame,
            Prayer::PERFECT_PASSING => InducementDuration::UntilEndOfGame,
            Prayer::INTENSIVE_TRAINING => InducementDuration::UntilEndOfGame,
            _ => InducementDuration::UntilEndOfDrive,
        }
    }

    /// Java: eventMessage() — the message appended to a player event report.
    pub fn event_message(self) -> &'static str {
        match self {
            Prayer::STILETTO => " gains Stab",
            Prayer::IRON_MAN => " gains 1 AV",
            Prayer::KNUCKLE_DUSTERS => " gains Mighty Blow (+1)",
            Prayer::BAD_HABITS => " gains Loner (2+)",
            Prayer::GREASY_CLEATS => " loses 1 MA",
            Prayer::BLESSED_STATUE_OF_NUFFLE => " gains Pro",
            _ => "",
        }
    }

    /// The inverse of `name()`: resolve the SCREAMING_SNAKE enum-constant name back to the
    /// variant. The prayer handlers carry the prayer as that string (it is the key
    /// `field_model.prayer_enhancements` and `apply_prayer_player_effect` both match on), so this
    /// is what lets them reach `event_message()` and `get_name()` for their reports.
    pub fn for_enum_name(name: &str) -> Option<Self> {
        match name {
            "TREACHEROUS_TRAPDOOR" => Some(Prayer::TREACHEROUS_TRAPDOOR),
            "FRIENDS_WITH_THE_REF" => Some(Prayer::FRIENDS_WITH_THE_REF),
            "STILETTO" => Some(Prayer::STILETTO),
            "IRON_MAN" => Some(Prayer::IRON_MAN),
            "KNUCKLE_DUSTERS" => Some(Prayer::KNUCKLE_DUSTERS),
            "BAD_HABITS" => Some(Prayer::BAD_HABITS),
            "GREASY_CLEATS" => Some(Prayer::GREASY_CLEATS),
            "BLESSED_STATUE_OF_NUFFLE" => Some(Prayer::BLESSED_STATUE_OF_NUFFLE),
            "MOLES_UNDER_THE_PITCH" => Some(Prayer::MOLES_UNDER_THE_PITCH),
            "PERFECT_PASSING" => Some(Prayer::PERFECT_PASSING),
            "FAN_INTERACTION" => Some(Prayer::FAN_INTERACTION),
            "NECESSARY_VIOLENCE" => Some(Prayer::NECESSARY_VIOLENCE),
            "FOULING_FRENZY" => Some(Prayer::FOULING_FRENZY),
            "THROW_A_ROCK" => Some(Prayer::THROW_A_ROCK),
            "UNDER_SCRUTINY" => Some(Prayer::UNDER_SCRUTINY),
            "INTENSIVE_TRAINING" => Some(Prayer::INTENSIVE_TRAINING),
            _ => None,
        }
    }

    /// Java: name() — enum constant name used for serialization.
    pub fn name(self) -> &'static str {
        match self {
            Prayer::TREACHEROUS_TRAPDOOR => "TREACHEROUS_TRAPDOOR",
            Prayer::FRIENDS_WITH_THE_REF => "FRIENDS_WITH_THE_REF",
            Prayer::STILETTO => "STILETTO",
            Prayer::IRON_MAN => "IRON_MAN",
            Prayer::KNUCKLE_DUSTERS => "KNUCKLE_DUSTERS",
            Prayer::BAD_HABITS => "BAD_HABITS",
            Prayer::GREASY_CLEATS => "GREASY_CLEATS",
            Prayer::BLESSED_STATUE_OF_NUFFLE => "BLESSED_STATUE_OF_NUFFLE",
            Prayer::MOLES_UNDER_THE_PITCH => "MOLES_UNDER_THE_PITCH",
            Prayer::PERFECT_PASSING => "PERFECT_PASSING",
            Prayer::FAN_INTERACTION => "FAN_INTERACTION",
            Prayer::NECESSARY_VIOLENCE => "NECESSARY_VIOLENCE",
            Prayer::FOULING_FRENZY => "FOULING_FRENZY",
            Prayer::THROW_A_ROCK => "THROW_A_ROCK",
            Prayer::UNDER_SCRUTINY => "UNDER_SCRUTINY",
            Prayer::INTENSIVE_TRAINING => "INTENSIVE_TRAINING",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_prayers_have_names() {
        let prayers = [
            Prayer::TREACHEROUS_TRAPDOOR, Prayer::FRIENDS_WITH_THE_REF, Prayer::STILETTO,
            Prayer::IRON_MAN, Prayer::KNUCKLE_DUSTERS, Prayer::BAD_HABITS, Prayer::GREASY_CLEATS,
            Prayer::BLESSED_STATUE_OF_NUFFLE, Prayer::MOLES_UNDER_THE_PITCH, Prayer::PERFECT_PASSING,
            Prayer::FAN_INTERACTION, Prayer::NECESSARY_VIOLENCE, Prayer::FOULING_FRENZY,
            Prayer::THROW_A_ROCK, Prayer::UNDER_SCRUTINY, Prayer::INTENSIVE_TRAINING,
        ];
        for p in &prayers {
            assert!(!p.get_name().is_empty());
        }
    }

}
