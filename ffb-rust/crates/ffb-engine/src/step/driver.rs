//! Driver — replaces engine.rs once all step bodies are implemented.
//! Contains `make_step`, a LIFO `StepStack`, and the `DriverGameState` game loop.
//! Uses `Box<dyn Step>` (no `StepKind` enum) and `SequenceStep` for pushes.

use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::model::team::Team;
use ffb_model::enums::Rules;
use ffb_model::events::GameEvent;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::state_hash::state_hash;

use crate::action::Action;
use crate::legal_actions::TeamSide;
use super::framework::{Step, StepOutcome, StepAction, StepId, StepParameter, SequenceStep};

// ── NoOpStep ─────────────────────────────────────────────────────────────────

/// Fallback for StepId variants not yet wired to a concrete struct.
/// Returns `next()` immediately without doing anything.
struct NoOpStep(StepId);

impl Step for NoOpStep {
    fn id(&self) -> StepId { self.0 }
    fn start(&mut self, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }
    fn handle_command(&mut self, _action: &Action, _game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        StepOutcome::next()
    }
}

// ── make_step ────────────────────────────────────────────────────────────────

/// Dispatch a `StepId` to the concrete BB2025 step struct that implements it.
/// Unimplemented StepIds fall back to `NoOpStep`.
pub fn make_step(id: StepId) -> Box<dyn Step> {
    use crate::step::bb2025::block::*;
    use crate::step::bb2025::end::*;
    use crate::step::bb2025::foul::*;
    use crate::step::bb2025::inducements::*;
    use crate::step::bb2025::kickoff::*;
    use crate::step::bb2025::move_::*;
    use crate::step::bb2025::mutliblock::*;
    use crate::step::bb2025::pass::*;
    use crate::step::bb2025::punt::*;
    use crate::step::bb2025::shared::*;
    use crate::step::bb2025::special::*;
    use crate::step::bb2025::start::*;
    use crate::step::bb2025::ttm::*;
    use crate::step::bb2025::step_auto_gaze_zoat::StepAutoGazeZoat;
    use crate::step::bb2025::step_baleful_hex::StepBalefulHex;
    use crate::step::bb2025::step_black_ink::StepBlackInk;
    use crate::step::bb2025::step_catch_of_the_day::StepCatchOfTheDay;
    use crate::step::bb2025::step_end_furious_outburst::StepEndFuriousOutburst;
    use crate::step::bb2025::step_end_turn::StepEndTurn;
    use crate::step::bb2025::step_look_into_my_eyes::StepLookIntoMyEyes;
    use crate::step::bb2025::step_prayer::StepPrayer;
    use crate::step::bb2025::step_raiding_party::StepRaidingParty;
    use crate::step::bb2025::step_select_blitz_target::StepSelectBlitzTarget;
    use crate::step::bb2025::step_then_i_started_blastin::StepThenIStartedBlastin;
    use crate::step::bb2025::step_treacherous::StepTreacherous;
    use crate::step::bb2025::step_wisdom_of_the_white_dwarf::StepWisdomOfTheWhiteDwarf;
    use crate::step::game::start::step_init_start_game::StepInitStartGame;
    use crate::step::game::start::step_weather::StepWeather;
    use crate::step::mixed::block::step_both_down::StepBothDown;
    use crate::step::mixed::end::step_dedicated_fans::StepDedicatedFans;
    use crate::step::mixed::start::step_petty_cash::StepPettyCash;
    use crate::step::mixed::step_init_look_into_my_eyes::StepInitLookIntoMyEyes;
    use crate::step::mixed::foul::step_eject_player::StepEjectPlayer;
    use crate::step::mixed::foul::step_pile_driver::StepPileDriver;
    use crate::step::mixed::multiblock::step_dispatch_dump_off::StepDispatchDumpOff;
    use crate::step::mixed::multiblock::step_double_strength::StepDoubleStrength;
    use crate::step::mixed::shared::step_consume_parameter::StepConsumeParameter;
    use crate::step::mixed::shared::step_set_defender::StepSetDefender;
    use crate::step::mixed::start::step_spectators::StepSpectators;
    use crate::step::phase::kickoff::step_coin_choice::StepCoinChoice;
    use crate::step::phase::kickoff::step_receive_choice::StepReceiveChoice;
    use crate::step::phase::kickoff::step_end_kickoff::StepEndKickoff;
    use crate::step::phase::kickoff::step_touchback::StepTouchback;
    use crate::step::step_goto_label::StepGotoLabel;

    match id {
        // ── Start of game ────────────────────────────────────────────────────
        StepId::InitStartGame          => Box::new(StepInitStartGame::new()),
        StepId::Spectators             => Box::new(StepSpectators::new()),
        StepId::Weather                => Box::new(StepWeather::new()),
        StepId::CoinChoice             => Box::new(StepCoinChoice::new()),
        StepId::ReceiveChoice          => Box::new(StepReceiveChoice::new()),
        // ── Kickoff ──────────────────────────────────────────────────────────
        StepId::InitKickoff            => Box::new(StepInitKickoff::new()),
        StepId::Kickoff                => Box::new(step_kickoff::StepKickoff::new()),
        StepId::Setup                  => Box::new(step_setup::StepSetup::new()),
        StepId::KickoffScatterRoll     => Box::new(step_kickoff_scatter_roll::StepKickoffScatterRoll::new()),
        StepId::KickoffScatterRollAskAfter => Box::new(step_kickoff_scatter_roll_ask_after::StepKickoffScatterRollAskAfter::new()),
        StepId::KickoffResultRoll      => Box::new(step_kickoff_result_roll::StepKickoffResultRoll::new()),
        StepId::ApplyKickoffResult     => Box::new(step_apply_kickoff_result::StepApplyKickoffResult::new(String::new(), String::new())),
        StepId::EndKickoff             => Box::new(StepEndKickoff::new()),
        StepId::BlitzTurn              => Box::new(StepBlitzTurn::new()),
        StepId::Swarming               => Box::new(step_swarming::StepSwarming::new()),
        StepId::Touchback              => Box::new(StepTouchback::new()),
        // ── Select / activation ──────────────────────────────────────────────
        StepId::InitSelecting          => Box::new(step_init_selecting::StepInitSelecting::new(String::new())),
        StepId::EndSelecting           => Box::new(step_end_selecting::StepEndSelecting::new()),
        StepId::InitActivation         => Box::new(StepInitActivation::new()),
        StepId::StandUp                => Box::new(step_stand_up::StepStandUp::new(String::new())),
        // ── Movement ─────────────────────────────────────────────────────────
        StepId::InitMoving             => Box::new(step_init_moving::StepInitMoving::new(String::new())),
        StepId::Move                   => Box::new(step_move::StepMove::new()),
        StepId::GoForIt                => Box::new(step_go_for_it::StepGoForIt::new(String::new())),
        StepId::MoveDodge              => Box::new(step_move_dodge::StepMoveDodge::new(String::new())),
        StepId::FallDown               => Box::new(step_fall_down::StepFallDown::new()),
        StepId::EndMoving              => Box::new(step_end_moving::StepEndMoving::new()),
        StepId::HypnoticGaze           => Box::new(step_hypnotic_gaze::StepHypnoticGaze::new(String::new())),
        StepId::Jump                   => Box::new(step_jump::StepJump::new(String::new())),
        StepId::Shadowing              => Box::new(step_shadowing::StepShadowing::new()),
        StepId::PickUp                 => Box::new(step_pick_up::StepPickUp::new(String::new())),
        // ── Block ────────────────────────────────────────────────────────────
        StepId::InitBlocking           => Box::new(step_init_blocking::StepInitBlocking::new(String::new())),
        StepId::BlockRoll              => Box::new(step_block_roll::StepBlockRoll::new()),
        StepId::BlockChoice            => Box::new(step_block_choice::StepBlockChoice::new(String::new(), String::new(), String::new())),
        StepId::Pushback               => Box::new(step_pushback::StepPushback::new()),
        StepId::Followup               => Box::new(step_followup::StepFollowup::new()),
        StepId::EndBlocking            => Box::new(step_end_blocking::StepEndBlocking::new()),
        StepId::DropFallingPlayers     => Box::new(step_drop_falling_players::StepDropFallingPlayers::new()),
        StepId::PlaceBall              => Box::new(step_place_ball::StepPlaceBall::new()),
        StepId::BlockChainsaw          => Box::new(step_block_chainsaw::StepBlockChainsaw::new(String::new(), String::new())),
        StepId::BreatheFire            => Box::new(step_breathe_fire::StepBreatheFire::new(String::new(), String::new())),
        StepId::Chomp                  => Box::new(step_chomp::StepChomp::new(String::new())),
        StepId::HitAndRun              => Box::new(step_hit_and_run::StepHitAndRun::new()),
        StepId::Trickster              => Box::new(step_trickster::StepTrickster::new()),
        // ── Foul ─────────────────────────────────────────────────────────────
        StepId::InitFouling            => Box::new(step_init_fouling::StepInitFouling::new(String::new())),
        StepId::Bribes                 => Box::new(step_bribes::StepBribes::new(String::new())),
        StepId::EndFouling             => Box::new(step_end_fouling::StepEndFouling::new()),
        // ── Punt ─────────────────────────────────────────────────────────────
        StepId::InitPunt               => Box::new(step_init_punt::StepInitPunt::new(String::new())),
        StepId::PuntDirection          => Box::new(step_punt_direction::StepPuntDirection::new(String::new())),
        StepId::PuntDistance           => Box::new(step_punt_distance::StepPuntDistance::new()),
        StepId::EndPunt                => Box::new(step_end_punt::StepEndPunt::new()),
        // ── Pass / ball ──────────────────────────────────────────────────────
        StepId::Pass                   => Box::new(step_pass::StepPass::new(String::new(), String::new(), String::new())),
        StepId::Intercept              => Box::new(step_intercept::StepIntercept::new(String::new())),
        StepId::ResolvePass            => Box::new(step_resolve_pass::StepResolvePass::new()),
        StepId::HandOver               => Box::new(step_hand_over::StepHandOver::new()),
        StepId::MissedPass             => Box::new(step_missed_pass::StepMissedPass::new()),
        StepId::EndPassing             => Box::new(step_end_passing::StepEndPassing::new()),
        StepId::HailMaryPass           => Box::new(step_hail_mary_pass::StepHailMaryPass::new(String::new())),
        StepId::CatchScatterThrowIn    => Box::new(step_catch_scatter_throw_in::StepCatchScatterThrowIn::new()),
        // ── Inducements ──────────────────────────────────────────────────────
        StepId::InitInducement         => Box::new(step_init_inducement::StepInitInducement::default()),
        StepId::EndInducement          => Box::new(step_end_inducement::StepEndInducement::new(false)),
        StepId::ThrowARock             => Box::new(step_throw_a_rock::StepThrowARock::new(false)),
        StepId::WeatherMage            => Box::new(step_weather_mage::StepWeatherMage::new()),
        // ── Multi-block ──────────────────────────────────────────────────────
        StepId::MultipleBlockFork      => Box::new(step_multiple_block_fork::StepMultipleBlockFork::new(vec![])),
        StepId::BlockRollMultiple      => Box::new(step_block_roll_multiple::StepBlockRollMultiple::new()),
        StepId::ApothecaryMultiple     => Box::new(step_apothecary_multiple::StepApothecaryMultiple::new(String::new())),
        // ── Negatraits ───────────────────────────────────────────────────────
        StepId::BoneHead               => { use crate::step::action::common::step_bone_head::StepBoneHead; Box::new(StepBoneHead::new()) }
        StepId::ReallyStupid           => { use crate::step::action::common::step_really_stupid::StepReallyStupid; Box::new(StepReallyStupid::new()) }
        StepId::WildAnimal             => { use crate::step::bb2016::step_wild_animal::StepWildAnimal; Box::new(StepWildAnimal::new(String::new())) }
        // ── Block skills ─────────────────────────────────────────────────────
        StepId::Juggernaut             => { use crate::step::action::block::step_juggernaut::StepJuggernaut; Box::new(StepJuggernaut::new()) }
        StepId::Dauntless              => { use crate::step::action::block::step_dauntless::StepDauntless; Box::new(StepDauntless::new()) }
        StepId::DumpOff                => { use crate::step::action::block::step_dump_off::StepDumpOff; Box::new(StepDumpOff::new()) }
        StepId::Stab                   => { use crate::step::action::block::step_stab::StepStab; Box::new(StepStab::new()) }
        StepId::Wrestle                => { use crate::step::action::block::step_wrestle::StepWrestle; Box::new(StepWrestle::new()) }
        // ── Move skills ──────────────────────────────────────────────────────
        StepId::DivingTackle           => { use crate::step::action::move_::step_diving_tackle::StepDivingTackle; Box::new(StepDivingTackle::new()) }
        StepId::Tentacles              => { use crate::step::mixed::move_::step_tentacles::StepTentacles; Box::new(StepTentacles::new()) }
        // ── Select skills ────────────────────────────────────────────────────
        StepId::JumpUp                 => { use crate::step::action::select::step_jump_up::StepJumpUp; Box::new(StepJumpUp::new()) }
        // ── Pass/foul skills ─────────────────────────────────────────────────
        StepId::Animosity              => { use crate::step::action::pass::step_animosity::StepAnimosity; Box::new(StepAnimosity::new(String::new())) }
        StepId::FoulAppearance         => { use crate::step::mixed::step_foul_appearance::StepFoulAppearance; Box::new(StepFoulAppearance::new(String::new())) }
        StepId::Bombardier             => { use crate::step::action::pass::step_bombardier::StepBombardier; Box::new(StepBombardier::new()) }
        StepId::PassBlock              => { use crate::step::mixed::pass::step_pass_block::StepPassBlock; Box::new(StepPassBlock::new()) }
        StepId::SafeThrow              => { use crate::step::bb2016::pass::step_safe_throw::StepSafeThrow; Box::new(StepSafeThrow::new()) }
        // ── Mixed special skills ─────────────────────────────────────────────
        StepId::AnimalSavagery         => { use crate::step::mixed::shared::step_animal_savagery::StepAnimalSavagery; Box::new(StepAnimalSavagery::new(String::new())) }
        StepId::UnchannelledFury       => { use crate::step::mixed::step_unchannelled_fury::StepUnchannelledFury; Box::new(StepUnchannelledFury::new(String::new())) }
        // ── Shared ───────────────────────────────────────────────────────────
        StepId::BloodLust              => Box::new(step_blood_lust::StepBloodLust::new(String::new())),
        StepId::EndFeeding             => Box::new(step_end_feeding::StepEndFeeding::new()),
        StepId::ForgoneStalling        => Box::new(step_forgone_stalling::StepForgoneStalling::new()),
        StepId::GettingEven            => Box::new(step_getting_even::StepGettingEven::new()),
        StepId::HandleDropPlayerContext => Box::new(step_handle_drop_player_context::StepHandleDropPlayerContext::new()),
        StepId::InitFeeding            => Box::new(step_init_feeding::StepInitFeeding::new()),
        StepId::StallingPlayer         => Box::new(StepStallingPlayer::new()),
        StepId::SteadyFooting          => Box::new(step_steady_footing::StepSteadyFooting::new(String::new(), String::new())),
        StepId::TakeRoot               => Box::new(step_take_root::StepTakeRoot::new()),
        StepId::Apothecary             => Box::new(step_apothecary::StepApothecary::new()),
        // ── End of turn / game ────────────────────────────────────────────────
        StepId::EndTurn                => Box::new(StepEndTurn::new()),
        StepId::Mvp                    => Box::new(step_mvp::StepMvp::new()),
        StepId::InitEndGame            => Box::new(step_init_end_game::StepInitEndGame::new(String::new())),
        StepId::Winnings               => Box::new(step_winnings::StepWinnings),
        StepId::PlayerLoss             => Box::new(step_player_loss::StepPlayerLoss),
        // ── Special (bomb) ────────────────────────────────────────────────────
        StepId::InitBomb               => Box::new(step_init_bomb::StepInitBomb::new(String::new())),
        StepId::RecheckExplodeSkill    => Box::new(StepRecheckExplodeSkill::new()),
        StepId::ResolveBomb            => Box::new(step_resolve_bomb::StepResolveBomb::new()),
        StepId::SpecialEffect          => Box::new(step_special_effect::StepSpecialEffect::new(String::new())),
        // ── Start of game inducements ─────────────────────────────────────────
        StepId::BuyInducements         => Box::new(step_buy_inducements::StepBuyInducements::new()),
        StepId::MasterChef             => Box::new(StepMasterChef::new()),
        StepId::Prayers                => Box::new(step_prayers::StepPrayers::new()),
        StepId::Prayer                 => Box::new(StepPrayer::new(0, "")),
        // ── TTM ──────────────────────────────────────────────────────────────
        StepId::AlwaysHungry           => Box::new(step_always_hungry::StepAlwaysHungry::new(String::new(), String::new())),
        StepId::DispatchScatterPlayer  => Box::new(step_dispatch_scatter_player::StepDispatchScatterPlayer::new()),
        StepId::EndScatterPlayer       => Box::new(step_end_scatter_player::StepEndScatterPlayer::new()),
        StepId::EndThrowTeamMate       => Box::new(step_end_throw_team_mate::StepEndThrowTeamMate::new()),
        StepId::InitScatterPlayer      => Box::new(step_init_scatter_player::StepInitScatterPlayer::new()),
        StepId::InitThrowTeamMate      => Box::new(step_init_throw_team_mate::StepInitThrowTeamMate::new(String::new())),
        StepId::RightStuff             => Box::new(step_right_stuff::StepRightStuff::new(String::new())),
        StepId::Swoop                  => Box::new(step_swoop::StepSwoop::new(String::new())),
        StepId::ThrowTeamMate          => Box::new(step_throw_team_mate::StepThrowTeamMate::new()),
        // ── BB2025 misc ──────────────────────────────────────────────────────
        StepId::AutoGazeZoat           => Box::new(StepAutoGazeZoat::new()),
        StepId::BalefulHex             => Box::new(StepBalefulHex::new()),
        StepId::BlackInk               => Box::new(StepBlackInk::new()),
        StepId::CatchOfTheDay          => Box::new(StepCatchOfTheDay::new()),
        StepId::EndFuriousOutburst     => Box::new(StepEndFuriousOutburst::new()),
        StepId::LookIntoMyEyes         => Box::new(StepLookIntoMyEyes::new()),
        StepId::RaidingParty           => Box::new(StepRaidingParty::new()),
        StepId::SelectBlitzTarget      => Box::new(StepSelectBlitzTarget::new()),
        StepId::ThenIStartedBlastin    => Box::new(StepThenIStartedBlastin::new()),
        StepId::Treacherous            => Box::new(StepTreacherous::new()),
        StepId::WisdomOfTheWhiteDwarf  => Box::new(StepWisdomOfTheWhiteDwarf::new()),
        // ── Mixed start ─────────────────────────────────────────────────────
        StepId::PettyCash              => Box::new(StepPettyCash::new()),
        // ── Mixed root ──────────────────────────────────────────────────────
        StepId::InitLookIntoMyEyes     => Box::new(StepInitLookIntoMyEyes::new()),
        // ── Mixed end ───────────────────────────────────────────────────────
        StepId::DedicatedFans          => Box::new(StepDedicatedFans::new()),
        // ── Mixed foul ──────────────────────────────────────────────────────
        StepId::EjectPlayer            => Box::new(StepEjectPlayer::new()),
        StepId::PileDriver             => Box::new(StepPileDriver::new()),
        // ── Mixed multiblock ────────────────────────────────────────────────
        StepId::DispatchDumpOff        => Box::new(StepDispatchDumpOff::new()),
        StepId::DoubleStrength         => Box::new(StepDoubleStrength::new()),
        // ── Mixed shared ────────────────────────────────────────────────────
        StepId::BothDown               => Box::new(StepBothDown::new()),
        StepId::ConsumeParameter       => Box::new(StepConsumeParameter::new()),
        StepId::SetDefender            => Box::new(StepSetDefender::new()),
        // ── Control / framework ──────────────────────────────────────────────
        StepId::GotoLabel              => Box::new(StepGotoLabel::new()),
        // ── Everything else → NoOp ───────────────────────────────────────────
        other                          => Box::new(NoOpStep(other)),
    }
}

