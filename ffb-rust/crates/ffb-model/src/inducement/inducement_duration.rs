/// 1:1 translation of `com.fumbbl.ffb.inducement.InducementDuration`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InducementDuration {
    UNTIL_END_OF_GAME,
    UNTIL_END_OF_DRIVE,
    UNTIL_END_OF_TURN,
    WHILE_HOLDING_THE_BALL,
    UNTIL_USED,
    UNTIL_END_OF_OPPONENTS_TURN,
    UNTIL_END_OF_HALF,
}

impl InducementDuration {
    /// Java: getId()
    pub fn get_id(self) -> i32 {
        match self {
            InducementDuration::UNTIL_END_OF_GAME => 1,
            InducementDuration::UNTIL_END_OF_DRIVE => 2,
            InducementDuration::UNTIL_END_OF_TURN => 3,
            InducementDuration::WHILE_HOLDING_THE_BALL => 4,
            InducementDuration::UNTIL_USED => 5,
            InducementDuration::UNTIL_END_OF_OPPONENTS_TURN => 6,
            InducementDuration::UNTIL_END_OF_HALF => 7,
        }
    }

    /// Java: getName()
    pub fn get_name(self) -> &'static str {
        match self {
            InducementDuration::UNTIL_END_OF_GAME => "untilEndOfGame",
            InducementDuration::UNTIL_END_OF_DRIVE => "untilEndOfDrive",
            InducementDuration::UNTIL_END_OF_TURN => "untilEndOfTurn",
            InducementDuration::WHILE_HOLDING_THE_BALL => "whileHoldingTheBall",
            InducementDuration::UNTIL_USED => "untilUsed",
            InducementDuration::UNTIL_END_OF_OPPONENTS_TURN => "untilEndOfOpponentsTurn",
            InducementDuration::UNTIL_END_OF_HALF => "untilEndOfHalf",
        }
    }

    /// Java: getDescription()
    pub fn get_description(self) -> &'static str {
        match self {
            InducementDuration::UNTIL_END_OF_GAME => "For the entire game",
            InducementDuration::UNTIL_END_OF_DRIVE => "For this drive",
            InducementDuration::UNTIL_END_OF_TURN => "For this turn",
            InducementDuration::WHILE_HOLDING_THE_BALL => "While holding the ball",
            InducementDuration::UNTIL_USED => "Single use",
            InducementDuration::UNTIL_END_OF_OPPONENTS_TURN => "For opponent's turn",
            InducementDuration::UNTIL_END_OF_HALF => "For this half",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_id_returns_correct_id() {
        assert_eq!(InducementDuration::UNTIL_END_OF_GAME.get_id(), 1);
        assert_eq!(InducementDuration::UNTIL_END_OF_DRIVE.get_id(), 2);
        assert_eq!(InducementDuration::UNTIL_END_OF_HALF.get_id(), 7);
    }

    #[test]
    fn get_name_returns_correct_name() {
        assert_eq!(InducementDuration::UNTIL_END_OF_GAME.get_name(), "untilEndOfGame");
        assert_eq!(InducementDuration::UNTIL_END_OF_DRIVE.get_name(), "untilEndOfDrive");
    }

    #[test]
    fn all_variants_have_descriptions() {
        let all = [
            InducementDuration::UNTIL_END_OF_GAME,
            InducementDuration::UNTIL_END_OF_DRIVE,
            InducementDuration::UNTIL_END_OF_TURN,
            InducementDuration::WHILE_HOLDING_THE_BALL,
            InducementDuration::UNTIL_USED,
            InducementDuration::UNTIL_END_OF_OPPONENTS_TURN,
            InducementDuration::UNTIL_END_OF_HALF,
        ];
        for d in &all {
            assert!(!d.get_description().is_empty());
        }
    }
}
