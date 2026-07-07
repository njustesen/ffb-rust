use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.SendToBoxReason.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SendToBoxReason {
    MNG, FOUL_BAN, SECRET_WEAPON_BAN, FOULED, BLOCKED, CROWD_PUSHED, CROWD_KICKED,
    DODGE_FAIL, GFI_FAIL, KICKED, JUMP_FAIL, STABBED, HIT_BY_ROCK, EATEN,
    HIT_BY_THROWN_PLAYER, LANDING_FAIL, PILED_ON, CHAINSAW, BITTEN, NURGLES_ROT,
    RAISED, LIGHTNING, FIREBALL, KO_ON_PILING_ON, BOMB, BALL_AND_CHAIN,
    PLAGUE_RIDDEN, PROJECTILE_VOMIT, TRAP_DOOR_FALL, OFFICIOUS_REF, THROWN_KEG,
    THREW_TWO_BOMBS, BREATHE_FIRE, THEN_I_STARTED_BLASTIN, QUICK_BITE,
    SABOTEUR, SABOTAGED,
}

impl SendToBoxReason {
    pub fn get_name(self) -> &'static str {
        match self {
            SendToBoxReason::MNG => "mng",
            SendToBoxReason::FOUL_BAN => "foulBan",
            SendToBoxReason::SECRET_WEAPON_BAN => "secretWeaponBan",
            SendToBoxReason::FOULED => "fouled",
            SendToBoxReason::BLOCKED => "blocked",
            SendToBoxReason::CROWD_PUSHED => "crowdPushed",
            SendToBoxReason::CROWD_KICKED => "crowdKicked",
            SendToBoxReason::DODGE_FAIL => "dodgeFail",
            SendToBoxReason::GFI_FAIL => "gfiFail",
            SendToBoxReason::KICKED => "kicked",
            SendToBoxReason::JUMP_FAIL => "leapFail",
            SendToBoxReason::STABBED => "stabbed",
            SendToBoxReason::HIT_BY_ROCK => "hitByRock",
            SendToBoxReason::EATEN => "eaten",
            SendToBoxReason::HIT_BY_THROWN_PLAYER => "hitByThrownPlayer",
            SendToBoxReason::LANDING_FAIL => "landingFail",
            SendToBoxReason::PILED_ON => "piledOn",
            SendToBoxReason::CHAINSAW => "chainsaw",
            SendToBoxReason::BITTEN => "bitten",
            SendToBoxReason::NURGLES_ROT => "nurglesRot",
            SendToBoxReason::RAISED => "raised",
            SendToBoxReason::LIGHTNING => "lightning",
            SendToBoxReason::FIREBALL => "fireball",
            SendToBoxReason::KO_ON_PILING_ON => "koOnPilingOn",
            SendToBoxReason::BOMB => "bomb",
            SendToBoxReason::BALL_AND_CHAIN => "ballAndChain",
            SendToBoxReason::PLAGUE_RIDDEN => "plagueRidden",
            SendToBoxReason::PROJECTILE_VOMIT => "projectileVomit",
            SendToBoxReason::TRAP_DOOR_FALL => "trapDoorFall",
            SendToBoxReason::OFFICIOUS_REF => "officiousRef",
            SendToBoxReason::THROWN_KEG => "thrownKeg",
            SendToBoxReason::THREW_TWO_BOMBS => "threwToBombs",
            SendToBoxReason::BREATHE_FIRE => "breatheFire",
            SendToBoxReason::THEN_I_STARTED_BLASTIN => "startedBlastin",
            SendToBoxReason::QUICK_BITE => "quickBite",
            SendToBoxReason::SABOTEUR => "saboteur",
            SendToBoxReason::SABOTAGED => "sabotaged",
        }
    }

    pub fn get_reason(self) -> &'static str {
        match self {
            SendToBoxReason::MNG => "is recovering from a Serious Injury",
            SendToBoxReason::FOUL_BAN => "was banned for fouling",
            SendToBoxReason::SECRET_WEAPON_BAN => "was banned for using a Secret Weapon",
            SendToBoxReason::FOULED => "was fouled",
            SendToBoxReason::BLOCKED => "was blocked",
            SendToBoxReason::CROWD_PUSHED => "got pushed into the crowd",
            SendToBoxReason::CROWD_KICKED => "got kicked into the crowd",
            SendToBoxReason::DODGE_FAIL => "failed a dodge",
            SendToBoxReason::GFI_FAIL => "failed to go for it",
            SendToBoxReason::KICKED => "got kicked",
            SendToBoxReason::JUMP_FAIL => "failed a leap",
            SendToBoxReason::STABBED => "has been stabbed",
            SendToBoxReason::HIT_BY_ROCK => "has been hit by a rock",
            SendToBoxReason::EATEN => "has been eaten",
            SendToBoxReason::HIT_BY_THROWN_PLAYER => "has been hit by a thrown player",
            SendToBoxReason::LANDING_FAIL => "failed to land after being thrown",
            SendToBoxReason::PILED_ON => "was piled upon",
            SendToBoxReason::CHAINSAW => "has been hit by a chainsaw",
            SendToBoxReason::BITTEN => "was bitten by a team-mate",
            SendToBoxReason::NURGLES_ROT => "has been infected with Nurgle's Rot",
            SendToBoxReason::RAISED => "has been raised from the dead",
            SendToBoxReason::LIGHTNING => "has been hit by a lightning bolt",
            SendToBoxReason::FIREBALL => "has been hit by a fireball",
            SendToBoxReason::KO_ON_PILING_ON => "has been knocked out while Piling On",
            SendToBoxReason::BOMB => "has been hit by a bomb",
            SendToBoxReason::BALL_AND_CHAIN => "has been hit by a ball and chain",
            SendToBoxReason::PLAGUE_RIDDEN => "is now plague ridden",
            SendToBoxReason::PROJECTILE_VOMIT => "has been hit by projectile vomit",
            SendToBoxReason::TRAP_DOOR_FALL => "fell down a trapdoor",
            SendToBoxReason::OFFICIOUS_REF => "was banned by the officious ref",
            SendToBoxReason::THROWN_KEG => "was hit by a beer keg",
            SendToBoxReason::THREW_TWO_BOMBS => "was spotted throwing a second bomb",
            SendToBoxReason::BREATHE_FIRE => "has been hit by breathe fire",
            SendToBoxReason::THEN_I_STARTED_BLASTIN => "was blasted",
            SendToBoxReason::QUICK_BITE => "got injured by quick bite",
            SendToBoxReason::SABOTEUR => "was knocked out sabotaging an opponent",
            SendToBoxReason::SABOTAGED => "was taken out by a sabotaged weapon",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.get_name().eq_ignore_ascii_case(name))
    }

    fn all() -> &'static [SendToBoxReason] {
        &[
            Self::MNG, Self::FOUL_BAN, Self::SECRET_WEAPON_BAN, Self::FOULED, Self::BLOCKED,
            Self::CROWD_PUSHED, Self::CROWD_KICKED, Self::DODGE_FAIL, Self::GFI_FAIL, Self::KICKED,
            Self::JUMP_FAIL, Self::STABBED, Self::HIT_BY_ROCK, Self::EATEN, Self::HIT_BY_THROWN_PLAYER,
            Self::LANDING_FAIL, Self::PILED_ON, Self::CHAINSAW, Self::BITTEN, Self::NURGLES_ROT,
            Self::RAISED, Self::LIGHTNING, Self::FIREBALL, Self::KO_ON_PILING_ON, Self::BOMB,
            Self::BALL_AND_CHAIN, Self::PLAGUE_RIDDEN, Self::PROJECTILE_VOMIT, Self::TRAP_DOOR_FALL,
            Self::OFFICIOUS_REF, Self::THROWN_KEG, Self::THREW_TWO_BOMBS, Self::BREATHE_FIRE,
            Self::THEN_I_STARTED_BLASTIN, Self::QUICK_BITE, Self::SABOTEUR, Self::SABOTAGED,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mng_get_name_returns_camel_case() {
        assert_eq!(SendToBoxReason::MNG.get_name(), "mng");
        assert_eq!(SendToBoxReason::FOUL_BAN.get_name(), "foulBan");
        assert_eq!(SendToBoxReason::SECRET_WEAPON_BAN.get_name(), "secretWeaponBan");
    }

    #[test]
    fn for_name_case_insensitive() {
        assert_eq!(SendToBoxReason::for_name("mng"), Some(SendToBoxReason::MNG));
        assert_eq!(SendToBoxReason::for_name("MNG"), Some(SendToBoxReason::MNG));
        assert_eq!(SendToBoxReason::for_name("foulBan"), Some(SendToBoxReason::FOUL_BAN));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(SendToBoxReason::for_name("NOT_VALID"), None);
    }

    #[test]
    fn get_reason_returns_non_empty_string() {
        assert!(!SendToBoxReason::MNG.get_reason().is_empty());
        assert!(!SendToBoxReason::FOULED.get_reason().is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", SendToBoxReason::MNG).is_empty());
    }

}
