/// 1:1 translation of com.fumbbl.ffb.client.state.logic.ClientAction.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientAction {
    ALL_YOU_CAN_EAT,
    AUTO_GAZE_ZOAT,
    BALEFUL_HEX,
    BEER_BARREL_BASH,
    BLACK_INK,
    BLITZ,
    BLOCK,
    BOMB,
    BOUNDING_LEAP,
    BREATHE_FIRE,
    CATCH_OF_THE_DAY,
    CHAINSAW,
    CHOMP,
    END_MOVE,
    FRENZIED_RUSH,
    FORGO,
    FOUL,
    FUMBLEROOSKIE,
    FURIOUS_OUTBURST,
    GAZE,
    GAZE_ZOAT,
    GORED_BY_THE_BULL,
    HAIL_MARY_BOMB,
    HAIL_MARY_PASS,
    HIT_AND_RUN,
    HAND_OVER,
    INCORPOREAL,
    JUMP,
    KICK_EM_BLITZ,
    KICK_EM_BLOCK,
    KICK_TEAM_MATE,
    LOOK_INTO_MY_EYES,
    MOVE,
    MULTIPLE_BLOCK,
    PROJECTILE_VOMIT,
    PASS,
    PASS_LONG,
    PASS_SHORT,
    PUNT,
    RAIDING_PARTY,
    RECOVER,
    SECURE_THE_BALL,
    SHOT_TO_NOTHING,
    SHOT_TO_NOTHING_BOMB,
    SLASHING_NAILS,
    STAB,
    STAND_UP,
    STAND_UP_BLITZ,
    THE_FLASHING_BLADE,
    THEN_I_STARTED_BLASTIN,
    THROW_TEAM_MATE,
    TREACHEROUS,
    VICIOUS_VINES,
    WISDOM,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_variant_equals_itself() {
        assert_eq!(ClientAction::MOVE, ClientAction::MOVE);
    }

    #[test]
    fn distinct_variants_are_not_equal() {
        assert_ne!(ClientAction::MOVE, ClientAction::BLOCK);
    }

    #[test]
    fn variant_is_copy_and_clone() {
        let a = ClientAction::PASS;
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientAction::WISDOM).is_empty());
    }
}
