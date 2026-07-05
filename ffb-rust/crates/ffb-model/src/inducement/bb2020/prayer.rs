/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2020.Prayer`.
use crate::inducement::inducement_duration::InducementDuration;

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
            Prayer::TREACHEROUS_TRAPDOOR => InducementDuration::UNTIL_END_OF_HALF,
            Prayer::FRIENDS_WITH_THE_REF => InducementDuration::UNTIL_END_OF_DRIVE,
            Prayer::MOLES_UNDER_THE_PITCH => InducementDuration::UNTIL_END_OF_HALF,
            Prayer::UNDER_SCRUTINY => InducementDuration::UNTIL_END_OF_HALF,
            Prayer::IRON_MAN => InducementDuration::UNTIL_END_OF_GAME,
            Prayer::BLESSED_STATUE_OF_NUFFLE => InducementDuration::UNTIL_END_OF_GAME,
            Prayer::PERFECT_PASSING => InducementDuration::UNTIL_END_OF_GAME,
            Prayer::INTENSIVE_TRAINING => InducementDuration::UNTIL_END_OF_GAME,
            _ => InducementDuration::UNTIL_END_OF_DRIVE,
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

    #[test]
    fn treacherous_trapdoor_affects_both_teams() {
        assert!(Prayer::TREACHEROUS_TRAPDOOR.affects_both_teams());
        assert!(!Prayer::FOULING_FRENZY.affects_both_teams());
    }

    #[test]
    fn changing_player_prayers_are_flagged() {
        assert!(Prayer::STILETTO.is_changing_player());
        assert!(Prayer::BAD_HABITS.is_changing_player());
        assert!(!Prayer::FOULING_FRENZY.is_changing_player());
        assert!(!Prayer::FRIENDS_WITH_THE_REF.is_changing_player());
    }
}
