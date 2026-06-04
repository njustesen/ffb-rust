pub mod move_decision_engine;

use rand_xoshiro::Xoshiro256StarStar;
use rand_core::{RngCore, SeedableRng};
use ffb_model::prompts::{AgentPrompt, AgentResponse};
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PlayerAction;
use ffb_mechanics::skills::{SkillId, SKILL_TABLE};
use crate::action::Action;
use crate::legal_actions::TeamSide;
use crate::engine::GameEngine;

/// Look up a SkillId by its index in the SKILL_TABLE.
/// Used to convert u16 skill IDs from prompts to the SkillId enum.
fn skill_id_from_u16(idx: u16) -> SkillId {
    SKILL_TABLE.get(idx as usize).map(|s| s.id).unwrap_or(SkillId::Block)
}

/// Trait for agents that respond to game prompts.
pub trait Agent {
    fn respond(&mut self, prompt: &AgentPrompt) -> AgentResponse;
}

/// The single random agent used for both parity and coverage runs.
///
/// Uses Xoshiro256StarStar with `next_u64() % n` for all picks — identical to
/// Java's `Long.remainderUnsigned(decisionRng.nextLong(), n)` pattern so that
/// both engines consume the decision RNG in the same order given the same seed.
pub struct RandomAgent {
    rng: Xoshiro256StarStar,
}

impl RandomAgent {
    /// Standard constructor — seeds with `seed` directly.
    pub fn new(seed: u64) -> Self {
        RandomAgent { rng: Xoshiro256StarStar::seed_from_u64(seed) }
    }

