use ffb_model::model::skill::skill::Skill;

use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::influences::Influences;

/// 1:1 translation of com.fumbbl.ffb.client.state.logic.interaction.ActionContext.
pub struct ActionContext {
    actions: Vec<ClientAction>,
    influences: Vec<Influences>,
    block_alternatives: Vec<Skill>,
}

impl ActionContext {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            influences: Vec::new(),
            block_alternatives: Vec::new(),
        }
    }

    pub fn get_actions(&self) -> &Vec<ClientAction> {
        &self.actions
    }

    pub fn get_influences(&self) -> &Vec<Influences> {
        &self.influences
    }

    pub fn get_block_alternatives(&self) -> &Vec<Skill> {
        &self.block_alternatives
    }

    pub fn add_action(&mut self, action: ClientAction) {
        self.actions.push(action);
    }

    pub fn add_influence(&mut self, influence: Influences) {
        self.influences.push(influence);
    }

    pub fn add_block_alternative(&mut self, block_alternative: Skill) {
        self.block_alternatives.push(block_alternative);
    }

    pub fn merge(&mut self, mut other: ActionContext) -> &mut Self {
        self.actions.append(&mut other.actions);
        self.influences.append(&mut other.influences);
        self.block_alternatives.append(&mut other.block_alternatives);
        self
    }
}

impl Default for ActionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::SkillCategory;

    #[test]
    fn new_context_has_empty_lists() {
        let ctx = ActionContext::new();
        assert!(ctx.get_actions().is_empty());
        assert!(ctx.get_influences().is_empty());
        assert!(ctx.get_block_alternatives().is_empty());
    }

    #[test]
    fn add_action_and_influence_and_skill() {
        let mut ctx = ActionContext::new();
        ctx.add_action(ClientAction::MOVE);
        ctx.add_influence(Influences::HAS_ACTED);
        ctx.add_block_alternative(Skill::new("Block", SkillCategory::General));

        assert_eq!(ctx.get_actions(), &vec![ClientAction::MOVE]);
        assert_eq!(ctx.get_influences(), &vec![Influences::HAS_ACTED]);
        assert_eq!(ctx.get_block_alternatives().len(), 1);
    }

    #[test]
    fn merge_combines_lists_from_both_contexts() {
        let mut ctx = ActionContext::new();
        ctx.add_action(ClientAction::MOVE);
        ctx.add_influence(Influences::HAS_ACTED);

        let mut other = ActionContext::new();
        other.add_action(ClientAction::BLOCK);
        other.add_influence(Influences::IS_JUMPING);
        other.add_block_alternative(Skill::new("Block", SkillCategory::General));

        ctx.merge(other);

        assert_eq!(ctx.get_actions(), &vec![ClientAction::MOVE, ClientAction::BLOCK]);
        assert_eq!(
            ctx.get_influences(),
            &vec![Influences::HAS_ACTED, Influences::IS_JUMPING]
        );
        assert_eq!(ctx.get_block_alternatives().len(), 1);
    }

    #[test]
    fn merge_returns_mutable_reference_to_self() {
        let mut ctx = ActionContext::new();
        let other = ActionContext::new();
        let merged = ctx.merge(other);
        assert!(merged.get_actions().is_empty());
    }
}
