mod runner;
mod log_format;
mod state_hash;
mod comparator;
mod update_progress;
mod network_test;
#[allow(dead_code)] mod debug_rng;

use std::collections::HashMap;
use serde::Serialize;
use rand_core::SeedableRng;
use ffb_engine::agent::{RandomAgent, Agent};
use ffb_engine::engine::GameEngine;
use ffb_engine::legal_actions::{legal_block_targets, legal_move_targets, legal_foul_targets, TeamSide};
use ffb_engine::action::Action;
use ffb_mechanics::mechanics::path_probability::{find_all_paths, OpponentOnField, PathContext, PlayerMoveContext};
use ffb_model::types::FieldCoordinate;
use ffb_model::events::GameEvent;
use ffb_model::enums::{PlayerAction, Rules, SeriousInjuryKind};
use ffb_mechanics::skills::{SKILL_TABLE, SkillId};
use runner::make_team_from_roster;

// ── Data structures ────────────────────────────────────────────────────────────

#[derive(Default, Serialize)]
struct RollStats {
    total: u32,
    success: u32,
    failure: u32,
    rerolled: u32,
}

impl RollStats {
    fn record(&mut self, success: bool, rerolled: bool) {
        self.total += 1;
        if success { self.success += 1; } else { self.failure += 1; }
        if rerolled { self.rerolled += 1; }
    }
}

#[derive(Default, Serialize)]
struct BlockStats {
    total: u32,
    rerolled: u32,
    by_dice: HashMap<i32, u32>,
}

#[derive(Default, Serialize)]
struct PassStats {
    total: u32,
    rerolled: u32,
    by_distance: HashMap<String, u32>,
    by_result: HashMap<String, u32>,
    by_distance_and_result: HashMap<String, u32>,
}

#[derive(Default, Serialize)]
struct InjuryStats {
    total: u32,
    armor_only: u32,
    ko: u32,
    cas: u32,
    dead: u32,
    by_serious_injury: HashMap<String, u32>,
}

#[derive(Default, Serialize)]
struct CoverageReport {
    games: u32,
    home_wins: u32,
    away_wins: u32,
    draws: u32,
    touchdowns_home: u32,
    touchdowns_away: u32,
    matchups: Vec<MatchupSummary>,

    // Player activations
    activations: HashMap<String, u32>,

    // Rolls
    dodge_rolls: RollStats,
    go_for_it_rolls: RollStats,
    catch_rolls: RollStats,
    interception_rolls: RollStats,
    jump_rolls: RollStats,
    jump_up_rolls: RollStats,
    dauntless_rolls: RollStats,
    loner_rolls: RollStats,
    pro_rolls: RollStats,
    foul_appearance_rolls: RollStats,
    always_hungry_rolls: RollStats,
    blood_lust_rolls: RollStats,
    animosity_rolls: RollStats,
    confusion_rolls: RollStats,
    hypnotic_gaze_rolls: RollStats,
    escape_rolls: RollStats,
    right_stuff_rolls: RollStats,
    safe_throw_rolls: RollStats,
    stand_up_rolls: RollStats,
    pick_me_up_rolls: RollStats,
    breathe_fire_rolls: RollStats,
    projectile_vomit_rolls: RollStats,
    baleful_hex_rolls: RollStats,
    look_into_my_eyes_rolls: RollStats,
    weeping_dagger_rolls: RollStats,

    block_rolls: BlockStats,
    pass_rolls: PassStats,

    // Injuries
    injuries: InjuryStats,

    // Skills
    skill_used: HashMap<String, u32>,
    skill_declined: HashMap<String, u32>,
    skill_names: HashMap<String, String>,

    // Kickoff
    kickoff_events: HashMap<String, u32>,
    kickoff_pitch_invasions: u32,
    kickoff_rocks_thrown: u32,
    kickoff_riots: u32,

    // Ball events
    touchdowns: u32,
    throw_ins: u32,
    scatter_balls: u32,
    pass_deviates: u32,
    touchbacks: u32, // from AgentPrompt::Touchback (not a GameEvent — tracked from prompts)
    ball_picked_up: u32,
    kickoff_scatters: u32,

    // Blocks / pushes
    // (chain_pushes removed: squares.len() > 1 means "multiple push destinations", not chain)
    // (crowd_pushes removed: squares are filtered to on-pitch by engine, no off-pitch coord present)
    total_pushbacks: u32,
    push_destinations_hist: HashMap<String, u32>, // "1" | "2" | "3+" → count

    // Fouls
    fouls: u32,
    players_ejected: u32,
    argue_the_call_rolls: RollStats,
    bribes_rolls: RollStats,

    // Movement
    players_fell_down: u32,
    player_moved_events: u32,
    scatter_players: u32,

    // Inducements / misc
    apothecary_used: u32,
    inducements_bought: HashMap<String, u32>,
    wizard_used: u32,
    cards_played: u32,
    secret_weapon_bans: u32,

    // Re-rolls
    rerolls_by_source: HashMap<String, u32>,
    rerolls_total: u32,

    // Weather / game flow
    weather_changes: u32,
    heat_exhaustion: u32,
    turn_ends: u32,
    half_starts: u32,
    coin_throws: u32,
    mvp_rolls: u32,

    // Misc rolls
    chainsaw_rolls: u32,
    trap_door_rolls: u32,
    piling_on_events: u32,
    animal_savagery_rolls: RollStats,
    team_captain_saves: u32,
    swarming_rolls: u32,
    bomb_explosions: u32,
    throw_team_mate_rolls: u32,
}

#[derive(Serialize)]
struct MatchupSummary {
    home: String,
    away: String,
    seeds: u32,
    home_wins: u32,
    away_wins: u32,
    draws: u32,
    touchdowns_home: u32,
    touchdowns_away: u32,
}