// ── StepEntry ────────────────────────────────────────────────────────────────

/// A stacked step: concrete step + optional goto label.
pub struct DriverStepEntry {
    pub step: Box<dyn Step>,
    pub label: Option<String>,
}

impl DriverStepEntry {
    pub fn new(step: Box<dyn Step>) -> Self { DriverStepEntry { step, label: None } }
    pub fn labelled(step: Box<dyn Step>, label: impl Into<String>) -> Self {
        DriverStepEntry { step, label: Some(label.into()) }
    }
    pub fn id(&self) -> StepId { self.step.id() }
}

fn seq_step_to_driver_entry(s: SequenceStep) -> DriverStepEntry {
    let mut step = make_step(s.step_id);
    for param in &s.params { step.set_parameter(param); }
    DriverStepEntry { step, label: s.label }
}

// ── DriverStepStack ──────────────────────────────────────────────────────────

/// LIFO stack of `DriverStepEntry`. Top = last element.
pub struct DriverStepStack {
    steps: Vec<DriverStepEntry>,
}

impl DriverStepStack {
    pub fn new() -> Self { DriverStepStack { steps: Vec::new() } }
    pub fn push(&mut self, entry: DriverStepEntry) { self.steps.push(entry); }
    pub fn push_sequence(&mut self, seq: Vec<SequenceStep>) {
        for s in seq.into_iter().rev() { self.steps.push(seq_step_to_driver_entry(s)); }
    }
    pub fn pop(&mut self) -> Option<DriverStepEntry> { self.steps.pop() }
    pub fn peek(&self) -> Option<&DriverStepEntry> { self.steps.last() }
    pub fn len(&self) -> usize { self.steps.len() }
    pub fn is_empty(&self) -> bool { self.steps.is_empty() }

