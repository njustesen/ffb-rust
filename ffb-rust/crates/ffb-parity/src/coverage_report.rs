/// Shared coverage report types and HTML generation.
/// Used by both ffb-parity (inline during parity runs) and can be used standalone.

use std::collections::HashMap;
use serde::Serialize;
use ffb_model::events::GameEvent;
use ffb_model::enums::{PlayerAction, SeriousInjuryKind};
use ffb_mechanics::skills::SKILL_TABLE;

// ── Data structures ────────────────────────────────────────────────────────────

#[derive(Default, Serialize)]
pub struct RollStats {
    pub total: u32,
    pub success: u32,
    pub failure: u32,
    pub rerolled: u32,
}

impl RollStats {
    pub fn record(&mut self, success: bool, rerolled: bool) {
        self.total += 1;
        if success { self.success += 1; } else { self.failure += 1; }
        if rerolled { self.rerolled += 1; }
    }
}

#[derive(Default, Serialize)]
pub struct BlockStats {
    pub total: u32,
    pub rerolled: u32,
    pub by_dice: HashMap<i32, u32>,
    /// Selected block result (Skull/BothDown/Pushback/PowPushback/Pow) counts.
    pub by_result: HashMap<String, u32>,
}

#[derive(Default, Serialize)]
pub struct PassStats {
    pub total: u32,
    pub rerolled: u32,
    pub by_distance: HashMap<String, u32>,
    pub by_result: HashMap<String, u32>,
    pub by_distance_and_result: HashMap<String, u32>,
}

#[derive(Default, Serialize)]
pub struct InjuryStats {
    pub total: u32,
    pub armor_only: u32,
    pub ko: u32,
    pub cas: u32,
    pub dead: u32,
    pub by_serious_injury: HashMap<String, u32>,
}

#[derive(Default, Serialize)]
pub struct CoverageReport {
    pub games: u32,
    pub home_wins: u32,
    pub away_wins: u32,
    pub draws: u32,
    pub touchdowns_home: u32,
    pub touchdowns_away: u32,
    pub matchups: Vec<MatchupSummary>,

    pub activations: HashMap<String, u32>,

    pub dodge_rolls: RollStats,
    pub go_for_it_rolls: RollStats,
    pub catch_rolls: RollStats,
    pub pickup_rolls: RollStats,
    pub interception_rolls: RollStats,
    pub jump_rolls: RollStats,
    pub jump_up_rolls: RollStats,
    pub dauntless_rolls: RollStats,
    pub loner_rolls: RollStats,
    pub pro_rolls: RollStats,
    pub foul_appearance_rolls: RollStats,
    pub always_hungry_rolls: RollStats,
    pub blood_lust_rolls: RollStats,
    pub animosity_rolls: RollStats,
    pub confusion_rolls: RollStats,
    pub hypnotic_gaze_rolls: RollStats,
    pub escape_rolls: RollStats,
    pub right_stuff_rolls: RollStats,
    pub safe_throw_rolls: RollStats,
    pub stand_up_rolls: RollStats,
    pub pick_me_up_rolls: RollStats,
    pub breathe_fire_rolls: RollStats,
    pub projectile_vomit_rolls: RollStats,
    pub baleful_hex_rolls: RollStats,
    pub look_into_my_eyes_rolls: RollStats,
    pub weeping_dagger_rolls: RollStats,

    pub block_rolls: BlockStats,
    pub pass_rolls: PassStats,

    pub injuries: InjuryStats,

    pub skill_used: HashMap<String, u32>,
    pub skill_declined: HashMap<String, u32>,
    pub skill_names: HashMap<String, String>,

    pub kickoff_events: HashMap<String, u32>,
    pub kickoff_pitch_invasions: u32,
    pub kickoff_rocks_thrown: u32,
    pub kickoff_riots: u32,

    pub touchdowns: u32,
    pub throw_ins: u32,
    pub scatter_balls: u32,
    pub pass_deviates: u32,
    pub touchbacks: u32,
    pub ball_picked_up: u32,
    pub kickoff_scatters: u32,

    pub total_pushbacks: u32,
    pub push_destinations_hist: HashMap<String, u32>,

    pub fouls: u32,
    pub players_ejected: u32,
    pub argue_the_call_rolls: RollStats,
    pub bribes_rolls: RollStats,

    pub players_fell_down: u32,
    pub player_moved_events: u32,
    pub scatter_players: u32,

    pub apothecary_used: u32,
    pub inducements_bought: HashMap<String, u32>,
    pub wizard_used: u32,
    pub cards_played: u32,
    pub secret_weapon_bans: u32,

    pub rerolls_by_source: HashMap<String, u32>,
    pub rerolls_total: u32,

    pub weather_changes: u32,
    pub heat_exhaustion: u32,
    pub turn_ends: u32,
    pub half_starts: u32,
    pub coin_throws: u32,
    pub mvp_rolls: u32,

    pub chainsaw_rolls: u32,
    pub trap_door_rolls: u32,
    pub piling_on_events: u32,
    pub animal_savagery_rolls: RollStats,
    pub team_captain_saves: u32,
    pub swarming_rolls: u32,
    pub bomb_explosions: u32,
    pub throw_team_mate_rolls: u32,

    // ── Prayers / cards / inducement registration ───────────────────────────────
    pub prayer_rolls: HashMap<String, u32>,
    pub prayer_amount_events: u32,
    pub card_effects_by_name: HashMap<String, u32>,
    pub cards_deactivated: HashMap<String, u32>,
    pub cards_and_inducements_bought_reports: u32,
    pub inducements_registered: HashMap<String, u32>,
    pub petty_cash_events: u32,

    // ── Referee / discipline ─────────────────────────────────────────────────────
    pub referee_spots_foul_rolls: RollStats,
    pub biased_ref_rolls: RollStats,
    pub coach_bans: u32,

    // ── Kickoff misc events ──────────────────────────────────────────────────────
    pub kickoff_sequence_activations_exhausted: u32,
    pub solid_defence_rolls: u32,
    pub cheering_fans_events: u32,
    pub kickoff_extra_rerolls: u32,
    pub quick_snap_rolls: u32,
    pub kickoff_timeouts: u32,
    pub kickoff_officious_ref_events: u32,
    pub kickoff_dodgy_snack_events: u32,
    pub dodgy_snack_rolls: u32,
    pub kickoff_extra_reroll_bb2016_events: u32,
    pub kickoff_throw_a_rock_bb2016_events: u32,
    pub kickoff_pitch_invasion_bb2016_events: u32,

