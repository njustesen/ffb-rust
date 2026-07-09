use crate::kickoff::kickoff_result::KickoffResult as IKickoffResult;

/// 1:1 translation of `com.fumbbl.ffb.kickoff.bb2016.KickoffResult`.
///
/// BB2016 kickoff event enum.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KickoffResult {
    GET_THE_REF,
    RIOT,
    PERFECT_DEFENCE,
    HIGH_KICK,
    CHEERING_FANS,
    WEATHER_CHANGE,
    BRILLIANT_COACHING,
    QUICK_SNAP,
    BLITZ,
    THROW_A_ROCK,
    PITCH_INVASION,
}

impl IKickoffResult for KickoffResult {
    fn get_name(&self) -> &str {
        match self {
            KickoffResult::GET_THE_REF => "Get the Ref",
            KickoffResult::RIOT => "Riot",
            KickoffResult::PERFECT_DEFENCE => "Perfect Defence",
            KickoffResult::HIGH_KICK => "High Kick",
            KickoffResult::CHEERING_FANS => "Cheering Fans",
            KickoffResult::WEATHER_CHANGE => "Weather Change",
            KickoffResult::BRILLIANT_COACHING => "Brilliant Coaching",
            KickoffResult::QUICK_SNAP => "Quick Snap",
            KickoffResult::BLITZ => "Blitz",
            KickoffResult::THROW_A_ROCK => "Throw a Rock",
            KickoffResult::PITCH_INVASION => "Pitch Invasion",
        }
    }

    fn get_description(&self) -> &str {
        match self {
            KickoffResult::GET_THE_REF => "Each coach receives a free bribe.",
            KickoffResult::RIOT => "The referee adjusts the clock after the riot clears.",
            KickoffResult::PERFECT_DEFENCE => "The kicking team may reorganize its players.",
            KickoffResult::HIGH_KICK => "A player on the receiving team may try to catch the ball directly.",
            KickoffResult::CHEERING_FANS => "The team with the most enthusiastic fans gains a re-roll.",
            KickoffResult::WEATHER_CHANGE => "The weather changes suddenly.",
            KickoffResult::BRILLIANT_COACHING => "The team with the best coaching gains a re-roll.",
            KickoffResult::QUICK_SNAP => "The offence may reposition their players 1 square each.",
            KickoffResult::BLITZ => "The defence receives a free turn for moving and blitzing.",
            KickoffResult::THROW_A_ROCK => "A random player is hit by a rock and suffers an injury.",
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
        assert!(!KickoffResult::RIOT.is_fan_reroll());
    }

    #[test]
    fn brilliant_coaching_is_coach_reroll() {
        assert!(KickoffResult::BRILLIANT_COACHING.is_coach_reroll());
        assert!(!KickoffResult::BLITZ.is_coach_reroll());
    }

    #[test]
    fn all_have_names() {
        let all = [
            KickoffResult::GET_THE_REF, KickoffResult::RIOT, KickoffResult::PERFECT_DEFENCE,
            KickoffResult::HIGH_KICK, KickoffResult::CHEERING_FANS, KickoffResult::WEATHER_CHANGE,
            KickoffResult::BRILLIANT_COACHING, KickoffResult::QUICK_SNAP, KickoffResult::BLITZ,
            KickoffResult::THROW_A_ROCK, KickoffResult::PITCH_INVASION,
        ];
        for r in &all {
            assert!(!r.get_name().is_empty());
            assert!(!r.get_description().is_empty());
        }
    }
}
