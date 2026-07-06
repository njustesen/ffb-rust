/// 1:1 translation of `com.fumbbl.ffb.server.InjuryResult`.
use crate::util::util_server_game::UtilServerGame;
use ffb_model::enums::{
    ApothecaryMode, ApothecaryStatus, PlayerState, SeriousInjuryKind,
    PS_PRONE, PS_STUNNED, PS_KNOCKED_OUT, PS_BADLY_HURT, PS_SERIOUS_INJURY, PS_RIP, PS_RESERVE,
};
use ffb_model::model::SoundId;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::model::game::Game;
use ffb_model::util::util_box::UtilBox;
use crate::injury::InjuryContext;

/// Precedence order for player states (higher index = more severe).
/// Used in `applyTo` to prevent downgrading injuries (e.g. RIP → BH).
pub static BASE_PRECEDENCE: &[u32] = &[
    PS_PRONE,
    PS_STUNNED,
    PS_KNOCKED_OUT,
    PS_BADLY_HURT,
    PS_SERIOUS_INJURY,
    PS_RIP,
    PS_RESERVE,
];

pub struct InjuryResult {
    pub already_reported: bool,
    /// `true` until `passed_regeneration()` is called.
    pub pre_regeneration: bool,
    pub injury_context: InjuryContext,
}

impl InjuryResult {
    pub fn new() -> Self {
        Self {
            already_reported: false,
            pre_regeneration: true,
            injury_context: InjuryContext::new(ApothecaryMode::Attacker),
        }
    }

    pub fn injury_context(&self) -> &InjuryContext {
        &self.injury_context
    }

    pub fn injury_context_mut(&mut self) -> &mut InjuryContext {
        &mut self.injury_context
    }

    pub fn set_injury_context(&mut self, context: InjuryContext) {
        self.injury_context = context;
    }

    pub fn set_already_reported(&mut self, already_reported: bool) {
        self.already_reported = already_reported;
    }

    pub fn is_already_reported(&self) -> bool {
        self.already_reported
    }

    pub fn is_pre_regeneration(&self) -> bool {
        self.pre_regeneration
    }

    pub fn passed_regeneration(&mut self) {
        self.pre_regeneration = false;
    }

    /// Returns the precedence index for a PlayerState base, or `None` if not in the list.
    /// Higher index = more severe (used to prevent downgrading).
    pub fn precedence(state: PlayerState) -> Option<usize> {
        BASE_PRECEDENCE.iter().position(|&b| b == state.base())
    }

    /// Whether applying `new_state` would be an upgrade (more severe) relative to `old_state`.
    pub fn is_worse_than(new_state: PlayerState, old_state: PlayerState) -> bool {
        match (Self::precedence(new_state), Self::precedence(old_state)) {
            (Some(new_idx), Some(old_idx)) => new_idx > old_idx,
            _ => false,
        }
    }

