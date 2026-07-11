use crate::client::state::logic::client_action::ClientAction;

/// 1:1 translation of com.fumbbl.ffb.client.state.logic.Influences.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Influences {
    BALL_ACTIONS_DUE_TO_TREACHEROUS,
    HAS_ACTED,
    HANDS_OVER_TO_ANYONE,
    IS_THROWING_HAIL_MARY,
    IS_JUMPING,
    VOMIT_DUE_TO_PUTRID_REGURGITATION,
    INCORPOREAL_ACTIVE,
}

impl Influences {
    pub fn get_influenced_actions(self) -> Vec<ClientAction> {
        match self {
            Influences::BALL_ACTIONS_DUE_TO_TREACHEROUS => vec![
                ClientAction::PASS,
                ClientAction::HAND_OVER,
                ClientAction::SHOT_TO_NOTHING,
            ],
            Influences::HAS_ACTED => vec![ClientAction::END_MOVE],
            Influences::HANDS_OVER_TO_ANYONE => vec![ClientAction::HAND_OVER],
            Influences::IS_THROWING_HAIL_MARY => {
                vec![ClientAction::HAIL_MARY_BOMB, ClientAction::HAIL_MARY_PASS]
            }
            Influences::IS_JUMPING => vec![ClientAction::JUMP],
            Influences::VOMIT_DUE_TO_PUTRID_REGURGITATION => vec![ClientAction::PROJECTILE_VOMIT],
            Influences::INCORPOREAL_ACTIVE => {
                vec![ClientAction::INCORPOREAL, ClientAction::END_MOVE]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_acted_influences_end_move() {
        assert_eq!(
            Influences::HAS_ACTED.get_influenced_actions(),
            vec![ClientAction::END_MOVE]
        );
    }

    #[test]
    fn ball_actions_due_to_treacherous_has_three_actions() {
        let actions = Influences::BALL_ACTIONS_DUE_TO_TREACHEROUS.get_influenced_actions();
        assert_eq!(
            actions,
            vec![
                ClientAction::PASS,
                ClientAction::HAND_OVER,
                ClientAction::SHOT_TO_NOTHING,
            ]
        );
    }

    #[test]
    fn incorporeal_active_influences_two_actions() {
        let actions = Influences::INCORPOREAL_ACTIVE.get_influenced_actions();
        assert_eq!(
            actions,
            vec![ClientAction::INCORPOREAL, ClientAction::END_MOVE]
        );
    }

    #[test]
    fn is_throwing_hail_mary_influences_both_bomb_and_pass() {
        let actions = Influences::IS_THROWING_HAIL_MARY.get_influenced_actions();
        assert_eq!(
            actions,
            vec![ClientAction::HAIL_MARY_BOMB, ClientAction::HAIL_MARY_PASS]
        );
    }
}