    pub fn goto_label(&mut self, label: &str) -> Result<(), String> {
        while let Some(top) = self.steps.last() {
            if top.label.as_deref() == Some(label) { return Ok(()); }
            self.steps.pop();
        }
        Err(format!("goto unknown label '{label}'"))
    }

    pub fn publish(&mut self, param: &StepParameter) {
        for entry in self.steps.iter_mut().rev() {
            if entry.step.set_parameter(param) { return; }
        }
    }
}

impl Default for DriverStepStack {
    fn default() -> Self { Self::new() }
}

// ── DriverGameState ───────────────────────────────────────────────────────────

/// Replacement for `GameState` using `Box<dyn Step>` instead of `StepKind`.
/// Same external API as the engine.rs `GameState` so callers only need to swap the type.
pub struct DriverGameState {
    pub game: Game,
    pub rng: GameRng,
    stack: DriverStepStack,
    current: Option<DriverStepEntry>,
    forwarded: Option<Action>,
    pending_prompt: Option<AgentPrompt>,
    pub events: Vec<GameEvent>,
    initial_hash: String,
}

impl DriverGameState {
    pub fn from_game(game: Game, seed: u64) -> Self {
        DriverGameState {
            game, rng: GameRng::new(seed), stack: DriverStepStack::new(),
            current: None, forwarded: None, pending_prompt: None, events: Vec::new(),
            initial_hash: String::new(),
        }
    }

