use criterion::{criterion_group, criterion_main, Criterion};
use ffb_core::model::game_state::GameState;
use ffb_core::model::player::{Player, PlayerStats};
use ffb_core::model::team::Team;
use ffb_core::rng::GameRng;
use ffb_core::skills::SkillSet;
use ffb_core::types::{PlayerId, TeamId};
use ffb_sim::simulation::{NullStrategy, SimulationLoop};

fn make_full_state() -> GameState {
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
    GameState::new(home, away)
}

fn bench_null_game(c: &mut Criterion) {
    let home_strat = NullStrategy;
    let away_strat = NullStrategy;
    c.bench_function("game_null_strategy", |b| {
        b.iter(|| {
            let state = make_full_state();
            let mut rng = GameRng::new_live(42);
            SimulationLoop::run(state, &home_strat, &away_strat, &mut rng)
        });
    });
}

criterion_group!(benches, bench_null_game);
criterion_main!(benches);
