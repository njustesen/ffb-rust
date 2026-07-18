/// 1:1 translation of `com.fumbbl.ffb.inducement.bb2025.Prayer`.
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
    DAZZLING_CATCHING,
    FAN_INTERACTION,
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
            Prayer::DAZZLING_CATCHING => "Dazzling Catching",
            Prayer::FAN_INTERACTION => "Fan Interaction",
            Prayer::FOULING_FRENZY => "Fouling Frenzy",
            Prayer::THROW_A_ROCK => "Throw a Rock",
            Prayer::UNDER_SCRUTINY => "Under Scrutiny",
            Prayer::INTENSIVE_TRAINING => "Intensive Training",
        }
    }

    /// Java: getDescription() — the prayer's rules text.
    pub fn get_description(self) -> &'static str {
        match self {
            Prayer::TREACHEROUS_TRAPDOOR => {
                "Trapdoors appear. On a roll of 1 a player stepping on them falls through them"
            }
            Prayer::FRIENDS_WITH_THE_REF => "Argue the call succeeds on 5+",
            Prayer::STILETTO => "One random player available to play this game gains Stab",
            Prayer::IRON_MAN => "One chosen player available to play this game improves AV by 1 (Max 11+)",
            Prayer::KNUCKLE_DUSTERS => "One chosen player available to play this game gains Mighty Blow",
            Prayer::BAD_HABITS => {
                "D3 random opponent players available to play this game without Loner gain Loner (2+)"
            }
            Prayer::GREASY_CLEATS => {
                "One random opponent player available to play this game has his MA reduced by 1"
            }
            Prayer::BLESSED_STATUE_OF_NUFFLE => "One random player available to play this game gains Pro",
            Prayer::MOLES_UNDER_THE_PITCH => "Rushes from opposing players have a -1 modifier",
            Prayer::PERFECT_PASSING => "Completions generate 2 instead of 1 spp",
            Prayer::DAZZLING_CATCHING => {
                "Caught passes generate 1 spp (from both teams, does not have to be accurate)"
            }
            Prayer::FAN_INTERACTION => "Casualties caused by crowd pushes generate 2 spp",
            Prayer::FOULING_FRENZY => "Casualties caused by fouls generate 2 spp",
            Prayer::THROW_A_ROCK => {
                "Once a game at the start of any turn, a randomly selected opponent on the pitch gets knocked down on a 4+"
            }
            Prayer::UNDER_SCRUTINY => "Fouls by opposing players are always spotted if armour is broken",
            Prayer::INTENSIVE_TRAINING => "One random player available to play this game gains a chosen Primary skill",
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

    /// Java: getDuration() — all BB2025 prayers last UNTIL_END_OF_GAME.
    pub fn get_duration(self) -> InducementDuration {
        InducementDuration::UntilEndOfGame
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
            Prayer::DAZZLING_CATCHING => "DAZZLING_CATCHING",
            Prayer::FAN_INTERACTION => "FAN_INTERACTION",
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
            Prayer::DAZZLING_CATCHING, Prayer::FAN_INTERACTION, Prayer::FOULING_FRENZY,
            Prayer::THROW_A_ROCK, Prayer::UNDER_SCRUTINY, Prayer::INTENSIVE_TRAINING,
        ];
        for p in &prayers {
            assert!(!p.get_name().is_empty());
        }
    }

    #[test]
    fn dazzling_catching_is_bb2025_only() {
        assert_eq!(Prayer::DAZZLING_CATCHING.get_name(), "Dazzling Catching");
    }

    #[test]
    fn all_bb2025_prayers_last_until_end_of_game() {
        assert_eq!(Prayer::FOULING_FRENZY.get_duration(), InducementDuration::UntilEndOfGame);
        assert_eq!(Prayer::FRIENDS_WITH_THE_REF.get_duration(), InducementDuration::UntilEndOfGame);
    }

    #[test]
    fn event_message_returns_correct_strings() {
        assert_eq!(Prayer::STILETTO.event_message(), " gains Stab");
        assert_eq!(Prayer::BLESSED_STATUE_OF_NUFFLE.event_message(), " gains Pro");
        assert_eq!(Prayer::DAZZLING_CATCHING.event_message(), "");
        assert_eq!(Prayer::THROW_A_ROCK.event_message(), "");
    }

    #[test]
    fn name_and_is_changing_player_for_bb2025_variants() {
        assert_eq!(Prayer::DAZZLING_CATCHING.name(), "DAZZLING_CATCHING");
        assert_eq!(Prayer::UNDER_SCRUTINY.name(), "UNDER_SCRUTINY");
        assert!(Prayer::INTENSIVE_TRAINING.is_changing_player());
        assert!(!Prayer::DAZZLING_CATCHING.is_changing_player());
        assert!(Prayer::TREACHEROUS_TRAPDOOR.affects_both_teams());
        assert!(!Prayer::FAN_INTERACTION.affects_both_teams());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", Prayer::TREACHEROUS_TRAPDOOR).is_empty());
    }

    #[test]
    fn get_description_returns_rules_text() {
        assert_eq!(Prayer::FRIENDS_WITH_THE_REF.get_description(), "Argue the call succeeds on 5+");
        assert_eq!(
            Prayer::INTENSIVE_TRAINING.get_description(),
            "One random player available to play this game gains a chosen Primary skill"
        );
        assert!(!Prayer::DAZZLING_CATCHING.get_description().is_empty());
    }
}
