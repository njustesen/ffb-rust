use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.AnimationType.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimationType {
    PASS, THROW_TEAM_MATE, KICK_TEAM_MATE, KICK,
    SPELL_FIREBALL, SPELL_LIGHTNING, SPELL_ZAP,
    KICKOFF_BLITZ, KICKOFF_BLIZZARD, KICKOFF_BRILLIANT_COACHING,
    KICKOFF_CHEERING_FANS, KICKOFF_GET_THE_REF, KICKOFF_HIGH_KICK,
    KICKOFF_NICE, KICKOFF_PERFECT_DEFENSE, KICKOFF_SOLID_DEFENSE,
    KICKOFF_PITCH_INVASION, KICKOFF_OFFICIOUS_REF, KICKOFF_POURING_RAIN,
    KICKOFF_QUICK_SNAP, KICKOFF_RIOT, KICKOFF_TIMEOUT,
    KICKOFF_SWELTERING_HEAT, KICKOFF_THROW_A_ROCK, KICKOFF_VERY_SUNNY,
    KICKOFF_CHARGE, KICKOFF_DODGY_SNACK,
    HAIL_MARY_PASS, THROW_A_ROCK, THROW_BOMB, HAIL_MARY_BOMB, BOMB_EXPLOSION,
    CARD, THROW_KEG, FUMBLED_KEG, TRICKSTER, BREATHE_FIRE,
    THEN_I_STARTED_BLASTIN,
    PRAYER_TREACHEROUS_TRAPDOOR, PRAYER_BAD_HABITS,
    PRAYER_BLESSED_STATUE_OF_NUFFLE, PRAYER_FAN_INTERACTION,
    PRAYER_FOULING_FRENZY, PRAYER_FRIENDS_WITH_THE_REF,
    PRAYER_GREASY_CLEATS, PRAYER_INTENSIVE_TRAINING, PRAYER_IRON_MAN,
    PRAYER_KNUCKLE_DUSTERS, PRAYER_MOLES_UNDER_THE_PITCH,
    PRAYER_NECESSARY_VIOLENCE, PRAYER_PERFECT_PASSING, PRAYER_STILETTO,
    PRAYER_THROW_A_ROCK, PRAYER_DAZZLING_CATCHING, PRAYER_UNDER_SCRUTINY,
}

