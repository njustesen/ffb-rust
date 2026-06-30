use serde::{Deserialize, Serialize};

/// Dugout box — where injured/KO'd/reserve players sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoxType {
    Reserves,
    Out,
}

impl BoxType {
    pub fn id(self) -> u8 {
        match self {
            BoxType::Reserves => 1,
            BoxType::Out => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BoxType::Reserves => "reserves",
            BoxType::Out => "out",
        }
    }

    pub fn from_id(id: u8) -> Option<BoxType> {
        match id {
            1 => Some(BoxType::Reserves),
            2 => Some(BoxType::Out),
            _ => None,
        }
    }

    pub fn from_name(name: &str) -> Option<BoxType> {
        match name.to_lowercase().as_str() {
            "reserves" => Some(BoxType::Reserves),
            "out" => Some(BoxType::Out),
            _ => None,
        }
    }
}

/// Why a player was removed from the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SendToBoxReason {
    Mng,
    FoulBan,
    SecretWeaponBan,
    Fouled,
    Blocked,
    CrowdPushed,
    CrowdKicked,
    DodgeFail,
    GfiFail,
    Kicked,
    JumpFail,
    Stabbed,
    HitByRock,
    Eaten,
    HitByThrownPlayer,
    LandingFail,
    PiledOn,
    Chainsaw,
    Bitten,
    NurglesRot,
    Raised,
    Lightning,
    Fireball,
    KoOnPilingOn,
    Bomb,
    BallAndChain,
    PlagueRidden,
    ProjectileVomit,
    TrapDoorFall,
    OficiousRef,
    ThrownKeg,
    ThrewTwoBombs,
    BreatheFire,
    ThenIStartedBlastin,
    QuickBite,
    Saboteur,
    Sabotaged,
}

