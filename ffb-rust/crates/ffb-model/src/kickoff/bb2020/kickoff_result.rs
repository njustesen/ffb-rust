use crate::kickoff::kickoff_result::KickoffResult as IKickoffResult;

/// 1:1 translation of `com.fumbbl.ffb.kickoff.bb2020.KickoffResult`.
///
/// BB2020 kickoff event enum.
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
    BLITZ,
    OFFICIOUS_REF,
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
            KickoffResult::BLITZ => "Blitz",
            KickoffResult::OFFICIOUS_REF => "Officious Ref",
            KickoffResult::PITCH_INVASION => "Pitch Invasion",
        }
    }

    fn get_description(&self) -> &str {
        match self {
            KickoffResult::GET_THE_REF => "Each coach receives a free bribe.",
            KickoffResult::TIME_OUT => "Turn marker moves back by one if kicking team is on turn 6, 7 or 8 or forward otherwise.",
            KickoffResult::SOLID_DEFENCE => "The kicking team may reorganize D3+3 of its players.",
            KickoffResult::HIGH_KICK => "A player on the receiving team may try to catch the ball directly.",
            KickoffResult::CHEERING_FANS => "The team with the most enthusiastic fans gains a Prayer to Nuffle.",
            KickoffResult::WEATHER_CHANGE => "The weather changes suddenly.",
            KickoffResult::BRILLIANT_COACHING => "The team with the best coaching gains a re-roll.",
            KickoffResult::QUICK_SNAP => "The offence may reposition D3+3 of their open players 1 square each.",
            KickoffResult::BLITZ => "The defence receives a free turn for moving and blitzing. TTM is allowed but no team re-rolls can be used.",
            KickoffResult::OFFICIOUS_REF => "A random player gets into an argument with the ref and might be sent off.",
            KickoffResult::PITCH_INVASION => "Random players are being stunned by the crowd.",
        }
    }

    fn is_fan_reroll(&self) -> bool {
        matches!(self, KickoffResult::CHEERING_FANS)
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
    fn cheering_fans_is_fan_reroll() {
        assert!(KickoffResult::CHEERING_FANS.is_fan_reroll());
    }

    #[test]
    fn brilliant_coaching_is_coach_reroll() {
        assert!(KickoffResult::BRILLIANT_COACHING.is_coach_reroll());
        assert!(!KickoffResult::OFFICIOUS_REF.is_coach_reroll());
    }

    #[test]
    fn all_have_non_empty_names() {
        let all = [
            KickoffResult::GET_THE_REF, KickoffResult::TIME_OUT, KickoffResult::SOLID_DEFENCE,
        ];
        for r in &all {
            assert!(!r.get_name().is_empty());
        }
    }
}