impl CoverageReport {
    fn tally(&mut self, ev: &GameEvent) {
        match ev {
            GameEvent::DodgeRoll { success, rerolled, .. } =>
                self.dodge_rolls.record(*success, *rerolled),
            GameEvent::GoForItRoll { success, rerolled, .. } =>
                self.go_for_it_rolls.record(*success, *rerolled),
            GameEvent::CatchRoll { success, rerolled, .. } =>
                self.catch_rolls.record(*success, *rerolled),
            GameEvent::InterceptionRoll { success, .. } =>
                self.interception_rolls.record(*success, false),
            GameEvent::JumpRoll { success, .. } =>
                self.jump_rolls.record(*success, false),
            GameEvent::JumpUpRoll { success, .. } =>
                self.jump_up_rolls.record(*success, false),
            GameEvent::DauntlessRoll { success, .. } =>
                self.dauntless_rolls.record(*success, false),
            GameEvent::LonerRoll { success, .. } =>
                self.loner_rolls.record(*success, false),
            GameEvent::ProRoll { success, .. } =>
                self.pro_rolls.record(*success, false),
            GameEvent::FoulAppearanceRoll { failed, .. } =>
                self.foul_appearance_rolls.record(!failed, false),
            GameEvent::AlwaysHungry { success, .. } =>
                self.always_hungry_rolls.record(*success, false),
            GameEvent::BloodLustRoll { success, .. } =>
                self.blood_lust_rolls.record(*success, false),
            GameEvent::AnimosityRoll { success, .. } =>
                self.animosity_rolls.record(*success, false),
            GameEvent::ConfusionRoll { confused, .. } =>
                self.confusion_rolls.record(!confused, false),
            GameEvent::HypnoticGazeRoll { success, .. } =>
                self.hypnotic_gaze_rolls.record(*success, false),
            GameEvent::EscapeRoll { success, .. } =>
                self.escape_rolls.record(*success, false),
            GameEvent::RightStuffRoll { success, .. } =>
                self.right_stuff_rolls.record(*success, false),
            GameEvent::SafeThrowRoll { success, .. } =>
                self.safe_throw_rolls.record(*success, false),
            GameEvent::StandUpRoll { success, .. } =>
                self.stand_up_rolls.record(*success, false),
            GameEvent::PickMeUpRoll { success, .. } =>
                self.pick_me_up_rolls.record(*success, false),
            GameEvent::BreatheFireRoll { knock_down, rerolled, .. } =>
                self.breathe_fire_rolls.record(*knock_down, *rerolled),
            GameEvent::ProjectileVomitRoll { success, rerolled, .. } =>
                self.projectile_vomit_rolls.record(*success, *rerolled),
            GameEvent::BalefulHexRoll { success, rerolled, .. } =>
                self.baleful_hex_rolls.record(*success, *rerolled),
            GameEvent::LookIntoMyEyesRoll { success, rerolled, .. } =>
                self.look_into_my_eyes_rolls.record(*success, *rerolled),
            GameEvent::WeepingDaggerRoll { .. } => {
                self.weeping_dagger_rolls.total += 1;
            }
            GameEvent::ArgueTheCall { success, .. } =>
                self.argue_the_call_rolls.record(*success, false),
            GameEvent::BribesRoll { success, .. } =>
                self.bribes_rolls.record(*success, false),
            GameEvent::AnimalSavagery { success, .. } =>
                self.animal_savagery_rolls.record(*success, false),

            GameEvent::BlockRoll { nr_of_dice, rerolled, .. } => {
                self.block_rolls.total += 1;
                if *rerolled { self.block_rolls.rerolled += 1; }
                *self.block_rolls.by_dice.entry(*nr_of_dice).or_default() += 1;
            }

            GameEvent::PassRoll { distance, result, rerolled, .. } => {
                self.pass_rolls.total += 1;
                if *rerolled { self.pass_rolls.rerolled += 1; }
                *self.pass_rolls.by_distance.entry(distance.name().to_string()).or_default() += 1;
                *self.pass_rolls.by_result.entry(result.name().to_string()).or_default() += 1;
                let key = format!("{}|{}", distance.name(), result.name());
                *self.pass_rolls.by_distance_and_result.entry(key).or_default() += 1;
            }

            GameEvent::ThrowTeamMateRoll { .. } => {
                self.throw_team_mate_rolls += 1;
            }

            GameEvent::Injury { was_ko, was_cas, serious_injury, armor_roll, injury_roll, .. } => {
                self.injuries.total += 1;
                if *was_ko { self.injuries.ko += 1; }
                if *was_cas { self.injuries.cas += 1; }
                if let Some(si) = serious_injury {
                    if matches!(si, SeriousInjuryKind::Dead) {
                        self.injuries.dead += 1;
                    }
                    *self.injuries.by_serious_injury.entry(format!("{si:?}")).or_default() += 1;
                }
                if armor_roll.is_some() && injury_roll.is_none() && !was_ko && !was_cas {
                    self.injuries.armor_only += 1;
                }
            }

            GameEvent::PlayerAction { action, .. } => {
                let name = player_action_name(action);
                *self.activations.entry(name.to_string()).or_default() += 1;
            }

            GameEvent::PlayerMoved { .. } => { self.player_moved_events += 1; }
            GameEvent::PlayerFellDown { .. } => { self.players_fell_down += 1; }

            GameEvent::Pushback { squares, .. } => {
                self.total_pushbacks += 1;
                let bucket = match squares.len() {
                    1 => "1",
                    2 => "2",
                    _ => "3+",
                };
                *self.push_destinations_hist.entry(bucket.to_string()).or_default() += 1;
            }

            GameEvent::ScatterBall { .. } => { self.scatter_balls += 1; }
            GameEvent::ScatterPlayer { .. } => { self.scatter_players += 1; }
            GameEvent::ThrowIn { .. } => { self.throw_ins += 1; }
            GameEvent::PassDeviate { .. } => { self.pass_deviates += 1; }
            GameEvent::BallPickedUp { .. } => { self.ball_picked_up += 1; }
            GameEvent::KickoffScatter { .. } => { self.kickoff_scatters += 1; }

            GameEvent::Touchdown { .. } => { self.touchdowns += 1; }

            GameEvent::KickoffResultEvent { result } => {
                *self.kickoff_events.entry(result.name().to_string()).or_default() += 1;
            }
            GameEvent::KickoffPitchInvasion { .. } => { self.kickoff_pitch_invasions += 1; }
            GameEvent::KickoffRiot => { self.kickoff_riots += 1; }
            GameEvent::KickoffThrowARock { .. } => { self.kickoff_rocks_thrown += 1; }

            GameEvent::SkillUse { skill_id, used, .. } => {
                let name = skill_name_from_u16(*skill_id);
                if *used {
                    *self.skill_used.entry(name).or_default() += 1;
                } else {
                    *self.skill_declined.entry(name).or_default() += 1;
                }
            }

            GameEvent::Foul { .. } => { self.fouls += 1; }
            GameEvent::PlayerEjected { .. } => { self.players_ejected += 1; }

            GameEvent::ReRoll { source, .. } => {
                self.rerolls_total += 1;
                *self.rerolls_by_source.entry(source.name.clone()).or_default() += 1;
            }

            GameEvent::ApothecaryChoice { healed, .. } => {
                if *healed { self.apothecary_used += 1; }
            }

            GameEvent::BuyInducement { inducement_id, .. } => {
                *self.inducements_bought.entry(inducement_id.clone()).or_default() += 1;
            }

            GameEvent::WizardUse { .. } => { self.wizard_used += 1; }
            GameEvent::PlayCard { .. } => { self.cards_played += 1; }
            GameEvent::SecretWeaponBan { .. } => { self.secret_weapon_bans += 1; }

            GameEvent::WeatherChange { .. } => { self.weather_changes += 1; }
            GameEvent::HeatExhaustion { .. } => { self.heat_exhaustion += 1; }
            GameEvent::TurnEnd { .. } => { self.turn_ends += 1; }
            GameEvent::StartHalf { .. } => { self.half_starts += 1; }
            GameEvent::CoinThrow { .. } => { self.coin_throws += 1; }
            GameEvent::MvpRoll { .. } => { self.mvp_rolls += 1; }

            GameEvent::ChainsawRoll { .. } => { self.chainsaw_rolls += 1; }
            GameEvent::TrapDoor { .. } => { self.trap_door_rolls += 1; }
            GameEvent::PilingOn { .. } => { self.piling_on_events += 1; }
            GameEvent::TeamCaptainRoll { reroll_saved, .. } => {
                if *reroll_saved { self.team_captain_saves += 1; }
            }
            GameEvent::SwarmingPlayersRoll { .. } => { self.swarming_rolls += 1; }
            GameEvent::BombExplodesAfterCatch { .. } | GameEvent::BombOutOfBounds { .. } => {
                self.bomb_explosions += 1;
            }

            GameEvent::RegenerationRoll { success, .. } => {
                // Tally under a generic category — rare enough to not need its own field
                let _ = success;
            }
            GameEvent::BallScattered { .. } => {
                self.scatter_balls += 1;
            }

            // Game flow events not tallied individually
            GameEvent::ReceiveChoice { .. }
            | GameEvent::GameOptions { .. }
            | GameEvent::DoubleHiredStarPlayer
            | GameEvent::TimeoutEnforced { .. }
            | GameEvent::WinningsRoll { .. }
            | GameEvent::HandOver { .. }
            | GameEvent::SwoopPlayer { .. }
            | GameEvent::RiotousRookies { .. }
            | GameEvent::KickoffPitchInvasionStun { .. }
            | GameEvent::PassBlock { .. }
            | GameEvent::PassBlockEligible { .. }
            | GameEvent::PettyCash { .. }
            | GameEvent::CardDeactivated { .. }
            | GameEvent::CardEffectRoll { .. }
            | GameEvent::DefectingPlayers { .. }
            | GameEvent::CoachBanned { .. }
            | GameEvent::Leader { .. }
            | GameEvent::ThenIStartedBlastin { .. }
            | GameEvent::PrayerRoll { .. }
            | GameEvent::SpellEffectRoll { .. }
            | GameEvent::MasterChefRoll { .. }
            => {}
        }
    }
}

fn player_action_name(action: &PlayerAction) -> &'static str {
    match action {
        PlayerAction::Move | PlayerAction::BlitzMove | PlayerAction::HandOverMove
        | PlayerAction::PassMove | PlayerAction::FoulMove | PlayerAction::GazeMove
        | PlayerAction::KickTeamMateMove | PlayerAction::ThrowTeamMateMove
        | PlayerAction::PuntMove | PlayerAction::PutridRegurgitationMove => "Move",
        PlayerAction::Block | PlayerAction::PutridRegurgitationBlock
        | PlayerAction::KickEmBlock => "Block",
        PlayerAction::Blitz | PlayerAction::BlitzSelect | PlayerAction::StandUpBlitz
        | PlayerAction::PutridRegurgitationBlitz | PlayerAction::KickEmBlitz => "Blitz",
        PlayerAction::Pass => "Pass",
        PlayerAction::HandOver => "HandOver",
        PlayerAction::Foul => "Foul",
        PlayerAction::StandUp => "StandUp",
        PlayerAction::ThrowTeamMate => "ThrowTeamMate",
        PlayerAction::RemoveConfusion => "RemoveConfusion",
        PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::AutoGazeZoat => "HypnoticGaze",
        PlayerAction::MultipleBlock => "MultipleBlock",
        PlayerAction::HailMaryPass => "HailMaryPass",
        PlayerAction::DumpOff => "DumpOff",
        PlayerAction::ThrowBomb => "ThrowBomb",
        PlayerAction::HailMaryBomb => "HailMaryBomb",
        PlayerAction::Swoop => "Swoop",
        PlayerAction::KickTeamMate => "KickTeamMate",
        PlayerAction::Treacherous => "Treacherous",
        PlayerAction::WisdomOfTheWhiteDwarf => "WisdomOfTheWhiteDwarf",
        PlayerAction::ThrowKeg => "ThrowKeg",
        PlayerAction::RaidingParty => "RaidingParty",
        PlayerAction::MaximumCarnage => "MaximumCarnage",
        PlayerAction::LookIntoMyEyes => "LookIntoMyEyes",
        PlayerAction::BalefulHex => "BalefulHex",
        PlayerAction::AllYouCanEat => "AllYouCanEat",
        PlayerAction::BlackInk => "BlackInk",
        PlayerAction::CatchOfTheDay => "CatchOfTheDay",
        PlayerAction::ThenIStartedBlastin => "ThenIStartedBlastin",
        PlayerAction::TheFlashingBlade => "TheFlashingBlade",
        PlayerAction::ViciousVines => "ViciousVines",
        PlayerAction::FuriousOutburst => "FuriousOutburst",
        PlayerAction::SecureTheBall => "SecureTheBall",
        PlayerAction::BreatheFire => "BreatheFire",
        PlayerAction::Chainsaw => "Chainsaw",
        PlayerAction::Stab => "Stab",
        PlayerAction::ProjectileVomit => "ProjectileVomit",
        PlayerAction::Chomp => "Chomp",
        PlayerAction::Punt => "Punt",
        PlayerAction::Forgo => "Forgo",
        PlayerAction::Incorporeal => "Incorporeal",
    }
}