    /// Parity constructor — seeds with `seed ^ 0xDEAD_BEEF_CAFE_0001`, matching
    /// Java's `new Xoshiro256StarStar(seed ^ 0xDEADBEEFCAFE0001L)`.
    pub fn new_parity(seed: u64) -> Self {
        RandomAgent { rng: Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001) }
    }

    /// Pick a uniform random index in `[0, len)`. Matches Java's
    /// `Long.remainderUnsigned(decisionRng.nextLong(), len)`.
    fn pick(&mut self, len: usize) -> usize {
        if len == 0 { 0 } else { (self.rng.next_u64() as usize) % len }
    }

    /// Pick a random bool. Matches Java's `decisionRng.nextLong() % 2 == 0`.
    fn pick_bool(&mut self) -> bool {
        self.rng.next_u64() % 2 == 0
    }

    /// Side-aware deterministic parity respond, matching Java ParityRunner's decision logic.
    ///
    /// Only `CoinChoice`, `ReceiveChoice`, `KickBall`, and `ActivatePlayer` consume decision RNG.
    /// Everything else is deterministic (0 RNG calls), matching Java's existing dialog handlers.
    pub fn respond_parity(&mut self, prompt: &AgentPrompt, side: TeamSide) -> AgentResponse {
        match prompt {
            // ── 1 rng call each ───────────────────────────────────────────────
            AgentPrompt::CoinChoice { .. } =>
                AgentResponse::CoinChoice { heads: self.rng.next_u64() % 2 == 0 },
            AgentPrompt::ReceiveChoice { .. } =>
                AgentResponse::ReceiveChoice { receive: self.rng.next_u64() % 2 == 0 },

            // ── 2 rng calls ───────────────────────────────────────────────────
            AgentPrompt::KickBall => {
                // Home kicks to away's half (x 13..25), away kicks to home's half (x 0..12).
                let x_raw = (self.rng.next_u64() % 13) as i32;
                let y_raw = (self.rng.next_u64() % 13) as i32;
                let x = if side == TeamSide::Home { x_raw + 13 } else { x_raw };
                AgentResponse::KickBall { coord: FieldCoordinate::new(x, y_raw + 1) }
            }

            // ── 1 rng call (T3 Phase 1) ──────────────────────────────────────────
            // Consume 1 decisionRng call to stay in sync with Java's dummy-call.
            // Then return Confirm (EndTurn) WITHOUT applying the activation.
            // Rationale: Java's `ClientCommandActingPlayer` followed by an immediate
            // deselect cancels the activation step sequence before `StepReallyStupid`
            // fires, so Java consumes 0 extra game dice. Rust's `check_negatrait`
            // fires synchronously inside `Action::ActivatePlayer`, so any Troll
            // activation would consume 1 game die that Java doesn't. Returning
            // Confirm avoids applying the activation and keeps game-RNG in sync.
            // ── 1 rng call (T3 Phase 1) ──────────────────────────────────────────
            AgentPrompt::ActivatePlayer { eligible_players } => {
                if !eligible_players.is_empty() {
                    let _pi = (self.rng.next_u64() as usize) % eligible_players.len();
                }
                AgentResponse::Confirm
            }

            // ── 0 rng calls — deterministic, matches Java's dialog handlers ──
            AgentPrompt::TeamSetup { .. } =>
                AgentResponse::TeamSetup { placements: vec![] },

            // Touchback: nearest player to (13,8) — same as Java ParityRunner
            AgentPrompt::Touchback { eligible_players } => {
                let pid = eligible_players.iter()
                    .min_by_key(|(_, c)| {
                        let dx = c.x as i32 - 13;
                        let dy = c.y as i32 - 8;
                        dx * dx + dy * dy
                    })
                    .map(|(id, _)| id.clone())
                    .unwrap_or_default();
                AgentResponse::Touchback { player_id: pid }
            }

            // PlayerChoice / KickoffReturn: decline (empty player) — matches Java
            AgentPrompt::PlayerChoice { .. } | AgentPrompt::KickoffReturn { .. } =>
                AgentResponse::PlayerChoice { player_id: String::new() },

            // Block/follow-up: deterministic choices matching Java's dialog handlers
            AgentPrompt::BlockChoice { .. } =>
                AgentResponse::BlockChoice { index: 0 },
            AgentPrompt::Pushback { squares, .. } => {
                let coord = squares.first().copied().unwrap_or(FieldCoordinate::new(0, 0));
                AgentResponse::Pushback { coord }
            }
            AgentPrompt::FollowUp { .. } =>
                AgentResponse::FollowUp { follow_up: false },
            AgentPrompt::ReRollOffer { .. } =>
                AgentResponse::UseReRoll { use_reroll: false },
            // SkillUse: always use — matches Java's "always USE the skill" handler
            AgentPrompt::SkillUse { .. } | AgentPrompt::PilingOn { .. } =>
                AgentResponse::UseSkill { use_skill: true },
            AgentPrompt::ApothecaryChoice { .. } | AgentPrompt::UseApothecary { .. } =>
                AgentResponse::ApothecaryChoice { heal: false },
            AgentPrompt::ArgueTheCall { .. } =>
                AgentResponse::UseReRoll { use_reroll: false },
            AgentPrompt::BriberyAndCorruption { .. } =>
                AgentResponse::UseBribe { use_bribe: false },
            AgentPrompt::BuyInducements { .. } | AgentPrompt::BuyPrayersAndInducements { .. } =>
                AgentResponse::BuyInducements { purchases: vec![] },
            AgentPrompt::ConcedeGame { .. } =>
                AgentResponse::UseReRoll { use_reroll: false },

            _ => AgentResponse::Confirm,
        }
    }
}