    // ── Skill / special-action rolls ─────────────────────────────────────────────
    pub then_i_started_blastin_rolls: RollStats,
    pub then_i_started_blastin_fumbles: u32,
    pub master_chef_rolls: u32,
    pub master_chef_rerolls_stolen: u32,
    pub keg_throws: RollStats,
    pub keg_throw_fumbles: u32,
    pub blitz_rolls: u32,
    pub special_effect_rolls: HashMap<String, u32>,
    pub throw_at_stalling_player_rolls: RollStats,
    pub throw_at_player_rolls: RollStats,
    pub fumblerooskie_uses: RollStats,
    pub all_you_can_eat_rolls: RollStats,
    pub pump_up_the_crowd_rerolls: u32,
    pub select_blitz_targets: u32,
    pub select_gaze_targets: u32,
    pub hit_and_run_events: u32,
    pub kick_team_mate_fumbles: u32,
    pub bite_spectator_events: u32,
    pub leader_events: u32,
    pub defecting_players: u32,
    pub regeneration_rolls: RollStats,
    pub apothecary_rolls: u32,
    pub block_actions: u32,
    pub swoop_player_events: u32,
    pub hand_overs: u32,
    pub riotous_rookies: u32,
    pub riotous_rookies_players: u32,
    pub kickoff_pitch_invasion_stuns: u32,
    pub pass_blocks: u32,
    pub pass_block_eligible_events: u32,
    pub spell_effect_rolls: u32,
    pub player_notes: HashMap<String, u32>,
    pub players_added: u32,
    pub no_players_to_field_events: u32,

    // ── Game flow misc ───────────────────────────────────────────────────────────
    pub receive_choices: HashMap<String, u32>,
    pub game_options_snapshots: u32,
    pub double_hired_star_players: u32,
    pub timeouts_enforced: u32,
    pub winnings_rolls: u32,

    // ── Diagnostics fed from outside tally() (e.g. UniformAgent unhandled-prompt
    // tracking) — not populated by `tally()` itself, see `record_unhandled_prompt`.
    pub unhandled_prompts: HashMap<String, u32>,

    /// Raw count per `GameEvent` serde tag, taken straight off the event rather than from a
    /// hand-written `tally()` arm.
    ///
    /// Every field above needs its own match arm, so a variant nobody wrote an arm for reads as
    /// zero and is indistinguishable from one the engine never emits — the exact confusion a
    /// coverage report exists to prevent. This map is derived from the serialised tag, so it
    /// counts all ~130 variants whether or not `tally()` knows about them.
    pub event_type_counts: HashMap<String, u32>,
}

/// The `#[serde(tag = "type", rename_all = "camelCase")]` discriminant of a `GameEvent`.
/// Serialising just to read the tag is not free, but this runs in analysis tools, not the engine.
fn event_type_name(ev: &GameEvent) -> String {
    serde_json::to_value(ev)
        .ok()
        .and_then(|v| v.get("type").and_then(|t| t.as_str()).map(str::to_string))
        .unwrap_or_else(|| "<unserialisable>".to_string())
}

#[derive(Serialize, Clone)]
pub struct MatchupSummary {
    pub home: String,
    pub away: String,
    pub seeds: u32,
    pub home_wins: u32,
    pub away_wins: u32,
    pub draws: u32,
    pub touchdowns_home: u32,
    pub touchdowns_away: u32,
}