    /// Java: `InjuryResult.applyTo(IStep)` — applies injury outcome to game state.
    ///
    /// Sets the defender's player state (respecting precedence order),
    /// deactivates the player if stunned and on the active team,
    /// and moves KO/casualty/reserve players to the dugout box.
    /// Also updates PlayerResult (secret weapon flag, serious injury, send-to-box) and
    /// TeamResult injury counters, and awards the attacker a casualty SPP count.
    ///
    /// client-only: BloodSpot — visual effect, not ported.
    pub fn apply_to(&self, game: &mut Game) {
        let ctx = &self.injury_context;
        let defender_id = match ctx.defender_id.as_deref() {
            Some(id) => id.to_owned(),
            None => return,
        };
        let new_state = match ctx.injury {
            Some(s) => s,
            None => return,
        };

        // Java: if (NamedProperties.getsSentOffAtEndOfDrive) playerResult.setHasUsedSecretWeapon(true)
        let defender_has_secret_weapon = game.player(&defender_id)
            .map(|p| p.has_skill_property(NamedProperties::GETS_SENT_OFF_AT_END_OF_DRIVE))
            .unwrap_or(false);
        let defender_is_home = game.team_home.player(&defender_id).is_some();
        if defender_has_secret_weapon {
            let tr = game.game_result.team_result_mut(defender_is_home);
            tr.player_result_mut(&defender_id).has_used_secret_weapon = true;
        }

        // Respect precedence: only apply if new_state is worse than existing.
        let current = game.field_model.player_state(&defender_id);
        let should_set = match current {
            None => true,
            Some(cur) => {
                let cur_rank = BASE_PRECEDENCE.iter().position(|&b| b == cur.base());
                match cur_rank {
                    None => true,
                    Some(cr) => {
                        let new_rank = BASE_PRECEDENCE.iter().position(|&b| b == new_state.base());
                        new_rank.map(|nr| nr > cr).unwrap_or(false)
                    }
                }
            }
        };

        if should_set {
            let existing = game.field_model.player_state(&defender_id).unwrap_or(new_state);
            game.field_model.set_player_state(&defender_id, existing.change_base(new_state.base()));
        }

        // Java: STUNNED → deactivate if on acting team OR on the bombardier's team (bomb hits friendlies).
        if new_state.base() == PS_STUNNED {
            let (home_bomb, away_bomb) = if let Some(ref orig_id) = game.original_bombardier.clone() {
                let bombardier_is_home = game.team_home.player(orig_id).is_some();
                (bombardier_is_home, !bombardier_is_home)
            } else {
                (false, false)
            };
            let should_deactivate = if defender_is_home {
                game.home_playing || home_bomb
            } else {
                !game.home_playing || away_bomb
            };
            if should_deactivate {
                let state = game.field_model.player_state(&defender_id).unwrap_or(new_state);
                game.field_model.set_player_state(&defender_id, state.change_active(false));
            }
        }

        // KO / casualty / reserve → put into dugout box.
        let final_state = game.field_model.player_state(&defender_id).unwrap_or(new_state);
        let base = final_state.base();
        if base == PS_KNOCKED_OUT || final_state.is_casualty() || base == PS_RESERVE {
            UtilBox::put_player_into_box(game, &defender_id);
            UtilServerGame::update_player_state_dependent_properties(game);
            UtilServerGame::check_for_wasted_skills(game, &defender_id);
        }

        // Java: death is also a serious injury — update PlayerResult serious injury.
        if new_state.base() == PS_RIP {
            let tr = game.game_result.team_result_mut(defender_is_home);
            let pr = tr.player_result_mut(&defender_id);
            pr.serious_injury = Some(SeriousInjuryKind::Dead);
        } else {
            // Java: else if playerResult.getSeriousInjury() != null → set decay (multiblock)
            // Java: else → set serious injury + decay from context
            if let Some(si) = ctx.serious_injury {
                let tr = game.game_result.team_result_mut(defender_is_home);
                let pr = tr.player_result_mut(&defender_id);
                if pr.serious_injury.is_some() {
                    pr.serious_injury_decay = Some(si);
                } else {
                    pr.serious_injury = Some(si);
                    pr.serious_injury_decay = ctx.serious_injury_decay;
                }
            }
        }

        // Java: if (injuryContext.getSendToBoxReason() != null) update player result.
        if let Some(reason) = ctx.send_to_box_reason {
            let attacker_id = ctx.attacker_id.clone().unwrap_or_default();
            let tr = game.game_result.team_result_mut(defender_is_home);
            let pr = tr.player_result_mut(&defender_id);
            pr.send_to_box_reason = Some(reason);
            pr.send_to_box_turn = ctx.send_to_box_turn;
            pr.send_to_box_half = ctx.send_to_box_half;
            pr.send_to_box_by_player_id = if attacker_id.is_empty() { None } else { Some(attacker_id) };
        }

        // Java: if (injuryContext.getSufferedInjury() != null && updateStats) — update TeamResult.
        if let Some(suffered) = ctx.suffered_injury {
            if ctx.is_caused_by_opponent {
                // Java: apothecary RESULT_CHOICE + RESERVE → count as BADLY_HURT
                let count_state = if ctx.apothecary_status == ApothecaryStatus::ResultChoice
                    && suffered.base() == PS_RESERVE
                {
                    PlayerState::new(PS_BADLY_HURT)
                } else {
                    suffered
                };
                game.game_result.team_result_mut(defender_is_home).suffer_injury(count_state);

                // Java: if attacker caused casualty → increment attacker's casualty counter.
                if let Some(attacker_id) = &ctx.attacker_id {
                    let casualty_is_against_opponent = game.team_home.player(attacker_id).is_some() != defender_is_home;
                    if suffered.is_casualty() && ctx.is_worth_spps && casualty_is_against_opponent {
                        let attacker_is_home = game.team_home.player(attacker_id).is_some();
                        let attacker_team_id = if attacker_is_home {
                            game.team_home.id.clone()
                        } else {
                            game.team_away.id.clone()
                        };
                        let has_additional_spp = game.prayer_state.get_additional_cas_spp_teams().contains(&attacker_team_id);
                        let attacker_id_owned = attacker_id.clone();
                        let pr = game.game_result.team_result_mut(attacker_is_home)
                            .player_result_mut(&attacker_id_owned);
                        pr.casualties += 1;
                        if has_additional_spp {
                            pr.casualties_with_additional_spp += 1;
                        }
                    }
                }
            }
            // client-only: BloodSpot — field model visual, not needed for server logic
        }
    }

