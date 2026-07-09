use crate::kickoff::kickoff_result::KickoffResult as IKickoffResult;

/// 1:1 translation of `com.fumbbl.ffb.kickoff.bb2025.KickoffResult`.
///
/// BB2025 kickoff event enum.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KickoffResult {
    GET_THE_REF,
    TIME_OUT,
    SOLID_DEFENCE,
    HIGH_KICK,
    CHEERING_FANS,
    WEATHER_CHANGE,
    BRILLIANT_COACHING,
    QUICK_SNAP,
    CHARGE,
    DODGY_SNACK,
    PITCH_INVASION,
}

impl IKickoffResult for KickoffResult {
    fn get_name(&self) -> &str {
        match self {
            KickoffResult::GET_THE_REF => "Get the Ref",
            KickoffResult::TIME_OUT => "Time-out",
            KickoffResult::SOLID_DEFENCE => "Solid Defence",
            KickoffResult::HIGH_KICK => "High Kick",
            KickoffResult::CHEERING_FANS => "Cheering Fans",
            KickoffResult::WEATHER_CHANGE => "Weather Change",
            KickoffResult::BRILLIANT_COACHING => "Brilliant Coaching",
            KickoffResult::QUICK_SNAP => "Quick Snap",
            KickoffResult::CHARGE => "Charge",
            KickoffResult::DODGY_SNACK => "Dodgy Snack",
            KickoffResult::PITCH_INVASION => "Pitch Invasion",
        }
    }

    fn get_description(&self) -> &str {
        match self {
            KickoffResult::GET_THE_REF => "Each coach receives a free bribe.",
            KickoffResult::TIME_OUT => "Turn marker moves back by one if kicking team is on turn 6, 7 or 8 or forward otherwise.",
            KickoffResult::SOLID_DEFENCE => "The kicking team may setup D3+3 of its players again.",
            KickoffResult::HIGH_KICK => "A player on the receiving team may try to catch the ball directly.",
            KickoffResult::CHEERING_FANS => "The team with the most enthusiastic fans gains an additonal offensive assist on their next block.",
            KickoffResult::WEATHER_CHANGE => "The weather changes suddenly.",
            KickoffResult::BRILLIANT_COACHING => "The team with the best coaching gains a re-roll.",
            KickoffResult::QUICK_SNAP => "The offence may reposition D3+3 of their open players 1 square each.",
            KickoffResult::CHARGE => "The kicking team can select D3+3 open players to perform Move, Blitz, TTM and KTM actions as it was a regular team turn.",
            KickoffResult::DODGY_SNACK => "A random player gets either -MA and -AV for the Drive or is sent to reserves.",
            KickoffResult::PITCH_INVASION => "Random players are being stunned by the crowd.",
        }
    }

    fn is_coach_reroll(&self) -> bool {
        matches!(self, KickoffResult::BRILLIANT_COACHING)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kickoff::kickoff_result::KickoffResult as IKickoffResult;

    #[test]
    fn cheering_fans_not_fan_reroll_in_bb2025() {
        // BB2025 Cheering Fans grants an offensive assist, not a re-roll
        assert!(!KickoffResult::CHEERING_FANS.is_fan_reroll());
    }

    #[test]
    fn brilliant_coaching_is_coach_reroll() {
        assert!(KickoffResult::BRILLIANT_COACHING.is_coach_reroll());
    }

    #[test]
    fn charge_has_name() {
        assert_eq!(KickoffResult::CHARGE.get_name(), "Charge");
    }

    #[test]
    fn dodgy_snack_has_description() {
        assert!(!KickoffResult::DODGY_SNACK.get_description().is_empty());
    }
}
