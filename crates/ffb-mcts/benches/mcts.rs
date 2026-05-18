use criterion::{criterion_group, criterion_main, Criterion};
use ffb_core::model::game_state::GameState;
use ffb_core::model::player::{Player, PlayerStats};
use ffb_core::model::team::Team;
use ffb_core::rng::GameRng;
use ffb_core::skills::SkillSet;
use ffb_core::types::{FieldCoordinate, PlayerId, PlayerState, TeamId, TurnMode};
use ffb_mcts::{MctsConfig, MctsSearch, OutcomeController, RolloutDepth};
use ffb_sim::simulation::NullStrategy;

fn make_mid_game_state() -> GameState {
    let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
    for i in 0..11u8 {
        home.add_player(Player::new(
            PlayerId(format!("h{i}")),
            format!("HP{i}"),
            "lineman".into(),
            TeamId::Home,
            i + 1,
            PlayerStats::new(6, 3, 4, 8, None),
            SkillSet::empty(),
        ));
    }
    let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
    for i in 0..11u8 {
        away.add_player(Player::new(
            PlayerId(format!("a{i}")),
            format!("AP{i}"),
            "lineman".into(),
            TeamId::Away,
            i + 1,
            PlayerStats::new(5, 4, 3, 9, None),
            SkillSet::empty(),
        ));
    }
    let mut state = GameState::new(home, away);
    // Place players in a mid-game position
    for i in 0..11u8 {
        state.field.place_player(
            PlayerId(format!("h{i}")),
            TeamId::Home,
            FieldCoordinate::new(i + 2, 5),
            PlayerState::Standing,
        );
        state.field.place_player(
            PlayerId(format!("a{i}")),
            TeamId::Away,
            FieldCoordinate::new(i + 13, 10),
            PlayerState::Standing,
        );
    }
    state.turn_mode = TurnMode::Regular;
    state.home_is_active = true;
    state
}

fn bench_mcts_100(c: &mut Criterion) {
    let state = make_mid_game_state();
    c.bench_function("mcts_100", |b| {
        b.iter(|| {
            let mut rng = GameRng::new_live(42);
            let cfg = MctsConfig {
                budget: 100,
                rollout_depth: RolloutDepth::None,
                outcome_controller: OutcomeController::Stochastic,
                c_ucb: 1.414,
                team: TeamId::Home,
                rollout_strategy: Box::new(NullStrategy),
            };
            MctsSearch::search(&state, &cfg, &mut rng)
        });
    });
}

fn bench_mcts_1000(c: &mut Criterion) {
    let state = make_mid_game_state();
    c.bench_function("mcts_1000", |b| {
        b.iter(|| {
            let mut rng = GameRng::new_live(42);
            let cfg = MctsConfig {
                budget: 1000,
                rollout_depth: RolloutDepth::None,
                outcome_controller: OutcomeController::Stochastic,
                c_ucb: 1.414,
                team: TeamId::Home,
                rollout_strategy: Box::new(NullStrategy),
            };
            MctsSearch::search(&state, &cfg, &mut rng)
        });
    });
}

criterion_group!(benches, bench_mcts_100, bench_mcts_1000);
criterion_main!(benches);
