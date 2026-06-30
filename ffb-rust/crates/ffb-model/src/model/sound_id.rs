use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.SoundId.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoundId {
    BLOCK, BLUNDER, BOUNCE, CATCH, CHAINSAW, CLICK, DING, DODGE, DUH, EW,
    EXPLODE, FALL, FIREBALL, FOUL, HYPNO, INJURY, KICK, KO, LIGHTNING,
    ZAP, METAL, NOMNOM, ORGAN, PICKUP, QUESTION, RIP, ROAR, ROOT, SLURP,
    STAB, STEP, SWOOP, THROW, TOUCHDOWN, WHISTLE, WOOOAAAH,
    SPEC_AAH, SPEC_BOO, SPEC_CHEER, SPEC_CLAP, SPEC_CRICKETS, SPEC_HURT,
    SPEC_LAUGH, SPEC_OOH, SPEC_SHOCK, SPEC_STOMP,
    PUMP_CROWD, TRAPDOOR, VOMIT, YOINK,
}

impl SoundId {
    pub fn get_name(self) -> &'static str {
        match self {
            SoundId::BLOCK => "block",
            SoundId::BLUNDER => "blunder",
            SoundId::BOUNCE => "bounce",
            SoundId::CATCH => "catch",
            SoundId::CHAINSAW => "chainsaw",
            SoundId::CLICK => "click",
            SoundId::DING => "ding",
            SoundId::DODGE => "dodge",
            SoundId::DUH => "duh",
            SoundId::EW => "ew",
            SoundId::EXPLODE => "explode",
            SoundId::FALL => "fall",
            SoundId::FIREBALL => "fireball",
            SoundId::FOUL => "foul",
            SoundId::HYPNO => "hypno",
            SoundId::INJURY => "injury",
            SoundId::KICK => "kick",
            SoundId::KO => "ko",
            SoundId::LIGHTNING => "lightning",
            SoundId::ZAP => "zap",
            SoundId::METAL => "metal",
            SoundId::NOMNOM => "nomnom",
            SoundId::ORGAN => "organ",
            SoundId::PICKUP => "pickup",
            SoundId::QUESTION => "question",
            SoundId::RIP => "rip",
            SoundId::ROAR => "roar",
            SoundId::ROOT => "root",
            SoundId::SLURP => "slurp",
            SoundId::STAB => "stab",
            SoundId::STEP => "step",
            SoundId::SWOOP => "swoop",
            SoundId::THROW => "throw",
            SoundId::TOUCHDOWN => "touchdown",
            SoundId::WHISTLE => "whistle",
            SoundId::WOOOAAAH => "woooaaah",
            SoundId::SPEC_AAH => "specAah",
            SoundId::SPEC_BOO => "specBoo",
            SoundId::SPEC_CHEER => "specCheer",
            SoundId::SPEC_CLAP => "specClap",
            SoundId::SPEC_CRICKETS => "specCrickets",
            SoundId::SPEC_HURT => "specHurt",
            SoundId::SPEC_LAUGH => "specLaugh",
            SoundId::SPEC_OOH => "specOoh",
            SoundId::SPEC_SHOCK => "specShock",
            SoundId::SPEC_STOMP => "specStomp",
            SoundId::PUMP_CROWD => "pumpcrowd",
            SoundId::TRAPDOOR => "trapdoor",
            SoundId::VOMIT => "vomit",
            SoundId::YOINK => "yoink",
        }
    }

    pub fn is_spectator_sound(self) -> bool {
        matches!(self,
            SoundId::SPEC_AAH | SoundId::SPEC_BOO | SoundId::SPEC_CHEER | SoundId::SPEC_CLAP |
            SoundId::SPEC_CRICKETS | SoundId::SPEC_HURT | SoundId::SPEC_LAUGH |
            SoundId::SPEC_OOH | SoundId::SPEC_SHOCK | SoundId::SPEC_STOMP
        )
    }
    pub fn for_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.get_name().eq_ignore_ascii_case(name))
    }

    fn all() -> &'static [Self] {
        &[
            Self::BLOCK, Self::BLUNDER, Self::BOUNCE, Self::CATCH, Self::CHAINSAW, Self::CLICK,
            Self::DING, Self::DODGE, Self::DUH, Self::EW, Self::EXPLODE, Self::FALL,
            Self::FIREBALL, Self::FOUL, Self::HYPNO, Self::INJURY, Self::KICK, Self::KO,
            Self::LIGHTNING, Self::ZAP, Self::METAL, Self::NOMNOM, Self::ORGAN, Self::PICKUP,
            Self::QUESTION, Self::RIP, Self::ROAR, Self::ROOT, Self::SLURP, Self::STAB,
            Self::STEP, Self::SWOOP, Self::THROW, Self::TOUCHDOWN, Self::WHISTLE, Self::WOOOAAAH,
            Self::SPEC_AAH, Self::SPEC_BOO, Self::SPEC_CHEER, Self::SPEC_CLAP, Self::SPEC_CRICKETS,
            Self::SPEC_HURT, Self::SPEC_LAUGH, Self::SPEC_OOH, Self::SPEC_SHOCK, Self::SPEC_STOMP,
            Self::PUMP_CROWD, Self::TRAPDOOR, Self::VOMIT, Self::YOINK,
        ]
    }
}
