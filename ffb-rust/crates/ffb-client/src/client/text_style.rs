/// 1:1 translation of `TextStyle.java`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TextStyle {
    NONE,
    BOLD,
    HOME,
    HOME_BOLD,
    AWAY,
    AWAY_BOLD,
    SPECTATOR,
    SPECTATOR_BOLD,
    ADMIN,
    ADMIN_BOLD,
    DEV,
    DEV_BOLD,
    ROLL,
    NEEDED_ROLL,
    EXPLANATION,
    TURN,
    TURN_HOME,
    TURN_AWAY,
    MENTION,
}

impl TextStyle {
    pub fn get_name(&self) -> &'static str {
        match self {
            TextStyle::NONE => "",
            TextStyle::BOLD => "bold",
            TextStyle::HOME => "home",
            TextStyle::HOME_BOLD => "homeBold",
            TextStyle::AWAY => "away",
            TextStyle::AWAY_BOLD => "awayBold",
            TextStyle::SPECTATOR => "spectator",
            TextStyle::SPECTATOR_BOLD => "spectatorBold",
            TextStyle::ADMIN => "admin",
            TextStyle::ADMIN_BOLD => "adminBold",
            TextStyle::DEV => "dev",
            TextStyle::DEV_BOLD => "devBold",
            TextStyle::ROLL => "roll",
            TextStyle::NEEDED_ROLL => "neededRoll",
            TextStyle::EXPLANATION => "explanation",
            TextStyle::TURN => "turn",
            TextStyle::TURN_HOME => "turnHome",
            TextStyle::TURN_AWAY => "turnAway",
            TextStyle::MENTION => "mention",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_name_none_is_empty() {
        assert_eq!(TextStyle::NONE.get_name(), "");
    }

    #[test]
    fn get_name_bold() {
        assert_eq!(TextStyle::BOLD.get_name(), "bold");
    }

    #[test]
    fn get_name_home_bold() {
        assert_eq!(TextStyle::HOME_BOLD.get_name(), "homeBold");
    }

    #[test]
    fn get_name_mention() {
        assert_eq!(TextStyle::MENTION.get_name(), "mention");
    }
}