    /// Java: `InjuryResult.report(IStep)` — delegates to `StateMechanic.reportInjury`.
    /// Note: step code uses `crate::injury::InjuryResult.report()` directly; this
    /// variant is kept for tests and legacy callers.
    pub fn report(&mut self, game: &mut Game) {
        // Translate to the canonical InjuryResult used by the trait, emit, then sync back.
        let mut canonical = crate::injury::InjuryResult::new(self.injury_context.apothecary_mode);
        canonical.injury_context = self.injury_context.clone();
        canonical.already_reported = self.already_reported;
        canonical.pre_regeneration = self.pre_regeneration;
        let mechanic = crate::mechanic::state_mechanic_for(game.rules);
        mechanic.report_injury(game, &mut canonical);
        self.already_reported = canonical.already_reported;
    }

    /// Java: `InjuryResult.handleIgnoringArmourBreaks(IStep, Player, Game)`.
    ///
    /// If armour was broken AND the defender has `ignoreFirstArmourBreak`:
    /// resets `armor_broken`, sets injury to PRONE, and returns `true`.
    ///
    /// Deactivates the Force Shield (or equivalent) card via UtilServerCards::deactivate_card_for_player.
    pub fn handle_ignoring_armour_breaks(&mut self, game: &mut Game) -> bool {
        if !self.injury_context.armor_broken {
            return false;
        }
        if self.injury_context.armor_roll.is_none() {
            return false;
        }
        let has_property = self.injury_context.defender_id.as_deref()
            .and_then(|id| game.player(id))
            .map(|p| p.has_skill_property(NamedProperties::IGNORE_FIRST_ARMOUR_BREAK))
            .unwrap_or(false);

        if has_property {
            self.injury_context.armor_broken = false;
            self.injury_context.injury = Some(PlayerState::new(PS_PRONE));
            // Java: UtilServerCards.deactivateCard(gameState, player) — deactivate Force Shield
            if let Some(defender_id) = self.injury_context.defender_id.clone() {
                use crate::util::util_server_cards::UtilServerCards;
                UtilServerCards::deactivate_card_for_player(game, &defender_id);
            }
            return true;
        }
        false
    }

