use criterion::{criterion_group, criterion_main, Criterion};
use ffb_core::model::game_state::GameState;
use ffb_core::model::player::{Player, PlayerStats};
use ffb_core::model::team::Team;
use ffb_core::skills::SkillSet;
use ffb_core::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

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
    let mut state = GameState::new(home, away);
    // Place all 22 players
    for i in 0..11u8 {
        state.field.place_player(
            PlayerId(format!("h{i}")),
            TeamId::Home,
            FieldCoordinate::new(i + 1, 5),
            PlayerState::Standing,
        );
        state.field.place_player(
            PlayerId(format!("a{i}")),
            TeamId::Away,
            FieldCoordinate::new(i + 14, 5),
            PlayerState::Standing,
        );
    }
    state
}

fn bench_state_clone(c: &mut Criterion) {
    let state = make_full_state();
    c.bench_function("state_clone", |b| {
        b.iter(|| state.clone());
    });
}

criterion_group!(benches, bench_state_clone);
criterion_main!(benches);
