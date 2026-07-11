//! 1:1 translation of `com.fumbbl.ffb.client.state.IPlayerPopupMenuKeys`, a marker interface
//! of `int` constants aliasing AWT `KeyEvent.VK_*` virtual key codes used to bind popup-menu
//! shortcuts to player actions.
//!
//! Java's `KeyEvent.VK_*` constants are the standard AWT virtual key code values; there is no
//! AWT dependency in this crate, so the raw `i32` values are reproduced directly (these are
//! fixed platform constants, not application logic — see the JDK's `java.awt.event.KeyEvent`
//! for the canonical values). `VK_EXCLAMATION_MARK`/`VK_PLUS`/`VK_ASTERISK` are also standard
//! `KeyEvent` constants (shifted/numpad punctuation keys), not typos.

#![allow(dead_code)]

pub const KEY_GAZE_ZOAT: i32 = VK_A;
pub const KEY_TREACHEROUS: i32 = VK_A;
pub const KEY_PUNT: i32 = VK_A;

pub const KEY_BLOCK: i32 = VK_B;

pub const KEY_CHAINSAW: i32 = VK_C;
pub const KEY_KICK_EM_BLOCK: i32 = VK_C;

pub const KEY_CATCH_OF_THE_DAY: i32 = VK_D;
pub const KEY_THE_FLASHING_BLADE: i32 = VK_D;

pub const KEY_END_MOVE: i32 = VK_E;
pub const KEY_SECURE_THE_BALL: i32 = VK_E;

pub const KEY_FORGO: i32 = VK_F;
pub const KEY_FOUL: i32 = VK_F;
pub const KEY_FUMBLEROOSKIE: i32 = VK_F;

pub const KEY_GAZE: i32 = VK_G;

pub const KEY_HAIL_MARY_PASS: i32 = VK_H;
pub const KEY_HAIL_MARY_BOMB: i32 = VK_H;
pub const KEY_HAND_OVER: i32 = VK_H;
pub const KEY_HIT_AND_RUN: i32 = VK_H;

pub const KEY_BREATHE_FIRE: i32 = VK_I;
pub const KEY_KICK_EM_BLITZ: i32 = VK_I;

pub const KEY_JUMP: i32 = VK_J;

pub const KEY_KICK_TEAM_MATE: i32 = VK_K;

pub const KEY_BEER_BARREL_BASH: i32 = VK_L;
pub const KEY_BLACK_INK: i32 = VK_L;
pub const KEY_LONG: i32 = VK_L;

pub const KEY_MOVE: i32 = VK_M;

pub const KEY_BOUNDING_LEAP: i32 = VK_N;
pub const KEY_SHOT_TO_NOTHING: i32 = VK_N;

pub const KEY_BOMB: i32 = VK_O;
pub const KEY_GORED_BY_THE_BULL: i32 = VK_O;

pub const KEY_PASS: i32 = VK_P;

pub const KEY_FRENZIED_RUSH: i32 = VK_Q;
pub const KEY_STAB: i32 = VK_Q;

pub const KEY_RANGE_GRID: i32 = VK_R;
pub const KEY_RECOVER: i32 = VK_R;

pub const KEY_SHORT: i32 = VK_S;
pub const KEY_SLASHING_NAILS: i32 = VK_S;
pub const KEY_STAND_UP: i32 = VK_S;

pub const KEY_THROW_TEAM_MATE: i32 = VK_T;

pub const KEY_INCORPOREAL: i32 = VK_U;
pub const KEY_MULTIPLE_BLOCK: i32 = VK_U;

pub const KEY_PROJECTILE_VOMIT: i32 = VK_V;
pub const KEY_SHOT_TO_NOTHING_BOMB: i32 = VK_V;

pub const KEY_STAND_UP_BLITZ: i32 = VK_W;
pub const KEY_WISDOM: i32 = VK_W;

pub const KEY_BALEFUL_HEX: i32 = VK_X;

pub const KEY_ALL_YOU_CAN_EAT: i32 = VK_Y;
pub const KEY_LOOK_INTO_MY_EYES: i32 = VK_Y;

