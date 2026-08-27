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

#[cfg(test)]
/// Test-only step that rolls one d6 in `start()` — a detectable side effect used to prove the
/// driver does NOT run leftover stack steps after the game is Finished.
struct RngStep;

#[cfg(test)]
impl Step for RngStep {
    fn id(&self) -> StepId { StepId::NoOp }
    fn start(&mut self, _game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        rng.d6();
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
    use crate::step::mixed::shared::step_end_player_action::StepEndPlayerAction;
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
        // Java resolves teamId from the ACTING_TEAM init parameter (handleActingTeam); the step's
        // own execute maps acting_team → team id on first run, but ONLY while team_id is None —
        // constructing with new(String::new()) pre-set it to Some("") and the resolution never
        // ran, so the retain filter compared "" against real team ids and silently dropped every
        // acting-team injury (dark_elf bb2020 seed 53: the multiple-block attacker's KO was never
        // applied and he stood up next turn where Java had him in the KO box).
        StepId::ApothecaryMultiple     => Box::new(step_apothecary_multiple::StepApothecaryMultiple::default()),
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
        StepId::CloudBurster           => { use crate::step::bb2020::pass::step_cloud_burster::StepCloudBurster; Box::new(StepCloudBurster::new(String::new())) }
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
        StepId::EndPlayerAction        => Box::new(StepEndPlayerAction::new()),
        StepId::SetDefender            => Box::new(StepSetDefender::new()),
        // ── Control / framework ──────────────────────────────────────────────
        StepId::GotoLabel              => Box::new(StepGotoLabel::new()),
        StepId::NextStep               => { use crate::step::step_next_step::StepNextStep; Box::new(StepNextStep::new()) }
        StepId::NextStepAndRepeat      => { use crate::step::step_next_step_and_repeat::StepNextStepAndRepeat; Box::new(StepNextStepAndRepeat::new()) }
        StepId::NoOp                   => Box::new(NoOpStep(StepId::NoOp)),
        StepId::ResetToMove            => { use crate::step::step_reset_to_move::StepResetToMove; Box::new(StepResetToMove::new()) }
        // ── Block mechanics ──────────────────────────────────────────────────
        StepId::BlockBallAndChain      => { use crate::step::mixed::block::step_block_ball_and_chain::StepBlockBallAndChain; Box::new(StepBlockBallAndChain::new()) }
        StepId::BlockDodge             => { use crate::step::mixed::step_block_dodge::StepBlockDodge; Box::new(StepBlockDodge::new()) }
        StepId::BlockStatistics        => { use crate::step::action::block::step_block_statistics::StepBlockStatistics; Box::new(StepBlockStatistics::new()) }
        StepId::Horns                  => { use crate::step::action::block::step_horns::StepHorns; Box::new(StepHorns::new()) }
        StepId::ProjectileVomit        => { use crate::step::mixed::block::step_projectile_vomit::StepProjectileVomit; Box::new(StepProjectileVomit::new()) }
        // ── Multi-block skills ───────────────────────────────────────────────
        StepId::DauntlessMultiple      => { use crate::step::mixed::multiblock::step_dauntless_multiple::StepDauntlessMultiple; Box::new(StepDauntlessMultiple::new()) }
        StepId::FoulAppearanceMultiple => { use crate::step::mixed::multiblock::step_foul_appearance_multiple::StepFoulAppearanceMultiple; Box::new(StepFoulAppearanceMultiple::new(String::new())) }
        StepId::ReportStabInjury       => { use crate::step::bb2020::multiblock::step_report_stab_injury::StepReportStabInjury; Box::new(StepReportStabInjury::new()) }
        StepId::StateMultipleRolls     => { use crate::step::bb2020::step_state_multiple_rolls::StepStateMultipleRolls; Box::new(StepStateMultipleRolls::new()) }
        // ── Foul mechanics ───────────────────────────────────────────────────
        StepId::Foul                   => { use crate::step::mixed::foul::step_foul::StepFoul; Box::new(StepFoul::new()) }
        StepId::FoulChainsaw           => { use crate::step::mixed::foul::step_foul_chainsaw::StepFoulChainsaw; Box::new(StepFoulChainsaw::new(String::new())) }
        StepId::Referee                => { use crate::step::action::foul::step_referee::StepReferee; Box::new(StepReferee::new()) }
        // ── Move mechanics ───────────────────────────────────────────────────
        StepId::DropActingPlayer       => { use crate::step::mixed::step_drop_acting_player::StepDropActingPlayer; Box::new(StepDropActingPlayer::new()) }
        StepId::DropDivingTackler      => { use crate::step::mixed::move_::step_drop_diving_tackler::StepDropDivingTackler; Box::new(StepDropDivingTackler::new()) }
        StepId::MoveBallAndChain       => { use crate::step::mixed::move_::step_move_ball_and_chain::StepMoveBallAndChain; Box::new(StepMoveBallAndChain::new()) }
        StepId::ResetFumblerooskie     => { use crate::step::mixed::move_::step_reset_fumblerooskie::StepResetFumblerooskie; Box::new(StepResetFumblerooskie::new()) }
        StepId::TrapDoor               => { use crate::step::mixed::move_::step_trap_door::StepTrapDoor; Box::new(StepTrapDoor::new()) }
        // ── Pass mechanics ───────────────────────────────────────────────────
        StepId::AllYouCanEat           => { use crate::step::mixed::pass::step_all_you_can_eat::StepAllYouCanEat; Box::new(StepAllYouCanEat::new()) }
        StepId::DispatchPassing        => { use crate::step::action::pass::step_dispatch_passing::StepDispatchPassing; Box::new(StepDispatchPassing::new(String::new(), String::new(), String::new())) }
        StepId::InitPassing            => { use crate::step::mixed::pass::step_init_passing::StepInitPassing; Box::new(StepInitPassing::new()) }
        // ── KickTeamMate ─────────────────────────────────────────────────────
        StepId::InitKickTeamMate       => { use crate::step::action::ktm::step_init_kick_team_mate::StepInitKickTeamMate; Box::new(StepInitKickTeamMate::new(String::new())) }
        StepId::KickTeamMate           => { use crate::step::action::ktm::step_kick_team_mate::StepKickTeamMate; Box::new(StepKickTeamMate::new(String::new())) }
        StepId::KickTeamMateDoubleRolled => { use crate::step::action::ktm::step_kick_team_mate_double_rolled::StepKickTeamMateDoubleRolled; Box::new(StepKickTeamMateDoubleRolled::new()) }
        StepId::EndKickTeamMate        => { use crate::step::action::ktm::step_end_kick_team_mate::StepEndKickTeamMate; Box::new(StepEndKickTeamMate::new()) }
        // ── TTM ──────────────────────────────────────────────────────────────
        StepId::EatTeamMate            => { use crate::step::action::ttm::step_eat_team_mate::StepEatTeamMate; Box::new(StepEatTeamMate::new()) }
        StepId::FumbleTtmPass          => { use crate::step::bb2016::ttm::step_fumble_ttm_pass::StepFumbleTtmPass; Box::new(StepFumbleTtmPass::new()) }
        // ── Kickoff events ───────────────────────────────────────────────────
        StepId::KickoffAnimation       => { use crate::step::phase::kickoff::step_kickoff_animation::StepKickoffAnimation; Box::new(StepKickoffAnimation::new()) }
        StepId::KickoffReturn          => { use crate::step::phase::kickoff::step_kickoff_return::StepKickoffReturn; Box::new(StepKickoffReturn::new()) }
        StepId::RiotousRookies         => { use crate::step::phase::inducement::step_riotous_rookies::StepRiotousRookies; Box::new(StepRiotousRookies::new()) }
        // ── Mixed special ────────────────────────────────────────────────────
        StepId::EndBomb                => { use crate::step::mixed::special::step_end_bomb::StepEndBomb; Box::new(StepEndBomb::new()) }
        StepId::EndThenIStartedBlastin => { use crate::step::mixed::step_end_then_i_started_blastin::StepEndThenIStartedBlastin; Box::new(StepEndThenIStartedBlastin::new()) }
        StepId::EndThrowKeg            => { use crate::step::mixed::step_end_throw_keg::StepEndThrowKeg; Box::new(StepEndThrowKeg::new()) }
        StepId::FirstMoveFuriousOutburst => { use crate::step::mixed::step_first_move_furious_outburst::StepFirstMoveFuriousOutburst; Box::new(StepFirstMoveFuriousOutburst::new(String::new())) }
        StepId::InitFuriousOutburst    => { use crate::step::mixed::step_init_furious_outburst::StepInitFuriousOutburst; Box::new(StepInitFuriousOutburst::new(String::new())) }
        StepId::SecondMoveFuriousOutburst => { use crate::step::mixed::step_second_move_furious_outburst::StepSecondMoveFuriousOutburst; Box::new(StepSecondMoveFuriousOutburst::new(String::new())) }
        StepId::ThrowKeg               => { use crate::step::mixed::step_throw_keg::StepThrowKeg; Box::new(StepThrowKeg::new()) }
        StepId::Wizard                 => { use crate::step::mixed::step_wizard::StepWizard; Box::new(StepWizard::new()) }
        // ── Mixed blitz ──────────────────────────────────────────────────────
        StepId::RemoveTargetSelectionState => { use crate::step::mixed::blitz::step_remove_target_selection_state::StepRemoveTargetSelectionState; Box::new(StepRemoveTargetSelectionState::new()) }
        StepId::SelectBlitzTargetEnd   => { use crate::step::mixed::blitz::step_select_blitz_target_end::StepSelectBlitzTargetEnd; Box::new(StepSelectBlitzTargetEnd::new()) }
        // ── Mixed shared/end ─────────────────────────────────────────────────
        StepId::PickMeUp               => { use crate::step::mixed::shared::step_pick_me_up::StepPickMeUp; Box::new(StepPickMeUp::new()) }
        StepId::PenaltyShootout        => { use crate::step::mixed::end::step_penalty_shootout::StepPenaltyShootout; Box::new(StepPenaltyShootout::new()) }
        // ── Mixed inducements ────────────────────────────────────────────────
        StepId::PlayCard               => { use crate::step::mixed::inducements::step_play_card::StepPlayCard; Box::new(StepPlayCard::new()) }
        // ── Skills (mixed) ───────────────────────────────────────────────────
        StepId::Pro                    => { use crate::step::mixed::step_pro::StepPro; Box::new(StepPro::new()) }
        StepId::QuickBite              => { use crate::step::mixed::step_quick_bite::StepQuickBite; Box::new(StepQuickBite::new()) }
        // ── End of game ──────────────────────────────────────────────────────
        StepId::EndGame                => { use crate::step::game::end::step_end_game::StepEndGame; Box::new(StepEndGame::new()) }
        // ── BB2020-specific ──────────────────────────────────────────────────
        StepId::AssignTouchdowns       => { use crate::step::bb2020::end::step_assign_touchdowns::StepAssignTouchdowns; Box::new(StepAssignTouchdowns::new()) }
        StepId::BuyCardsAndInducements => { use crate::step::bb2020::start::step_buy_cards_and_inducements::StepBuyCardsAndInducements; Box::new(StepBuyCardsAndInducements::new()) }
        StepId::CheckStalling          => { use crate::step::bb2020::shared::step_check_stalling::StepCheckStalling; Box::new(StepCheckStalling::new()) }
        StepId::SelectGazeTarget       => { use crate::step::bb2020::gaze::step_select_gaze_target::StepSelectGazeTarget; Box::new(StepSelectGazeTarget::new()) }
        StepId::SelectGazeTargetEnd    => { use crate::step::bb2020::gaze::step_select_gaze_target_end::StepSelectGazeTargetEnd; Box::new(StepSelectGazeTargetEnd::new()) }
        StepId::SetActingPlayerAndTeam => { use crate::step::bb2020::step_set_acting_player_and_team::StepSetActingPlayerAndTeam; Box::new(StepSetActingPlayerAndTeam::new()) }
        StepId::SetActingTeam          => { use crate::step::bb2020::step_set_acting_team::StepSetActingTeam; Box::new(StepSetActingTeam::new()) }
        // ── BB2016-specific ──────────────────────────────────────────────────
        StepId::BuyCards               => { use crate::step::bb2016::start::step_buy_cards::StepBuyCards; Box::new(StepBuyCards::new()) }
        StepId::FanFactor              => { use crate::step::bb2016::end::step_fan_factor::StepFanFactor; Box::new(StepFanFactor::new()) }
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

fn seq_step_to_driver_entry(s: SequenceStep, rules: Rules) -> DriverStepEntry {
    let mut step = make_step_for(s.step_id, rules);
    for param in &s.params { step.set_parameter(param); }
    DriverStepEntry { step, label: s.label }
}

/// Edition-aware `make_step`. BB2016 has its own step classes for a handful of steps whose
/// dice/logic differ from the shared BB2020+ (`mixed`) implementations (Java routes these via the
/// per-ruleset SequenceGenerator factory). Route those to the BB2016 impls when the game is BB2016;
/// every other step — and every non-BB2016 edition — falls through to the shared `make_step`, so
/// BB2020/BB2025 behaviour is byte-for-byte unchanged.
pub fn make_step_for(id: StepId, rules: Rules) -> Box<dyn Step> {
    if rules == Rules::Bb2020 {
        match id {
            // BB2020 and BB2025 have genuinely DIFFERENT kickoff event tables, and the shared
            // (bb2025) step implements the BB2025 rules: Cheering Fans grants BB2020 a PRAYER TO
            // NUFFLE (`handleCheeringFans` pushes `StepId.PRAYER`) but grants BB2025 extra
            // offensive block assists; the tables differ in membership too (BB2020 Officious Ref
            // vs BB2025 Charge / Dodgy Snack). `make_step_for` had NO `Rules::Bb2020` arm at all,
            // so every bb2020 step fell through to the bb2025 default and bb2020 games ran BB2025
            // kickoff rules.
            // NOT ApplyKickoffResult: routing the whole step to the bb2020 file regressed human to
            // 12/100 even with the prayer chain correct, because that file is STALER than the
            // shared one for the events they have in common (its own header flags QuickSnap /
            // SolidDefence / HighKick as TODO). Per the bb2016 campaign's lesson, edition-gate the
            // ONE differing event inside the shared step instead — see `handle_cheering_fans`.
            StepId::Prayer =>
                return Box::new(crate::step::bb2020::StepPrayer::default()),
            // BB2020 Throw-Team-Mate step-set. These twins were translated with the rest of the
            // port and then sat dead: the driver had no BB2020 TTM arm, so a BB2020 throw ran the
            // BB2025 chain. Nothing noticed for as long as it didn't because BB2020 never reached a
            // throw at all — both harnesses filtered throwable team-mates on the raw `canBeThrown`
            // property, which bb2020's Right Stuff does not register (it registers
            // `canBeThrownIfStrengthIs3orLess`), so the candidate list was always empty.
            //
            // The two chains genuinely differ: BB2020 scatters the thrown player 3x d8 and carries
            // PASS_DEVIATES / WILDLY_INACCURATE / crash-landing; BB2025 replaces those with
            // bullseye and swoop. Running BB2025's chain under BB2020 landed an accurate throw
            // on-target and standing where Java scattered it and then failed the landing roll
            // (ogre bb2020 seed 6: java h09 Prone at 7,6 with pass_used set, rust h09 Standing).
            // Routing the WHOLE set was measured and is worse: the outer twins
            // (ThrowTeamMate / InitThrowTeamMate / EndThrowTeamMate / RightStuff / AlwaysHungry) are
            // staler than the shared chain and no longer match the generator that pushes them — the
            // throw never rolled at all, two players took armour rolls and the turn ended on a
            // turnover (ogre bb2020 seed 6). Same lesson as ApplyKickoffResult above: route only the
            // steps whose BB2020 behaviour genuinely differs, and leave the rest shared.
            // Routing DispatchScatterPlayer / InitScatterPlayer alone was measured too, and is also
            // wrong: the SHARED StepThrowTeamMate hands the scatter over by PUBLISHING
            // PASS_RESULT + USING_BULLSEYE, and the bb2020 twin reads a different parameter set, so
            // the hand-over silently drops. Whatever BB2020 needs has to be edition-gated INSIDE the
            // shared chain, the way handle_cheering_fans does above.
            _ => {}
        }
    }
    if rules == Rules::Bb2016 {
        match id {
            // BB2016 StepSpectators rolls 2D6 per team (spectators + fame via ReportSpectators),
            // vs the mixed BB2020+ single-D3 fan-factor (bb2016 amazon seed 1 pregame divergence).
            StepId::Spectators => return Box::new(crate::step::bb2016::start::StepSpectators::new()),
            // BB2016 has its own kickoff path: the "Changing Weather → Nice" gust scatters the ball
            // exactly ONE square (bb2020+/bb2025 scatter up to three, with a different bounds check),
            // and the kickoff bounds + catch/scatter/throw-in flow differ. Route the whole bb2016
            // kickoff chain so it matches stock Java (bb2016 amazon seed 1 pos15: Java 1-square gust
            // then scatterBall d8, vs the bb2025 step's 3-square gust that spuriously touchbacked).
            StepId::KickoffScatterRoll =>
                return Box::new(crate::step::bb2016::StepKickoffScatterRoll::new()),
            StepId::KickoffResultRoll =>
                return Box::new(crate::step::bb2016::StepKickoffResultRoll::new()),
            StepId::ApplyKickoffResult =>
                return Box::new(crate::step::bb2016::StepApplyKickoffResult::new(String::new(), String::new())),
            StepId::CatchScatterThrowIn =>
                return Box::new(crate::step::bb2016::StepCatchScatterThrowIn::new()),
            // BB2016 missed pass: 3× single-square scatter (d8 each) from the pass target, then
            // publish CATCH_SCATTER_THROW_IN_MODE=CATCH_MISSED_PASS so the (bb2016) CatchScatterThrowIn
            // resolves the catch/bounce. The shared bb2025 StepMissedPass publishes NOTHING when the
            // ball lands in-bounds (bb2025 resolves the landing via a different downstream path), so
            // an uncaught missed pass never bounced (amazon seed1 i=201: Java bounces the ball off the
            // empty landing square, Rust left it put → 1 fewer d8 + wrong ball square).
            StepId::MissedPass =>
                return Box::new(crate::step::bb2016::pass::step_missed_pass::StepMissedPass::new()),
            // ── BB2016 pass step-set ─────────────────────────────────────────────────────────────
            // Java's per-ruleset SequenceGenerator factory builds the BB2016 Pass sequence out of
            // `server.step.bb2016.pass.*`; `bb2016::move_::step_end_selecting` already pushes the
            // bb2016 Pass SEQUENCE, but the driver was still instantiating the bb2025 step classes
            // for every StepId in it. The editions differ materially: bb2016 uses the BB2016
            // `PassMechanic` throwing-range table (via `find_passing_distance`) where the bb2025
            // steps use the shared bb2020+ table, so an out-of-range bb2016 throw was executed
            // instead of refused (underworld seed 72 i=74: Java's InitPassing refuses the
            // 13-square throw and the turn ends with zero dice; Rust rolled the accuracy d6 and
            // offered an interception, desyncing the shared stream).
            StepId::InitPassing =>
                return Box::new(crate::step::bb2016::pass::step_init_passing::StepInitPassing::new()),
            // ── BB2016 activation / move / block / foul step-set (approach A: 1:1 bb2016 steps,
            // completed with the AgentPrompt bridge) ─────────────────────────────────────────────
            StepId::InitSelecting =>
                return Box::new(crate::step::bb2016::move_::step_init_selecting::StepInitSelecting::new(String::new())),
            StepId::InitMoving =>
                return Box::new(crate::step::bb2016::move_::step_init_moving::StepInitMoving::new(String::new())),
            StepId::Move =>
                return Box::new(crate::step::bb2016::move_::step_move::StepMove::new()),
            StepId::MoveDodge =>
                return Box::new(crate::step::bb2016::move_::step_move_dodge::StepMoveDodge::new(String::new())),
            StepId::GoForIt =>
                return Box::new(crate::step::bb2016::move_::step_go_for_it::StepGoForIt::new(String::new())),
            StepId::Jump =>
                return Box::new(crate::step::bb2016::move_::step_jump::StepJump::new(String::new())),
            StepId::HypnoticGaze =>
                return Box::new(crate::step::bb2016::move_::step_hypnotic_gaze::StepHypnoticGaze::new(String::new())),
            StepId::EndMoving =>
                return Box::new(crate::step::bb2016::move_::step_end_moving::StepEndMoving::new()),
            StepId::EndSelecting =>
                return Box::new(crate::step::bb2016::move_::step_end_selecting::StepEndSelecting::new()),
            StepId::InitBlocking =>
                return Box::new(crate::step::bb2016::StepInitBlocking::new()),
            StepId::BlockRoll =>
                return Box::new(crate::step::bb2016::block::step_block_roll::StepBlockRoll::new()),
            StepId::BlockChoice =>
                return Box::new(crate::step::bb2016::block::step_block_choice::StepBlockChoice::new()),
            StepId::BlockDodge =>
                return Box::new(crate::step::bb2016::block::step_block_dodge::StepBlockDodge::new()),
            StepId::BothDown =>
                return Box::new(crate::step::bb2016::block::step_both_down::StepBothDown::new()),
            StepId::Followup =>
                return Box::new(crate::step::bb2016::block::step_followup::StepFollowup::new()),
            StepId::EndBlocking =>
                return Box::new(crate::step::bb2016::block::step_end_blocking::StepEndBlocking::new()),
            StepId::Pushback =>
                return Box::new(crate::step::bb2016::StepPushback::new()),
            StepId::Foul =>
                return Box::new(crate::step::bb2016::foul::step_foul::StepFoul::new()),
            StepId::InitFouling =>
                return Box::new(crate::step::bb2016::foul::step_init_fouling::StepInitFouling::new()),
            StepId::EndFouling =>
                return Box::new(crate::step::bb2016::foul::step_end_fouling::StepEndFouling::new()),
            StepId::EjectPlayer =>
                return Box::new(crate::step::bb2016::foul::step_eject_player::StepEjectPlayer::new()),
            StepId::FallDown =>
                return Box::new(crate::step::bb2016::StepFallDown::new()),
            StepId::PickUp =>
                return Box::new(crate::step::bb2016::step_pick_up::StepPickUp::new(String::new())),
            // BB2016 Throw-Team-Mate landing: the bb2016 ThrowTeamMate generator adds RIGHT_STUFF with
            // NO params (matching Java bb2016 ThrowTeamMate.java), and the bb2016 StepRightStuff
            // NEXT_STEPs to the generator's jump→APOTHECARY_THROWN_PLAYER. The shared bb2025
            // StepRightStuff instead GOTOs its (unset) goto-label param → 'goto unknown label ""' →
            // stack drained → game ends abnormally (orc bb2016 seed35 i=249: the Troll throws a Goblin
            // → Rust bailed on the landing, Java continued). Route bb2016 to its own StepRightStuff.
            StepId::RightStuff =>
                return Box::new(crate::step::bb2016::ttm::step_right_stuff::StepRightStuff::new()),
            // BB2016 Throw-Team-Mate step-set. The bb2016 TTM generator pushes these StepIds; without
            // routing they fall through to the bb2025 TTM impls, whose scatter differs (bb2016 scatters
            // the thrown player 3× d8 via StepInitScatterPlayer/UtilThrowTeamMateSequence; bb2025 uses
            // DispatchScatterPlayer). Route the whole set to bb2016 (edition-gated; bb2025 untouched).
            // Gates renegades/ogre/goblin/underworld bb2016 (all Troll-TTM rosters).
            StepId::ThrowTeamMate =>
                return Box::new(crate::step::bb2016::ttm::step_throw_team_mate::StepThrowTeamMate::new()),
            StepId::InitThrowTeamMate =>
                return Box::new(crate::step::bb2016::ttm::step_init_throw_team_mate::StepInitThrowTeamMate::new()),
            StepId::InitScatterPlayer =>
                return Box::new(crate::step::bb2016::ttm::step_init_scatter_player::StepInitScatterPlayer::new()),
            StepId::EndScatterPlayer =>
                return Box::new(crate::step::bb2016::ttm::step_end_scatter_player::StepEndScatterPlayer::new()),
            StepId::AlwaysHungry =>
                return Box::new(crate::step::bb2016::ttm::step_always_hungry::StepAlwaysHungry::new()),
            StepId::EndThrowTeamMate =>
                return Box::new(crate::step::bb2016::ttm::step_end_throw_team_mate::StepEndThrowTeamMate::new()),
            _ => {}
        }
    }
    make_step(id)
}

// ── DriverStepStack ──────────────────────────────────────────────────────────

/// LIFO stack of `DriverStepEntry`. Top = last element.
pub struct DriverStepStack {
    steps: Vec<DriverStepEntry>,
    /// Edition of the game owning this stack — threaded into `make_step_for` so sequences pushed
    /// at runtime (pregame, kickoff, block, …) build the correct per-ruleset step impls. Defaults
    /// to BB2025; set from `game.rules` at construction.
    rules: Rules,
}

impl DriverStepStack {
    pub fn new() -> Self { DriverStepStack { steps: Vec::new(), rules: Rules::Bb2025 } }
    pub fn new_with_rules(rules: Rules) -> Self { DriverStepStack { steps: Vec::new(), rules } }
    pub fn push(&mut self, entry: DriverStepEntry) { self.steps.push(entry); }
    pub fn push_sequence(&mut self, seq: Vec<SequenceStep>) {
        for s in seq.into_iter().rev() { self.steps.push(seq_step_to_driver_entry(s, self.rules)); }
    }
    pub fn pop(&mut self) -> Option<DriverStepEntry> { self.steps.pop() }
    /// Insert an entry at an absolute stack position (0 = bottom). Used by the driver's
    /// `push_self` handling to slot the current instance BELOW freshly pushed sequences.
    pub fn insert(&mut self, index: usize, entry: DriverStepEntry) { self.steps.insert(index, entry); }
    pub fn peek(&self) -> Option<&DriverStepEntry> { self.steps.last() }
    pub fn len(&self) -> usize { self.steps.len() }
    pub fn is_empty(&self) -> bool { self.steps.is_empty() }

    /// Java: `StepStack.clear()`.
    pub fn clear(&mut self) { self.steps.clear(); }

    pub fn goto_label(&mut self, label: &str) -> Result<(), String> {
        while let Some(top) = self.steps.last() {
            if top.label.as_deref() == Some(label) { return Ok(()); }
            self.steps.pop();
        }
        // Java throws a StepException here. A missing label drains the entire stack, which
        // downstream looks like a silent, premature game end — always say so on stderr.
        eprintln!("FFB DRIVER ERROR: goto unknown label '{label}' — step stack drained, game will end");
        Err(format!("goto unknown label '{label}'"))
    }

    /// Java `StepStack.publishStepParameter`: deliver top-of-stack downward; a step whose
    /// setParameter *consumes* the key (see `Step::consumes_parameter`) stops the delivery —
    /// but only AFTER receiving it. Everything below stays untouched. Note both prior Rust
    /// semantics were wrong: first-accepting-wins starved multi-consumer params like
    /// COORDINATE_TO (MoveBallAndChain AND Move AND GoForIt all read it — StepMove never got
    /// it, so players never moved), while deliver-to-all let one inducement window's
    /// END_INDUCEMENT_PHASE/HOME_TEAM/INDUCEMENT_PHASE publishes clobber the other pending
    /// window's steps (Java's StepEndInducement consumes those keys precisely to prevent that).
    /// `already_consumed` mirrors self-delivery having consumed the parameter (Java delivers
    /// to one more stack step before its post-delivery isConsumed() check breaks the loop).
    pub fn publish(&mut self, param: &StepParameter, already_consumed: bool) {
        let mut consumed = already_consumed;
        for entry in self.steps.iter_mut().rev() {
            entry.step.set_parameter(param);
            consumed = consumed || entry.step.consumes_parameter(param);
            if consumed { return; }
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
    pub(crate) pending_prompt: Option<AgentPrompt>,
    rng_step_seq: u32,
    /// True exactly when the most recently dispatched outcome was
    /// `StepAction::Continue` — i.e. the step is waiting for an external
    /// command (whether or not that wait is surfaced as an `AgentPrompt`).
    /// `StepAction::Repeat` intentionally leaves this false: unlike
    /// `Continue`, `Repeat` means "call `start()` again immediately" (see
    /// CLAUDE.md's "Loop pattern"). Without this flag, a step that legitimately
    /// returns `Continue` with no prompt (e.g. `StepInitStartGame` waiting on
    /// two separate `CLIENT_START_GAME` network commands, which has no
    /// client-side dialog) would be busy-looped on `start()` forever by
    /// `drive()`, since the loop used to treat "no prompt" as "not actually
    /// waiting".
    waiting_for_command: bool,
    pub events: Vec<GameEvent>,
    initial_hash: String,
}

impl DriverGameState {
    pub fn from_game(game: Game, seed: u64) -> Self {
        let rules = game.rules;
        let mut game = game;
        // Mirror `ParityRunner.seedCollectionsShuffleRng`: Java's one-arg
        // `Collections.shuffle(list)` draws from a shared `Random` inside `java.util.Collections`,
        // a SECOND stream alongside the DiceRoller. The harness seeds Java's field per game with
        // this exact expression, so seeding ours identically makes both engines draw the same
        // permutations (prayer selection, per-player picks). Keep the constant in sync with
        // ParityRunner.
        game.collections_rng = ffb_model::model::game::CollectionsRng::new(
            ffb_model::util::java_random::JavaRandom::new((seed as i64) ^ 0x5EED_C011_3C71_04));
        DriverGameState {
            game, rng: GameRng::new(seed), stack: DriverStepStack::new_with_rules(rules),
            current: None, forwarded: None, pending_prompt: None, rng_step_seq: 0,
            waiting_for_command: false, events: Vec::new(),
            initial_hash: String::new(),
        }
    }

    pub fn new(home: Team, away: Team, rules: Rules, seed: u64) -> Self {
        Self::new_with_options(home, away, rules, seed, &[])
    }

    /// As `new_with_options`, but pushes the **edition-aware** generator start-game
    /// sequence (`generator::{bb2016,bb2020,bb2025}::start_game::StartGame::build_sequence()`
    /// — `InitStartGame` → `Spectators`/`Weather` → `PettyCash` → `BuyInducements`/
    /// `BuyCards`+`BuyInducements`/`BuyCardsAndInducements`, which then dynamically push
    /// their own Kickoff/CoinChoice sequence) instead of `sequences::start_game_sequence()`'s
    /// flattened, PettyCash/BuyInducements-skipping pregame (`InitStartGame` → `Spectators`
    /// → `Weather` → `CoinChoice` → `ReceiveChoice` → `InitKickoff` → ... directly).
    ///
    /// `new`/`new_with_options` deliberately keep the flattened sequence — it's what the
    /// Java-parity RNG contract (`AGENT_CONTRACT.md`) and `RandomAgent` were built and
    /// verified against, and changing dice/prompt ordering there would desync byte-for-byte
    /// parity tests. This constructor exists for callers that need the *real* pregame flow
    /// (petty cash + inducement purchasing actually reachable) and don't need Java-parity
    /// RNG-stream sync — e.g. `UniformAgent`-driven mechanic-coverage runs.
    pub fn new_full_pregame(home: Team, away: Team, rules: Rules, seed: u64, options: &[(&str, &str)]) -> Self {
        let mut game = Game::new(home, away, rules);
        for (key, value) in options { game.options.set(*key, *value); }
        let mut gs = DriverGameState::from_game(game, seed);
        gs.initial_hash = state_hash(&gs.game);
        let seq = match rules {
            Rules::Bb2016 => crate::step::generator::bb2016::start_game::StartGame::build_sequence(),
            Rules::Bb2020 => crate::step::generator::bb2020::start_game::StartGame::build_sequence(),
            Rules::Bb2025 | Rules::Common => crate::step::generator::bb2025::start_game::StartGame::build_sequence(),
        };
        gs.stack.push_sequence(seq);
        gs.run_until_prompt();
        // Same rationale as `new_with_options`: synthesize both CLIENT_START_GAME sends so
        // construction doesn't stall on the InitStartGame handshake.
        gs.apply_action(Action::StartGame { home: true });
        gs.apply_action(Action::StartGame { home: false });
        gs
    }

    /// As `new`, but applies `options` (game-option id → value string, e.g.
    /// `[("inducements", "true")]`) to the freshly-built `Game` before driving the
    /// pregame — `Game::new` starts every option unset/disabled (mirrors Java's
    /// per-ruleset option config, which this synchronous single-process constructor
    /// has no ruleset-loader hook to apply), so callers that need a specific option
    /// enabled from the very first step (e.g. `INDUCEMENTS`, so `StepBuyInducements`
    /// actually fires) must set it before the pregame sequence runs, not after.
    pub fn new_with_options(home: Team, away: Team, rules: Rules, seed: u64, options: &[(&str, &str)]) -> Self {
        use crate::step::sequences::start_game_sequence_for;
        let mut game = Game::new(home, away, rules);
        for (key, value) in options { game.options.set(*key, *value); }
        let mut gs = DriverGameState::from_game(game, seed);
        gs.initial_hash = state_hash(&gs.game);
        // Java StepBuyInducements.leaveStep grants the special-rule / skill inducements
        // (Bribery and Corruption → REROLL_ARGUE, Bugman's XXXXXX → REROLL_ONES_ON_KOS) as a
        // dice-free consequence of the drafted rosters. The parity `start_game_sequence()`
        // omits the petty-cash/shopping steps, so apply that grant directly here — matching
        // Java's headless start flow, which DOES run BuyInducements (dwarf Bribery and
        // Corruption argue re-roll). Done before the start sequence runs so the inducement is
        // present for the whole game, exactly as in Java.
        crate::step::bb2025::start::step_buy_inducements::grant_special_rule_inducements(&mut gs.game);
        gs.stack.push_sequence(start_game_sequence_for(rules));
        gs.run_until_prompt();
        // `start_game_sequence()` begins with `StepInitStartGame`, which (matching Java's
        // `StepInitStartGame`) only proceeds once BOTH coaches have sent `CLIENT_START_GAME`
        // (`fStartedHome && fStartedAway`), and waits (`Continue`, no dialog/prompt — this isn't
        // a UI choice, just two independent network commands) until then. This constructor is a
        // synchronous, single-process entry point that is only ever called once both `home` and
        // `away` teams are already fully formed (its own doc: "Initialize the engine once both
        // teams are present" — see `ffb-server::game_state::GameState::start_game`), so both
        // coaches being "ready" is already an established precondition here — there is no
        // earlier async join/team-load handshake modeled at this layer for a real per-client
        // signal to arrive later. Synthesize both `CLIENT_START_GAME` commands immediately so
        // construction doesn't stall waiting for network events this driver never delivers.
        gs.apply_action(Action::StartGame { home: true });
        gs.apply_action(Action::StartGame { home: false });
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

    /// Java: `gameState.getStepStack().clear()`. Also drops the separately-held
    /// running step — Java's `StepStack` has no split "current step" concept, so a
    /// faithful clear must reset both halves of this driver's split representation
    /// or the just-running step would resume after the stack is emptied.
    pub fn clear_step_stack(&mut self) {
        self.stack.clear();
        self.current = None;
        self.pending_prompt = None;
        self.forwarded = None;
    }

    /// Java: `((EndGame) factory.forName("EndGame")).pushSequence(new EndGame.SequenceParams(gameState, adminMode))`.
    /// Reuses the same `end_game_sequence` helper the in-engine `StepEndTurn` variants push.
    pub fn push_end_game_sequence(&mut self, admin_mode: bool) {
        use crate::step::sequences::end_game_sequence;
        self.push_sequence(end_game_sequence(admin_mode));
    }

    fn apply_effects(&mut self, entry: &mut DriverStepEntry, outcome: &mut StepOutcome) {
        self.events.append(&mut outcome.events);
        for seq in outcome.pushes.drain(..) { self.stack.push_sequence(seq); }
        // Java AbstractStep.publishParameter: `setParameter(pParameter)` on the publishing step
        // itself first, then `stepStack.publishStepParameter(pParameter)` to the stack
        // (delivery stops at, but includes, the first consuming step — see stack.publish).
        for param in outcome.published.drain(..) {
            entry.step.set_parameter(&param);
            let self_consumed = entry.step.consumes_parameter(&param);
            self.stack.publish(&param, self_consumed);
        }
    }

    /// Java `pushCurrentStepOnStack()` (StepOutcome::push_self): re-insert the CURRENT step
    /// instance (fields intact) BELOW the sequences the same outcome pushed, so it resumes
    /// after they finish. `apply_effects` has already pushed `outcome.pushes` on top of the
    /// stack, so the instance is inserted underneath those freshly pushed entries.
    fn apply_push_self(&mut self, entry: DriverStepEntry, pushed_len: usize) {
        let insert_at = self.stack.len().saturating_sub(pushed_len);
        self.stack.insert(insert_at, entry);
    }

    pub fn apply_action(&mut self, action: Action) {
        let mut entry = self.current.take().expect("apply_action() with no waiting step");
        let def_before_cmd = self.game.defender_id.clone();
        let pos_before_cmd = self.game.acting_player.player_id.clone()
            .and_then(|p| self.game.field_model.player_coordinate(&p)).map(|c| (c.x, c.y));
        let mut outcome = entry.step.handle_command(&action, &mut self.game, &mut self.rng);
        if std::env::var_os("FFB_POSCHG").is_some() {
            let now = self.game.acting_player.player_id.clone()
                .and_then(|p| self.game.field_model.player_coordinate(&p)).map(|c| (c.x, c.y));
            if now != pos_before_cmd {
                eprintln!("POSCHG(cmd) step={:?} pid={:?} {:?} -> {:?}", entry.step.id(),
                    self.game.acting_player.player_id, pos_before_cmd, now);
            }
        }
        if std::env::var_os("FFB_DEFCHG").is_some() && self.game.defender_id != def_before_cmd {
            eprintln!("DEFCHG(cmd) step={:?} {:?} -> {:?} (acting={:?} pa={:?})",
                entry.step.id(), def_before_cmd, self.game.defender_id,
                self.game.acting_player.player_id, self.game.acting_player.player_action);
        }
        let pushed_len: usize = outcome.pushes.iter().map(|s| s.len()).sum();
        self.apply_effects(&mut entry, &mut outcome);
        self.pending_prompt = None;
        if outcome.push_self {
            self.apply_push_self(entry, pushed_len);
            self.waiting_for_command = false;
        } else {
            self.dispatch(entry, action, outcome);
        }
        self.drive();
    }

    pub fn run_until_prompt(&mut self) { self.drive(); }

    fn drive(&mut self) {
        loop {
            // Java: once StepEndGame flips the game to FINISHED the server tears the match down —
            // no further steps run. Rust pushes end_game_sequence ON TOP of whatever turn/kickoff
            // sequence was already queued for the (never-played) next turn, so without this guard the
            // driver kept popping those leftover steps after the game ended — a phantom EndTurn cycle
            // that re-ran KO recovery / PlayerLoss and corrupted the final state (seed 19: home_04
            // wrongly recovered from KO and the active team flipped to away). Halt and drop the stack.
            if self.game.is_finished() {
                self.stack.clear();
                self.current = None;
                self.pending_prompt = None;
                self.waiting_for_command = false;
                return;
            }
            if self.current.is_some() && (self.pending_prompt.is_some() || self.waiting_for_command) { return; }
            if self.current.is_none() {
                match self.stack.pop() {
                    Some(s) => self.current = Some(s),
                    None => { self.pending_prompt = None; self.waiting_for_command = false; return; }
                }
            }
            let mut entry = self.current.take().unwrap();
            // Step-dispatch trace, enabled via FFB_DRIVE_TRACE (companion to lib.rs's FFB_TRACE
            // dice/agent trace): one line per step the driver runs. This is the primary tool for
            // diagnosing silent stalls and premature game-ends in headless runs.
            if std::env::var_os("FFB_DRIVE_TRACE").is_some() {
                eprintln!("DRIVE step={:?} stack_len={} forwarded={} rng={}", entry.step.id(), self.stack.len(), self.forwarded.is_some(), self.rng.call_count);
            }
            // FFB_RNG_STEPS: a GLOBAL, ordered list of every step that actually consumed engine
            // dice, printed as `RNGSTEP <n> step=<id> <from>-><to>`. Unlike a drive-trace window
            // this needs no alignment: two builds can be diffed line for line and the first
            // differing entry IS the divergence. Windows bounded by RUST_STEP have produced three
            // wrong root causes in the §12 work (docs/BACKLOG.md).
            let rng_before_step = self.rng.call_count;
            // FFB_RR_STEPS: which step changed a team's re-roll bank. Same shape as
            // FFB_RNG_STEPS, and it exists for the same reason: the `r` field is in the state hash,
            // so an accounting error is a divergence, but nothing else says WHO changed it. Only
            // visible once an agent actually accepts re-rolls -- under the random contract the
            // counters never move.
            let rr_before_step = (self.game.turn_data_home.rerolls, self.game.turn_data_away.rerolls);
            let def_before_step = self.game.defender_id.clone();
            let pos_before_step = self.game.acting_player.player_id.clone()
                .and_then(|p| self.game.field_model.player_coordinate(&p)).map(|c| (c.x, c.y));
            // FFB_BALLCHG: every step that changes the ball's position or its loose/in-play
            // flags, printed as `BALLCHG step=<id> <before> -> <after>`. The dice-based traces
            // answer "which step rolled"; this one answers "what put the ball there", which is
            // the question when a scatter fires with no pick-up roll in front of it.
            let ball_before_step = (
                self.game.field_model.ball_coordinate.map(|c| (c.x, c.y)),
                self.game.field_model.ball_moving,
                self.game.field_model.ball_in_play,
            );
            let mut outcome = match self.forwarded.take() {
                Some(cmd) => {
                    let mut o = entry.step.handle_command(&cmd, &mut self.game, &mut self.rng);
                    if std::env::var_os("FFB_POSCHG").is_some() {
                let now = self.game.acting_player.player_id.clone()
                    .and_then(|p| self.game.field_model.player_coordinate(&p)).map(|c| (c.x, c.y));
                if now != pos_before_step {
                    eprintln!("POSCHG step={:?} pid={:?} {:?} -> {:?}", entry.step.id(),
                        self.game.acting_player.player_id, pos_before_step, now);
                }
            }
            if std::env::var_os("FFB_DEFCHG").is_some() && self.game.defender_id != def_before_step {
                        eprintln!("DEFCHG(fwd) step={:?} {:?} -> {:?} (acting={:?} pa={:?})",
                            entry.step.id(), def_before_step, self.game.defender_id,
                            self.game.acting_player.player_id, self.game.acting_player.player_action);
                    }
                    // Same as apply_action: a forwarded command's outcome carries events, pushed
                    // sequences, and published parameters too — dropping them silently diverged
                    // from every other dispatch path.
                    let pushed_len: usize = o.pushes.iter().map(|s| s.len()).sum();
                    self.apply_effects(&mut entry, &mut o);
                    if o.push_self {
                        self.apply_push_self(entry, pushed_len);
                        self.waiting_for_command = false;
                    } else {
                        self.dispatch(entry, cmd, o);
                    }
                    if self.pending_prompt.is_some() || self.waiting_for_command { return; }
                    continue;
                }
                None => entry.step.start(&mut self.game, &mut self.rng),
            };
            if std::env::var_os("FFB_DEFCHG").is_some() && self.game.defender_id != def_before_step {
                eprintln!("DEFCHG step={:?} {:?} -> {:?} (acting={:?} pa={:?})",
                    entry.step.id(), def_before_step, self.game.defender_id,
                    self.game.acting_player.player_id, self.game.acting_player.player_action);
            }
            if std::env::var_os("FFB_BALLCHG").is_some() {
                let ball_now = (
                    self.game.field_model.ball_coordinate.map(|c| (c.x, c.y)),
                    self.game.field_model.ball_moving,
                    self.game.field_model.ball_in_play,
                );
                if ball_now != ball_before_step {
                    eprintln!("BALLCHG step={:?} {:?} -> {:?} (rng={} acting={:?})",
                        entry.step.id(), ball_before_step, ball_now,
                        self.rng.call_count, self.game.acting_player.player_id);
                }
            }
            if std::env::var_os("FFB_RR_STEPS").is_some() {
                let rr_now = (self.game.turn_data_home.rerolls, self.game.turn_data_away.rerolls);
                if rr_now != rr_before_step {
                    eprintln!("RRSTEP step={:?} home {}->{} away {}->{} (half={} rng={})",
                        entry.step.id(), rr_before_step.0, rr_now.0, rr_before_step.1, rr_now.1,
                        self.game.half, self.rng.call_count);
                }
            }
            if std::env::var_os("FFB_RNG_STEPS").is_some() && self.rng.call_count != rng_before_step {
                self.rng_step_seq += 1;
                eprintln!("RNGSTEP {} step={:?} {}->{} pid={:?} pa={:?}", self.rng_step_seq,
                    entry.step.id(), rng_before_step, self.rng.call_count,
                    self.game.acting_player.player_id, self.game.acting_player.player_action);
            }
            let pushed_len: usize = outcome.pushes.iter().map(|s| s.len()).sum();
            self.apply_effects(&mut entry, &mut outcome);
            if outcome.push_self {
                self.apply_push_self(entry, pushed_len);
                self.waiting_for_command = false;
            } else {
                self.dispatch_after_start(entry, outcome);
            }
            if self.pending_prompt.is_some() || self.waiting_for_command { return; }
        }
    }

    fn dispatch(&mut self, entry: DriverStepEntry, action: Action, outcome: StepOutcome) {
        self.waiting_for_command = matches!(outcome.action, StepAction::Continue);
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
        self.waiting_for_command = matches!(outcome.action, StepAction::Continue);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_stack_clear_empties_stack() {
        let mut stack = DriverStepStack::new();
        stack.push(DriverStepEntry::new(Box::new(NoOpStep(StepId::NoOp))));
        stack.push(DriverStepEntry::new(Box::new(NoOpStep(StepId::NoOp))));
        assert_eq!(stack.len(), 2);
        stack.clear();
        assert!(stack.is_empty());
    }

    #[test]
    fn clear_step_stack_drops_stack_and_current() {
        let mut gs = new_game(1);
        // A fresh game is parked waiting on a prompt — `current` is populated.
        assert!(gs.current_prompt().is_some());
        gs.stack.push(DriverStepEntry::new(Box::new(NoOpStep(StepId::NoOp))));
        assert!(!gs.stack.is_empty());

        gs.clear_step_stack();

        assert!(gs.stack.is_empty());
        assert!(gs.current.is_none());
        assert!(gs.pending_prompt.is_none());
    }

    #[test]
    fn make_step_for_routes_bb2016_spectators_but_not_bb2025() {
        // bb2016 Spectators (2D6 + fame) is a different class from the mixed bb2020+ (1D3).
        // make_step_for must route it for bb2016 only; every other edition uses make_step unchanged.
        use ffb_model::enums::Rules;
        // Round-trips its id in all editions (sanity).
        assert_eq!(make_step_for(StepId::Spectators, Rules::Bb2016).id(), StepId::Spectators);
        assert_eq!(make_step_for(StepId::Spectators, Rules::Bb2025).id(), StepId::Spectators);
        // A non-overridden step is identical across editions (delegates to make_step).
        assert_eq!(make_step_for(StepId::Weather, Rules::Bb2016).id(), StepId::Weather);
        assert_eq!(make_step_for(StepId::Weather, Rules::Bb2025).id(), StepId::Weather);
        // BB2016 kickoff chain (1-square gust etc.) is routed to the bb2016 impls; all round-trip.
        for id in [StepId::KickoffScatterRoll, StepId::KickoffResultRoll, StepId::ApplyKickoffResult, StepId::CatchScatterThrowIn] {
            assert_eq!(make_step_for(id, Rules::Bb2016).id(), id, "bb2016 routing of {id:?}");
            assert_eq!(make_step_for(id, Rules::Bb2025).id(), id, "bb2025 passthrough of {id:?}");
        }
    }

    #[test]
    fn push_end_game_sequence_pushes_seven_steps_and_drives_to_finished() {
        let mut gs = new_game(2);
        gs.clear_step_stack();
        gs.push_end_game_sequence(true);
        assert_eq!(gs.stack.len(), 7);
        gs.run_until_prompt();
        assert!(gs.is_finished());
    }

    #[test]
    fn finished_game_drops_leftover_stack_without_running_it() {
        // Java: a FINISHED game runs no further steps. Rust pushes end_game_sequence ON TOP of a
        // leftover (never-played) next-turn sequence; once EndGame flips the game to Finished the
        // driver must drop those leftover steps instead of popping them (a phantom cycle that
        // re-ran KO recovery / PlayerLoss corrupted the final state — seed 19).
        let mut gs = new_game(2);
        gs.clear_step_stack();
        // Leftover steps beneath the end-game push; each rolls a die if it ever runs.
        gs.stack.push(DriverStepEntry::new(Box::new(RngStep)));
        gs.stack.push(DriverStepEntry::new(Box::new(RngStep)));
        gs.push_end_game_sequence(true);
        let rng_before = gs.rng.call_count;
        gs.run_until_prompt();
        assert!(gs.is_finished(), "EndGame flipped the game to Finished");
        assert!(gs.stack.is_empty(), "leftover steps were dropped once finished");
        assert!(gs.current.is_none());
        assert_eq!(gs.rng.call_count, rng_before,
            "leftover RngSteps never ran — no dice rolled after the game ended");
    }

    /// Exhaustive replacement for the per-file `id_is_*` tests: every `StepId`
    /// variant constructs via `make_step` and round-trips its `id()`.
    /// The wildcard-free `match` below fails to compile when a variant is added
    /// or removed, forcing this list (and the pinned count) to be updated.
    #[test]
    fn make_step_round_trips_every_step_id() {
        macro_rules! all_step_ids {
            ($($v:ident),* $(,)?) => {{
                #[allow(dead_code)]
                fn assert_exhaustive(id: StepId) {
                    match id { $(StepId::$v => {}),* }
                }
                [$(StepId::$v),*]
            }};
        }
        let ids = all_step_ids![
            InitStartGame, Spectators, Weather, Kickoff, Setup, KickoffScatterRoll, KickoffResultRoll,
            ApplyKickoffResult, EndKickoff, CoinChoice, ReceiveChoice, Touchback, InitSelecting, EndSelecting,
            InitActivation, StandUp, JumpUp, ResetFumblerooskie, InitMoving, Move, GoForIt, MoveDodge,
            FallDown, EndMoving, InitBlocking, BlockRoll, BlockChoice, BlockDodge, Pushback, Followup,
            BothDown, EndBlocking, DropFallingPlayers, PlaceBall, InitFouling, Foul, Referee, Bribes,
            EjectPlayer, EndFouling, InitPassing, Pass, DispatchPassing, Intercept, ResolvePass, HandOver,
            MissedPass, EndPassing, PickUp, CatchScatterThrowIn, Apothecary, EndPlayerAction, EndTurn,
            EndGame, Mvp, NoOp, GotoLabel, NextStep, AlwaysHungry, DispatchScatterPlayer, EndScatterPlayer,
            EndThrowTeamMate, InitScatterPlayer, InitThrowTeamMate, RightStuff, ThrowTeamMate, FumbleTtmPass,
            InitKickoff, KickoffScatterRollAskAfter, KickoffAnimation, KickoffReturn, BuyCardsAndInducements,
            BuyInducements, BuyCards, PettyCash, BlitzTurn, Jump, Swarming, HypnoticGaze, Shadowing,
            BlockChainsaw, BreatheFire, Chomp, HitAndRun, Trickster, Juggernaut, BlockBallAndChain, MoveBallAndChain,
            DivingTackle, Dauntless, DauntlessMultiple, DoubleStrength, SetDefender, Stab, HandleDropPlayerContext,
            FoulChainsaw, FoulAppearance, FoulAppearanceMultiple, AnimalSavagery, Animosity, BoneHead,
            ReallyStupid, WildAnimal, TakeRoot, ForgoneStalling, CheckStalling, StallingPlayer, AutoGazeZoat,
            BalefulHex, BlackInk, Bombardier, Horns, GettingEven, ProjectileVomit, QuickBite, RaidingParty,
            SafeThrow, Swoop, Tentacles, Treacherous, UnchannelledFury, WisdomOfTheWhiteDwarf, Wrestle,
            HailMaryPass, PassBlock, DumpOff, DispatchDumpOff, InitInducement, EndInducement, WeatherMage,
            Wizard, ThrowARock, InitKickTeamMate, EndKickTeamMate, KickTeamMate, KickTeamMateDoubleRolled,
            AssignTouchdowns, InitEndGame, Winnings, PlayerLoss, FanFactor, DedicatedFans, MasterChef,
            RiotousRookies, Prayer, Prayers, PrayerRoll, InitPunt, PuntDirection, PuntDistance, EndPunt,
            InitBomb, EndBomb, ResolveBomb, Bombardier2, SelectGazeTarget, SelectGazeTargetEnd, LookIntoMyEyes,
            InitLookIntoMyEyes, ApothecaryMultiple, BlockRollMultiple, MultipleBlockFork, ReportStabInjury, NextStepAndRepeat,
            InitFeeding, EndFeeding, EatTeamMate, AllYouCanEat, InitFuriousOutburst, FirstMoveFuriousOutburst,
            SecondMoveFuriousOutburst, EndFuriousOutburst, SpecialEffect, ConsumeParameter, SetActingPlayerAndTeam,
            SetActingTeam, StateMultipleRolls, SteadyFooting, PickMeUp, PileDriver, CatchOfTheDay, RecheckExplodeSkill,
            DropActingPlayer, DropDivingTackler, ThenIStartedBlastin, EndThenIStartedBlastin, ThrowKeg,
            EndThrowKeg, BlockStatistics, SelectBlitzTarget, SelectBlitzTargetEnd, RemoveTargetSelectionState,
            ResetToMove, PenaltyShootout, TrapDoor, Pro, RevertEndTurn, BloodLust, PlayCard, CloudBurster,
        ];
        assert_eq!(ids.len(), 200, "StepId variant count changed - update this list and pin");
        for id in ids {
            assert_eq!(make_step(id).id(), id, "make_step({id:?}) built a step with a mismatched id");
        }
    }
}