    pub fn new(home: Team, away: Team, rules: Rules, seed: u64) -> Self {
        use crate::step::sequences::start_game_sequence;
        let game = Game::new(home, away, rules);
        let mut gs = DriverGameState::from_game(game, seed);
        gs.initial_hash = state_hash(&gs.game);
        gs.stack.push_sequence(start_game_sequence());
        gs.run_until_prompt();
        gs
    }

    pub fn initial_state_hash(&self) -> &str { &self.initial_hash }
    pub fn current_prompt(&self) -> Option<&AgentPrompt> { self.pending_prompt.as_ref() }
    pub fn take_events(&mut self) -> Vec<GameEvent> { std::mem::take(&mut self.events) }
    pub fn active_side(&self) -> TeamSide {
        if self.game.home_playing { TeamSide::Home } else { TeamSide::Away }
    }
    pub fn is_finished(&self) -> bool { self.game.is_finished() }
    pub fn rng_call_count(&self) -> u64 { self.rng.call_count }
    pub fn state_hash_str(&self) -> String { state_hash(&self.game) }

    pub fn apply(&mut self, _side: TeamSide, action: Action) -> Result<Vec<GameEvent>, String> {
        self.apply_action(action);
        Ok(self.take_events())
    }

    pub fn push_sequence(&mut self, seq: Vec<SequenceStep>) { self.stack.push_sequence(seq); }