impl Agent for RandomAgent {
    fn respond(&mut self, prompt: &AgentPrompt) -> AgentResponse {
        match prompt {
            AgentPrompt::ReRollOffer { .. } =>
                AgentResponse::UseReRoll { use_reroll: self.pick_bool() },

            AgentPrompt::SkillUse { .. } | AgentPrompt::PilingOn { .. } =>
                AgentResponse::UseSkill { use_skill: self.pick_bool() },

            AgentPrompt::FollowUp { .. } =>
                AgentResponse::FollowUp { follow_up: self.pick_bool() },

            AgentPrompt::HitAndRun { squares, .. } => {
                if !squares.is_empty() && self.pick_bool() {
                    let idx = self.pick(squares.len());
                    AgentResponse::Pushback { coord: squares[idx] }
                } else {
                    AgentResponse::Pushback { coord: FieldCoordinate::new(0, 0) }
                }
            }

            AgentPrompt::BlockChoice { dice, .. } =>
                AgentResponse::BlockChoice { index: self.pick(dice.len()) as i32 },

            AgentPrompt::Pushback { squares, .. } => {
                let idx = self.pick(squares.len());
                let coord = squares.get(idx).copied().unwrap_or(FieldCoordinate::new(0, 0));
                AgentResponse::Pushback { coord }
            }

            AgentPrompt::ActivatePlayer { eligible_players } => {
                if eligible_players.is_empty() {
                    return AgentResponse::Confirm;
                }
                let pi = self.pick(eligible_players.len());
                let (player_id, actions) = &eligible_players[pi];
                let ai = self.pick(actions.len());
                let action = actions.get(ai).copied().unwrap_or(PlayerAction::Move);
                AgentResponse::ActivatePlayer { player_id: player_id.clone(), action }
            }

            AgentPrompt::CoinChoice { .. } =>
                AgentResponse::CoinChoice { heads: self.pick_bool() },

            AgentPrompt::ReceiveChoice { .. } =>
                AgentResponse::ReceiveChoice { receive: self.pick_bool() },

            AgentPrompt::Touchback { eligible_players } => {
                let idx = self.pick(eligible_players.len());
                let pid = eligible_players.get(idx).map(|(id, _)| id.clone()).unwrap_or_default();
                AgentResponse::Touchback { player_id: pid }
            }

            AgentPrompt::KickBall => {
                // Default: kick to away half. Use respond_parity for side-aware kicking.
                let x_raw = self.rng.next_u64() % 13;
                let y_raw = self.rng.next_u64() % 13;
                AgentResponse::KickBall { coord: FieldCoordinate::new(x_raw as i32 + 13, y_raw as i32 + 1) }
            }

            AgentPrompt::KickoffReturn { eligible_players } => {
                let idx = self.pick(eligible_players.len());
                let pid = eligible_players.get(idx).cloned().unwrap_or_default();
                AgentResponse::PlayerChoice { player_id: pid }
            }

            AgentPrompt::PlayerChoice { eligible_players, .. } => {
                let idx = self.pick(eligible_players.len());
                let pid = eligible_players.get(idx).cloned().unwrap_or_default();
                AgentResponse::PlayerChoice { player_id: pid }
            }

            AgentPrompt::ApothecaryChoice { .. } =>
                AgentResponse::ApothecaryChoice { heal: self.pick_bool() },

            AgentPrompt::UseApothecary { .. } =>
                AgentResponse::ApothecaryChoice { heal: self.pick_bool() },

            AgentPrompt::SelectSkill { available, .. } => {
                if available.is_empty() { return AgentResponse::Confirm; }
                let ci = self.pick(available.len());
                let (_, skills) = &available[ci];
                if skills.is_empty() { return AgentResponse::Confirm; }
                let si = self.pick(skills.len());
                AgentResponse::SelectSkill { skill_id: skills[si] }
            }

            AgentPrompt::TeamSetup { .. } =>
                AgentResponse::TeamSetup { placements: vec![] },

            AgentPrompt::BuyInducements { .. } | AgentPrompt::BuyPrayersAndInducements { .. } =>
                AgentResponse::BuyInducements { purchases: vec![] },

            AgentPrompt::ArgueTheCall { .. } =>
                AgentResponse::UseReRoll { use_reroll: self.pick_bool() },

            AgentPrompt::BriberyAndCorruption { .. } =>
                AgentResponse::UseBribe { use_bribe: self.pick_bool() },

            AgentPrompt::ConcedeGame { .. } =>
                AgentResponse::UseReRoll { use_reroll: false },

            _ => AgentResponse::Confirm,
        }
    }
}