fn skill_name_from_u16(id: u16) -> String {
    SKILL_TABLE.get(id as usize)
        .map(|s| s.id.class_name().to_string())
        .unwrap_or_else(|| format!("Unknown({})", id))
}

/// After an activation, attempt skill-based special actions that aren't offered via eligible_players.
/// These actions (HypnoticGaze, BreatheFire, ProjectileVomit) must be sent directly by the client.
fn try_skill_actions(engine: &mut GameEngine, side: TeamSide, pid: &str,
    rng: &mut rand_xoshiro::Xoshiro256StarStar, events: &mut Vec<GameEvent>) {
    let side_team = if side == TeamSide::Home { &engine.game.team_home } else { &engine.game.team_away };
    let player = match side_team.player(pid) { Some(p) => p.clone(), None => return };

    // HypnoticGaze (Vampire, Zoat): can gaze any standing opponent on the pitch
    if player.has_skill(SkillId::HypnoticGaze) && !player.used_skills.contains(&SkillId::HypnoticGaze) {
        if let Some(target_id) = any_opponent_on_pitch(&engine.game, pid, side, rng) {
            if let Ok(evs) = engine.apply(side, Action::HypnoticGaze { target_id }) {
                events.extend(evs);
            }
        }
    }

    // BreatheFire (Lizardman Kroxigor): targets an adjacent standing opponent
    if player.has_skill(SkillId::BreatheFire) && !player.used_skills.contains(&SkillId::BreatheFire) {
        let targets = legal_block_targets(&engine.game, pid, side);
        if !targets.is_empty() {
            let idx = (rand_core::RngCore::next_u64(rng) as usize) % targets.len();
            if let Ok(evs) = engine.apply(side, Action::BreatheFire { target_id: targets[idx].clone() }) {
                events.extend(evs);
            }
        }
    }

    // Read turn flags into locals before any mutable borrow of engine
    let (pass_used, ttm_used) = {
        let td = if side == TeamSide::Home { &engine.game.turn_data_home } else { &engine.game.turn_data_away };
        (td.pass_used, td.ttm_used)
    };

    // ThrowBomb (Bombardier): throw a bomb at a random on-pitch location
    if player.has_skill(SkillId::Bombardier) && !pass_used {
        let coord = random_pass_coord(&engine.game, pid, side, rng);
        if let Ok(evs) = engine.apply(side, Action::ThrowBomb { coord }) {
            events.extend(evs);
        }
    }

    // ThrowTeamMate: Troll/Ogre throws an adjacent small teammate
    if player.has_skill(SkillId::ThrowTeamMate) && !ttm_used {
        if let Some((thrown_id, coord)) = find_throw_target(&engine.game, pid, side, rng) {
            if let Ok(evs) = engine.apply(side, Action::ThrowTeamMate { player_id: thrown_id, coord }) {
                events.extend(evs);
            }
        }
    }

    // ProjectileVomit (Nurgle Rotspawn): targets an adjacent standing opponent
    if player.has_skill(SkillId::ProjectileVomit) && !player.used_skills.contains(&SkillId::ProjectileVomit) {
        let targets = legal_block_targets(&engine.game, pid, side);
        if !targets.is_empty() {
            let idx = (rand_core::RngCore::next_u64(rng) as usize) % targets.len();
            if let Ok(evs) = engine.apply(side, Action::ProjectileVomit { target_id: targets[idx].clone() }) {
                events.extend(evs);
            }
        }
    }
}

/// Build a path for Leap players: allows stepping into occupied squares (triggers JumpRoll).
/// Use find_all_paths to enumerate every square the player can reach this activation,
/// correctly handling MA, GoForIt, Sprint (extra_gfi=1), BurstOfSpeed, dodge modifiers etc.
/// Returns the path to a randomly chosen reachable destination.
fn all_reachable_path(game: &ffb_model::model::game::Game, player_id: &str,
    side: TeamSide, rng: &mut rand_xoshiro::Xoshiro256StarStar) -> Vec<FieldCoordinate> {
    let is_home = matches!(side, TeamSide::Home);
    let my_team  = if is_home { &game.team_home  } else { &game.team_away };
    let opp_team = if is_home { &game.team_away  } else { &game.team_home };
    let player = match my_team.player(player_id) { Some(p) => p, None => return vec![] };
    let coord  = match game.field_model.player_coordinate(player_id) { Some(c) => c, None => return vec![] };

    let occupied: std::collections::HashSet<FieldCoordinate> = my_team.players.iter()
        .chain(opp_team.players.iter())
        .filter(|pl| pl.id != *player_id)
        .filter_map(|pl| game.field_model.player_coordinate(&pl.id))
        .collect();

    let opponents: Vec<OpponentOnField> = opp_team.players.iter().filter_map(|q| {
        let qc = game.field_model.player_coordinate(&q.id)?;
        let qs = game.field_model.player_state(&q.id)?;
        Some(OpponentOnField {
            coord: qc,
            has_tackle_zones: qs.has_tacklezones(),
            has_diving_tackle: q.has_skill(SkillId::DivingTackle),
            has_prehensile_tail: q.has_skill(SkillId::PrehensileTail),
            has_disturbing_presence: q.has_skill(SkillId::DisturbingPresence),
            is_titchy: q.has_skill(SkillId::Titchy),
        })
    }).collect();

    let extra_gfi = if player.has_skill(SkillId::Sprint) { 1 }
        else if player.has_skill(SkillId::BurstOfSpeed) && !player.used_skills.contains(&SkillId::BurstOfSpeed) { 1 }
        else { 0 };

    let player_ctx = PlayerMoveContext {
        start: coord,
        movement_allowance: player.movement,
        current_move: game.acting_player.current_move,
        agility: player.agility,
        strength: player.strength,
        rules: game.rules,
        has_two_heads: player.has_skill(SkillId::TwoHeads),
        ignore_tackle_zones: player.has_skill(SkillId::Incorporeal),
        has_break_tackle: player.has_skill(SkillId::BreakTackle),
        gfi_modifier_total: 0, // weather effects on GFI omitted for simplicity
        extra_gfi,
    };
    let field_ctx = PathContext { occupied, opponents };
    let path_map = find_all_paths(&player_ctx, &field_ctx);
    if path_map.is_empty() { return vec![]; }

    // Uniform random selection from all reachable squares.
    let mut dests: Vec<FieldCoordinate> = path_map.keys().cloned().collect();
    dests.sort_by_key(|c| (c.x, c.y)); // deterministic ordering before random pick
    let idx = (rand_core::RngCore::next_u64(rng) as usize) % dests.len();
    path_map[&dests[idx]].path.clone()
}

fn build_leap_path(game: &ffb_model::model::game::Game, player_id: &str,
    rng: &mut rand_xoshiro::Xoshiro256StarStar, max_steps: usize) -> Vec<FieldCoordinate> {
    let mut path: Vec<FieldCoordinate> = Vec::new();
    let mut current = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return path,
    };
    let mut visited = std::collections::HashSet::new();
    visited.insert(current);
    for _ in 0..max_steps {
        // Include occupied squares (player will Leap over them if destination is occupied)
        let candidates: Vec<FieldCoordinate> = current.neighbours()
            .into_iter()
            .filter(|c| c.is_on_pitch() && !visited.contains(c))
            .collect();
        if candidates.is_empty() { break; }
        let idx = (rand_core::RngCore::next_u64(rng) as usize) % candidates.len();
        current = candidates[idx];
        visited.insert(current);
        path.push(current);
        // Stop after landing on an occupied square (player falls prone if leap fails)
        if game.field_model.player_at(current).is_some() { break; }
    }
    path
}