    fn apply_effects(&mut self, outcome: &mut StepOutcome) {
        self.events.append(&mut outcome.events);
        for seq in outcome.pushes.drain(..) { self.stack.push_sequence(seq); }
        for param in outcome.published.drain(..) { self.stack.publish(&param); }
    }

    pub fn apply_action(&mut self, action: Action) {
        let mut entry = self.current.take().expect("apply_action() with no waiting step");
        let mut outcome = entry.step.handle_command(&action, &mut self.game, &mut self.rng);
        self.apply_effects(&mut outcome);
        self.pending_prompt = None;
        self.dispatch(entry, action, outcome);
        self.drive();
    }

    pub fn run_until_prompt(&mut self) { self.drive(); }

    fn drive(&mut self) {
        loop {
            if self.current.is_some() && self.pending_prompt.is_some() { return; }
            if self.current.is_none() {
                match self.stack.pop() {
                    Some(s) => self.current = Some(s),
                    None => { self.pending_prompt = None; return; }
                }
            }
            let mut entry = self.current.take().unwrap();
            let mut outcome = match self.forwarded.take() {
                Some(cmd) => {
                    let o = entry.step.handle_command(&cmd, &mut self.game, &mut self.rng);
                    self.dispatch(entry, cmd, o);
                    if self.pending_prompt.is_some() { return; }
                    continue;
                }
                None => entry.step.start(&mut self.game, &mut self.rng),
            };
            self.apply_effects(&mut outcome);
            self.dispatch_after_start(entry, outcome);
            if self.pending_prompt.is_some() { return; }
        }
    }

