use criterion::{criterion_group, criterion_main, Criterion};
use ffb_core::model::game_state::{ActingPlayer, GameState};
use ffb_core::model::player::{Player, PlayerStats};
use ffb_core::model::team::Team;
use ffb_core::pathfinding::find_paths;
use ffb_core::skills::SkillSet;
use ffb_core::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

fn make_pathfinding_state() -> (GameState, PlayerId) {
    let pid = PlayerId("p1".into());
    let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
    home.add_player(Player::new(
        pid.clone(),
        "Lineman".into(),
        "lineman".into(),
        TeamId::Home,
        1,
        PlayerStats::new(6, 3, 4, 8, None),
        SkillSet::empty(),
    ));
    let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
    let mut state = GameState::new(home, away);
    state.field.place_player(pid.clone(), TeamId::Home, FieldCoordinate::new(13, 8), PlayerState::Standing);
    state.acting_player = Some(ActingPlayer::new(pid.clone(), TeamId::Home));
    state.home_is_active = true;
    (state, pid)
}

fn bench_pathfinding(c: &mut Criterion) {
    let (state, pid) = make_pathfinding_state();
    let player = state.home.player_by_id(&pid).unwrap();
    let acting = state.acting_player.as_ref().unwrap();
    let movement_remaining = player.effective_ma().saturating_sub(acting.movement_used);

    c.bench_function("pathfinding_single_player", |b| {
        b.iter(|| {
            find_paths(&state.field, player, &pid, TeamId::Home, movement_remaining)
        });
    });
}

criterion_group!(benches, bench_pathfinding);
criterion_main!(benches);