/// (Kept for reference — replaced by all_reachable_path for normal moves.)
#[allow(dead_code)]
fn build_random_path_legacy(game: &ffb_model::model::game::Game, player_id: &str,
    rng: &mut rand_xoshiro::Xoshiro256StarStar, max_steps: usize) -> Vec<FieldCoordinate> {
    let mut path: Vec<FieldCoordinate> = Vec::new();
    let mut current = match game.field_model.player_coordinate(player_id) {
        Some(c) => c,
        None => return path,
    };
    let mut visited = std::collections::HashSet::new();
    visited.insert(current);
    for _ in 0..max_steps {
        let candidates: Vec<FieldCoordinate> = current.neighbours()
            .into_iter()
            .filter(|c| c.is_on_pitch() && game.field_model.player_at(*c).is_none() && !visited.contains(c))
            .collect();
        if candidates.is_empty() { break; }
        let idx = (rand_core::RngCore::next_u64(rng) as usize) % candidates.len();
        current = candidates[idx];
        visited.insert(current);
        path.push(current);
    }
    path
}

/// Pick a random on-pitch coordinate to throw a pass to (tries to find a square with a
/// teammate on it for a catch chance; falls back to a random pitch square).
fn random_pass_coord(game: &ffb_model::model::game::Game, player_id: &str,
    side: TeamSide, rng: &mut rand_xoshiro::Xoshiro256StarStar) -> FieldCoordinate {
    let same_team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    // Try to throw to a teammate
    let teammates: Vec<FieldCoordinate> = same_team.players.iter()
        .filter(|p| p.id != *player_id)
        .filter_map(|p| game.field_model.player_coordinate(&p.id))
        .filter(|c| c.is_on_pitch())
        .collect();
    if !teammates.is_empty() {
        let idx = (rand_core::RngCore::next_u64(rng) as usize) % teammates.len();
        return teammates[idx];
    }
    // Fallback: random on-pitch coordinate
    let x = ((rand_core::RngCore::next_u64(rng) as i32).abs() % 26) as i32;
    let y = ((rand_core::RngCore::next_u64(rng) as i32).abs() % 14 + 1) as i32;
    FieldCoordinate::new(x, y)
}

/// Find an adjacent throwable teammate (has RightStuff/Stunty) and a random landing coord.
/// Used for ThrowTeamMate and KickTeamMate actions.
fn find_throw_target(game: &ffb_model::model::game::Game, thrower_id: &str,
    side: TeamSide, rng: &mut rand_xoshiro::Xoshiro256StarStar) -> Option<(String, FieldCoordinate)> {
    let thrower_coord = game.field_model.player_coordinate(thrower_id)?;
    let same_team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    // Find adjacent teammates that can be thrown (RightStuff or Stunty skill)
    let throwable: Vec<String> = same_team.players.iter()
        .filter(|p| p.id != *thrower_id)
        .filter(|p| p.has_skill(SkillId::RightStuff) || p.has_skill(SkillId::Stunty))
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(thrower_coord))
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    if throwable.is_empty() { return None; }
    let idx = (rand_core::RngCore::next_u64(rng) as usize) % throwable.len();
    let thrown_id = throwable[idx].clone();
    // Throw to a random on-pitch square away from the thrower
    let x = ((rand_core::RngCore::next_u64(rng) as i32).abs() % 26).max(0);
    let y = ((rand_core::RngCore::next_u64(rng) as i32).abs() % 14 + 1).max(1);
    Some((thrown_id, FieldCoordinate::new(x, y)))
}

/// Find any opponent on the pitch (for ranged actions like HypnoticGaze).
fn any_opponent_on_pitch(game: &ffb_model::model::game::Game, _player_id: &str,
    side: TeamSide, rng: &mut rand_xoshiro::Xoshiro256StarStar) -> Option<String> {
    let opponent_team = match side {
        TeamSide::Home => &game.team_away,
        TeamSide::Away => &game.team_home,
    };
    let candidates: Vec<String> = opponent_team.players.iter()
        .filter(|p| game.field_model.player_coordinate(&p.id).is_some())
        .filter(|p| game.field_model.player_state(&p.id).map(|s| s.has_tacklezones()).unwrap_or(false))
        .map(|p| p.id.clone())
        .collect();
    if candidates.is_empty() { return None; }
    let idx = (rand_core::RngCore::next_u64(rng) as usize) % candidates.len();
    Some(candidates[idx].clone())
}

/// Find a random adjacent teammate who can receive a hand-off.
fn adjacent_teammate(game: &ffb_model::model::game::Game, player_id: &str,
    side: TeamSide) -> Option<String> {
    let coord = game.field_model.player_coordinate(player_id)?;
    let same_team = match side {
        TeamSide::Home => &game.team_home,
        TeamSide::Away => &game.team_away,
    };
    let candidates: Vec<String> = same_team.players.iter()
        .filter(|p| p.id != *player_id)
        .filter(|p| {
            game.field_model.player_coordinate(&p.id)
                .map(|c| c.is_adjacent(coord))
                .unwrap_or(false)
        })
        .map(|p| p.id.clone())
        .collect();
    candidates.into_iter().next()
}

fn edition_to_rules(edition: &str) -> Rules {
    match edition {
        "bb2016" => Rules::Bb2016,
        "bb2020" => Rules::Bb2020,
        _ => Rules::Bb2025,
    }
}

// ── Game loop ──────────────────────────────────────────────────────────────────