impl SendToBoxReason {
    pub fn name(self) -> &'static str {
        match self {
            SendToBoxReason::Mng => "mng",
            SendToBoxReason::FoulBan => "foulBan",
            SendToBoxReason::SecretWeaponBan => "secretWeaponBan",
            SendToBoxReason::Fouled => "fouled",
            SendToBoxReason::Blocked => "blocked",
            SendToBoxReason::CrowdPushed => "crowdPushed",
            SendToBoxReason::CrowdKicked => "crowdKicked",
            SendToBoxReason::DodgeFail => "dodgeFail",
            SendToBoxReason::GfiFail => "gfiFail",
            SendToBoxReason::Kicked => "kicked",
            SendToBoxReason::JumpFail => "leapFail",
            SendToBoxReason::Stabbed => "stabbed",
            SendToBoxReason::HitByRock => "hitByRock",
            SendToBoxReason::Eaten => "eaten",
            SendToBoxReason::HitByThrownPlayer => "hitByThrownPlayer",
            SendToBoxReason::LandingFail => "landingFail",
            SendToBoxReason::PiledOn => "piledOn",
            SendToBoxReason::Chainsaw => "chainsaw",
            SendToBoxReason::Bitten => "bitten",
            SendToBoxReason::NurglesRot => "nurglesRot",
            SendToBoxReason::Raised => "raised",
            SendToBoxReason::Lightning => "lightning",
            SendToBoxReason::Fireball => "fireball",
            SendToBoxReason::KoOnPilingOn => "koOnPilingOn",
            SendToBoxReason::Bomb => "bomb",
            SendToBoxReason::BallAndChain => "ballAndChain",
            SendToBoxReason::PlagueRidden => "plagueRidden",
            SendToBoxReason::ProjectileVomit => "projectileVomit",
            SendToBoxReason::TrapDoorFall => "trapDoorFall",
            SendToBoxReason::OficiousRef => "officiousRef",
            SendToBoxReason::ThrownKeg => "thrownKeg",
            SendToBoxReason::ThrewTwoBombs => "threwToBombs",
            SendToBoxReason::BreatheFire => "breatheFire",
            SendToBoxReason::ThenIStartedBlastin => "startedBlastin",
            SendToBoxReason::QuickBite => "quickBite",
            SendToBoxReason::Saboteur => "saboteur",
            SendToBoxReason::Sabotaged => "sabotaged",
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            SendToBoxReason::Mng => "is recovering from a Serious Injury",
            SendToBoxReason::FoulBan => "was banned for fouling",
            SendToBoxReason::SecretWeaponBan => "was banned for using a Secret Weapon",
            SendToBoxReason::Fouled => "was fouled",
            SendToBoxReason::Blocked => "was blocked",
            SendToBoxReason::CrowdPushed => "got pushed into the crowd",
            SendToBoxReason::CrowdKicked => "got kicked into the crowd",
            SendToBoxReason::DodgeFail => "failed a dodge",
            SendToBoxReason::GfiFail => "failed to go for it",
            SendToBoxReason::Kicked => "got kicked",
            SendToBoxReason::JumpFail => "failed a leap",
            SendToBoxReason::Stabbed => "has been stabbed",
            SendToBoxReason::HitByRock => "has been hit by a rock",
            SendToBoxReason::Eaten => "has been eaten",
            SendToBoxReason::HitByThrownPlayer => "has been hit by a thrown player",
            SendToBoxReason::LandingFail => "failed to land after being thrown",
            SendToBoxReason::PiledOn => "was piled upon",
            SendToBoxReason::Chainsaw => "has been hit by a chainsaw",
            SendToBoxReason::Bitten => "was bitten by a team-mate",
            SendToBoxReason::NurglesRot => "has been infected with Nurgle's Rot",
            SendToBoxReason::Raised => "has been raised from the dead",
            SendToBoxReason::Lightning => "has been hit by a lightning bolt",
            SendToBoxReason::Fireball => "has been hit by a fireball",
            SendToBoxReason::KoOnPilingOn => "has been knocked out while Piling On",
            SendToBoxReason::Bomb => "has been hit by a bomb",
            SendToBoxReason::BallAndChain => "has been hit by a ball and chain",
            SendToBoxReason::PlagueRidden => "is now plague ridden",
            SendToBoxReason::ProjectileVomit => "has been hit by projectile vomit",
            SendToBoxReason::TrapDoorFall => "fell down a trapdoor",
            SendToBoxReason::OficiousRef => "was banned by the officious ref",
            SendToBoxReason::ThrownKeg => "was hit by a beer keg",
            SendToBoxReason::ThrewTwoBombs => "was spotted throwing a second bomb",
            SendToBoxReason::BreatheFire => "has been hit by breathe fire",
            SendToBoxReason::ThenIStartedBlastin => "was blasted",
            SendToBoxReason::QuickBite => "got injured by quick bite",
            SendToBoxReason::Saboteur => "was knocked out sabotaging an opponent",
            SendToBoxReason::Sabotaged => "was taken out by a sabotaged weapon",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.name().eq_ignore_ascii_case(name))
    }

    fn all() -> &'static [Self] {
        use SendToBoxReason::*;
        &[
            Mng, FoulBan, SecretWeaponBan, Fouled, Blocked, CrowdPushed, CrowdKicked,
            DodgeFail, GfiFail, Kicked, JumpFail, Stabbed, HitByRock, Eaten,
            HitByThrownPlayer, LandingFail, PiledOn, Chainsaw, Bitten, NurglesRot,
            Raised, Lightning, Fireball, KoOnPilingOn, Bomb, BallAndChain, PlagueRidden,
            ProjectileVomit, TrapDoorFall, OficiousRef, ThrownKeg, ThrewTwoBombs,
            BreatheFire, ThenIStartedBlastin, QuickBite, Saboteur, Sabotaged,
        ]
    }
}

/// Administrative status of a team on FUMBBL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamStatus {
    New,
    Active,
    PendingApproval,
    Blocked,
    Retired,
    WaitingForOpponent,
    SkillRollsPending,
}

