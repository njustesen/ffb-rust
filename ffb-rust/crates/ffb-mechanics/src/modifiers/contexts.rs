use ffb_model::enums::PassingDistance;
use ffb_model::model::player::PlayerId;
use ffb_model::types::FieldCoordinate;

/// Context for armor roll modifier computation.
#[derive(Debug, Clone)]
pub struct ArmorContext {
    pub defender_id: PlayerId,
    pub attacker_id: Option<PlayerId>,
    pub is_stab: bool,
    pub is_foul: bool,
    pub foul_assists: i32,
    pub is_ttm: bool,
}

impl ArmorContext {
    pub fn block(defender_id: PlayerId, attacker_id: PlayerId) -> Self {
        ArmorContext {
            defender_id,
            attacker_id: Some(attacker_id),
            is_stab: false,
            is_foul: false,
            foul_assists: 0,
            is_ttm: false,
        }
    }

    pub fn foul(defender_id: PlayerId, attacker_id: PlayerId, assists: i32) -> Self {
        ArmorContext {
            defender_id,
            attacker_id: Some(attacker_id),
            is_stab: false,
            is_foul: true,
            foul_assists: assists,
            is_ttm: false,
        }
    }
}

/// Context for injury roll modifier computation.
#[derive(Debug, Clone)]
pub struct InjuryContext {
    pub defender_id: PlayerId,
    pub attacker_id: Option<PlayerId>,
    pub is_stab: bool,
    pub is_foul: bool,
    pub is_vomit_like: bool,
    pub is_chainsaw: bool,
    pub is_ttm: bool,
    pub attacker_mode: bool,
}

impl InjuryContext {
    pub fn block(defender_id: PlayerId, attacker_id: PlayerId) -> Self {
        InjuryContext {
            defender_id,
            attacker_id: Some(attacker_id),
            is_stab: false,
            is_foul: false,
            is_vomit_like: false,
            is_chainsaw: false,
            is_ttm: false,
            attacker_mode: true,
        }
    }

    pub fn set_defender_mode(&mut self) {
        self.attacker_mode = false;
    }
}

/// Context for dodge roll modifier computation.
#[derive(Debug, Clone)]
pub struct DodgeContext {
    pub player_id: PlayerId,
    pub source: FieldCoordinate,
    pub target: FieldCoordinate,
    pub use_break_tackle: bool,
}

/// Context for catch roll modifier computation.
#[derive(Debug, Clone)]
pub struct CatchContext {
    pub player_id: PlayerId,
    pub ball_coord: FieldCoordinate,
    pub use_diving_catch: bool,
}

/// Context for pass roll modifier computation.
#[derive(Debug, Clone)]
pub struct PassContext {
    pub thrower_id: PlayerId,
    pub distance: PassingDistance,
    pub is_ttm: bool,
}

/// Context for interception roll modifier computation.
#[derive(Debug, Clone)]
pub struct InterceptionContext {
    pub interceptor_id: PlayerId,
    pub thrower_id: PlayerId,
    pub ball_coord: FieldCoordinate,
}

/// Context for jump roll modifier computation.
#[derive(Debug, Clone)]
pub struct JumpContext {
    pub player_id: PlayerId,
    pub source: FieldCoordinate,
    pub target: FieldCoordinate,
}

/// Context for go-for-it modifier computation.
#[derive(Debug, Clone)]
pub struct GoForItContext {
    pub player_id: PlayerId,
    pub target: FieldCoordinate,
}

/// Context for pickup modifier computation.
#[derive(Debug, Clone)]
pub struct PickupContext {
    pub player_id: PlayerId,
    pub coord: FieldCoordinate,
}

/// Context for right-stuff (thrown player landing) modifier computation.
#[derive(Debug, Clone)]
pub struct RightStuffContext {
    pub player_id: PlayerId,
    pub landing_coord: FieldCoordinate,
}

/// Context for hypnotic gaze modifier computation.
#[derive(Debug, Clone)]
pub struct GazeContext {
    pub player_id: PlayerId,
    pub target_id: PlayerId,
}

/// Context for jump-up modifier computation.
#[derive(Debug, Clone)]
pub struct JumpUpContext {
    pub player_id: PlayerId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armor_context_block() {
        let ctx = ArmorContext::block("p1".into(), "p2".into());
        assert!(!ctx.is_foul);
        assert!(!ctx.is_stab);
        assert_eq!(ctx.foul_assists, 0);
    }

    #[test]
    fn injury_context_defender_mode() {
        let mut ctx = InjuryContext::block("p1".into(), "p2".into());
        assert!(ctx.attacker_mode);
        ctx.set_defender_mode();
        assert!(!ctx.attacker_mode);
    }
}