impl AnimationType {
    pub fn get_name(self) -> &'static str {
        match self {
            AnimationType::PASS => "pass",
            AnimationType::THROW_TEAM_MATE => "throwTeamMate",
            AnimationType::KICK_TEAM_MATE => "kickTeamMate",
            AnimationType::KICK => "kick",
            AnimationType::SPELL_FIREBALL => "spellFireball",
            AnimationType::SPELL_LIGHTNING => "spellLightning",
            AnimationType::SPELL_ZAP => "spellZap",
            AnimationType::KICKOFF_BLITZ => "kickoffBlitz",
            AnimationType::KICKOFF_BLIZZARD => "kickoffBlizzard",
            AnimationType::KICKOFF_BRILLIANT_COACHING => "kickoffBrilliantCoaching",
            AnimationType::KICKOFF_CHEERING_FANS => "kickoffCheeringFans",
            AnimationType::KICKOFF_GET_THE_REF => "kickoffGetTheRef",
            AnimationType::KICKOFF_HIGH_KICK => "kickoffHighKick",
            AnimationType::KICKOFF_NICE => "kickoffNice",
            AnimationType::KICKOFF_PERFECT_DEFENSE => "kickoffPerfectDefense",
            AnimationType::KICKOFF_SOLID_DEFENSE => "kickoffSolidDefense",
            AnimationType::KICKOFF_PITCH_INVASION => "kickoffPitchInvasion",
            AnimationType::KICKOFF_OFFICIOUS_REF => "kickoffOfficiousRef",
            AnimationType::KICKOFF_POURING_RAIN => "kickoffPouringRain",
            AnimationType::KICKOFF_QUICK_SNAP => "kickoffQuickSnap",
            AnimationType::KICKOFF_RIOT => "kickoffRiot",
            AnimationType::KICKOFF_TIMEOUT => "kickoffTimeout",
            AnimationType::KICKOFF_SWELTERING_HEAT => "kickoffSwelteringHeat",
            AnimationType::KICKOFF_THROW_A_ROCK => "kickoffThrowARock",
            AnimationType::KICKOFF_VERY_SUNNY => "kickoffVerySunny",
            AnimationType::KICKOFF_CHARGE => "kickoffCharge",
            AnimationType::KICKOFF_DODGY_SNACK => "kickoffDodgySnack",
            AnimationType::HAIL_MARY_PASS => "hailMaryPass",
            AnimationType::THROW_A_ROCK => "throwARock",
            AnimationType::THROW_BOMB => "throwBomb",
            AnimationType::HAIL_MARY_BOMB => "hailMaryBomb",
            AnimationType::BOMB_EXPLOSION => "bombExplosion",
            AnimationType::CARD => "card",
            AnimationType::THROW_KEG => "throwKeg",
            AnimationType::FUMBLED_KEG => "fumbledKeg",
            AnimationType::TRICKSTER => "trickster",
            AnimationType::BREATHE_FIRE => "breatheFire",
            AnimationType::THEN_I_STARTED_BLASTIN => "thenIStartedBlastin",
            AnimationType::PRAYER_TREACHEROUS_TRAPDOOR => "prayerTrapdoor",
            AnimationType::PRAYER_BAD_HABITS => "badhabits",
            AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE => "blessedStatueOfNuffle",
            AnimationType::PRAYER_FAN_INTERACTION => "fanInteraction",
            AnimationType::PRAYER_FOULING_FRENZY => "foulingFrenzy",
            AnimationType::PRAYER_FRIENDS_WITH_THE_REF => "friendsWithTheRef",
            AnimationType::PRAYER_GREASY_CLEATS => "greasyCleats",
            AnimationType::PRAYER_INTENSIVE_TRAINING => "intensiveTraining",
            AnimationType::PRAYER_IRON_MAN => "ironMan",
            AnimationType::PRAYER_KNUCKLE_DUSTERS => "knuckleDusters",
            AnimationType::PRAYER_MOLES_UNDER_THE_PITCH => "molesUnderThePitch",
            AnimationType::PRAYER_NECESSARY_VIOLENCE => "necessaryViolence",
            AnimationType::PRAYER_PERFECT_PASSING => "perfectPassing",
            AnimationType::PRAYER_STILETTO => "stiletto",
            AnimationType::PRAYER_THROW_A_ROCK => "throwARockPrayer",
            AnimationType::PRAYER_DAZZLING_CATCHING => "dazzlingCatching",
            AnimationType::PRAYER_UNDER_SCRUTINY => "underScrutiny",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.get_name().eq_ignore_ascii_case(name))
    }

    fn all() -> &'static [AnimationType] {
        &[
            AnimationType::PASS, AnimationType::THROW_TEAM_MATE, AnimationType::KICK_TEAM_MATE,
            AnimationType::KICK, AnimationType::SPELL_FIREBALL, AnimationType::SPELL_LIGHTNING,
            AnimationType::SPELL_ZAP, AnimationType::KICKOFF_BLITZ, AnimationType::KICKOFF_BLIZZARD,
            AnimationType::KICKOFF_BRILLIANT_COACHING, AnimationType::KICKOFF_CHEERING_FANS,
            AnimationType::KICKOFF_GET_THE_REF, AnimationType::KICKOFF_HIGH_KICK,
            AnimationType::KICKOFF_NICE, AnimationType::KICKOFF_PERFECT_DEFENSE,
            AnimationType::KICKOFF_SOLID_DEFENSE, AnimationType::KICKOFF_PITCH_INVASION,
            AnimationType::KICKOFF_OFFICIOUS_REF, AnimationType::KICKOFF_POURING_RAIN,
            AnimationType::KICKOFF_QUICK_SNAP, AnimationType::KICKOFF_RIOT,
            AnimationType::KICKOFF_TIMEOUT, AnimationType::KICKOFF_SWELTERING_HEAT,
            AnimationType::KICKOFF_THROW_A_ROCK, AnimationType::KICKOFF_VERY_SUNNY,
            AnimationType::KICKOFF_CHARGE, AnimationType::KICKOFF_DODGY_SNACK,
            AnimationType::HAIL_MARY_PASS, AnimationType::THROW_A_ROCK, AnimationType::THROW_BOMB,
            AnimationType::HAIL_MARY_BOMB, AnimationType::BOMB_EXPLOSION, AnimationType::CARD,
            AnimationType::THROW_KEG, AnimationType::FUMBLED_KEG, AnimationType::TRICKSTER,
            AnimationType::BREATHE_FIRE, AnimationType::THEN_I_STARTED_BLASTIN,
            AnimationType::PRAYER_TREACHEROUS_TRAPDOOR, AnimationType::PRAYER_BAD_HABITS,
            AnimationType::PRAYER_BLESSED_STATUE_OF_NUFFLE, AnimationType::PRAYER_FAN_INTERACTION,
            AnimationType::PRAYER_FOULING_FRENZY, AnimationType::PRAYER_FRIENDS_WITH_THE_REF,
            AnimationType::PRAYER_GREASY_CLEATS, AnimationType::PRAYER_INTENSIVE_TRAINING,
            AnimationType::PRAYER_IRON_MAN, AnimationType::PRAYER_KNUCKLE_DUSTERS,
            AnimationType::PRAYER_MOLES_UNDER_THE_PITCH, AnimationType::PRAYER_NECESSARY_VIOLENCE,
            AnimationType::PRAYER_PERFECT_PASSING, AnimationType::PRAYER_STILETTO,
            AnimationType::PRAYER_THROW_A_ROCK, AnimationType::PRAYER_DAZZLING_CATCHING,
            AnimationType::PRAYER_UNDER_SCRUTINY,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_get_name_is_lowercase() {
        assert_eq!(AnimationType::PASS.get_name(), "pass");
        assert_eq!(AnimationType::SPELL_FIREBALL.get_name(), "spellFireball");
    }

    #[test]
    fn for_name_case_insensitive() {
        assert_eq!(AnimationType::for_name("pass"), Some(AnimationType::PASS));
        assert_eq!(AnimationType::for_name("PASS"), Some(AnimationType::PASS));
        assert_eq!(AnimationType::for_name("invalid"), None);
    }
}
