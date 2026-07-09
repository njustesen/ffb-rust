/// 1:1 translation of Java InducementPhase enum.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InducementPhase {
    END_OF_OPPONENT_TURN,
    END_OF_OWN_TURN,
    START_OF_OWN_TURN,
    AFTER_KICKOFF_TO_OPPONENT,
    AFTER_INDUCEMENTS_PURCHASED,
    BEFORE_KICKOFF_SCATTER,
    END_OF_TURN_NOT_HALF,
    BEFORE_SETUP,
}

impl InducementPhase {
    pub fn get_name(&self) -> &'static str {
        match self {
            InducementPhase::END_OF_OPPONENT_TURN => "endOfOpponentTurn",
            InducementPhase::END_OF_OWN_TURN => "endOfOwnTurn",
            InducementPhase::START_OF_OWN_TURN => "startOfOwnTurn",
            InducementPhase::AFTER_KICKOFF_TO_OPPONENT => "afterKickoffToOpponent",
            InducementPhase::AFTER_INDUCEMENTS_PURCHASED => "afterInducementsPurchased",
            InducementPhase::BEFORE_KICKOFF_SCATTER => "beforeKickoffScatter",
            InducementPhase::END_OF_TURN_NOT_HALF => "endOfTurnNotHalf",
            InducementPhase::BEFORE_SETUP => "beforeSetup",
        }
    }

    pub fn get_description(&self) -> &'static str {
        match self {
            InducementPhase::END_OF_OPPONENT_TURN => "at end of opponent turn",
            InducementPhase::END_OF_OWN_TURN => "at end of own turn",
            InducementPhase::START_OF_OWN_TURN => "at start of own turn",
            InducementPhase::AFTER_KICKOFF_TO_OPPONENT => "after Kickoff to opponent",
            InducementPhase::AFTER_INDUCEMENTS_PURCHASED => "after Inducements are purchased",
            InducementPhase::BEFORE_KICKOFF_SCATTER => "before Kickoff Scatter",
            InducementPhase::END_OF_TURN_NOT_HALF => "at end of turn, not half",
            InducementPhase::BEFORE_SETUP => "before setting up",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_matches_java() {
        assert_eq!(InducementPhase::START_OF_OWN_TURN.get_name(), "startOfOwnTurn");
    }

    #[test]
    fn test_description_matches_java() {
        assert_eq!(InducementPhase::BEFORE_SETUP.get_description(), "before setting up");
    }
}