impl TeamStatus {
    pub fn id(self) -> u8 {
        match self {
            TeamStatus::New => 0,
            TeamStatus::Active => 1,
            TeamStatus::PendingApproval => 2,
            TeamStatus::Blocked => 3,
            TeamStatus::Retired => 4,
            TeamStatus::WaitingForOpponent => 5,
            TeamStatus::SkillRollsPending => 6,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TeamStatus::New => "New",
            TeamStatus::Active => "Active",
            TeamStatus::PendingApproval => "Pending Approval",
            TeamStatus::Blocked => "Blocked",
            TeamStatus::Retired => "Retired",
            TeamStatus::WaitingForOpponent => "Waiting for Opponent",
            TeamStatus::SkillRollsPending => "Skill Rolls Pending",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        [
            TeamStatus::New,
            TeamStatus::Active,
            TeamStatus::PendingApproval,
            TeamStatus::Blocked,
            TeamStatus::Retired,
            TeamStatus::WaitingForOpponent,
            TeamStatus::SkillRollsPending,
        ]
        .iter()
        .copied()
        .find(|v| v.name().eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_type_round_trip_id() {
        for id in 1u8..=2 {
            let b = BoxType::from_id(id).unwrap();
            assert_eq!(b.id(), id);
        }
    }

    #[test]
    fn send_to_box_reason_serde() {
        let r = SendToBoxReason::CrowdPushed;
        let json = serde_json::to_string(&r).unwrap();
        let back: SendToBoxReason = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn box_type_count_is_two() {
        assert_eq!(BoxType::from_id(1).is_some() as u8 + BoxType::from_id(2).is_some() as u8, 2);
    }

    #[test]
    fn box_type_from_name_reserves() {
        assert_eq!(BoxType::from_name("reserves"), Some(BoxType::Reserves));
        assert_eq!(BoxType::from_name("out"), Some(BoxType::Out));
    }

    #[test]
    fn box_type_from_id_unknown_is_none() {
        assert!(BoxType::from_id(99).is_none());
    }

    #[test]
    fn send_to_box_reason_mng_name() {
        assert_eq!(SendToBoxReason::Mng.name(), "mng");
    }

    #[test]
    fn send_to_box_reason_foul_ban_name() {
        assert_eq!(SendToBoxReason::FoulBan.name(), "foulBan");
    }

    #[test]
    fn send_to_box_reason_blocked_reason() {
        assert_eq!(SendToBoxReason::Blocked.reason(), "was blocked");
    }

    #[test]
    fn team_status_count_is_seven() {
        let all = [
            TeamStatus::New, TeamStatus::Active, TeamStatus::PendingApproval, TeamStatus::Blocked,
            TeamStatus::Retired, TeamStatus::WaitingForOpponent, TeamStatus::SkillRollsPending,
        ];
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn team_status_active_id_is_one() {
        assert_eq!(TeamStatus::Active.id(), 1);
    }

    #[test]
    fn team_status_new_id_is_zero() {
        assert_eq!(TeamStatus::New.id(), 0);
    }

    #[test]
    fn team_status_active_name() {
        assert_eq!(TeamStatus::Active.name(), "Active");
    }

    #[test]
    fn team_status_all_have_non_empty_names() {
        for s in [
            TeamStatus::New, TeamStatus::Active, TeamStatus::PendingApproval, TeamStatus::Blocked,
            TeamStatus::Retired, TeamStatus::WaitingForOpponent, TeamStatus::SkillRollsPending,
        ] {
            assert!(!s.name().is_empty());
        }
    }

    #[test]
    fn send_to_box_reason_all_have_non_empty_names() {
        for r in [
            SendToBoxReason::Mng, SendToBoxReason::FoulBan, SendToBoxReason::Fouled,
            SendToBoxReason::Blocked, SendToBoxReason::CrowdPushed, SendToBoxReason::DodgeFail,
            SendToBoxReason::GfiFail, SendToBoxReason::Kicked, SendToBoxReason::Stabbed,
            SendToBoxReason::Lightning, SendToBoxReason::Fireball, SendToBoxReason::Bomb,
        ] {
            assert!(!r.name().is_empty());
        }
    }

    #[test]
    fn send_to_box_reason_all_have_non_empty_reasons() {
        for r in [
            SendToBoxReason::Mng, SendToBoxReason::FoulBan, SendToBoxReason::Fouled,
            SendToBoxReason::Blocked, SendToBoxReason::CrowdPushed, SendToBoxReason::DodgeFail,
            SendToBoxReason::GfiFail, SendToBoxReason::Kicked, SendToBoxReason::Stabbed,
            SendToBoxReason::Lightning, SendToBoxReason::Fireball, SendToBoxReason::Bomb,
        ] {
            assert!(!r.reason().is_empty());
        }
    }

    #[test]
    fn team_status_retired_name() {
        assert_eq!(TeamStatus::Retired.name(), "Retired");
    }

    #[test]
    fn team_status_pending_approval_name() {
        assert_eq!(TeamStatus::PendingApproval.name(), "Pending Approval");
    }

    #[test]
    fn box_type_reserves_name() {
        assert_eq!(BoxType::Reserves.name(), "reserves");
    }

    #[test]
    fn box_type_out_name() {
        assert_eq!(BoxType::Out.name(), "out");
    }
}