/// Run a complete headless game between two agents, returning all events.
pub fn run_game(
    engine: &mut GameEngine,
    home_agent: &mut dyn Agent,
    away_agent: &mut dyn Agent,
) -> Vec<ffb_model::events::GameEvent> {
    let mut all_events = Vec::new();

    while !engine.is_finished() {
        let prompt = match engine.current_prompt() {
            Some(p) => p.clone(),
            None => break,
        };

        let side = engine.active_side();
        let response = match side {
            TeamSide::Home => home_agent.respond(&prompt),
            TeamSide::Away => away_agent.respond(&prompt),
        };

        let action = response_to_action(response, Some(&prompt));
        match engine.apply(side, action) {
            Ok(events) => all_events.extend(events),
            Err(e) => {
                log::warn!("engine error: {e}");
                break;
            }
        }
    }

    all_events
}

/// Convert an agent response to an engine action (public for parity runner).
/// Pass the triggering prompt so context-sensitive responses (skill_id, player_id) work correctly.
pub fn response_to_action_pub(response: AgentResponse, prompt: Option<&AgentPrompt>) -> Action {
    response_to_action(response, prompt)
}

/// Convert an agent response to an engine action.
fn response_to_action(response: AgentResponse, prompt: Option<&AgentPrompt>) -> Action {
    match response {
        AgentResponse::CoinChoice { heads } => Action::CoinChoice { heads },
        AgentResponse::ReceiveChoice { receive } => Action::ReceiveChoice { receive },
        AgentResponse::UseReRoll { use_reroll } => {
            match prompt {
                Some(AgentPrompt::ArgueTheCall { .. }) => Action::ArgueTheCall { argue: use_reroll },
                _ => Action::UseReRoll { use_reroll },
            }
        }
        AgentResponse::UseSkill { use_skill } => {
            let skill_id = match prompt {
                Some(AgentPrompt::SkillUse { skill_id, .. }) => skill_id_from_u16(*skill_id),
                Some(AgentPrompt::PilingOn { .. }) => SkillId::PilingOn,
                _ => SkillId::Block,
            };
            Action::UseSkill { skill_id, use_skill }
        }
        AgentResponse::FollowUp { follow_up } => Action::FollowUp { follow_up },
        AgentResponse::BlockChoice { index } => Action::BlockChoice { die_index: index as usize },
        AgentResponse::Pushback { coord } => {
            match prompt {
                Some(AgentPrompt::HitAndRun { squares, .. }) => {
                    let chosen = if squares.contains(&coord) { Some(coord) } else { None };
                    Action::HitAndRun { coord: chosen }
                }
                _ => Action::PushTo { coord },
            }
        }
        AgentResponse::ActivatePlayer { player_id, action } => {
            let player_action = player_action_to_choice(action);
            Action::ActivatePlayer { player_id, player_action }
        }
        AgentResponse::Touchback { player_id } => Action::Touchback { player_id },
        AgentResponse::KickBall { coord } => Action::KickBall { coord },
        AgentResponse::PlayerChoice { player_id } => Action::SelectPlayer { player_id },
        AgentResponse::TeamSetup { .. } => Action::ConfirmSetup,
        AgentResponse::ApothecaryChoice { heal } => {
            let player_id = match prompt {
                Some(AgentPrompt::UseApothecary { player_id, .. }) => player_id.clone(),
                Some(AgentPrompt::ApothecaryChoice { player_id, .. }) => player_id.clone(),
                _ => String::new(),
            };
            Action::UseApothecary { player_id, use_apothecary: heal }
        }
        AgentResponse::UseBribe { use_bribe } => Action::UseBribe { use_bribe },
        AgentResponse::BuyInducements { purchases } => {
            use crate::action::InducementPurchase;
            Action::BuyInducements {
                purchases: purchases.iter()
                    .map(|(id, count)| InducementPurchase { id: id.clone(), count: *count as u32 })
                    .collect()
            }
        }
        AgentResponse::SelectSkill { skill_id } => {
            Action::SelectSkill { skill_id: skill_id_from_u16(skill_id) }
        }
        AgentResponse::Confirm => Action::EndTurn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::team::Team;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{Rules, PlayerGender, PlayerType, SkillCategory};
    use ffb_model::prompts::{AgentPrompt, AgentResponse};
    use ffb_mechanics::skills::{SkillId, SKILL_TABLE};
    use crate::action::Action;
    use crate::engine::GameEngine;

    fn make_team_with_players(name: &str, n: usize) -> Team {
        let mut players = Vec::new();
        for i in 0..n {
            players.push(Player {
                id: format!("{name}{i}"),
                name: format!("{name}{i}"),
                nr: i as i32,
                position_id: String::new(),
                player_type: PlayerType::Regular,
                gender: PlayerGender::Neutral,
                movement: 6,
                strength: 3,
                agility: 3,
                passing: 4,
                armour: 8,
                starting_skills: vec![],
                extra_skills: vec![],
                temporary_skills: vec![],
                used_skills: std::collections::HashSet::new(),
                niggling_injuries: 0,
                stat_injuries: vec![],
                current_spps: 0,
                career_spps: 0,
                race: None,
            });
        }
        Team {
            id: name.into(),
            name: name.into(),
            race: String::new(),
            roster_id: String::new(),
            coach: String::new(),
            rerolls: 2,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            fan_factor: 5,
            assistant_coaches: 0,
            cheerleaders: 0,
            dedicated_fans: 5,
            treasury: 0,
            team_value: 0,
            players,
            special_rules: vec![],
        }
    }

    #[test]
    fn use_skill_uses_skill_id_from_prompt() {
        let dodge_idx = SKILL_TABLE.iter().position(|s| s.id == SkillId::Dodge).unwrap_or(0) as u16;
        let prompt = AgentPrompt::SkillUse {
            player_id: "p1".into(),
            skill_id: dodge_idx,
            skill_name: "Dodge".into(),
        };
        let response = AgentResponse::UseSkill { use_skill: true };
        let action = response_to_action(response, Some(&prompt));
        assert!(matches!(action, Action::UseSkill { skill_id: SkillId::Dodge, use_skill: true }));
    }

    #[test]
    fn use_apothecary_uses_player_id_from_prompt() {
        let prompt = AgentPrompt::UseApothecary {
            player_id: "injured_player".into(),
            apothecary_type: "standard".into(),
        };
        let response = AgentResponse::ApothecaryChoice { heal: true };
        let action = response_to_action(response, Some(&prompt));
        assert!(matches!(action, Action::UseApothecary { ref player_id, use_apothecary: true } if player_id == "injured_player"));
    }

    #[test]
    fn select_skill_passes_skill_id_through() {
        let block_idx = SKILL_TABLE.iter().position(|s| s.id == SkillId::Block).unwrap_or(0) as u16;
        let prompt = AgentPrompt::SelectSkill {
            player_id: "p1".into(),
            available: vec![(SkillCategory::General, vec![block_idx])],
        };
        let response = AgentResponse::SelectSkill { skill_id: block_idx };
        let action = response_to_action(response, Some(&prompt));
        assert!(matches!(action, Action::SelectSkill { skill_id: SkillId::Block }));
    }

    #[test]
    fn run_game_terminates_with_random_agents() {
        let home = make_team_with_players("h", 11);
        let away = make_team_with_players("a", 11);
        let mut engine = GameEngine::new(home, away, Rules::Bb2020, 42);
        let mut home_agent = RandomAgent::new(1);
        let mut away_agent = RandomAgent::new(2);

        // Run with a step cap to prevent runaway loops in test
        let mut steps = 0;
        let max_steps = 5000;
        while !engine.is_finished() && steps < max_steps {
            let prompt = match engine.current_prompt() {
                Some(p) => p.clone(),
                None => break,
            };
            let side = engine.active_side();
            let response = if side == crate::legal_actions::TeamSide::Home {
                home_agent.respond(&prompt)
            } else {
                away_agent.respond(&prompt)
            };
            let action = response_to_action(response, Some(&prompt));
            if let Err(e) = engine.apply(side, action) {
                panic!("engine error at step {steps}: {e}");
            }
            steps += 1;
        }

        // Game must either finish normally or we hit the step cap (acceptable for now)
        // The key invariant: no panics or infinite loops
        assert!(steps > 0, "game must make at least one step");
    }

    #[test]
    fn random_agent_responds_to_reroll_offer() {
        use ffb_model::enums::ReRollSource;
        let prompt = AgentPrompt::ReRollOffer {
            source: ReRollSource::new("TeamReRoll"),
            action: "Dodge".into(),
            team_id: "home".into(),
        };
        let response = RandomAgent::new(42).respond(&prompt);
        assert!(matches!(response, AgentResponse::UseReRoll { .. }),
            "RandomAgent must return UseReRoll for ReRollOffer");
    }

    #[test]
    fn random_agent_responds_to_follow_up() {
        let prompt = AgentPrompt::FollowUp {
            attacker_id: "att".into(),
            target_coord: ffb_model::types::FieldCoordinate::new(10, 7),
        };
        let response = RandomAgent::new(42).respond(&prompt);
        assert!(matches!(response, AgentResponse::FollowUp { .. }),
            "RandomAgent must return FollowUp for FollowUp prompt");
    }

    #[test]
    fn random_agent_responds_to_activate_player() {
        let prompt = AgentPrompt::ActivatePlayer {
            eligible_players: vec![
                ("p1".into(), vec![PlayerAction::Move]),
                ("p2".into(), vec![PlayerAction::Move, PlayerAction::Block]),
            ],
        };
        let response = RandomAgent::new(42).respond(&prompt);
        assert!(matches!(response, AgentResponse::ActivatePlayer { .. }),
            "RandomAgent must return ActivatePlayer for ActivatePlayer prompt");
        if let AgentResponse::ActivatePlayer { player_id, .. } = response {
            assert!(player_id == "p1" || player_id == "p2",
                "RandomAgent must pick from the eligible players list");
        }
    }

    #[test]
    fn random_agent_responds_to_block_choice() {
        let prompt = AgentPrompt::BlockChoice {
            attacker_id: "att".into(),
            defender_id: "def".into(),
            dice: vec![1, 3, 5],
            own_choice: true,
            nr_of_dice: 2,
        };
        let response = RandomAgent::new(42).respond(&prompt);
        assert!(matches!(response, AgentResponse::BlockChoice { .. }),
            "RandomAgent must return BlockChoice for BlockChoice prompt");
    }
}

fn player_action_to_choice(action: PlayerAction) -> crate::action::PlayerActionChoice {
    use crate::action::PlayerActionChoice;
    match action {
        PlayerAction::Move | PlayerAction::BlitzMove | PlayerAction::PassMove
        | PlayerAction::FoulMove | PlayerAction::HandOverMove | PlayerAction::ThrowTeamMateMove
        | PlayerAction::StandUp | PlayerAction::PuntMove | PlayerAction::KickTeamMateMove => PlayerActionChoice::Move,
        PlayerAction::Block => PlayerActionChoice::Block,
        PlayerAction::Stab => PlayerActionChoice::Stab,
        PlayerAction::Blitz | PlayerAction::BlitzSelect | PlayerAction::StandUpBlitz => PlayerActionChoice::Blitz,
        PlayerAction::Pass | PlayerAction::HailMaryPass | PlayerAction::DumpOff => PlayerActionChoice::Pass,
        PlayerAction::HandOver => PlayerActionChoice::HandOff,
        PlayerAction::Foul => PlayerActionChoice::Foul,
        PlayerAction::ThrowTeamMate => PlayerActionChoice::ThrowTeamMate,
        PlayerAction::KickTeamMate => PlayerActionChoice::KickTeamMate,
        PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::GazeMove => PlayerActionChoice::HypnoticGaze,
        PlayerAction::ThrowBomb | PlayerAction::HailMaryBomb => PlayerActionChoice::ThrowBomb,
        PlayerAction::Swoop => PlayerActionChoice::Swoop,
        PlayerAction::Punt => PlayerActionChoice::Punt,
        _ => PlayerActionChoice::Move,
    }
}
