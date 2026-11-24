//! BACKLOG §P P3 gate: the server's model-change diff against real Java traffic.
//!
//! Every replay of the corpus is replayed with the Java model changes it recorded (`replay_observe`:
//! the same reader §O's state check uses). Between two consecutive syncs the Rust server would
//! have to produce a model-change list taking the earlier game to the later one; this runs
//! `model_change_diff::diff` on exactly that pair and proves the list by replaying it
//! (`self_check`), for the home view and -- via `transform_list` on the transformed games -- the
//! away coach's mirrored view. Uncovered paths are tallied: they are where the live server falls
//! back to sending the whole game.
use std::collections::BTreeMap;
use std::path::PathBuf;

use ffb_model::model::change::model_change_diff::{diff, self_check, self_check_with, transform_list};
use ffb_model::model::game::Game;

use super::ffbr;
use super::states::replay_observe;

/// The path with player ids / indexes collapsed, so the tally groups by kind.
fn kind(path: &str) -> String {
    let mut out = String::new();
    let mut skip = false;
    for c in path.chars() {
        match c {
            '[' => {
                skip = true;
                out.push_str("[*]");
            }
            ']' => skip = false,
            _ if !skip => out.push(c),
            _ => {}
        }
    }
    out
}

/// What Java's `Game.transform` / `FieldModel.transform` do not carry over (a fresh model's
/// defaults): the away client's copy keeps these from the changes it applied, a re-transformed
/// game resets them, so the away comparison leaves them out.
fn strip_untransformed(g: &mut Game) {
    g.concession_possible = false;
    g.last_defender_id = None;
    g.field_model.bomb_coordinate = None;
    g.field_model.bomb_moving = false;
    g.field_model.out_of_bounds = false;
    g.field_model.target_selection_state = None;
    g.field_model.chomped.clear();
}

pub fn sync_check(inputs: &[PathBuf]) -> i32 {
    let replays = ffbr::collect_inputs(inputs);
    let (mut syncs, mut exact, mut fallback, mut wrong, mut away_wrong, mut replay_errors) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let mut uncovered: BTreeMap<String, u64> = BTreeMap::new();
    let mut failures: BTreeMap<String, u64> = BTreeMap::new();
    for (n, (id, path)) in replays.iter().enumerate() {
        let mut prev: Option<Game> = None;
        let mut observer = |_i: usize, game: &Game| {
            if let Some(before) = prev.as_ref() {
                syncs += 1;
                match diff(before, game) {
                    Err(paths) => {
                        fallback += 1;
                        let mut kinds: Vec<String> = paths.iter().map(|p| kind(p)).collect();
                        kinds.sort();
                        kinds.dedup();
                        for k in kinds {
                            *uncovered.entry(k).or_default() += 1;
                        }
                    }
                    Ok(list) => match self_check(before, game, &list) {
                        Err(e) => {
                            wrong += 1;
                            if wrong <= 3 {
                                // The player entries the list got wrong, for the log.
                                let mut g = before.clone();
                                let processor = ffb_model::model::change::model_change_processor::ModelChangeProcessor::new();
                                for c in list.iter() {
                                    if let Ok(d) = ffb_model::model::change::model_change_processor::DecodedModelChange::decode(&g, c) {
                                        let _ = processor.apply(&mut g, &d);
                                    }
                                }
                                let pd = |x: &Game| x.to_json_value().ok().and_then(|v| v.pointer("/fieldModel/playerDataArray").cloned()).unwrap_or_default();
                                let (got, want) = (pd(&g), pd(game));
                                if let (Some(ga), Some(wa)) = (got.as_array(), want.as_array()) {
                                    for (x, y) in ga.iter().zip(wa) {
                                        if x != y {
                                            eprintln!("  WRONG {id}: got {x} want {y}");
                                        }
                                    }
                                }
                                let changes: Vec<String> = list.iter().map(|c| format!("{}:{:?}={}", c.change_id.get_name(), c.key, c.value)).collect();
                                eprintln!("  list: {}", changes.join(" | ").chars().take(600).collect::<String>());
                            }
                            *failures.entry(e.chars().take(160).collect()).or_default() += 1;
                        }
                        Ok(()) => {
                            exact += 1;
                            let away = transform_list(before, &list)
                                .and_then(|away| self_check_with(&before.transform(), &game.transform(), &away, strip_untransformed));
                            if let Err(e) = away {
                                away_wrong += 1;
                                *failures.entry(format!("away: {}", e.chars().take(150).collect::<String>())).or_default() += 1;
                            }
                        }
                    },
                }
            }
            prev = Some(game.clone());
        };
        let states = replay_observe(path, &mut observer);
        if let Some(e) = &states.error {
            replay_errors += 1;
            if replay_errors <= 5 {
                eprintln!("  {id}: {e}");
            }
        }
        if (n + 1) % 100 == 0 {
            eprintln!("  [{}/{}] syncs {syncs} exact {exact} fallback {fallback} wrong {wrong} away-wrong {away_wrong}", n + 1, replays.len());
        }
    }
    println!("replays {} (replay errors {replay_errors}), syncs {syncs}", replays.len());
    println!("  diff exact (home + away views): {}", exact - away_wrong);
    println!("  whole-game fallback (uncovered path): {fallback}");
    println!("  WRONG list (self-check failed): {wrong}, away view wrong: {away_wrong}");
    println!("uncovered paths (syncs):");
    let mut u: Vec<_> = uncovered.into_iter().collect();
    u.sort_by(|a, b| b.1.cmp(&a.1));
    for (k, v) in u.iter().take(40) {
        println!("  {v:>8}  {k}");
    }
    if !failures.is_empty() {
        println!("self-check failures:");
        let mut f: Vec<_> = failures.into_iter().collect();
        f.sort_by(|a, b| b.1.cmp(&a.1));
        for (k, v) in f.iter().take(20) {
            println!("  {v:>8}  {k}");
        }
    }
    // A wrong list would desynchronise a client; a fallback only costs bandwidth.
    if wrong == 0 && away_wrong == 0 { 0 } else { 1 }
}