    /// Java: `InjuryResult.swapToAlternateContext(IStep, Game)`.
    ///
    /// If a modified injury context exists (set by apothecary), replaces the current
    /// context with it, clears `already_reported`, and handles `ignoreFirstArmourBreak`.
    pub fn swap_to_alternate_context(&mut self, game: &mut Game) {
        if self.injury_context.modified_injury_context.is_none() {
            return;
        }
        let modified = *self.injury_context.modified_injury_context.take().unwrap();
        self.injury_context = modified;
        self.already_reported = false;

        if self.handle_ignoring_armour_breaks(game) {
            self.injury_context.send_to_box_reason = None;
            self.injury_context.send_to_box_half = 0;
            self.injury_context.send_to_box_turn = 0;
            self.injury_context.apothecary_status = ApothecaryStatus::NoApothecary;
            self.injury_context.serious_injury = None;
            self.injury_context.serious_injury_decay = None;
            self.injury_context.sound = Some(SoundId::FALL);
        }
    }
}

impl Default for InjuryResult {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PS_BADLY_HURT, PS_RIP, PS_KNOCKED_OUT, PS_STUNNED, PS_STANDING, PS_SERIOUS_INJURY, Rules};
    use ffb_model::model::game::Game;
    use ffb_model::types::FieldCoordinate;
    use crate::step::framework::test_team;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn new_has_correct_defaults() {
        let r = InjuryResult::new();
        assert!(!r.already_reported);
        assert!(r.pre_regeneration);
    }

    #[test]
    fn passed_regeneration_clears_flag() {
        let mut r = InjuryResult::new();
        r.passed_regeneration();
        assert!(!r.pre_regeneration);
    }

    #[test]
    fn set_already_reported() {
        let mut r = InjuryResult::new();
        r.set_already_reported(true);
        assert!(r.is_already_reported());
    }

    #[test]
    fn precedence_known_states() {
        let rip = PlayerState::new(PS_RIP);
        let bh = PlayerState::new(PS_BADLY_HURT);
        assert!(InjuryResult::precedence(rip) > InjuryResult::precedence(bh));
    }

    #[test]
    fn is_worse_than_rip_over_ko() {
        let rip = PlayerState::new(PS_RIP);
        let ko = PlayerState::new(PS_KNOCKED_OUT);
        assert!(InjuryResult::is_worse_than(rip, ko));
        assert!(!InjuryResult::is_worse_than(ko, rip));
    }

    #[test]
    fn is_worse_than_same_state_is_false() {
        let stunned = PlayerState::new(PS_STUNNED);
        assert!(!InjuryResult::is_worse_than(stunned, stunned));
    }

    #[test]
    fn set_injury_context_replaces() {
        let mut r = InjuryResult::new();
        let mut ctx = InjuryContext::new(ApothecaryMode::Attacker);
        ctx.armor_broken = true;
        r.set_injury_context(ctx);
        assert!(r.injury_context.armor_broken);
    }

    // ── apply_to tests ───────────────────────────────────────────────────────

    #[test]
    fn apply_to_sets_player_state_when_standing() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("p1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        assert_eq!(game.field_model.player_state("p1").unwrap().base(), PS_STUNNED);
    }

    #[test]
    fn apply_to_does_not_downgrade_rip_to_ko() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", PlayerState::new(PS_RIP));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("p1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_KNOCKED_OUT));
        ir.apply_to(&mut game);

        // KO must not override RIP
        assert_eq!(game.field_model.player_state("p1").unwrap().base(), PS_RIP);
    }

    #[test]
    fn apply_to_ko_puts_player_in_box() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("p1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_KNOCKED_OUT));
        ir.apply_to(&mut game);

        // Player should have been moved to the box (no longer at pitch coordinate).
        let coord = game.field_model.player_coordinate("p1");
        assert!(coord.map(|c| c.is_box_coordinate()).unwrap_or(true));
    }

    #[test]
    fn apply_to_no_defender_id_is_noop() {
        let mut game = make_game();
        let mut ir = InjuryResult::new();
        // defender_id not set
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game); // must not panic
    }

    #[test]
    fn apply_to_no_injury_is_noop() {
        let mut game = make_game();
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("p1".into());
        // injury not set
        ir.apply_to(&mut game);

        assert_eq!(game.field_model.player_state("p1").unwrap().base(), PS_STANDING);
    }

    // ── handle_ignoring_armour_breaks tests ──────────────────────────────────

    #[test]
    fn handle_ignoring_armour_breaks_returns_false_when_not_broken() {
        let mut game = make_game();
        let mut ir = InjuryResult::new();
        ir.injury_context.armor_broken = false;
        ir.injury_context.armor_roll = Some([3, 4]);
        assert!(!ir.handle_ignoring_armour_breaks(&mut game));
    }

    #[test]
    fn handle_ignoring_armour_breaks_returns_false_when_no_armor_roll() {
        let mut game = make_game();
        let mut ir = InjuryResult::new();
        ir.injury_context.armor_broken = true;
        ir.injury_context.armor_roll = None;
        assert!(!ir.handle_ignoring_armour_breaks(&mut game));
    }

    #[test]
    fn handle_ignoring_armour_breaks_returns_false_when_no_property() {
        let mut game = make_game();
        let mut ir = InjuryResult::new();
        ir.injury_context.armor_broken = true;
        ir.injury_context.armor_roll = Some([3, 4]);
        ir.injury_context.defender_id = Some("p1".into());
        // p1 not in game / no skill → returns false
        assert!(!ir.handle_ignoring_armour_breaks(&mut game));
    }

    // ── swap_to_alternate_context tests ──────────────────────────────────────

    #[test]
    fn swap_to_alternate_context_no_modified_is_noop() {
        let mut game = make_game();
        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("p1".into());
        ir.already_reported = true;
        ir.swap_to_alternate_context(&mut game);
        // nothing changed
        assert!(ir.already_reported);
    }

    #[test]
    fn swap_to_alternate_context_swaps_and_resets_reported() {
        let mut game = make_game();
        let mut ir = InjuryResult::new();
        ir.already_reported = true;

        let mut modified = InjuryContext::new(ApothecaryMode::Attacker);
        modified.defender_id = Some("p1".into());
        modified.injury = Some(PlayerState::new(PS_BADLY_HURT));
        ir.injury_context.modified_injury_context = Some(Box::new(modified));

        ir.swap_to_alternate_context(&mut game);

        assert!(!ir.already_reported);
        assert_eq!(ir.injury_context.injury.unwrap().base(), PS_BADLY_HURT);
        assert!(ir.injury_context.modified_injury_context.is_none());
    }

    #[test]
    fn report_emits_injury_report() {
        let mut game = make_game();
        let mut ir = InjuryResult::new();
        ir.report(&mut game);
        assert!(ir.is_already_reported());
        assert!(game.report_list.has_report(ffb_model::report::ReportId::INJURY));
    }

    // ── PlayerResult / TeamResult wiring tests ───────────────────────────────

    fn add_player_to_game(game: &mut Game, team_id: &str, player_id: &str) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let p = Player {
            id: player_id.into(), name: player_id.into(), nr: 1,
            position_id: "pos".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            ..Default::default()
        };
        if team_id == "home" { game.team_home.players.push(p); }
        else { game.team_away.players.push(p); }
    }

    #[test]
    fn apply_to_updates_send_to_box_reason_on_player_result() {
        use ffb_model::enums::SendToBoxReason;
        let mut game = make_game();
        add_player_to_game(&mut game, "home", "h1");
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("h1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_KNOCKED_OUT));
        ir.injury_context.send_to_box_reason = Some(SendToBoxReason::Blocked);
        ir.injury_context.send_to_box_turn = 3;
        ir.injury_context.send_to_box_half = 1;
        ir.apply_to(&mut game);

        let pr = game.game_result.home.player_result("h1").unwrap();
        assert_eq!(pr.send_to_box_reason, Some(SendToBoxReason::Blocked));
        assert_eq!(pr.send_to_box_turn, 3);
        assert_eq!(pr.send_to_box_half, 1);
    }

    #[test]
    fn apply_to_updates_serious_injury_on_player_result() {
        use ffb_model::enums::SeriousInjuryKind;
        let mut game = make_game();
        add_player_to_game(&mut game, "home", "h1");
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("h1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        ir.injury_context.serious_injury = Some(SeriousInjuryKind::BrokenArmPa);
        ir.apply_to(&mut game);

        let pr = game.game_result.home.player_result("h1").unwrap();
        assert_eq!(pr.serious_injury, Some(SeriousInjuryKind::BrokenArmPa));
    }

    #[test]
    fn apply_to_increments_team_result_badly_hurt() {
        use ffb_model::enums::PS_STANDING;
        let mut game = make_game();
        add_player_to_game(&mut game, "home", "h1");
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("h1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_BADLY_HURT));
        ir.injury_context.suffered_injury = Some(PlayerState::new(PS_BADLY_HURT));
        ir.injury_context.is_caused_by_opponent = true;
        ir.injury_context.is_worth_spps = true;
        ir.apply_to(&mut game);

        assert_eq!(game.game_result.home.badly_hurt_suffered, 1);
    }

    #[test]
    fn apply_to_increments_attacker_casualty_count_for_spp_injury() {
        let mut game = make_game();
        add_player_to_game(&mut game, "home", "h1");  // attacker
        add_player_to_game(&mut game, "away", "a1");  // defender
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("a1", PlayerState::new(PS_STANDING));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("a1".into());
        ir.injury_context.attacker_id = Some("h1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_BADLY_HURT));
        ir.injury_context.suffered_injury = Some(PlayerState::new(PS_BADLY_HURT));
        ir.injury_context.is_caused_by_opponent = true;
        ir.injury_context.is_worth_spps = true;
        ir.apply_to(&mut game);

        // Attacker (home h1) should have casualty count incremented
        let pr = game.game_result.home.player_result("h1").unwrap();
        assert_eq!(pr.casualties, 1);
    }

    #[test]
    fn apply_to_does_not_award_spp_when_not_worth_spps() {
        let mut game = make_game();
        add_player_to_game(&mut game, "home", "h1");
        add_player_to_game(&mut game, "away", "a1");
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("a1", PlayerState::new(PS_STANDING));

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("a1".into());
        ir.injury_context.attacker_id = Some("h1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_BADLY_HURT));
        ir.injury_context.suffered_injury = Some(PlayerState::new(PS_BADLY_HURT));
        ir.injury_context.is_caused_by_opponent = true;
        ir.injury_context.is_worth_spps = false;  // not a ForSpp injury type
        ir.apply_to(&mut game);

        // No casualties on attacker
        assert!(game.game_result.home.player_results.get("h1").is_none()
            || game.game_result.home.player_results["h1"].casualties == 0);
    }

    // ── bomb team deactivation tests ─────────────────────────────────────────

    fn active_standing() -> PlayerState {
        PlayerState::new(PS_STANDING).change_active(true)
    }

    // Normal (non-bomb) STUNNED: active team player → deactivated.
    #[test]
    fn stunned_active_team_player_deactivated_no_bomb() {
        let mut game = make_game();
        game.home_playing = true;
        add_player_to_game(&mut game, "home", "h1");
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", active_standing());

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("h1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        // h1 is on home team, home is playing → deactivated
        let state = game.field_model.player_state("h1").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
        assert!(!state.is_active());
    }

    // Normal (non-bomb) STUNNED: inactive team player → NOT deactivated.
    #[test]
    fn stunned_inactive_team_player_not_deactivated_no_bomb() {
        let mut game = make_game();
        game.home_playing = true;
        add_player_to_game(&mut game, "away", "a1");
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("a1", active_standing());

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("a1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        // a1 is on away team, home is playing → a1 is inactive team → active bit preserved
        let state = game.field_model.player_state("a1").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
        assert!(state.is_active());
    }

    // Bomb: home bombardier stuns home player during away turn → deactivated (homeBomb).
    #[test]
    fn bomb_stuns_friendly_player_deactivated_when_bombardier_same_team() {
        let mut game = make_game();
        game.home_playing = false; // away's turn
        add_player_to_game(&mut game, "home", "bombardier");
        add_player_to_game(&mut game, "home", "friendly");
        game.field_model.set_player_coordinate("friendly", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("friendly", active_standing());
        game.original_bombardier = Some("bombardier".into()); // home bombardier

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("friendly".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        // friendly is home team, bombardier is home → homeBomb=true → deactivated even on away turn
        let state = game.field_model.player_state("friendly").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
        assert!(!state.is_active());
    }

    // Bomb: home bombardier stuns away player during away turn → deactivated by active-team rule.
    #[test]
    fn bomb_stuns_enemy_player_deactivated_by_active_team_rule() {
        let mut game = make_game();
        game.home_playing = false; // away's turn — away is active
        add_player_to_game(&mut game, "home", "bombardier");
        add_player_to_game(&mut game, "away", "enemy");
        game.field_model.set_player_coordinate("enemy", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("enemy", active_standing());
        game.original_bombardier = Some("bombardier".into()); // home bombardier

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("enemy".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        // enemy is away team (active on away's turn) → normal deactivation applies
        let state = game.field_model.player_state("enemy").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
        assert!(!state.is_active());
    }

    // Bomb: away bombardier stuns away player during home turn → deactivated (awayBomb).
    #[test]
    fn bomb_stuns_away_friendly_during_home_turn_deactivated() {
        let mut game = make_game();
        game.home_playing = true; // home's turn
        add_player_to_game(&mut game, "away", "bombardier");
        add_player_to_game(&mut game, "away", "friendly");
        game.field_model.set_player_coordinate("friendly", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("friendly", active_standing());
        game.original_bombardier = Some("bombardier".into()); // away bombardier

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("friendly".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        // friendly is away, bombardier is away → awayBomb=true → deactivated even on home's turn
        let state = game.field_model.player_state("friendly").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
        assert!(!state.is_active());
    }

    // Bomb: away bombardier stuns home player during home turn → deactivated by active-team rule.
    #[test]
    fn bomb_stuns_home_player_during_home_turn_deactivated_by_active_rule() {
        let mut game = make_game();
        game.home_playing = true; // home's turn
        add_player_to_game(&mut game, "away", "bombardier");
        add_player_to_game(&mut game, "home", "target");
        game.field_model.set_player_coordinate("target", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("target", active_standing());
        game.original_bombardier = Some("bombardier".into()); // away bombardier

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("target".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        // target is home (active on home's turn) → deactivated by active-team rule
        let state = game.field_model.player_state("target").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
        assert!(!state.is_active());
    }

    // No bomb: away player stunned while away is inactive → active bit preserved.
    #[test]
    fn no_bomb_away_player_stunned_inactive_turn_stays_active() {
        let mut game = make_game();
        game.home_playing = false; // away's turn — away is ACTIVE
        // Test: home player on away's turn → home is inactive → stays active
        add_player_to_game(&mut game, "home", "h1");
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_state("h1", active_standing());

        let mut ir = InjuryResult::new();
        ir.injury_context.defender_id = Some("h1".into());
        ir.injury_context.injury = Some(PlayerState::new(PS_STUNNED));
        ir.apply_to(&mut game);

        // h1 is home, away is playing → h1 is inactive team → active bit preserved
        let state = game.field_model.player_state("h1").unwrap();
        assert_eq!(state.base(), PS_STUNNED);
        assert!(state.is_active());
    }
}