pub const KEY_AUTO_GAZE_ZOAT: i32 = VK_Z;
pub const KEY_BLITZ: i32 = VK_Z;

pub const KEY_THEN_I_STARTED_BLASTIN: i32 = VK_EXCLAMATION_MARK;
pub const KEY_VICIOUS_VINES: i32 = VK_PLUS;
pub const KEY_FURIOUS_OUTBURST: i32 = VK_ASTERISK;
pub const KEY_MORE_ACTION: i32 = VK_TAB;

pub const KEY_CHOMP: i32 = VK_1;

pub const KEY_RAIDING_PARTY: i32 = VK_2;

// java: `java.awt.event.KeyEvent` virtual key codes referenced above (standard JDK values,
// reproduced directly since this crate has no AWT dependency).
const VK_A: i32 = 0x41;
const VK_B: i32 = 0x42;
const VK_C: i32 = 0x43;
const VK_D: i32 = 0x44;
const VK_E: i32 = 0x45;
const VK_F: i32 = 0x46;
const VK_G: i32 = 0x47;
const VK_H: i32 = 0x48;
const VK_I: i32 = 0x49;
const VK_J: i32 = 0x4A;
const VK_K: i32 = 0x4B;
const VK_L: i32 = 0x4C;
const VK_M: i32 = 0x4D;
const VK_N: i32 = 0x4E;
const VK_O: i32 = 0x4F;
const VK_P: i32 = 0x50;
const VK_Q: i32 = 0x51;
const VK_R: i32 = 0x52;
const VK_S: i32 = 0x53;
const VK_T: i32 = 0x54;
const VK_U: i32 = 0x55;
const VK_V: i32 = 0x56;
const VK_W: i32 = 0x57;
const VK_X: i32 = 0x58;
const VK_Y: i32 = 0x59;
const VK_Z: i32 = 0x5A;
const VK_1: i32 = 0x31;
const VK_2: i32 = 0x32;
const VK_TAB: i32 = 0x09;
const VK_EXCLAMATION_MARK: i32 = 0x0221;
const VK_PLUS: i32 = 0x0209;
const VK_ASTERISK: i32 = 0x020A;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_key_bindings_match_java_constants() {
        assert_eq!(KEY_GAZE_ZOAT, VK_A);
        assert_eq!(KEY_TREACHEROUS, VK_A);
        assert_eq!(KEY_PUNT, VK_A);
        assert_eq!(KEY_CHAINSAW, VK_C);
        assert_eq!(KEY_KICK_EM_BLOCK, VK_C);
        assert_eq!(KEY_CATCH_OF_THE_DAY, VK_D);
        assert_eq!(KEY_THE_FLASHING_BLADE, VK_D);
        assert_eq!(KEY_END_MOVE, VK_E);
        assert_eq!(KEY_SECURE_THE_BALL, VK_E);
        assert_eq!(KEY_FORGO, VK_F);
        assert_eq!(KEY_FOUL, VK_F);
        assert_eq!(KEY_FUMBLEROOSKIE, VK_F);
    }

    #[test]
    fn distinct_letter_keys_are_distinct() {
        let keys = [
            KEY_BLOCK,
            KEY_GAZE,
            KEY_JUMP,
            KEY_KICK_TEAM_MATE,
            KEY_MOVE,
            KEY_PASS,
            KEY_THROW_TEAM_MATE,
            KEY_BALEFUL_HEX,
        ];
        let mut sorted = keys.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len());
    }

    #[test]
    fn special_punctuation_keys_are_distinct_from_letters() {
        assert_eq!(KEY_THEN_I_STARTED_BLASTIN, VK_EXCLAMATION_MARK);
        assert_eq!(KEY_VICIOUS_VINES, VK_PLUS);
        assert_eq!(KEY_FURIOUS_OUTBURST, VK_ASTERISK);
        assert_eq!(KEY_MORE_ACTION, VK_TAB);
        assert_eq!(KEY_CHOMP, VK_1);
        assert_eq!(KEY_RAIDING_PARTY, VK_2);
    }
}
