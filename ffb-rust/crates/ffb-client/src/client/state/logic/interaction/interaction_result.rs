use ffb_model::enums::ClientStateId;
use ffb_model::types::{FieldCoordinate, MoveSquare, PushbackSquare, RangeRuler};
use ffb_model::model::SpecialEffect;

use crate::client::state::logic::interaction::action_context::ActionContext;

/// 1:1 translation of com.fumbbl.ffb.client.state.logic.interaction.InteractionResult.Kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Delegate,
    PreviewThrow,
    Handled,
    Ignore,
    Invalid,
    Perform,
    Reset,
    SelectAction,
}

/// 1:1 translation of com.fumbbl.ffb.client.state.logic.interaction.InteractionResult.
pub struct InteractionResult {
    kind: Kind,
    coordinate: Option<FieldCoordinate>,
    range_ruler: Option<RangeRuler>,
    pushback_squares: Option<Vec<PushbackSquare>>,
    special_effect: Option<SpecialEffect>,
    move_square: Option<MoveSquare>,
    path: Option<Vec<FieldCoordinate>>,
    delegate: Option<ClientStateId>,
    action_context: Option<ActionContext>,
}

impl InteractionResult {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            coordinate: None,
            range_ruler: None,
            pushback_squares: None,
            special_effect: None,
            move_square: None,
            path: None,
            delegate: None,
            action_context: None,
        }
    }

    pub fn delegate(delegate: ClientStateId) -> Self {
        Self::new(Kind::Delegate).with_delegate(delegate)
    }

    pub fn select_action(action_context: ActionContext) -> Self {
        Self::new(Kind::SelectAction).with_action_context(action_context)
    }

    pub fn invalid() -> Self {
        Self::new(Kind::Invalid)
    }

    pub fn reset() -> Self {
        Self::new(Kind::Reset)
    }

    pub fn perform() -> Self {
        Self::new(Kind::Perform)
    }

    pub fn ignore() -> Self {
        Self::new(Kind::Ignore)
    }

    pub fn handled() -> Self {
        Self::new(Kind::Handled)
    }

    pub fn preview_throw() -> Self {
        Self::new(Kind::PreviewThrow)
    }

    pub fn with_delegate(mut self, delegate: ClientStateId) -> Self {
        self.delegate = Some(delegate);
        self
    }

    pub fn with_action_context(mut self, action_context: ActionContext) -> Self {
        self.action_context = Some(action_context);
        self
    }

    pub fn with_coordinate(mut self, coordinate: FieldCoordinate) -> Self {
        self.coordinate = Some(coordinate);
        self
    }

    pub fn with_path(mut self, path: Vec<FieldCoordinate>) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_move_square(mut self, move_square: MoveSquare) -> Self {
        self.move_square = Some(move_square);
        self
    }

    pub fn with_special_effect(mut self, special_effect: SpecialEffect) -> Self {
        self.special_effect = Some(special_effect);
        self
    }

    pub fn with_range_ruler(mut self, range_ruler: RangeRuler) -> Self {
        self.range_ruler = Some(range_ruler);
        self
    }

    pub fn with_pushback_squares(mut self, pushback_squares: Vec<PushbackSquare>) -> Self {
        self.pushback_squares = Some(pushback_squares);
        self
    }

    pub fn get_kind(&self) -> Kind {
        self.kind
    }

    pub fn get_coordinate(&self) -> Option<FieldCoordinate> {
        self.coordinate
    }

    pub fn get_range_ruler(&self) -> Option<&RangeRuler> {
        self.range_ruler.as_ref()
    }

    pub fn get_pushback_squares(&self) -> Option<&Vec<PushbackSquare>> {
        self.pushback_squares.as_ref()
    }

    pub fn get_special_effect(&self) -> Option<SpecialEffect> {
        self.special_effect
    }

    pub fn get_path(&self) -> Option<&Vec<FieldCoordinate>> {
        self.path.as_ref()
    }

    pub fn get_move_square(&self) -> Option<MoveSquare> {
        self.move_square
    }

    pub fn get_delegate(&self) -> Option<ClientStateId> {
        self.delegate
    }

    pub fn get_action_context(&self) -> Option<&ActionContext> {
        self.action_context.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_has_ignore_kind() {
        assert_eq!(InteractionResult::ignore().get_kind(), Kind::Ignore);
    }

    #[test]
    fn handled_has_handled_kind() {
        assert_eq!(InteractionResult::handled().get_kind(), Kind::Handled);
    }

    #[test]
    fn perform_has_perform_kind() {
        assert_eq!(InteractionResult::perform().get_kind(), Kind::Perform);
    }

    #[test]
    fn invalid_has_invalid_kind() {
        assert_eq!(InteractionResult::invalid().get_kind(), Kind::Invalid);
    }

    #[test]
    fn reset_has_reset_kind() {
        assert_eq!(InteractionResult::reset().get_kind(), Kind::Reset);
    }

    #[test]
    fn preview_throw_has_preview_throw_kind() {
        assert_eq!(
            InteractionResult::preview_throw().get_kind(),
            Kind::PreviewThrow
        );
    }

    #[test]
    fn delegate_has_delegate_kind_and_payload() {
        let result = InteractionResult::delegate(ClientStateId::Move);
        assert_eq!(result.get_kind(), Kind::Delegate);
        assert_eq!(result.get_delegate(), Some(ClientStateId::Move));
    }

    #[test]
    fn select_action_has_select_action_kind_and_payload() {
        let mut ctx = ActionContext::new();
        ctx.add_action(crate::client::state::logic::client_action::ClientAction::MOVE);
        let result = InteractionResult::select_action(ctx);
        assert_eq!(result.get_kind(), Kind::SelectAction);
        assert_eq!(result.get_action_context().unwrap().get_actions().len(), 1);
    }

    #[test]
    fn with_coordinate_sets_payload() {
        let result = InteractionResult::perform().with_coordinate(FieldCoordinate::new(1, 2));
        assert_eq!(result.get_coordinate(), Some(FieldCoordinate::new(1, 2)));
    }
}
