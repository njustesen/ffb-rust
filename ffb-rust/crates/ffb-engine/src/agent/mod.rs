pub mod move_decision_engine;
mod random_agent;

pub use random_agent::RandomAgent;
pub(crate) use random_agent::response_to_action;

use crate::action::Action;
use crate::engine::GameEngine;
use crate::legal_actions::TeamSide;

/// Trait for agents that choose actions in the game.
pub trait Agent {
    fn act(&mut self, engine: &GameEngine) -> Action;
}

/// Run a complete headless game where `decide` returns one Action per engine step.
///
/// For parity (one agent for both sides):
/// ```ignore
/// run_game(engine, |e| agent.act(e));
/// ```
/// For coverage (separate agents per side):
/// ```ignore
/// run_game(engine, |e| {
///     if e.game.home_playing { home.act(e) } else { away.act(e) }
/// });
/// ```
pub fn run_game<F>(engine: &mut GameEngine, mut decide: F) -> Vec<ffb_model::events::GameEvent>
where
    F: FnMut(&mut GameEngine) -> Action,
{
    let mut all_events = Vec::new();
    while !engine.is_finished() {
        let side = engine.active_side();
        let action = decide(engine);
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

/// Convert an agent response to an engine action (public for compatibility).
pub fn response_to_action_pub(
    response: ffb_model::prompts::AgentResponse,
    prompt: Option<&ffb_model::prompts::AgentPrompt>,
) -> Action {
    random_agent::response_to_action(response, prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::team::Team;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{Rules, PlayerGender, PlayerType};

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
    fn run_game_terminates_with_random_agents() {
        let home = make_team_with_players("h", 11);
        let away = make_team_with_players("a", 11);
        let mut engine = GameEngine::new(home, away, Rules::Bb2020, 42);
        let mut home_agent = RandomAgent::new(1);
        let mut away_agent = RandomAgent::new(2);

        let mut steps = 0;
        let max_steps = 10_000;
        while !engine.is_finished() && steps < max_steps {
            let side = engine.active_side();
            let action = if side == TeamSide::Home {
                home_agent.act(&engine)
            } else {
                away_agent.act(&engine)
            };
            if let Err(e) = engine.apply(side, action) {
                panic!("engine error at step {steps}: {e}");
            }
            steps += 1;
        }
        assert!(steps > 0, "game must make at least one step");
    }
}