fn run_coverage_game(seed: u64, home_roster: &str, away_roster: &str, edition: &str) -> (Vec<GameEvent>, i32, i32, u32) {
    let rules = edition_to_rules(edition);
    let home = match make_team_from_roster(home_roster, "home", edition) {
        Ok(t) => t,
        Err(e) => { eprintln!("team error: {e}"); return (vec![], 0, 0, 0); }
    };
    let away = match make_team_from_roster(away_roster, "away", edition) {
        Ok(t) => t,
        Err(e) => { eprintln!("team error: {e}"); return (vec![], 0, 0, 0); }
    };

    let mut engine = GameEngine::new(home, away, rules, seed);
    // Use RandomAgent (not ParityAgent) so players actually get activated and actions complete.
    let mut home_rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(seed ^ 0xC0FFEE_0001);
    let mut away_rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(seed ^ 0xC0FFEE_0002);
    let mut home_agent = RandomAgent::new(seed);
    let mut away_agent = RandomAgent::new(seed ^ 0xFFFF_FFFF);
    let mut all_events: Vec<GameEvent> = Vec::new();
    let mut all_touchbacks: u32 = 0;

    for _ in 0..200_000 {
        if engine.is_finished() { break; }
        let prompt = match engine.current_prompt() { Some(p) => p.clone(), None => break };
        // Track Touchback prompts (not a GameEvent, only observable as a prompt)
        if matches!(prompt, ffb_model::prompts::AgentPrompt::Touchback { .. }) {
            all_touchbacks += 1;
        }
        let side = engine.active_side();
        let response = if side == TeamSide::Home {
            home_agent.respond(&prompt)
        } else {
            away_agent.respond(&prompt)
        };
        // Clone before consuming in response_to_action_pub
        let activated_action = match &response {
            ffb_model::prompts::AgentResponse::ActivatePlayer { action, .. } => Some(*action),
            _ => None,
        };
        let action = ffb_engine::agent::response_to_action_pub(response, Some(&prompt));
        match engine.apply(side, action) {
            Ok(evs) => all_events.extend(evs),
            Err(e) => { eprintln!("engine error seed {seed}: {e}"); break; }
        }

        // After a real ActivatePlayer selection, send the follow-up action that the headless
        // engine doesn't prompt for (path selection, block/foul/pass/handoff target).
        if let Some(act) = activated_action {
            if let Some(pid) = engine.game.acting_player.player_id.clone() {
                let rng = if side == TeamSide::Home { &mut home_rng } else { &mut away_rng };
                match act {
                    // Pure move: use find_all_paths to enumerate every reachable square,
                    // which correctly accounts for MA, GFI limit, Sprint (extra_gfi=1),
                    // BurstOfSpeed, dodge probability etc. Pick a random destination.
                    PlayerAction::Move | PlayerAction::BlitzMove => {
                        let side_team = if side == TeamSide::Home { &engine.game.team_home } else { &engine.game.team_away };
                        let has_leap = side_team.player(&pid).map(|p|
                            p.has_skill(SkillId::Leap) || p.has_skill(SkillId::PogoStick) || p.has_skill(SkillId::Pogo)
                        ).unwrap_or(false);
                        let path = if has_leap {
                            // Leap can enter occupied squares — find_all_paths blocks those,
                            // so use the custom leap-aware path builder instead.
                            build_leap_path(&engine.game, &pid, rng, 3)
                        } else {
                            all_reachable_path(&engine.game, &pid, side, rng)
                        };
                        if !path.is_empty() {
                            if let Ok(evs) = engine.apply(side, Action::Move { path }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // Block/Blitz: execute block against a random adjacent standing opponent
                    PlayerAction::Block | PlayerAction::Blitz | PlayerAction::StandUpBlitz => {
                        let targets = legal_block_targets(&engine.game, &pid, side);
                        if !targets.is_empty() {
                            let idx = (rand_core::RngCore::next_u64(rng) as usize) % targets.len();
                            let defender_id = targets[idx].clone();
                            if let Ok(evs) = engine.apply(side, Action::Block { defender_id }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // Stab/Chainsaw: same target selection as block
                    PlayerAction::Stab | PlayerAction::Chainsaw => {
                        let targets = legal_block_targets(&engine.game, &pid, side);
                        if !targets.is_empty() {
                            let idx = (rand_core::RngCore::next_u64(rng) as usize) % targets.len();
                            let defender_id = targets[idx].clone();
                            let action = if act == PlayerAction::Stab {
                                Action::Stab { defender_id }
                            } else {
                                Action::Block { defender_id }
                            };
                            if let Ok(evs) = engine.apply(side, action) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // StandUp: send a 1-square move so the engine can process the stand-up roll
                    // (players with MA ≤ 3 roll 4+ to stand; the roll fires during apply_move)
                    PlayerAction::StandUp => {
                        let targets = legal_move_targets(&engine.game, &pid);
                        if !targets.is_empty() {
                            let idx = (rand_core::RngCore::next_u64(rng) as usize) % targets.len();
                            let dest = targets[idx];
                            if let Ok(evs) = engine.apply(side, Action::Move { path: vec![dest] }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // HypnoticGaze (Vampire/Zoat): pick any opponent on the pitch
                    PlayerAction::Gaze | PlayerAction::GazeSelect | PlayerAction::GazeMove
                    | PlayerAction::AutoGazeZoat => {
                        if let Some(target_id) = any_opponent_on_pitch(&engine.game, &pid, side, rng) {
                            if let Ok(evs) = engine.apply(side, Action::HypnoticGaze { target_id }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // ProjectileVomit (Nurgle): target adjacent standing opponent
                    PlayerAction::ProjectileVomit => {
                        let targets = legal_block_targets(&engine.game, &pid, side);
                        if !targets.is_empty() {
                            let idx = (rand_core::RngCore::next_u64(rng) as usize) % targets.len();
                            let target_id = targets[idx].clone();
                            if let Ok(evs) = engine.apply(side, Action::ProjectileVomit { target_id }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // BreatheFire (Lizardman Kroxigor): target adjacent standing opponent
                    PlayerAction::BreatheFire => {
                        let targets = legal_block_targets(&engine.game, &pid, side);
                        if !targets.is_empty() {
                            let idx = (rand_core::RngCore::next_u64(rng) as usize) % targets.len();
                            let target_id = targets[idx].clone();
                            if let Ok(evs) = engine.apply(side, Action::BreatheFire { target_id }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // Pass: throw to a random coordinate on the pitch
                    PlayerAction::Pass | PlayerAction::HailMaryPass => {
                        let coord = random_pass_coord(&engine.game, &pid, side, rng);
                        if let Ok(evs) = engine.apply(side, Action::Pass { coord }) {
                            all_events.extend(evs);
                        }
                    }
                    // PassMove: move first, then the pass will be attempted on the next ActivatePlayer
                    PlayerAction::PassMove => {
                        let path = all_reachable_path(&engine.game, &pid, side, rng);
                        if !path.is_empty() {
                            if let Ok(evs) = engine.apply(side, Action::Move { path }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // Foul: kick a prone/stunned adjacent opponent
                    PlayerAction::Foul => {
                        let targets = legal_foul_targets(&engine.game, &pid, side);
                        if !targets.is_empty() {
                            let idx = (rand_core::RngCore::next_u64(rng) as usize) % targets.len();
                            let target_id = targets[idx].clone();
                            if let Ok(evs) = engine.apply(side, Action::Foul { target_id }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // FoulMove: move into position first
                    PlayerAction::FoulMove => {
                        let path = all_reachable_path(&engine.game, &pid, side, rng);
                        if !path.is_empty() {
                            if let Ok(evs) = engine.apply(side, Action::Move { path }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // ThrowTeamMate: throw an adjacent small teammate (with RightStuff) to any coord
                    PlayerAction::ThrowTeamMate | PlayerAction::ThrowTeamMateMove => {
                        if let Some((thrown_id, coord)) = find_throw_target(&engine.game, &pid, side, rng) {
                            if let Ok(evs) = engine.apply(side, Action::ThrowTeamMate { player_id: thrown_id, coord }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // KickTeamMate (BB2025): like ThrowTeamMate but kicks instead
                    PlayerAction::KickTeamMate | PlayerAction::KickTeamMateMove => {
                        if let Some((thrown_id, coord)) = find_throw_target(&engine.game, &pid, side, rng) {
                            if let Ok(evs) = engine.apply(side, Action::KickTeamMate { player_id: thrown_id, coord }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // HandOff: hand the ball to a random adjacent teammate
                    PlayerAction::HandOver => {
                        let receiver = adjacent_teammate(&engine.game, &pid, side);
                        if let Some(receiver_id) = receiver {
                            if let Ok(evs) = engine.apply(side, Action::HandOff { receiver_id }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    // HandOverMove: move first
                    PlayerAction::HandOverMove => {
                        let path = all_reachable_path(&engine.game, &pid, side, rng);
                        if !path.is_empty() {
                            if let Ok(evs) = engine.apply(side, Action::Move { path }) {
                                all_events.extend(evs);
                            }
                        }
                    }
                    _ => {}
                }

                // After the primary follow-up, also fire skill-based actions that are
                // not in eligible_players but must be sent directly by the client.
                let rng2 = if side == TeamSide::Home { &mut home_rng } else { &mut away_rng };
                try_skill_actions(&mut engine, side, &pid, rng2, &mut all_events);
            }
        }
    }

    let home_score = engine.game.game_result.home.score;
    let away_score = engine.game.game_result.away.score;
    (all_events, home_score, away_score, all_touchbacks)
}

// ── CLI ────────────────────────────────────────────────────────────────────────

struct CovArgs {
    home: String,
    away: String,
    edition: String,
    seed_start: u64,
    seed_end: u64,
    all_matchups: bool,
}

impl CovArgs {
    fn parse() -> Self {
        let raw: Vec<String> = std::env::args().skip(1).collect();
        let mut home = "lineman".to_string();
        let mut away = "lineman".to_string();
        let mut edition = "bb2025".to_string();
        let mut seed_start = 1u64;
        let mut seed_end = 100u64;
        let mut all_matchups = false;

        let mut i = 0;
        while i < raw.len() {
            match raw[i].as_str() {
                "--all-matchups" => all_matchups = true,
                "--home" if i + 1 < raw.len() => { home = raw[i+1].clone(); i += 1; }
                "--away" if i + 1 < raw.len() => { away = raw[i+1].clone(); i += 1; }
                "--edition" if i + 1 < raw.len() => { edition = raw[i+1].clone(); i += 1; }
                "--seeds" if i + 1 < raw.len() => {
                    let s = &raw[i+1];
                    if let Some(d) = s.find('-') {
                        seed_start = s[..d].parse().unwrap_or(1);
                        seed_end = s[d+1..].parse().unwrap_or(100);
                    } else {
                        seed_end = s.parse().unwrap_or(100);
                    }
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }
        CovArgs { home, away, edition, seed_start, seed_end, all_matchups }
    }
}

fn discover_matchups() -> Vec<(String, String)> {
    let mut matchups = Vec::new();
    let parity_dir = std::path::Path::new("parity");
    if let Ok(entries) = std::fs::read_dir(parity_dir) {
        let mut dirs: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        dirs.sort();
        for name in dirs {
            if let Some((home, away)) = name.split_once("_vs_") {
                matchups.push((home.to_string(), away.to_string()));
            }
        }
    }
    if matchups.is_empty() {
        matchups.push(("lineman".to_string(), "lineman".to_string()));
    }
    matchups
}

fn build_skill_names() -> HashMap<String, String> {
    SKILL_TABLE.iter()
        .map(|entry| (entry.id.class_name().to_string(), entry.id.class_name().to_string()))
        .collect()
}

fn main() {
    env_logger::init();
    let args = CovArgs::parse();

    let matchups: Vec<(String, String)> = if args.all_matchups {
        discover_matchups()
    } else {
        vec![(args.home.clone(), args.away.clone())]
    };

    let total_games = matchups.len() as u32 * (args.seed_end - args.seed_start + 1) as u32;
    println!("Running coverage analysis: {} matchups × {} seeds = {} games",
        matchups.len(), args.seed_end - args.seed_start + 1, total_games);

    let mut report = CoverageReport::default();

    for (home, away) in &matchups {
        let mut summary = MatchupSummary {
            home: home.clone(),
            away: away.clone(),
            seeds: (args.seed_end - args.seed_start + 1) as u32,
            home_wins: 0, away_wins: 0, draws: 0,
            touchdowns_home: 0, touchdowns_away: 0,
        };

        for seed in args.seed_start..=args.seed_end {
            let (events, home_score, away_score, touchbacks) = run_coverage_game(seed, home, away, &args.edition);
            report.touchbacks += touchbacks;

            for ev in &events {
                report.tally(ev);
            }

            report.games += 1;
            report.touchdowns_home += home_score as u32;
            report.touchdowns_away += away_score as u32;
            summary.touchdowns_home += home_score as u32;
            summary.touchdowns_away += away_score as u32;

            if home_score > away_score {
                report.home_wins += 1; summary.home_wins += 1;
            } else if away_score > home_score {
                report.away_wins += 1; summary.away_wins += 1;
            } else {
                report.draws += 1; summary.draws += 1;
            }

            if seed % 10 == 0 {
                println!("  {home} vs {away}: seed {seed}/{}", args.seed_end);
            }
        }

        report.matchups.push(summary);
    }

    report.skill_names = build_skill_names();

    let json = serde_json::to_string(&report).expect("serialization failed");
    let html = generate_html(&json);
    std::fs::write("coverage.html", &html).expect("failed to write coverage.html");
    println!("\nDone! Coverage report written to coverage.html");
    println!("  Games played: {}", report.games);
    println!("  Scores: {}-{} (home-away), {}/{}/{} W/D/L",
        report.touchdowns_home, report.touchdowns_away,
        report.home_wins, report.draws, report.away_wins);
    println!("  Events: {} turn-ends, {} player-actions, {} player-moves",
        report.turn_ends, report.activations.values().sum::<u32>(), report.player_moved_events);
    println!("  Rolls: {} blocks, {} dodges, {} GFIs, {} catches",
        report.block_rolls.total, report.dodge_rolls.total,
        report.go_for_it_rolls.total, report.catch_rolls.total);
    println!("  Injuries: {} total ({} KO, {} CAS, {} dead)",
        report.injuries.total, report.injuries.ko, report.injuries.cas, report.injuries.dead);
    println!("  Kickoff events: {}", report.kickoff_events.values().sum::<u32>());
}

// ── HTML generation ────────────────────────────────────────────────────────────

fn generate_html(json: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>FFB-Rust Coverage Dashboard</title>
<style>
  :root {{
    --bg: #0d1117;
    --surface: #161b22;
    --border: #30363d;
    --text: #e6edf3;
    --muted: #8b949e;
    --accent: #58a6ff;
    --green: #3fb950;
    --red: #f85149;
    --orange: #d29922;
    --purple: #bc8cff;
    --yellow: #e3b341;
  }}
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{ background: var(--bg); color: var(--text); font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; font-size: 14px; }}
  h1 {{ font-size: 24px; font-weight: 700; color: var(--accent); }}
  h2 {{ font-size: 16px; font-weight: 600; color: var(--text); margin-bottom: 12px; border-bottom: 1px solid var(--border); padding-bottom: 8px; }}
  h3 {{ font-size: 13px; font-weight: 600; color: var(--muted); text-transform: uppercase; letter-spacing: .06em; margin-bottom: 8px; }}
  .header {{ padding: 24px 32px 16px; border-bottom: 1px solid var(--border); display: flex; align-items: center; gap: 16px; }}
  .subtitle {{ color: var(--muted); font-size: 13px; }}
  .main {{ padding: 24px 32px; display: grid; gap: 24px; }}
  .card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 20px; }}
  .grid-2 {{ display: grid; grid-template-columns: 1fr 1fr; gap: 24px; }}
  .grid-3 {{ display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 24px; }}

  /* Stat tiles */
  .tiles {{ display: flex; flex-wrap: wrap; gap: 12px; margin-bottom: 0; }}
  .tile {{ background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 12px 16px; min-width: 120px; }}
  .tile .val {{ font-size: 22px; font-weight: 700; color: var(--accent); }}
  .tile .lbl {{ font-size: 11px; color: var(--muted); margin-top: 2px; text-transform: uppercase; letter-spacing: .05em; }}

  /* Bar charts */
  .bar-chart {{ display: flex; flex-direction: column; gap: 6px; }}
  .bar-row {{ display: flex; align-items: center; gap: 8px; }}
  .bar-label {{ min-width: 140px; max-width: 160px; color: var(--muted); font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex-shrink: 0; }}
  .bar-label.wide {{ min-width: 200px; max-width: 220px; }}
  .bar-track {{ flex: 1; background: var(--bg); border-radius: 3px; height: 14px; }}
  .bar-fill {{ height: 100%; border-radius: 3px; transition: width .2s; }}
  .bar-val {{ min-width: 50px; text-align: right; font-size: 12px; color: var(--muted); font-variant-numeric: tabular-nums; }}
  .bar-pct {{ min-width: 40px; text-align: right; font-size: 11px; color: var(--muted); font-variant-numeric: tabular-nums; }}

  /* Tables */
  table {{ width: 100%; border-collapse: collapse; font-size: 12px; }}
  th {{ text-align: left; padding: 6px 8px; color: var(--muted); font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: .04em; border-bottom: 1px solid var(--border); }}
  td {{ padding: 5px 8px; border-bottom: 1px solid #1c2128; }}
  tr:last-child td {{ border-bottom: none; }}
  tr:hover td {{ background: #1c2128; }}
  .num {{ text-align: right; font-variant-numeric: tabular-nums; }}
  .pct {{ text-align: right; font-variant-numeric: tabular-nums; color: var(--muted); }}

  /* Pill badges */
  .badge {{ display: inline-block; padding: 1px 6px; border-radius: 10px; font-size: 10px; font-weight: 600; }}
  .badge-green {{ background: #1a3a2a; color: var(--green); }}
  .badge-red {{ background: #3a1a1a; color: var(--red); }}
  .badge-yellow {{ background: #3a2a00; color: var(--yellow); }}

  /* Result bar */
  .result-bar {{ display: flex; height: 20px; border-radius: 4px; overflow: hidden; margin-top: 8px; }}
  .result-bar .seg {{ display: flex; align-items: center; justify-content: center; font-size: 10px; font-weight: 600; }}
  .seg-home {{ background: var(--accent); color: #000; }}
  .seg-draw {{ background: var(--muted); color: #000; }}
  .seg-away {{ background: var(--purple); color: #000; }}

  .section-note {{ font-size: 11px; color: var(--muted); margin-top: 8px; }}
  .two-col {{ display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }}
  .zero {{ color: #444; }}
</style>
</head>
<body>
<div class="header">
  <div>
    <h1>FFB-Rust Coverage Dashboard</h1>
    <div class="subtitle" id="subtitle">Loading…</div>
  </div>
</div>
<div class="main" id="main">
  <div class="card"><p style="color:var(--muted)">Loading data…</p></div>
</div>

<script>
const RAW = {json};

function fmt(n) {{ return n == null ? '—' : n.toLocaleString(); }}
function pct(n, d) {{ if (!d) return '—'; return (n/d*100).toFixed(1)+'%'; }}
function pctOf(n, d) {{ if (!d) return 0; return n/d*100; }}
function colorFor(i) {{
  const palette = ['#58a6ff','#3fb950','#d29922','#bc8cff','#f85149','#39d353','#ff7b72','#79c0ff','#ffa657','#56d364'];
  return palette[i % palette.length];
}}

function barChart(entries, maxVal, colorFn) {{
  if (!entries.length) return '<em style="color:var(--muted);font-size:12px">No data</em>';
  return entries.map(([label, val], i) => {{
    const w = maxVal > 0 ? (val / maxVal * 100).toFixed(1) : 0;
    const color = colorFn ? colorFn(label, i) : colorFor(i);
    return `<div class="bar-row">
      <span class="bar-label" title="${{label}}">${{label}}</span>
      <div class="bar-track"><div class="bar-fill" style="width:${{w}}%;background:${{color}}"></div></div>
      <span class="bar-val">${{fmt(val)}}</span>
    </div>`;
  }}).join('');
}}

function rollTable(rows) {{
  const header = `<tr><th>Roll Type</th><th class="num">Total</th><th class="pct">Success%</th><th class="pct">Fail%</th><th class="pct">Rerolled%</th></tr>`;
  const body = rows.filter(r => r[1] > 0).map(([name, stats]) => {{
    const t = stats.total || 0;
    return `<tr>
      <td>${{name}}</td>
      <td class="num">${{fmt(t)}}</td>
      <td class="pct">${{pct(stats.success, t)}}</td>
      <td class="pct">${{pct(stats.failure, t)}}</td>
      <td class="pct">${{pct(stats.rerolled, t)}}</td>
    </tr>`;
  }}).join('');
  if (!body) return '<em style="color:var(--muted);font-size:12px">No rolls recorded</em>';
  return `<table><thead>${{header}}</thead><tbody>${{body}}</tbody></table>`;
}}

function render() {{
  const D = RAW;
  const games = D.games || 0;
  const totalTDs = (D.touchdowns_home || 0) + (D.touchdowns_away || 0);

  document.getElementById('subtitle').textContent =
    `${{games.toLocaleString()}} games · ${{D.matchups ? D.matchups.length : 1}} matchup(s)`;

  const sections = [];

  // ── Overview ──────────────────────────────────────────────────────────────
  const homeWinPct = pctOf(D.home_wins, games);
  const drawPct = pctOf(D.draws, games);
  const awayWinPct = pctOf(D.away_wins, games);
  sections.push(`
    <div class="card">
      <h2>Overview</h2>
      <div class="tiles">
        <div class="tile"><div class="val">${{fmt(games)}}</div><div class="lbl">Games</div></div>
        <div class="tile"><div class="val">${{fmt(totalTDs)}}</div><div class="lbl">Total Touchdowns</div></div>
        <div class="tile"><div class="val">${{(totalTDs/Math.max(games,1)).toFixed(2)}}</div><div class="lbl">TDs / Game</div></div>
        <div class="tile"><div class="val">${{fmt(D.home_wins)}}</div><div class="lbl">Home Wins</div></div>
        <div class="tile"><div class="val">${{fmt(D.draws)}}</div><div class="lbl">Draws</div></div>
        <div class="tile"><div class="val">${{fmt(D.away_wins)}}</div><div class="lbl">Away Wins</div></div>
        <div class="tile"><div class="val">${{fmt(D.touchdowns)}}</div><div class="lbl">TD Events</div></div>
        <div class="tile"><div class="val">${{fmt(D.rerolls_total)}}</div><div class="lbl">Re-rolls Used</div></div>
        <div class="tile"><div class="val">${{fmt(D.fouls)}}</div><div class="lbl">Fouls</div></div>
        <div class="tile"><div class="val">${{fmt(D.weather_changes)}}</div><div class="lbl">Weather Changes</div></div>
      </div>
      <h3 style="margin-top:16px">Result Distribution</h3>
      <div class="result-bar">
        <div class="seg seg-home" style="width:${{homeWinPct.toFixed(1)}}%">${{homeWinPct > 5 ? pct(D.home_wins, games) : ''}}</div>
        <div class="seg seg-draw" style="width:${{drawPct.toFixed(1)}}%">${{drawPct > 5 ? pct(D.draws, games) : ''}}</div>
        <div class="seg seg-away" style="width:${{awayWinPct.toFixed(1)}}%">${{awayWinPct > 5 ? pct(D.away_wins, games) : ''}}</div>
      </div>
      <div class="section-note" style="margin-top:6px">
        <span class="badge badge-green">Home ${{pct(D.home_wins, games)}}</span>
        <span class="badge badge-yellow" style="margin-left:4px">Draw ${{pct(D.draws, games)}}</span>
        <span class="badge badge-red" style="margin-left:4px">Away ${{pct(D.away_wins, games)}}</span>
      </div>
    </div>`);

  // ── Matchups ──────────────────────────────────────────────────────────────
  if (D.matchups && D.matchups.length > 1) {{
    const rows = D.matchups.map(m => {{
      const tot = (m.touchdowns_home || 0) + (m.touchdowns_away || 0);
      return `<tr>
        <td>${{m.home}} vs ${{m.away}}</td>
        <td class="num">${{m.seeds}}</td>
        <td class="num">${{m.home_wins}}</td>
        <td class="num">${{m.draws}}</td>
        <td class="num">${{m.away_wins}}</td>
        <td class="num">${{tot}}</td>
        <td class="pct">${{(tot/Math.max(m.seeds,1)).toFixed(2)}}/g</td>
      </tr>`;
    }}).join('');
    sections.push(`
      <div class="card">
        <h2>Matchups</h2>
        <table>
          <thead><tr><th>Matchup</th><th class="num">Seeds</th><th class="num">Home W</th><th class="num">Draw</th><th class="num">Away W</th><th class="num">TDs</th><th class="pct">TDs/g</th></tr></thead>
          <tbody>${{rows}}</tbody>
        </table>
      </div>`);
  }}

  // ── Player Activations ────────────────────────────────────────────────────
  const activations = Object.entries(D.activations || {{}}).sort((a,b) => b[1]-a[1]);
  const maxAct = activations.length ? activations[0][1] : 1;
  sections.push(`
    <div class="card">
      <h2>Player Activations</h2>
      <div class="bar-chart">${{barChart(activations, maxAct)}}</div>
    </div>`);

  // ── Rolls Table ───────────────────────────────────────────────────────────
  const rollRows = [
    ['DodgeRoll', D.dodge_rolls],
    ['GoForItRoll', D.go_for_it_rolls],
    ['BlockRoll (any)', {{ total: D.block_rolls?.total || 0, success: 0, failure: 0, rerolled: D.block_rolls?.rerolled || 0 }}],
    ['PassRoll', {{ total: D.pass_rolls?.total || 0, success: 0, failure: 0, rerolled: D.pass_rolls?.rerolled || 0 }}],
    ['CatchRoll', D.catch_rolls],
    ['InterceptionRoll', D.interception_rolls],
    ['JumpRoll (Leap)', D.jump_rolls],
    ['JumpUpRoll', D.jump_up_rolls],
    ['DauntlessRoll', D.dauntless_rolls],
    ['LonerRoll', D.loner_rolls],
    ['ProRoll', D.pro_rolls],
    ['FoulAppearanceRoll', D.foul_appearance_rolls],
    ['AlwaysHungryRoll', D.always_hungry_rolls],
    ['BloodLustRoll', D.blood_lust_rolls],
    ['AnimosityRoll', D.animosity_rolls],
    ['ConfusionRoll', D.confusion_rolls],
    ['HypnoticGazeRoll', D.hypnotic_gaze_rolls],
    ['EscapeRoll', D.escape_rolls],
    ['RightStuffRoll', D.right_stuff_rolls],
    ['SafeThrowRoll', D.safe_throw_rolls],
    ['StandUpRoll', D.stand_up_rolls],
    ['PickMeUpRoll', D.pick_me_up_rolls],
    ['BreatheFireRoll', D.breathe_fire_rolls],
    ['ProjectileVomitRoll', D.projectile_vomit_rolls],
    ['BalefulHexRoll', D.baleful_hex_rolls],
    ['LookIntoMyEyesRoll', D.look_into_my_eyes_rolls],
    ['AnimalSavageryRoll', D.animal_savagery_rolls],
    ['ArgueTheCallRoll', D.argue_the_call_rolls],
    ['BribesRoll', D.bribes_rolls],
  ];
  sections.push(`
    <div class="card">
      <h2>Dice Rolls</h2>
      ${{rollTable(rollRows)}}
    </div>`);

  // ── Block Dice ────────────────────────────────────────────────────────────
  const blockByDice = Object.entries(D.block_rolls?.by_dice || {{}})
    .sort((a,b) => Number(a[0]) - Number(b[0]))
    .map(([k, v]) => [k === '-1' ? '2d (def)' : k === '1' ? '1d' : k === '2' ? '2d (att)' : `${{k}}d`, v]);
  const maxBlock = blockByDice.length ? Math.max(...blockByDice.map(e=>e[1])) : 1;
  sections.push(`
    <div class="card">
      <h2>Block Rolls by Dice Count</h2>
      <div class="tiles" style="margin-bottom:12px">
        <div class="tile"><div class="val">${{fmt(D.block_rolls?.total)}}</div><div class="lbl">Total Blocks</div></div>
        <div class="tile"><div class="val">${{fmt(D.block_rolls?.rerolled)}}</div><div class="lbl">Rerolled</div></div>
        <div class="tile"><div class="val">${{fmt(D.total_pushbacks)}}</div><div class="lbl">Total Pushbacks</div></div>
        <div class="tile"><div class="val">${{fmt(D.push_destinations_hist?.['1'])}}</div><div class="lbl">1-option push (edge)</div></div>
        <div class="tile"><div class="val">${{fmt(D.push_destinations_hist?.['2'])}}</div><div class="lbl">2-option push</div></div>
        <div class="tile"><div class="val">${{fmt(D.push_destinations_hist?.['3+'])}}</div><div class="lbl">3-option push</div></div>
      </div>
      <div class="bar-chart">${{barChart(blockByDice, maxBlock)}}</div>
    </div>`);

  // ── Pass ──────────────────────────────────────────────────────────────────
  const passByDist = Object.entries(D.pass_rolls?.by_distance || {{}}).sort((a,b)=>b[1]-a[1]);
  const passByResult = Object.entries(D.pass_rolls?.by_result || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxPassDist = passByDist.length ? passByDist[0][1] : 1;
  const maxPassRes = passByResult.length ? passByResult[0][1] : 1;
  sections.push(`
    <div class="card">
      <h2>Pass Rolls</h2>
      <div class="tiles" style="margin-bottom:12px">
        <div class="tile"><div class="val">${{fmt(D.pass_rolls?.total)}}</div><div class="lbl">Total Passes</div></div>
        <div class="tile"><div class="val">${{fmt(D.pass_deviates)}}</div><div class="lbl">Deviations</div></div>
        <div class="tile"><div class="val">${{fmt(D.throw_team_mate_rolls)}}</div><div class="lbl">Throw-Teammate</div></div>
      </div>
      <div class="two-col">
        <div>
          <h3>By Distance</h3>
          <div class="bar-chart">${{barChart(passByDist, maxPassDist)}}</div>
        </div>
        <div>
          <h3>By Result</h3>
          <div class="bar-chart">${{barChart(passByResult, maxPassRes)}}</div>
        </div>
      </div>
    </div>`);

  // ── Injuries ──────────────────────────────────────────────────────────────
  const inj = D.injuries || {{}};
  const injTotal = inj.total || 0;
  const injSections = [
    ['Armor Only', inj.armor_only || 0, '#8b949e'],
    ['KO', inj.ko || 0, '#d29922'],
    ['Casualty', inj.cas || 0, '#f85149'],
    ['Dead', inj.dead || 0, '#7a1a1a'],
  ];
  const maxInj = Math.max(...injSections.map(e=>e[1]), 1);
  const seriousEntries = Object.entries(inj.by_serious_injury || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxSi = seriousEntries.length ? seriousEntries[0][1] : 1;
  sections.push(`
    <div class="card">
      <h2>Injuries</h2>
      <div class="tiles" style="margin-bottom:12px">
        <div class="tile"><div class="val">${{fmt(injTotal)}}</div><div class="lbl">Total Injuries</div></div>
        <div class="tile"><div class="val">${{fmt(inj.ko)}}</div><div class="lbl">KOs</div></div>
        <div class="tile"><div class="val">${{fmt(inj.cas)}}</div><div class="lbl">Casualties</div></div>
        <div class="tile"><div class="val">${{fmt(inj.dead)}}</div><div class="lbl">Deaths</div></div>
        <div class="tile"><div class="val">${{pct(inj.dead, inj.cas)}}</div><div class="lbl">Death Rate (of CAS)</div></div>
      </div>
      <div class="two-col">
        <div>
          <h3>Severity</h3>
          <div class="bar-chart">${{barChart(injSections.map(e=>[e[0],e[1]]), maxInj, (l,i) => injSections[i]?.[2])}}</div>
        </div>
        <div>
          <h3>Serious Injury Types</h3>
          <div class="bar-chart">${{barChart(seriousEntries.slice(0,12), maxSi)}}</div>
        </div>
      </div>
    </div>`);

  // ── Skills ────────────────────────────────────────────────────────────────
  const skillUsed = Object.entries(D.skill_used || {{}}).sort((a,b)=>b[1]-a[1]);
  const skillDeclined = Object.entries(D.skill_declined || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxSU = skillUsed.length ? skillUsed[0][1] : 1;
  const maxSD = skillDeclined.length ? skillDeclined[0][1] : 1;
  sections.push(`
    <div class="card">
      <h2>Skill Usage</h2>
      <div class="two-col">
        <div>
          <h3>Used (${{skillUsed.length}} skills)</h3>
          <div class="bar-chart">${{barChart(skillUsed, maxSU)}}</div>
        </div>
        <div>
          <h3>Declined (${{skillDeclined.length}} skills)</h3>
          <div class="bar-chart">${{barChart(skillDeclined, maxSD)}}</div>
        </div>
      </div>
    </div>`);

  // ── Kickoff Events ────────────────────────────────────────────────────────
  const kickoffEntries = Object.entries(D.kickoff_events || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxKO = kickoffEntries.length ? kickoffEntries[0][1] : 1;
  sections.push(`
    <div class="card">
      <h2>Kickoff Events</h2>
      <div class="tiles" style="margin-bottom:12px">
        <div class="tile"><div class="val">${{fmt(D.kickoff_scatters)}}</div><div class="lbl">Kickoff Scatters</div></div>
        <div class="tile"><div class="val">${{fmt(D.touchbacks)}}</div><div class="lbl">Touchbacks</div></div>
        <div class="tile"><div class="val">${{fmt(D.kickoff_pitch_invasions)}}</div><div class="lbl">Pitch Invasions</div></div>
        <div class="tile"><div class="val">${{fmt(D.kickoff_riots)}}</div><div class="lbl">Riots</div></div>
        <div class="tile"><div class="val">${{fmt(D.kickoff_rocks_thrown)}}</div><div class="lbl">Rocks Thrown</div></div>
      </div>
      <div class="bar-chart">${{barChart(kickoffEntries, maxKO)}}</div>
    </div>`);

  // ── Ball Events ───────────────────────────────────────────────────────────
  const ballEntries = [
    ['Touchdowns', D.touchdowns || 0],
    ['Ball Picked Up', D.ball_picked_up || 0],
    ['Scatter Balls', D.scatter_balls || 0],
    ['Throw-ins', D.throw_ins || 0],
    ['Pass Deviates', D.pass_deviates || 0],
    ['Player Moved', D.player_moved_events || 0],
    ['Players Fell Down', D.players_fell_down || 0],
    ['Scatter Players', D.scatter_players || 0],
  ];
  const maxBall = Math.max(...ballEntries.map(e=>e[1]), 1);
  sections.push(`
    <div class="card">
      <h2>Ball &amp; Movement Events</h2>
      <div class="bar-chart">${{barChart(ballEntries, maxBall)}}</div>
    </div>`);

  // ── Fouls & Discipline ────────────────────────────────────────────────────
  sections.push(`
    <div class="card">
      <h2>Fouls &amp; Discipline</h2>
      <div class="tiles">
        <div class="tile"><div class="val">${{fmt(D.fouls)}}</div><div class="lbl">Fouls</div></div>
        <div class="tile"><div class="val">${{fmt(D.players_ejected)}}</div><div class="lbl">Players Ejected</div></div>
        <div class="tile"><div class="val">${{fmt(D.argue_the_call_rolls?.total)}}</div><div class="lbl">Argue the Call</div></div>
        <div class="tile"><div class="val">${{pct(D.argue_the_call_rolls?.success, D.argue_the_call_rolls?.total)}}</div><div class="lbl">Argue Success%</div></div>
        <div class="tile"><div class="val">${{fmt(D.bribes_rolls?.total)}}</div><div class="lbl">Bribes Used</div></div>
        <div class="tile"><div class="val">${{pct(D.bribes_rolls?.success, D.bribes_rolls?.total)}}</div><div class="lbl">Bribe Success%</div></div>
        <div class="tile"><div class="val">${{fmt(D.secret_weapon_bans)}}</div><div class="lbl">Secret Weapon Bans</div></div>
        <div class="tile"><div class="val">${{fmt(D.apothecary_used)}}</div><div class="lbl">Apothecary Used</div></div>
        <div class="tile"><div class="val">${{fmt(D.wizard_used)}}</div><div class="lbl">Wizard Used</div></div>
        <div class="tile"><div class="val">${{fmt(D.cards_played)}}</div><div class="lbl">Cards Played</div></div>
      </div>
    </div>`);

  // ── Re-rolls ──────────────────────────────────────────────────────────────
  const rrEntries = Object.entries(D.rerolls_by_source || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxRR = rrEntries.length ? rrEntries[0][1] : 1;
  sections.push(`
    <div class="card">
      <h2>Re-rolls</h2>
      <div class="tiles" style="margin-bottom:12px">
        <div class="tile"><div class="val">${{fmt(D.rerolls_total)}}</div><div class="lbl">Total Re-rolls</div></div>
        <div class="tile"><div class="val">${{(D.rerolls_total/Math.max(games,1)).toFixed(2)}}</div><div class="lbl">Re-rolls/Game</div></div>
      </div>
      <div class="bar-chart">${{barChart(rrEntries, maxRR)}}</div>
    </div>`);

  // ── Inducements ───────────────────────────────────────────────────────────
  const indEntries = Object.entries(D.inducements_bought || {{}}).sort((a,b)=>b[1]-a[1]);
  if (indEntries.length) {{
    const maxInd = indEntries[0][1];
    sections.push(`
      <div class="card">
        <h2>Inducements Bought</h2>
        <div class="bar-chart">${{barChart(indEntries, maxInd)}}</div>
      </div>`);
  }}

  // ── Misc ──────────────────────────────────────────────────────────────────
  const miscEntries = [
    ['Turn Ends', D.turn_ends || 0],
    ['Half Starts', D.half_starts || 0],
    ['Coin Throws', D.coin_throws || 0],
    ['MVP Rolls', D.mvp_rolls || 0],
    ['Chainsaw Rolls', D.chainsaw_rolls || 0],
    ['Piling On', D.piling_on_events || 0],
    ['Team Captain Saves', D.team_captain_saves || 0],
    ['Swarming Rolls', D.swarming_rolls || 0],
    ['Heat Exhaustion', D.heat_exhaustion || 0],
    ['Trap Door', D.trap_door_rolls || 0],
    ['Bomb Explosions', D.bomb_explosions || 0],
    ['Throw-Teammate Rolls', D.throw_team_mate_rolls || 0],
  ];
  const maxMisc = Math.max(...miscEntries.map(e=>e[1]), 1);
  sections.push(`
    <div class="card">
      <h2>Misc Events</h2>
      <div class="bar-chart">${{barChart(miscEntries, maxMisc)}}</div>
    </div>`);

  document.getElementById('main').innerHTML = sections.join('');
}}

render();
</script>
</body>
</html>
"#, json = json)
}
