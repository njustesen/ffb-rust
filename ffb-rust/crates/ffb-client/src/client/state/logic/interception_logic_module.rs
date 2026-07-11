//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.InterceptionLogicModule` (92 lines).
//!
//! Java's `InterceptionLogicModule` lets the coach pick which eligible player attempts to
//! intercept a pass. `playerInteraction`/`playerPeek` need `FantasyFootballClient` access, so
//! — matching the established `MoveLogicModule` convention — they are translated as inherent
//! methods taking `client` explicitly rather than trait overrides (see `logic_module.rs`'s
//! module doc on the narrower trait-default signature).
//!
//! Documented gap: `isInterceptor(Player<?> pPlayer)` calls
//! `UtilPassing.findInterceptors(game, game.getThrower(), game.getPassCoordinate())`. That
//! method needs a live `Game` and is only implemented per-rules-edition as a private helper
//! inside `ffb-engine`'s `step_intercept.rs` (bb2016/bb2020/bb2025), not exposed as a public,
//! shared API that `ffb-client` can call (see `ffb_model::util::util_passing::UtilPassing`'s own
//! doc comment: "These methods need a live Game reference and are therefore fully implemented in
//! ffb-engine"). There is also no equivalent of `Player.hasUnused(Skill)` — the Rust `Skill`
//! value type (`ffb_model::model::skill::skill::Skill`) carries only `name`/`category`, not a
//! `SkillId`, so it can't be checked against `Player::used_skills` (which is keyed by
//! `SkillId`). Both gaps mean `is_interceptor` cannot be computed faithfully here; it
//! conservatively always returns `false` (no player is ever offered as an interceptor),
//! matching this project's established conservative-gap convention (e.g.
//! `logic_module::player_can_not_move_placeholder`).

use std::collections::HashSet;

use ffb_model::enums::ClientStateId;
use ffb_model::model::acting_player::ActingPlayer;
use ffb_model::model::game::Game;
use ffb_model::model::player::Player;
use ffb_model::model::skill::skill::Skill;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::client_action::ClientAction;
use crate::client::state::logic::interaction::action_context::ActionContext;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;
use crate::client::state::logic::logic_module::LogicModule;

/// 1:1 translation of the `InterceptionLogicModule` class.
#[derive(Debug, Default)]
pub struct InterceptionLogicModule {
    /// java: `interceptionSkill`.
    interception_skill: Option<Skill>,
}

impl InterceptionLogicModule {
    /// java: `public InterceptionLogicModule(FantasyFootballClient pClient)`.
    pub fn new() -> Self {
        Self { interception_skill: None }
    }

    /// java: `public void setInterceptionSkill(Skill interceptionSkill)`.
    pub fn set_interception_skill(&mut self, interception_skill: Option<Skill>) {
        self.interception_skill = interception_skill;
    }

    /// java: `public InteractionResult playerInteraction(Player<?> pPlayer)` — see module doc
    /// regarding the trait-default signature.
    pub fn player_interaction(
        &self,
        client: &mut FantasyFootballClient,
        player: &Player,
    ) -> InteractionResult {
        if self.is_interceptor(client, player) {
            client
                .communication_mut()
                .send_interceptor_choice(Some(player), self.interception_skill.as_ref());
            InteractionResult::handled()
        } else {
            InteractionResult::ignore()
        }
    }

    /// java: `public InteractionResult playerPeek(Player<?> pPlayer)` — see module doc
    /// regarding the trait-default signature.
    pub fn player_peek(&self, client: &FantasyFootballClient, player: &Player) -> InteractionResult {
        if self.is_interceptor(client, player) {
            InteractionResult::perform()
        } else {
            InteractionResult::reset()
        }
    }

    /// java: `private boolean isInterceptor(Player<?> pPlayer)` — see module doc gap.
    fn is_interceptor(&self, _client: &FantasyFootballClient, _player: &Player) -> bool {
        // java: gap — see module doc comment (`UtilPassing.findInterceptors` and
        // `Player.hasUnused(Skill)` have no callable Rust equivalent here).
        false
    }
}

impl LogicModule for InterceptionLogicModule {
    /// java: `public ClientStateId getId()`.
    fn get_id(&self) -> ClientStateId {
        ClientStateId::Interception
    }

    /// java: `public void setUp() { interceptionSkill = null; }`.
    fn set_up(&mut self, _client: &mut FantasyFootballClient) {
        self.interception_skill = None;
    }

    /// java: `public Set<ClientAction> availableActions()`.
    fn available_actions(&self) -> HashSet<ClientAction> {
        HashSet::new()
    }

    /// java: `protected void performAvailableAction(Player<?> player, ClientAction action) {}`.
    fn perform_available_action(
        &mut self,
        _client: &mut FantasyFootballClient,
        _player: &Player,
        _action: ClientAction,
    ) {
    }

    /// java: `protected ActionContext actionContext(ActingPlayer actingPlayer)` — always
    /// throws `UnsupportedOperationException` in Java; faithfully translated as a panic.
    fn action_context(&self, _game: &Game, _acting_player: &ActingPlayer) -> ActionContext {
        panic!("actionContext for acting player is not supported in interception context")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillCategory};
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    fn make_client() -> FantasyFootballClient {
        let params = crate::client::client_parameters::ClientParameters::create_valid_params(&[
            "-spectator".into(),
            "-coach".into(),
            "bob".into(),
        ])
        .unwrap();
        FantasyFootballClient::new(params)
    }

    #[test]
    fn get_id_is_interception() {
        assert_eq!(InterceptionLogicModule::new().get_id(), ClientStateId::Interception);
    }

    #[test]
    fn available_actions_is_empty() {
        assert!(InterceptionLogicModule::new().available_actions().is_empty());
    }

    #[test]
    fn set_up_clears_interception_skill() {
        let mut module = InterceptionLogicModule::new();
        module.set_interception_skill(Some(Skill::new("Pass Block", SkillCategory::General)));
        let mut client = make_client();
        module.set_up(&mut client);
        assert!(module.interception_skill.is_none());
    }

    #[test]
    fn is_interceptor_is_always_false() {
        let module = InterceptionLogicModule::new();
        let client = make_client();
        let player = Player::default();
        assert!(!module.is_interceptor(&client, &player));
    }

    #[test]
    fn player_peek_resets_since_no_player_is_ever_an_interceptor() {
        let module = InterceptionLogicModule::new();
        let client = make_client();
        let player = Player::default();
        assert_eq!(
            module.player_peek(&client, &player).get_kind(),
            crate::client::state::logic::interaction::interaction_result::Kind::Reset
        );
    }

    #[test]
    #[should_panic(expected = "actionContext for acting player is not supported in interception context")]
    fn action_context_panics() {
        let module = InterceptionLogicModule::new();
        let game = make_game();
        let ap = ActingPlayer::new();
        module.action_context(&game, &ap);
    }
}