impl CoverageReport {
    pub fn tally(&mut self, ev: &GameEvent) {
        *self.event_type_counts.entry(event_type_name(ev)).or_default() += 1;
        match ev {
            GameEvent::DodgeRoll { success, rerolled, .. } =>
                self.dodge_rolls.record(*success, *rerolled),
            GameEvent::GoForItRoll { success, rerolled, .. } =>
                self.go_for_it_rolls.record(*success, *rerolled),
            GameEvent::CatchRoll { success, rerolled, .. } =>
                self.catch_rolls.record(*success, *rerolled),
            GameEvent::PickupRoll { success, rerolled, .. } =>
                self.pickup_rolls.record(*success, *rerolled),
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

            GameEvent::BlockRoll { nr_of_dice, rerolled, dice, selected_index, .. } => {
                self.block_rolls.total += 1;
                if *rerolled { self.block_rolls.rerolled += 1; }
                *self.block_rolls.by_dice.entry(*nr_of_dice).or_default() += 1;
                if let Some(&roll) = dice.get(*selected_index as usize) {
                    let result = ffb_mechanics::mechanics::block_result_for_roll(roll);
                    *self.block_rolls.by_result.entry(format!("{result:?}")).or_default() += 1;
                }
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

            GameEvent::ScatterBall { .. } | GameEvent::BallScattered { .. } => {
                self.scatter_balls += 1;
            }
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
            GameEvent::KickoffRiot { .. } => { self.kickoff_riots += 1; }
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
            GameEvent::RegenerationRoll { success, .. } =>
                self.regeneration_rolls.record(*success, false),

            GameEvent::PrayerRoll { prayer_id, .. } => {
                *self.prayer_rolls.entry(prayer_id.clone()).or_default() += 1;
            }
            GameEvent::PrayerAmount { .. } => { self.prayer_amount_events += 1; }
            GameEvent::CardEffectRoll { effect, .. } => {
                *self.card_effects_by_name.entry(effect.clone()).or_default() += 1;
            }
            GameEvent::CardDeactivated { card_id, .. } => {
                *self.cards_deactivated.entry(card_id.clone()).or_default() += 1;
            }
            GameEvent::CardsAndInducementsBought { .. } => {
                self.cards_and_inducements_bought_reports += 1;
            }
            GameEvent::Inducement { inducement_type, .. } => {
                *self.inducements_registered.entry(inducement_type.clone()).or_default() += 1;
            }
            GameEvent::PettyCash { .. } => { self.petty_cash_events += 1; }

            GameEvent::RefereeSpotsFoul { referee_spots_foul, .. } =>
                self.referee_spots_foul_rolls.record(*referee_spots_foul, false),
            GameEvent::BiasedRefRoll { referee_spots_foul, .. } =>
                self.biased_ref_rolls.record(*referee_spots_foul, false),
            GameEvent::CoachBanned { .. } => { self.coach_bans += 1; }

            GameEvent::KickoffSequenceActivationsExhausted { .. } => {
                self.kickoff_sequence_activations_exhausted += 1;
            }
            GameEvent::SolidDefenceRoll { .. } => { self.solid_defence_rolls += 1; }
            GameEvent::CheeringFans { .. } => { self.cheering_fans_events += 1; }
            GameEvent::KickoffExtraReRoll { .. } => { self.kickoff_extra_rerolls += 1; }
            GameEvent::QuickSnapRoll { .. } => { self.quick_snap_rolls += 1; }
            GameEvent::KickoffTimeout { .. } => { self.kickoff_timeouts += 1; }
            GameEvent::KickoffOfficiousRef { .. } => { self.kickoff_officious_ref_events += 1; }
            GameEvent::KickoffDodgySnack { .. } => { self.kickoff_dodgy_snack_events += 1; }
            GameEvent::DodgySnackRoll { .. } => { self.dodgy_snack_rolls += 1; }
            GameEvent::KickoffExtraReRollBb2016 { .. } => {
                self.kickoff_extra_reroll_bb2016_events += 1;
            }
            GameEvent::KickoffThrowARockBb2016 { .. } => {
                self.kickoff_throw_a_rock_bb2016_events += 1;
            }
            GameEvent::KickoffPitchInvasionBb2016 { .. } => {
                self.kickoff_pitch_invasion_bb2016_events += 1;
            }

            GameEvent::ThenIStartedBlastin { success, fumble, .. } => {
                self.then_i_started_blastin_rolls.record(*success, false);
                if *fumble { self.then_i_started_blastin_fumbles += 1; }
            }
            GameEvent::MasterChefRoll { rerolls_stolen, .. } => {
                self.master_chef_rolls += 1;
                self.master_chef_rerolls_stolen += *rerolls_stolen as u32;
            }
            GameEvent::KegThrow { success, fumble, .. } => {
                self.keg_throws.record(*success, false);
                if *fumble { self.keg_throw_fumbles += 1; }
            }
            GameEvent::BlitzRoll { .. } => { self.blitz_rolls += 1; }
            GameEvent::SpecialEffectRoll { effect, .. } => {
                *self.special_effect_rolls.entry(effect.clone()).or_default() += 1;
            }
            GameEvent::ThrowAtStallingPlayer { success, .. } =>
                self.throw_at_stalling_player_rolls.record(*success, false),
            GameEvent::ThrowAtPlayer { successful, .. } =>
                self.throw_at_player_rolls.record(*successful, false),
            GameEvent::Fumblerooskie { used, .. } =>
                self.fumblerooskie_uses.record(*used, false),
            GameEvent::AllYouCanEatRoll { success, rerolled, .. } =>
                self.all_you_can_eat_rolls.record(*success, *rerolled),
            GameEvent::PumpUpTheCrowdReRoll { .. } => { self.pump_up_the_crowd_rerolls += 1; }
            GameEvent::SelectBlitzTarget { .. } => { self.select_blitz_targets += 1; }
            GameEvent::SelectGazeTarget { .. } => { self.select_gaze_targets += 1; }
            GameEvent::HitAndRun { .. } => { self.hit_and_run_events += 1; }
            GameEvent::KickTeamMateFumble => { self.kick_team_mate_fumbles += 1; }
            GameEvent::BiteSpectator { .. } => { self.bite_spectator_events += 1; }
            GameEvent::Leader { .. } => { self.leader_events += 1; }
            GameEvent::DefectingPlayers { .. } => { self.defecting_players += 1; }
            GameEvent::ApothecaryRoll { roll, .. } => {
                if roll.is_some() { self.apothecary_rolls += 1; }
            }
            GameEvent::Block { .. } => { self.block_actions += 1; }
            GameEvent::SwoopPlayer { .. } => { self.swoop_player_events += 1; }
            GameEvent::HandOver { .. } => { self.hand_overs += 1; }
            GameEvent::RiotousRookies { player_count, .. } => {
                self.riotous_rookies += 1;
                self.riotous_rookies_players += *player_count as u32;
            }
            GameEvent::KickoffPitchInvasionStun { .. } => {
                self.kickoff_pitch_invasion_stuns += 1;
            }
            GameEvent::PassBlock { .. } => { self.pass_blocks += 1; }
            GameEvent::PassBlockEligible { .. } => { self.pass_block_eligible_events += 1; }
            GameEvent::SpellEffectRoll { .. } => { self.spell_effect_rolls += 1; }
            GameEvent::PlayerNote { note, .. } => {
                *self.player_notes.entry(note.clone()).or_default() += 1;
            }
            GameEvent::PlayerAdded { .. } => { self.players_added += 1; }
            GameEvent::NoPlayersToField { .. } => { self.no_players_to_field_events += 1; }

            GameEvent::ReceiveChoice { receive, .. } => {
                let key = if *receive { "receive" } else { "kick" };
                *self.receive_choices.entry(key.to_string()).or_default() += 1;
            }
            GameEvent::GameOptions { .. } => { self.game_options_snapshots += 1; }
            GameEvent::DoubleHiredStarPlayer => { self.double_hired_star_players += 1; }
            GameEvent::TimeoutEnforced { .. } => { self.timeouts_enforced += 1; }
            GameEvent::WinningsRoll { .. } => { self.winnings_rolls += 1; }
        }
    }

    /// Record a prompt the agent could not handle (fed from outside `tally()` by
    /// e.g. a `UniformAgent`'s unhandled-prompt tracking).
    pub fn record_unhandled_prompt(&mut self, prompt_name: &str) {
        *self.unhandled_prompts.entry(prompt_name.to_string()).or_default() += 1;
    }
}

pub fn player_action_name(action: &PlayerAction) -> &'static str {
    match action {
        PlayerAction::Move | PlayerAction::BlitzMove | PlayerAction::HandOverMove
        | PlayerAction::PassMove | PlayerAction::FoulMove | PlayerAction::GazeMove
        | PlayerAction::KickTeamMateMove | PlayerAction::ThrowTeamMateMove
        | PlayerAction::PuntMove | PlayerAction::PutridRegurgitationMove => "Move",
        PlayerAction::Block | PlayerAction::PutridRegurgitationBlock
        | PlayerAction::KickEmBlock => "Block",
        PlayerAction::Blitz | PlayerAction::BlitzSelect
        | PlayerAction::PutridRegurgitationBlitz | PlayerAction::KickEmBlitz => "Blitz",
        PlayerAction::StandUpBlitz => "StandUpBlitz",
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

pub fn skill_name_from_u16(id: u16) -> String {
    SKILL_TABLE.get(id as usize)
        .map(|s| s.id.class_name().to_string())
        .unwrap_or_else(|| format!("Unknown({})", id))
}

pub fn build_skill_names() -> HashMap<String, String> {
    SKILL_TABLE.iter()
        .map(|entry| (entry.id.class_name().to_string(), entry.id.class_name().to_string()))
        .collect()
}

// ── Inducement catalog (for checklist cross-reference) ─────────────────────────

/// One entry from an edition's `data/inducements/*.json` catalog, trimmed to what
/// the coverage checklist needs to flag "inducement X was never bought."
pub struct InducementCatalogEntry {
    pub id: String,
    pub name: String,
    /// True when the JSON entry has no `availability` gate (i.e. it's purchasable
    /// by any team in a normal game, not just under a special rule/edition quirk).
    pub universally_available: bool,
}

/// Known inducement ids/names for one edition ("bb2016" | "bb2020" | "bb2025",
/// defaulting to bb2025 for anything else). Backed by the same `include_str!`
/// JSON catalogs already loaded by `ffb_model::data::loader`.
pub fn known_inducements(edition: &str) -> Vec<InducementCatalogEntry> {
    use ffb_model::data::loader::{BB2016_INDUCEMENTS, BB2020_INDUCEMENTS, BB2025_INDUCEMENTS};
    let catalog = match edition {
        "bb2016" => &*BB2016_INDUCEMENTS,
        "bb2020" => &*BB2020_INDUCEMENTS,
        _ => &*BB2025_INDUCEMENTS,
    };
    catalog.inducements.iter()
        .map(|i| InducementCatalogEntry {
            id: i.id.clone(),
            name: i.name.clone(),
            universally_available: i.availability.is_empty(),
        })
        .collect()
}

// ── HTML generation ────────────────────────────────────────────────────────────

pub fn generate_html(json: &str) -> String {
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
  .tiles {{ display: flex; flex-wrap: wrap; gap: 12px; margin-bottom: 0; }}
  .tile {{ background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 12px 16px; min-width: 120px; }}
  .tile .val {{ font-size: 22px; font-weight: 700; color: var(--accent); }}
  .tile .lbl {{ font-size: 11px; color: var(--muted); margin-top: 2px; text-transform: uppercase; letter-spacing: .05em; }}
  .bar-chart {{ display: flex; flex-direction: column; gap: 6px; }}
  .bar-row {{ display: flex; align-items: center; gap: 8px; }}
  .bar-label {{ min-width: 140px; max-width: 160px; color: var(--muted); font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex-shrink: 0; }}
  .bar-track {{ flex: 1; background: var(--bg); border-radius: 3px; height: 14px; }}
  .bar-fill {{ height: 100%; border-radius: 3px; }}
  .bar-val {{ min-width: 50px; text-align: right; font-size: 12px; color: var(--muted); font-variant-numeric: tabular-nums; }}
  table {{ width: 100%; border-collapse: collapse; font-size: 12px; }}
  th {{ text-align: left; padding: 6px 8px; color: var(--muted); font-weight: 600; font-size: 11px; text-transform: uppercase; border-bottom: 1px solid var(--border); }}
  td {{ padding: 5px 8px; border-bottom: 1px solid #1c2128; }}
  tr:last-child td {{ border-bottom: none; }}
  .num {{ text-align: right; font-variant-numeric: tabular-nums; }}
  .pct {{ text-align: right; font-variant-numeric: tabular-nums; color: var(--muted); }}
  .badge {{ display: inline-block; padding: 1px 6px; border-radius: 10px; font-size: 10px; font-weight: 600; }}
  .badge-green {{ background: #1a3a2a; color: var(--green); }}
  .badge-red {{ background: #3a1a1a; color: var(--red); }}
  .badge-yellow {{ background: #3a2a00; color: var(--yellow); }}
  .result-bar {{ display: flex; height: 20px; border-radius: 4px; overflow: hidden; margin-top: 8px; }}
  .result-bar .seg {{ display: flex; align-items: center; justify-content: center; font-size: 10px; font-weight: 600; }}
  .seg-home {{ background: var(--accent); color: #000; }}
  .seg-draw {{ background: var(--muted); color: #000; }}
  .seg-away {{ background: var(--purple); color: #000; }}
  .two-col {{ display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }}
</style>
</head>
<body>
<div class="header">
  <div>
    <h1>FFB-Rust Coverage Dashboard</h1>
    <div class="subtitle" id="subtitle">Loading…</div>
  </div>
</div>
<div class="main" id="main"><div class="card"><p style="color:var(--muted)">Loading…</p></div></div>
<script>
const RAW = {json};
function fmt(n) {{ return n == null ? '—' : n.toLocaleString(); }}
function pct(n, d) {{ if (!d) return '—'; return (n/d*100).toFixed(1)+'%'; }}
function pctOf(n, d) {{ if (!d) return 0; return n/d*100; }}
function colorFor(i) {{
  const p = ['#58a6ff','#3fb950','#d29922','#bc8cff','#f85149','#39d353','#ff7b72','#79c0ff','#ffa657','#56d364'];
  return p[i % p.length];
}}
function barChart(entries, maxVal, colorFn) {{
  if (!entries.length) return '<em style="color:var(--muted);font-size:12px">No data</em>';
  return entries.map(([label, val], i) => {{
    const w = maxVal > 0 ? (val / maxVal * 100).toFixed(1) : 0;
    const color = colorFn ? colorFn(label, i) : colorFor(i);
    return `<div class="bar-row"><span class="bar-label" title="${{label}}">${{label}}</span><div class="bar-track"><div class="bar-fill" style="width:${{w}}%;background:${{color}}"></div></div><span class="bar-val">${{fmt(val)}}</span></div>`;
  }}).join('');
}}
function rollTable(rows) {{
  const header = `<tr><th>Roll Type</th><th class="num">Total</th><th class="pct">Success%</th><th class="pct">Rerolled%</th></tr>`;
  const body = rows.filter(r => r[1] && r[1].total > 0).map(([name, stats]) => {{
    const t = stats.total || 0;
    return `<tr><td>${{name}}</td><td class="num">${{fmt(t)}}</td><td class="pct">${{pct(stats.success, t)}}</td><td class="pct">${{pct(stats.rerolled, t)}}</td></tr>`;
  }}).join('');
  if (!body) return '<em style="color:var(--muted);font-size:12px">No rolls recorded</em>';
  return `<table><thead>${{header}}</thead><tbody>${{body}}</tbody></table>`;
}}
function render() {{
  const D = RAW;
  const games = D.games || 0;
  const totalTDs = (D.touchdowns_home || 0) + (D.touchdowns_away || 0);
  document.getElementById('subtitle').textContent = `${{games.toLocaleString()}} games · ${{D.matchups ? D.matchups.length : 1}} matchup(s)`;
  const sections = [];

  const homeWinPct = pctOf(D.home_wins, games);
  const drawPct = pctOf(D.draws, games);
  const awayWinPct = pctOf(D.away_wins, games);
  sections.push(`<div class="card"><h2>Overview</h2>
    <div class="tiles">
      <div class="tile"><div class="val">${{fmt(games)}}</div><div class="lbl">Games</div></div>
      <div class="tile"><div class="val">${{fmt(totalTDs)}}</div><div class="lbl">Touchdowns</div></div>
      <div class="tile"><div class="val">${{fmt(D.touchbacks)}}</div><div class="lbl">Touchbacks</div></div>
      <div class="tile"><div class="val">${{fmt(D.home_wins)}}</div><div class="lbl">Home Wins</div></div>
      <div class="tile"><div class="val">${{fmt(D.draws)}}</div><div class="lbl">Draws</div></div>
      <div class="tile"><div class="val">${{fmt(D.away_wins)}}</div><div class="lbl">Away Wins</div></div>
      <div class="tile"><div class="val">${{fmt(D.rerolls_total)}}</div><div class="lbl">Re-rolls</div></div>
      <div class="tile"><div class="val">${{fmt(D.fouls)}}</div><div class="lbl">Fouls</div></div>
      <div class="tile"><div class="val">${{fmt(D.weather_changes)}}</div><div class="lbl">Weather Changes</div></div>
    </div>
    <div class="result-bar" style="margin-top:12px">
      <div class="seg seg-home" style="width:${{homeWinPct.toFixed(1)}}%">${{homeWinPct > 5 ? pct(D.home_wins, games) : ''}}</div>
      <div class="seg seg-draw" style="width:${{drawPct.toFixed(1)}}%">${{drawPct > 5 ? pct(D.draws, games) : ''}}</div>
      <div class="seg seg-away" style="width:${{awayWinPct.toFixed(1)}}%">${{awayWinPct > 5 ? pct(D.away_wins, games) : ''}}</div>
    </div>
    <div style="margin-top:6px">
      <span class="badge badge-green">Home ${{pct(D.home_wins, games)}}</span>
      <span class="badge badge-yellow" style="margin-left:4px">Draw ${{pct(D.draws, games)}}</span>
      <span class="badge badge-red" style="margin-left:4px">Away ${{pct(D.away_wins, games)}}</span>
    </div></div>`);

  const activations = Object.entries(D.activations || {{}}).sort((a,b) => b[1]-a[1]);
  const maxAct = activations.length ? activations[0][1] : 1;
  sections.push(`<div class="card"><h2>Player Activations</h2><div class="bar-chart">${{barChart(activations, maxAct)}}</div></div>`);

  const rollRows = [
    ['DodgeRoll', D.dodge_rolls],['GoForItRoll', D.go_for_it_rolls],
    ['CatchRoll', D.catch_rolls],['InterceptionRoll', D.interception_rolls],
    ['JumpRoll', D.jump_rolls],['JumpUpRoll', D.jump_up_rolls],
    ['DauntlessRoll', D.dauntless_rolls],['LonerRoll', D.loner_rolls],
    ['ProRoll', D.pro_rolls],['FoulAppearanceRoll', D.foul_appearance_rolls],
    ['AlwaysHungryRoll', D.always_hungry_rolls],['BloodLustRoll', D.blood_lust_rolls],
    ['AnimosityRoll', D.animosity_rolls],['ConfusionRoll', D.confusion_rolls],
    ['HypnoticGazeRoll', D.hypnotic_gaze_rolls],['EscapeRoll', D.escape_rolls],
    ['RightStuffRoll', D.right_stuff_rolls],['StandUpRoll', D.stand_up_rolls],
    ['BreatheFireRoll', D.breathe_fire_rolls],['ProjectileVomitRoll', D.projectile_vomit_rolls],
    ['AnimalSavageryRoll', D.animal_savagery_rolls],['ArgueTheCallRoll', D.argue_the_call_rolls],
    ['BribesRoll', D.bribes_rolls],
  ];
  sections.push(`<div class="card"><h2>Dice Rolls</h2>${{rollTable(rollRows)}}</div>`);

  const blockByDice = Object.entries(D.block_rolls?.by_dice || {{}})
    .sort((a,b) => Number(a[0]) - Number(b[0]))
    .map(([k, v]) => [k === '-1' ? '2d (def)' : k === '1' ? '1d' : k === '2' ? '2d (att)' : `${{k}}d`, v]);
  const maxBlock = blockByDice.length ? Math.max(...blockByDice.map(e=>e[1])) : 1;
  sections.push(`<div class="card"><h2>Block Rolls</h2>
    <div class="tiles" style="margin-bottom:12px">
      <div class="tile"><div class="val">${{fmt(D.block_rolls?.total)}}</div><div class="lbl">Total Blocks</div></div>
      <div class="tile"><div class="val">${{fmt(D.block_rolls?.rerolled)}}</div><div class="lbl">Rerolled</div></div>
      <div class="tile"><div class="val">${{fmt(D.total_pushbacks)}}</div><div class="lbl">Pushbacks</div></div>
      <div class="tile"><div class="val">${{fmt(D.push_destinations_hist?.['1'])}}</div><div class="lbl">1-option push (edge)</div></div>
      <div class="tile"><div class="val">${{fmt(D.push_destinations_hist?.['3+'])}}</div><div class="lbl">3-option push</div></div>
    </div>
    <div class="bar-chart">${{barChart(blockByDice, maxBlock)}}</div></div>`);

  const passByDist = Object.entries(D.pass_rolls?.by_distance || {{}}).sort((a,b)=>b[1]-a[1]);
  const passByResult = Object.entries(D.pass_rolls?.by_result || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxPassDist = passByDist.length ? passByDist[0][1] : 1;
  const maxPassRes = passByResult.length ? passByResult[0][1] : 1;
  sections.push(`<div class="card"><h2>Pass Rolls</h2>
    <div class="tiles" style="margin-bottom:12px">
      <div class="tile"><div class="val">${{fmt(D.pass_rolls?.total)}}</div><div class="lbl">Total Passes</div></div>
      <div class="tile"><div class="val">${{fmt(D.pass_deviates)}}</div><div class="lbl">Pass Deviations</div></div>
      <div class="tile"><div class="val">${{fmt(D.throw_team_mate_rolls)}}</div><div class="lbl">Throw-Teammate</div></div>
    </div>
    <div class="two-col">
      <div><h3>By Distance</h3><div class="bar-chart">${{barChart(passByDist, maxPassDist)}}</div></div>
      <div><h3>By Result</h3><div class="bar-chart">${{barChart(passByResult, maxPassRes)}}</div></div>
    </div></div>`);

  const inj = D.injuries || {{}};
  const injSections = [['Armor Only', inj.armor_only||0,'#8b949e'],['KO',inj.ko||0,'#d29922'],['CAS',inj.cas||0,'#f85149'],['Dead',inj.dead||0,'#7a1a1a']];
  const maxInj = Math.max(...injSections.map(e=>e[1]), 1);
  const seriousEntries = Object.entries(inj.by_serious_injury || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxSi = seriousEntries.length ? seriousEntries[0][1] : 1;
  sections.push(`<div class="card"><h2>Injuries</h2>
    <div class="tiles" style="margin-bottom:12px">
      <div class="tile"><div class="val">${{fmt(inj.total)}}</div><div class="lbl">Total</div></div>
      <div class="tile"><div class="val">${{fmt(inj.ko)}}</div><div class="lbl">KOs</div></div>
      <div class="tile"><div class="val">${{fmt(inj.cas)}}</div><div class="lbl">CAS</div></div>
      <div class="tile"><div class="val">${{fmt(inj.dead)}}</div><div class="lbl">Deaths</div></div>
    </div>
    <div class="two-col">
      <div><h3>Severity</h3><div class="bar-chart">${{barChart(injSections.map(e=>[e[0],e[1]]), maxInj, (l,i)=>injSections[i]?.[2])}}</div></div>
      <div><h3>Serious Injury Types</h3><div class="bar-chart">${{barChart(seriousEntries.slice(0,12), maxSi)}}</div></div>
    </div></div>`);

  const skillUsed = Object.entries(D.skill_used || {{}}).sort((a,b)=>b[1]-a[1]);
  const skillDeclined = Object.entries(D.skill_declined || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxSU = skillUsed.length ? skillUsed[0][1] : 1;
  const maxSD = skillDeclined.length ? skillDeclined[0][1] : 1;
  sections.push(`<div class="card"><h2>Skill Usage</h2>
    <div class="two-col">
      <div><h3>Used (${{skillUsed.length}} skills)</h3><div class="bar-chart">${{barChart(skillUsed, maxSU)}}</div></div>
      <div><h3>Declined (${{skillDeclined.length}} skills)</h3><div class="bar-chart">${{barChart(skillDeclined, maxSD)}}</div></div>
    </div></div>`);

  const kickoffEntries = Object.entries(D.kickoff_events || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxKO = kickoffEntries.length ? kickoffEntries[0][1] : 1;
  sections.push(`<div class="card"><h2>Kickoff Events</h2>
    <div class="tiles" style="margin-bottom:12px">
      <div class="tile"><div class="val">${{fmt(D.kickoff_scatters)}}</div><div class="lbl">Kickoff Scatters</div></div>
      <div class="tile"><div class="val">${{fmt(D.touchbacks)}}</div><div class="lbl">Touchbacks</div></div>
      <div class="tile"><div class="val">${{fmt(D.kickoff_pitch_invasions)}}</div><div class="lbl">Pitch Invasions</div></div>
      <div class="tile"><div class="val">${{fmt(D.kickoff_riots)}}</div><div class="lbl">Riots</div></div>
    </div>
    <div class="bar-chart">${{barChart(kickoffEntries, maxKO)}}</div></div>`);

  const ballEntries = [['Touchdowns',D.touchdowns||0],['Ball Picked Up',D.ball_picked_up||0],['Scatter Balls',D.scatter_balls||0],['Throw-ins',D.throw_ins||0],['Pass Deviates',D.pass_deviates||0],['Players Fell Down',D.players_fell_down||0]];
  const maxBall = Math.max(...ballEntries.map(e=>e[1]), 1);
  sections.push(`<div class="card"><h2>Ball &amp; Movement Events</h2><div class="bar-chart">${{barChart(ballEntries, maxBall)}}</div></div>`);

  sections.push(`<div class="card"><h2>Fouls &amp; Discipline</h2>
    <div class="tiles">
      <div class="tile"><div class="val">${{fmt(D.fouls)}}</div><div class="lbl">Fouls</div></div>
      <div class="tile"><div class="val">${{fmt(D.players_ejected)}}</div><div class="lbl">Ejected</div></div>
      <div class="tile"><div class="val">${{pct(D.argue_the_call_rolls?.success, D.argue_the_call_rolls?.total)}}</div><div class="lbl">Argue Success%</div></div>
      <div class="tile"><div class="val">${{fmt(D.apothecary_used)}}</div><div class="lbl">Apothecary Used</div></div>
      <div class="tile"><div class="val">${{fmt(D.secret_weapon_bans)}}</div><div class="lbl">SW Bans</div></div>
      <div class="tile"><div class="val">${{fmt(D.bomb_explosions)}}</div><div class="lbl">Bomb Explosions</div></div>
    </div></div>`);

  const rrEntries = Object.entries(D.rerolls_by_source || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxRR = rrEntries.length ? rrEntries[0][1] : 1;
  sections.push(`<div class="card"><h2>Re-rolls</h2>
    <div class="tiles" style="margin-bottom:12px">
      <div class="tile"><div class="val">${{fmt(D.rerolls_total)}}</div><div class="lbl">Total</div></div>
      <div class="tile"><div class="val">${{(D.rerolls_total/Math.max(games,1)).toFixed(2)}}</div><div class="lbl">Per Game</div></div>
    </div>
    <div class="bar-chart">${{barChart(rrEntries, maxRR)}}</div></div>`);

  const miscEntries = [['Turn Ends',D.turn_ends||0],['Half Starts',D.half_starts||0],['Coin Throws',D.coin_throws||0],['MVP Rolls',D.mvp_rolls||0],['Chainsaw Rolls',D.chainsaw_rolls||0],['Piling On',D.piling_on_events||0],['Swarming Rolls',D.swarming_rolls||0],['Heat Exhaustion',D.heat_exhaustion||0],['Trap Door',D.trap_door_rolls||0]];
  const maxMisc = Math.max(...miscEntries.map(e=>e[1]), 1);
  sections.push(`<div class="card"><h2>Misc Events</h2><div class="bar-chart">${{barChart(miscEntries, maxMisc)}}</div></div>`);

  // ── Unhandled prompts (key diagnostic: should always be empty) ───────────────
  const unhandled = Object.entries(D.unhandled_prompts || {{}}).sort((a,b)=>b[1]-a[1]);
  const unhandledTotal = unhandled.reduce((s,[,v])=>s+v, 0);
  const maxUnhandled = unhandled.length ? unhandled[0][1] : 1;
  const unhandledStyle = unhandledTotal > 0 ? 'border-color:var(--red)' : '';
  sections.push(`<div class="card" style="${{unhandledStyle}}"><h2>Unhandled Prompts ${{unhandledTotal > 0 ? '<span class=\"badge badge-red\">' + fmt(unhandledTotal) + ' leaked</span>' : '<span class=\"badge badge-green\">none</span>'}}</h2>${{barChart(unhandled, maxUnhandled)}}</div>`);

  // ── Misc/Other events (Task-1 gap-fill counters) ──────────────────────────────
  const otherTiles = [
    ['Leader Events', D.leader_events||0], ['Defecting Players', D.defecting_players||0],
    ['Coach Bans', D.coach_bans||0], ['Hand Overs', D.hand_overs||0],
    ['Swoop Player', D.swoop_player_events||0], ['Riotous Rookies', D.riotous_rookies||0],
    ['Block Actions', D.block_actions||0], ['Apothecary Rolls', D.apothecary_rolls||0],
    ['Select Blitz Target', D.select_blitz_targets||0], ['Select Gaze Target', D.select_gaze_targets||0],
    ['Hit And Run', D.hit_and_run_events||0], ['Kick Team-Mate Fumble', D.kick_team_mate_fumbles||0],
    ['Bite Spectator', D.bite_spectator_events||0], ['Pass Block', D.pass_blocks||0],
    ['Pass Block Eligible', D.pass_block_eligible_events||0], ['Spell Effect Rolls', D.spell_effect_rolls||0],
    ['Players Added', D.players_added||0], ['No Players To Field', D.no_players_to_field_events||0],
    ['Petty Cash Events', D.petty_cash_events||0], ['Cards+Inducements Reports', D.cards_and_inducements_bought_reports||0],
    ['Double Hired Star', D.double_hired_star_players||0], ['Timeouts Enforced', D.timeouts_enforced||0],
    ['Winnings Rolls', D.winnings_rolls||0], ['Game Options Snapshots', D.game_options_snapshots||0],
    ['Kickoff Seq. Exhausted', D.kickoff_sequence_activations_exhausted||0], ['Solid Defence Rolls', D.solid_defence_rolls||0],
    ['Cheering Fans', D.cheering_fans_events||0], ['Kickoff Extra ReRoll', D.kickoff_extra_rerolls||0],
    ['Quick Snap Rolls', D.quick_snap_rolls||0], ['Kickoff Timeouts', D.kickoff_timeouts||0],
    ['Kickoff Officious Ref', D.kickoff_officious_ref_events||0], ['Kickoff Dodgy Snack', D.kickoff_dodgy_snack_events||0],
    ['Dodgy Snack Rolls', D.dodgy_snack_rolls||0], ['Kickoff Pitch Invasion Stuns', D.kickoff_pitch_invasion_stuns||0],
    ['BB2016 Extra ReRoll', D.kickoff_extra_reroll_bb2016_events||0], ['BB2016 Throw-A-Rock', D.kickoff_throw_a_rock_bb2016_events||0],
    ['BB2016 Pitch Invasion', D.kickoff_pitch_invasion_bb2016_events||0], ['Master Chef Rolls', D.master_chef_rolls||0],
    ['Master Chef Rerolls Stolen', D.master_chef_rerolls_stolen||0], ['Keg Throw Fumbles', D.keg_throw_fumbles||0],
    ['Blitz Rolls', D.blitz_rolls||0], ['Pump Up The Crowd ReRoll', D.pump_up_the_crowd_rerolls||0],
    ['Prayer Amount Events', D.prayer_amount_events||0], ['ThenIStartedBlastin Fumbles', D.then_i_started_blastin_fumbles||0],
  ];
  const maxOther = Math.max(...otherTiles.map(e=>e[1]), 1);
  sections.push(`<div class="card"><h2>Misc/Other Events</h2><div class="bar-chart">${{barChart(otherTiles, maxOther)}}</div></div>`);

  const rollRows2 = [
    ['RegenerationRoll', D.regeneration_rolls],['ThenIStartedBlastinRoll', D.then_i_started_blastin_rolls],
    ['KegThrowRoll', D.keg_throws],['ThrowAtStallingPlayerRoll', D.throw_at_stalling_player_rolls],
    ['ThrowAtPlayerRoll', D.throw_at_player_rolls],['FumblerooskieUse', D.fumblerooskie_uses],
    ['AllYouCanEatRoll', D.all_you_can_eat_rolls],['RefereeSpotsFoulRoll', D.referee_spots_foul_rolls],
    ['BiasedRefRoll', D.biased_ref_rolls],
  ];
  sections.push(`<div class="card"><h2>Additional Dice Rolls</h2>${{rollTable(rollRows2)}}</div>`);

  const prayerEntries = Object.entries(D.prayer_rolls || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxPrayer = prayerEntries.length ? prayerEntries[0][1] : 1;
  const cardEffectEntries = Object.entries(D.card_effects_by_name || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxCardEffect = cardEffectEntries.length ? cardEffectEntries[0][1] : 1;
  const inducementsRegEntries = Object.entries(D.inducements_registered || {{}}).sort((a,b)=>b[1]-a[1]);
  const maxIndReg = inducementsRegEntries.length ? inducementsRegEntries[0][1] : 1;
  sections.push(`<div class="card"><h2>Prayers, Cards &amp; Inducement Registration</h2>
    <div class="two-col">
      <div><h3>Prayers Rolled</h3><div class="bar-chart">${{barChart(prayerEntries, maxPrayer)}}</div></div>
      <div><h3>Card Effects</h3><div class="bar-chart">${{barChart(cardEffectEntries, maxCardEffect)}}</div></div>
    </div>
    <div style="margin-top:12px"><h3>Inducements Registered (start-of-half)</h3><div class="bar-chart">${{barChart(inducementsRegEntries, maxIndReg)}}</div></div></div>`);

  document.getElementById('main').innerHTML = sections.join('');
}}
render();
</script>
</body>
</html>
"#, json = json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn tally_prayer_roll_by_id() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::PrayerRoll { team_id: "t1".into(), roll: 4, prayer_id: "Blessed".into() });
        cov.tally(&GameEvent::PrayerRoll { team_id: "t1".into(), roll: 6, prayer_id: "Blessed".into() });
        assert_eq!(cov.prayer_rolls.get("Blessed"), Some(&2));
    }

    #[test]
    fn tally_leader_and_defecting_players() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::Leader { player_id: "p1".into(), reroll_available: true });
        cov.tally(&GameEvent::DefectingPlayers { player_ids: vec!["p1".into(), "p2".into()] });
        assert_eq!(cov.leader_events, 1);
        assert_eq!(cov.defecting_players, 1);
    }

    #[test]
    fn tally_coach_banned() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::CoachBanned { team_id: "t1".into() });
        assert_eq!(cov.coach_bans, 1);
    }

    #[test]
    fn tally_card_effect_roll_by_effect() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::CardEffectRoll { card_id: "c1".into(), roll: 3, effect: "Stun".into() });
        assert_eq!(cov.card_effects_by_name.get("Stun"), Some(&1));
    }

    #[test]
    fn tally_then_i_started_blastin_records_success_and_fumble() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::ThenIStartedBlastin {
            attacker_id: "p1".into(), defender_id: None, roll: 1, success: false, fumble: true,
        });
        assert_eq!(cov.then_i_started_blastin_rolls.total, 1);
        assert_eq!(cov.then_i_started_blastin_rolls.failure, 1);
        assert_eq!(cov.then_i_started_blastin_fumbles, 1);
    }

    #[test]
    fn tally_master_chef_roll_accumulates_rerolls_stolen() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::MasterChefRoll { team_id: "t1".into(), roll: 5, rerolls_stolen: 2 });
        cov.tally(&GameEvent::MasterChefRoll { team_id: "t1".into(), roll: 3, rerolls_stolen: 0 });
        assert_eq!(cov.master_chef_rolls, 2);
        assert_eq!(cov.master_chef_rerolls_stolen, 2);
    }

    #[test]
    fn tally_keg_throw_records_success_and_fumble() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::KegThrow {
            thrower_id: "p1".into(), target_id: Some("p2".into()), roll: 2, success: false, fumble: true,
        });
        assert_eq!(cov.keg_throws.total, 1);
        assert_eq!(cov.keg_throw_fumbles, 1);
    }

    #[test]
    fn tally_receive_choice_buckets_by_receive_flag() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::ReceiveChoice { team_id: "t1".into(), receive: true });
        cov.tally(&GameEvent::ReceiveChoice { team_id: "t2".into(), receive: false });
        assert_eq!(cov.receive_choices.get("receive"), Some(&1));
        assert_eq!(cov.receive_choices.get("kick"), Some(&1));
    }

    #[test]
    fn tally_double_hired_star_player() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::DoubleHiredStarPlayer);
        assert_eq!(cov.double_hired_star_players, 1);
    }

    #[test]
    fn tally_regeneration_roll_records_success() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::RegenerationRoll { player_id: "p1".into(), roll: 4, success: true });
        assert_eq!(cov.regeneration_rolls.total, 1);
        assert_eq!(cov.regeneration_rolls.success, 1);
    }

    #[test]
    fn tally_inducement_registered_by_type() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::Inducement { team_id: "t1".into(), inducement_type: "WanderingApothecary".into(), value: 1 });
        assert_eq!(cov.inducements_registered.get("WanderingApothecary"), Some(&1));
    }

    #[test]
    fn tally_block_action_and_swoop_and_handover() {
        let mut cov = CoverageReport::default();
        cov.tally(&GameEvent::Block { defender_id: "d1".into() });
        cov.tally(&GameEvent::SwoopPlayer { player_id: "p1".into(), coord: FieldCoordinate { x: 1, y: 1 } });
        cov.tally(&GameEvent::HandOver { from_id: "p1".into(), to_id: "p2".into() });
        assert_eq!(cov.block_actions, 1);
        assert_eq!(cov.swoop_player_events, 1);
        assert_eq!(cov.hand_overs, 1);
    }

    #[test]
    fn record_unhandled_prompt_increments_map() {
        let mut cov = CoverageReport::default();
        cov.record_unhandled_prompt("SelectSquare");
        cov.record_unhandled_prompt("SelectSquare");
        cov.record_unhandled_prompt("SelectPlayer");
        assert_eq!(cov.unhandled_prompts.get("SelectSquare"), Some(&2));
        assert_eq!(cov.unhandled_prompts.get("SelectPlayer"), Some(&1));
    }

    #[test]
    fn known_inducements_bb2025_includes_bribes() {
        let entries = known_inducements("bb2025");
        assert!(entries.iter().any(|e| e.id == "bribes" && e.universally_available));
    }
}