    fn dispatch(&mut self, entry: DriverStepEntry, action: Action, outcome: StepOutcome) {
        match outcome.action {
            StepAction::NextStep => {}
            StepAction::Continue | StepAction::Repeat => {
                self.pending_prompt = outcome.prompt;
                self.current = Some(entry);
            }
            StepAction::GotoLabel => {
                let label = outcome.goto_label.as_deref().unwrap_or("");
                let _ = self.stack.goto_label(label);
            }
            StepAction::NextStepAndRepeat => { self.forwarded = Some(action); }
            StepAction::GotoLabelAndRepeat => {
                let label = outcome.goto_label.as_deref().unwrap_or("");
                let _ = self.stack.goto_label(label);
                self.forwarded = Some(action);
            }
        }
    }

    fn dispatch_after_start(&mut self, entry: DriverStepEntry, outcome: StepOutcome) {
        match outcome.action {
            StepAction::NextStep => {}
            StepAction::Continue | StepAction::Repeat => {
                self.pending_prompt = outcome.prompt;
                self.current = Some(entry);
            }
            StepAction::GotoLabel | StepAction::GotoLabelAndRepeat => {
                let label = outcome.goto_label.as_deref().unwrap_or("");
                let _ = self.stack.goto_label(label);
            }
            // NextStepAndRepeat from start() has no command to forward — treat as NextStep.
            StepAction::NextStepAndRepeat => {}
        }
    }
}

/// Compatibility alias — keeps existing `use ffb_engine::step::GameState` imports working.
pub type GameState = DriverGameState;

/// Test helper — creates a fully-started game at the first prompt, used by agent.rs tests.
#[cfg(test)]
pub(crate) fn new_game(seed: u64) -> DriverGameState {
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;
    DriverGameState::new(test_team("home", 5), test_team("away", 7), Rules::Bb2025, seed)
}
