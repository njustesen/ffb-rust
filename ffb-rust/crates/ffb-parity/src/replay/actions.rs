//! BACKLOG §O O3: the action records of `FumbblReplayParser.parseFile`, 1:1
//! (`ffb-java/ffb/ffb-ai/src/main/java/com/fumbbl/ffb/ai/simulation/FumbblReplayParser.java`,
//! the pending-state machines around the model-change apply + the `build*Record` builders).
//!
//! [`Recorder::pre_sync`] = the PRE-SYNC CHECKS and the PRE-CHANGE PASS (one switch per report,
//! on the game BEFORE the sync's model changes), [`Recorder::post_sync`] = the POST-CHANGE PASS,
//! [`Recorder::finish`] = the final move flush + the end check. Every Java `try { .. } catch
//! (Exception e) { recordError(kind, e); }` is a `Result` whose `Err` counts into
//! `record_errors[kind]` -- statements before the failing one keep their side effects, exactly
//! as in Java.
//!
//! The two MoveDecisionEngine records (`player_select`, `move_target`) need the MDE scorers
//! (DOUBLE scores, `java.util.Random`); until they are ported [`Recorder::mde`] is `false` and
//! those records -- and only those -- are not produced (the gate then compares every other record).

use super::canonical::CanonicalJson;
use super::game_state_serializer::{self as gss, field_player_at, field_player_state, int, team_players, Obj};
use super::java_names;
use super::replay_report::ReplayReport;
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;
use std::collections::BTreeMap;

const AGENT_MODE: &str = "fumbbl_replay";

const PS_KNOCKED_OUT: u32 = 0x05;
const PS_BADLY_HURT: u32 = 0x06;
const PS_SERIOUS_INJURY: u32 = 0x07;
const PS_RIP: u32 = 0x08;
const PS_STANDING: u32 = 0x01;
const PS_PRONE: u32 = 0x03;
const PS_STUNNED: u32 = 0x04;

const CLASS_MIXED_INJURY: &str = "com.fumbbl.ffb.report.mixed.ReportInjury";
const CLASS_BB2016_INJURY: &str = "com.fumbbl.ffb.report.bb2016.ReportInjury";
const CLASS_MIXED_APOTHECARY_ROLL: &str = "com.fumbbl.ffb.report.mixed.ReportApothecaryRoll";
const CLASS_BB2016_APOTHECARY_ROLL: &str = "com.fumbbl.ffb.report.bb2016.ReportApothecaryRoll";
const CLASS_MIXED_ARGUE: &str = "com.fumbbl.ffb.report.mixed.ReportArgueTheCallRoll";
const CLASS_BB2016_ARGUE: &str = "com.fumbbl.ffb.report.bb2016.ReportArgueTheCallRoll";

type R<T> = Result<T, String>;

/// The record writer + every pending-state variable of `parseFile`.
#[allow(dead_code)] // next_move_action is read by the MoveDecisionEngine records (not ported yet)
pub struct Recorder {
    /// Whether the MoveDecisionEngine records are produced (not ported yet: always false).
    pub mde: bool,
    /// The record lines (`rec.toString()`), in write order.
    pub records: Vec<String>,
    /// `CUR.recordErrors`.
    pub record_errors: BTreeMap<String, u64>,
    /// The first error text of each kind (for the gate's diagnostics).
    pub first_errors: BTreeMap<String, String>,

    // ── RE_ROLL pending ──
    rr_player_id: Option<String>,
    rr_action_name: Option<String>,
    rr_min_roll: i32,
    rr_is_fumble: bool,
    rr_state: Option<CanonicalJson>,
    // ── FOLLOWUP_CHOICE pending ──
    fu_attacker_id: Option<String>,
    fu_attacker: (i32, i32),
    fu_target: (i32, i32),
    fu_state: Option<CanonicalJson>,
    fu_check_syncs: i32,
    // ── USE_APOTHECARY + APOTHECARY_CHOICE pending ──
    apo_player_id: Option<String>,
    apo_old_state_base: i32,
    apo_old_si: Option<String>,
    apo_decline_state: Option<CanonicalJson>,
    apo_countdown: i32,
    apo_new_state_base: i32,
    apo_new_si: Option<String>,
    // ── pass target pending ──
    pending_pass_coord: Option<FieldCoordinate>,
    pending_pass_state: Option<CanonicalJson>,
    pending_pass_deviated: bool,
    // ── USE_CHAINSAW pending ──
    pending_chainsaw: bool,
    pending_chainsaw_state: Option<CanonicalJson>,
    // ── PASS_BLOCK pending ──
    pb_player_id: Option<String>,
    pb_state: Option<CanonicalJson>,
    pb_old: (i32, i32),
    // ── TOUCHBACK pending ──
    touchback_state: Option<CanonicalJson>,
    touchback_pending: bool,
    // ── move_target (MDE) ──
    next_move_player_id: Option<String>,
    next_move_action: Option<&'static str>,
}

impl Default for Recorder {
    fn default() -> Self {
        Recorder {
            mde: false,
            records: Vec::new(),
            record_errors: BTreeMap::new(),
            first_errors: BTreeMap::new(),
            rr_player_id: None,
            rr_action_name: None,
            rr_min_roll: 0,
            rr_is_fumble: false,
            rr_state: None,
            fu_attacker_id: None,
            fu_attacker: (-1, -1),
            fu_target: (-1, -1),
            fu_state: None,
            fu_check_syncs: 0,
            apo_player_id: None,
            apo_old_state_base: -1,
            apo_old_si: None,
            apo_decline_state: None,
            apo_countdown: 0,
            apo_new_state_base: -1,
            apo_new_si: None,
            pending_pass_coord: None,
            pending_pass_state: None,
            pending_pass_deviated: false,
            pending_chainsaw: false,
            pending_chainsaw_state: None,
            pb_player_id: None,
            pb_state: None,
            pb_old: (-1, -1),
            touchback_state: None,
            touchback_pending: false,
            next_move_player_id: None,
            next_move_action: None,
        }
    }
}

impl Recorder {
    pub fn new() -> Self {
        Self::default()
    }

    fn write(&mut self, rec: CanonicalJson) {
        self.records.push(rec.to_json_string());
    }

    fn write_opt(&mut self, rec: Option<CanonicalJson>) {
        if let Some(r) = rec {
            self.write(r);
        }
    }

    /// `recordError(kind, e)`.
    fn record_error(&mut self, kind: &str, e: String) {
        *self.record_errors.entry(kind.to_string()).or_default() += 1;
        self.first_errors.entry(kind.to_string()).or_insert(e);
    }

    // ── PRE-SYNC CHECKS + PRE-CHANGE PASS ────────────────────────────────────

    pub fn pre_sync(&mut self, game: &Game, reports: &[ReplayReport]) {
        // RE_ROLL decline: pending from the last sync and no RE_ROLL in this one.
        if self.rr_state.is_some() {
            let has_re_roll = reports.iter().any(|r| r.is("RE_ROLL"));
            if !has_re_roll {
                let rec = build_re_roll_record(
                    self.rr_player_id.as_deref(),
                    self.rr_action_name.as_deref(),
                    self.rr_min_roll,
                    self.rr_is_fumble,
                    self.rr_state.clone(),
                    1,
                );
                self.write_opt(rec);
                self.rr_state = None;
                self.rr_player_id = None;
            }
        }
        // USE_APOTHECARY decline: countdown.
        if self.apo_player_id.is_some() && self.apo_countdown > 0 {
            let has_apo_roll = reports.iter().any(|r| r.is("APOTHECARY_ROLL"));
            if !has_apo_roll {
                self.apo_countdown -= 1;
                if self.apo_countdown <= 0 {
                    let rec = build_use_apothecary_record(
                        self.apo_player_id.as_deref(),
                        self.apo_old_state_base,
                        self.apo_old_si.as_deref(),
                        self.apo_decline_state.clone(),
                        1,
                    );
                    self.write_opt(rec);
                    self.apo_player_id = None;
                    self.apo_decline_state = None;
                }
            }
        }

        for report in reports {
            let mut rec: Option<CanonicalJson> = None;
            if let Err(e) = self.pre_report(game, report, &mut rec) {
                self.record_error(report.id, e);
            }
            if let Some(r) = rec {
                self.write(r);
            }

            // Post-switch: a failed, not re-rolled skill roll offers a re-roll.
            if self.rr_state.is_none()
                && report.skill_roll.is_some()
                && !report.is("BLOOD_LUST_ROLL")
                && !report.is("INTERCEPTION_ROLL")
                && !report.is("CHAINSAW_ROLL")
            {
                if let Err(e) = self.re_roll_offer(game, report) {
                    self.record_error("RE_ROLL_OFFER", e);
                }
            }

            // Post-switch: a block without CHAINSAW_ROLL declines the chainsaw.
            if self.pending_chainsaw && report.is("BLOCK_CHOICE") {
                let rec2 = build_dialog_record_with_state("USE_CHAINSAW", 1, Obj::new().build(), self.pending_chainsaw_state.clone());
                self.write_opt(rec2);
                self.pending_chainsaw = false;
                self.pending_chainsaw_state = None;
            }
        }
    }

    fn re_roll_offer(&mut self, game: &Game, report: &ReplayReport) -> R<()> {
        let sr = report.skill_roll.as_ref().expect("checked by the caller");
        if !sr.successful && !sr.re_rolled {
            let mut has_reroll = game.turn_data().rerolls > 0;
            if !has_reroll {
                if let Some(act_p) = game.acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
                    has_reroll = gss::java_skill_names(game, act_p)?.contains(&"Pro");
                }
            }
            if has_reroll {
                self.rr_player_id = sr.player_id.clone();
                self.rr_action_name = Some(report_id_to_re_roll_action(report.id));
                self.rr_min_roll = sr.minimum_roll;
                self.rr_is_fumble = report.is("PASS_ROLL") && !sr.successful;
                self.rr_state = Some(gss::serialize(game)?);
            }
        }
        Ok(())
    }

    /// The `switch (report.getId())` body; `rec` is Java's `rec` variable (kept when a later
    /// statement of the case throws).
    fn pre_report(&mut self, game: &Game, report: &ReplayReport, rec: &mut Option<CanonicalJson>) -> R<()> {
        match report.id {
            "BLOCK_CHOICE" => *rec = build_block_roll_properties_record(game, report)?,
            "SKILL_USE" => *rec = build_skill_use_record(game, report)?,
            "RECEIVE_CHOICE" => *rec = Some(build_flag_dialog_record(game, "RECEIVE_CHOICE", report.boolean("receiveChoice")?)?),
            "COIN_THROW" => {
                report.boolean("coinThrowHeads")?;
                *rec = Some(build_flag_dialog_record(game, "COIN_CHOICE", report.boolean("coinChoiceHeads")?)?)
            }
            "FOUL" => *rec = build_foul_target_record(game, report)?,
            "HAND_OVER" => *rec = build_handoff_target_record(game, report)?,
            "SELECT_GAZE_TARGET" => *rec = build_gaze_target_record(game, report)?,
            "RE_ROLL" => {
                if self.rr_state.is_some() {
                    *rec = build_re_roll_record(
                        self.rr_player_id.as_deref(),
                        self.rr_action_name.as_deref(),
                        self.rr_min_roll,
                        self.rr_is_fumble,
                        self.rr_state.clone(),
                        0,
                    );
                    self.rr_state = None;
                    self.rr_player_id = None;
                }
            }
            "PILING_ON" => *rec = build_piling_on_record(game, report)?,
            "INJURY" => {
                let is_injury_class = report.is_class(CLASS_MIXED_INJURY) || report.is_class(CLASS_BB2016_INJURY);
                let defender_id = if is_injury_class { report.defender_id()? } else { None };
                let inj_state = if is_injury_class { report.player_state("injury")? } else { None };
                let inj_si = if is_injury_class { report.serious_injury_name("seriousInjury")? } else { None };
                if let (Some(defender_id), Some(inj_state)) = (defender_id, inj_state) {
                    let base = inj_state.base();
                    if base == PS_BADLY_HURT || base == PS_SERIOUS_INJURY || base == PS_RIP {
                        if let Some(inj_player) = game.player(&defender_id) {
                            let inj_is_home = game.team_home.player(&inj_player.id).is_some();
                            let inj_td = if inj_is_home { &game.turn_data_home } else { &game.turn_data_away };
                            let apos = inj_td.apothecaries + inj_td.wandering_apothecaries;
                            if apos > 0 {
                                self.apo_player_id = Some(defender_id.clone());
                                self.apo_old_state_base = base as i32;
                                self.apo_old_si = inj_si;
                                self.apo_decline_state = Some(gss::serialize(game)?);
                                self.apo_countdown = 3;
                                self.apo_new_state_base = -1;
                                self.apo_new_si = None;
                            }
                        }
                    }
                }
            }
            "APOTHECARY_ROLL" => {
                let is_class = report.is_class(CLASS_MIXED_APOTHECARY_ROLL) || report.is_class(CLASS_BB2016_APOTHECARY_ROLL);
                let pid = if is_class { report.player_id()? } else { None };
                let new_state = if is_class { report.player_state("playerState")? } else { None };
                let new_si = if is_class { report.serious_injury_name("seriousInjury")? } else { None };
                if let (Some(pid), Some(apo)) = (pid, self.apo_player_id.clone()) {
                    if pid == apo {
                        *rec = build_use_apothecary_record(
                            self.apo_player_id.as_deref(),
                            self.apo_old_state_base,
                            self.apo_old_si.as_deref(),
                            self.apo_decline_state.clone(),
                            0,
                        );
                        self.apo_countdown = 0;
                        self.apo_new_state_base = new_state.map_or(-1, |s| s.base() as i32);
                        self.apo_new_si = new_si;
                        self.apo_player_id = None;
                        self.apo_decline_state = None;
                    }
                }
            }
            "APOTHECARY_CHOICE" => {
                if report.player_id()?.is_some() && self.apo_old_state_base != -1 && self.apo_new_state_base != -1 {
                    *rec = build_apothecary_choice_record(
                        game,
                        report,
                        self.apo_old_state_base,
                        self.apo_old_si.as_deref(),
                        self.apo_new_state_base,
                        self.apo_new_si.as_deref(),
                    )?;
                }
                self.apo_old_state_base = -1;
                self.apo_old_si = None;
                self.apo_new_state_base = -1;
                self.apo_new_si = None;
            }
            "PUSHBACK" => {
                let attacker_id = game.acting_player.player_id.clone();
                let defender_id = report.defender_id()?;
                if let (Some(attacker_id), Some(defender_id)) = (attacker_id, defender_id) {
                    if let (Some(att), Some(def)) = (game.player(&attacker_id), game.player(&defender_id)) {
                        let ac = game.field_model.player_coordinate(&att.id);
                        let dc = game.field_model.player_coordinate(&def.id);
                        if let (Some(ac), Some(dc)) = (ac, dc) {
                            self.fu_attacker_id = Some(attacker_id.clone());
                            self.fu_attacker = (ac.x, ac.y);
                            self.fu_target = (dc.x, dc.y);
                            self.fu_state = Some(gss::serialize(game)?);
                            self.fu_check_syncs = 3;
                        }
                    }
                }
            }
            "PASS_ROLL" => {
                self.pending_pass_coord = game.pass_coordinate;
                self.pending_pass_state = Some(gss::serialize(game)?);
                self.pending_pass_deviated = false;
            }
            "PASS_DEVIATE" => {
                let state_for_pass = match &self.pending_pass_state {
                    Some(s) => s.clone(),
                    None => gss::serialize(game)?,
                };
                if !report.boolean("throwTeamMate")? {
                    *rec = build_pass_target_record(report, self.pending_pass_coord, Some(state_for_pass))?;
                } else {
                    *rec = build_ttm_target_record(report, Some(state_for_pass))?;
                }
                self.pending_pass_deviated = true;
                self.pending_pass_coord = None;
                self.pending_pass_state = None;
            }
            "KICKOFF_SCATTER" => *rec = build_kickoff_target_record(game, report)?,
            "BLOOD_LUST_ROLL" => {
                let pid = report.player_id()?;
                *rec = build_simple_dialog_record(game, Some("BLOODLUST_ACTION"), 1, pid_param(pid.as_deref()))?;
            }
            "CHAINSAW_ROLL" => {
                let pid = report.player_id()?;
                *rec = build_simple_dialog_record(
                    game,
                    if self.pending_chainsaw_state.is_some() { None } else { Some("USE_CHAINSAW") },
                    0,
                    pid_param(pid.as_deref()),
                )?;
                if self.pending_chainsaw_state.is_some() {
                    *rec = build_dialog_record_with_state("USE_CHAINSAW", 0, pid_param(pid.as_deref()), self.pending_chainsaw_state.clone());
                    let target_rec = build_chainsaw_target_record(game, report)?;
                    self.write_opt(target_rec);
                }
                self.pending_chainsaw = false;
                self.pending_chainsaw_state = None;
            }
            "BRIBES_ROLL" => {
                let pid = report.player_id()?;
                *rec = build_simple_dialog_record(game, Some("BRIBES"), 0, pid_param(pid.as_deref()))?;
            }
            "ARGUE_THE_CALL" => {
                let pid = if report.is_class(CLASS_MIXED_ARGUE) || report.is_class(CLASS_BB2016_ARGUE) {
                    report.player_id()?
                } else {
                    None
                };
                *rec = build_simple_dialog_record(game, Some("ARGUE_THE_CALL"), 0, pid_param(pid.as_deref()))?;
            }
            "BRIBERY_AND_CORRUPTION_RE_ROLL" => *rec = Some(build_bribery_corruption_record(game)?),
            "INTERCEPTION_ROLL" => {
                let pid = report.player_id()?;
                *rec = build_simple_dialog_record(game, Some("INTERCEPTION"), 0, pid_param(pid.as_deref()))?;
            }
            "WIZARD_USE" => {
                let mut wp = Obj::new();
                let team_id = report.string("teamId")?;
                wp.str("team_id", team_id.as_deref().unwrap_or(""));
                if let Some(spell) = report.enum_constant("wizardSpell", java_names::SPECIAL_EFFECT)? {
                    let name = java_names::SPECIAL_EFFECT.iter().find(|(c, _)| *c == spell).map(|(_, n)| *n).unwrap_or_default();
                    wp.str("spell_type", name);
                }
                *rec = build_dialog_record_with_state("WIZARD_SPELL", 0, wp.build(), Some(gss::serialize(game)?));
            }
            "PASS_BLOCK" => {
                if report.boolean("passBlockAvailable")? {
                    let team_id = report.string("teamId")?;
                    let pb_team = if team_id.as_deref() == Some(game.team_home.id.as_str()) { &game.team_home } else { &game.team_away };
                    for p in team_players(pb_team) {
                        for s in gss::java_skill_names(game, p)? {
                            if s == "Pass Block" {
                                if let Some(pc) = game.field_model.player_coordinate(&p.id) {
                                    self.pb_player_id = Some(p.id.clone());
                                    self.pb_old = (pc.x, pc.y);
                                    self.pb_state = Some(gss::serialize(game)?);
                                }
                                break;
                            }
                        }
                        if self.pb_player_id.is_some() {
                            break;
                        }
                    }
                }
            }
            "PLAYER_ACTION" => {
                let acting_player_id = report.string("actingPlayerId")?;
                let action = report.enum_constant("playerAction", java_names::PLAYER_ACTION)?;
                // flushMoveTarget: MoveDecisionEngine (not ported) -- nothing pending without it.
                if action == Some("CHAINSAW") {
                    self.pending_chainsaw = true;
                    self.pending_chainsaw_state = Some(gss::serialize(game)?);
                } else {
                    self.pending_chainsaw = false;
                    self.pending_chainsaw_state = None;
                }
                // rec = buildPlayerSelectRecord(game, pa): MoveDecisionEngine (not ported).
                if action.is_some_and(is_movement_action) {
                    self.next_move_player_id = acting_player_id;
                    self.next_move_action = action;
                } else {
                    self.next_move_player_id = None;
                    self.next_move_action = None;
                }
            }
            "TURN_END" => {
                // flushMoveTarget: MoveDecisionEngine (not ported) -- nothing pending without it.
            }
            _ => {}
        }
        Ok(())
    }

    // ── POST-CHANGE PASS ──────────────────────────────────────────────────────

    pub fn post_sync(&mut self, game: &Game) {
        // FOLLOWUP_CHOICE: attacker position after the pushback.
        if self.fu_attacker_id.is_some() && self.fu_check_syncs > 0 {
            self.fu_check_syncs -= 1;
            let att = self.fu_attacker_id.as_deref().and_then(|id| game.player(id));
            if let Some(ac) = att.and_then(|a| game.field_model.player_coordinate(&a.id)) {
                let action = if (ac.x, ac.y) == self.fu_target {
                    0
                } else if (ac.x, ac.y) == self.fu_attacker {
                    1
                } else {
                    -1
                };
                if action >= 0 {
                    let rec = build_followup_record(self.fu_state.clone(), action);
                    self.write_opt(rec);
                    self.fu_attacker_id = None;
                    self.fu_state = None;
                    self.fu_check_syncs = 0;
                }
            }
            if self.fu_check_syncs <= 0 && self.fu_attacker_id.is_some() {
                self.fu_attacker_id = None;
                self.fu_state = None;
            }
        }

        // PASS target (accurate pass, no deviate).
        if self.pending_pass_state.is_some() && !self.pending_pass_deviated {
            let pass_target = game.pass_coordinate.or(self.pending_pass_coord);
            if let Some(t) = pass_target {
                let rec = build_coord_target_record("pass_target", Some(t), self.pending_pass_state.clone(), false);
                self.write_opt(rec);
            }
            self.pending_pass_state = None;
            self.pending_pass_coord = None;
        }

        // PASS_BLOCK: did the offered player move?
        if let Some(pb_id) = self.pb_player_id.clone() {
            if let Some(nc) = game.player(&pb_id).and_then(|p| game.field_model.player_coordinate(&p.id)) {
                let action = if (nc.x, nc.y) != self.pb_old { 0 } else { 1 };
                let rec = build_dialog_record_with_state("PASS_BLOCK", action, pid_param(Some(&pb_id)), self.pb_state.clone());
                self.write_opt(rec);
            }
            self.pb_player_id = None;
            self.pb_state = None;
        }

        // TOUCHBACK.
        let touchback = !game.turn_mode_null && game.turn_mode == TurnMode::Touchback;
        if touchback && !self.touchback_pending {
            match gss::serialize(game) {
                Ok(s) => {
                    self.touchback_state = Some(s);
                    self.touchback_pending = true;
                }
                Err(e) => self.record_error("TOUCHBACK_STATE", e),
            }
        } else if self.touchback_pending && !touchback {
            if let Some(bc) = game.field_model.ball_coordinate {
                if let Some(carrier) = field_player_at(game, bc) {
                    let rec = build_dialog_record_with_state("TOUCHBACK", 0, pid_param(Some(&carrier.id)), self.touchback_state.clone());
                    self.write_opt(rec);
                }
            }
            self.touchback_pending = false;
            self.touchback_state = None;
        }

        // Move target setup: MoveDecisionEngine.selectMoveTarget (not ported).
        if self.next_move_player_id.is_some() {
            self.next_move_player_id = None;
            self.next_move_action = None;
        }
    }

    /// The final `flushMoveTarget` (MDE only; nothing pending without it).
    pub fn finish(&mut self, _game: &Game) {}
}

/// `isMovementAction`: every `*_MOVE` constant and MOVE.
fn is_movement_action(action: &str) -> bool {
    action.ends_with("_MOVE") || action == "MOVE"
}

/// `reportIdToReRollAction`.
pub fn report_id_to_re_roll_action(id: &str) -> String {
    match id {
        "DODGE_ROLL" => "DODGE".into(),
        "GO_FOR_IT_ROLL" => "GO_FOR_IT".into(),
        "PASS_ROLL" => "PASS".into(),
        "CATCH_ROLL" => "CATCH".into(),
        other => other.strip_suffix("_ROLL").unwrap_or(other).to_string(),
    }
}

/// `playerStateBaseToString`.
fn player_state_base_to_string(base: i32) -> &'static str {
    match base as u32 {
        PS_BADLY_HURT => "BH",
        PS_SERIOUS_INJURY => "SI",
        PS_RIP => "RIP",
        PS_KNOCKED_OUT => "KO",
        _ => "UNKNOWN",
    }
}

/// `pidParam`.
fn pid_param(player_id: Option<&str>) -> CanonicalJson {
    let mut p = Obj::new();
    if let Some(id) = player_id {
        p.str("player_id", id);
    }
    p.build()
}

fn empty_array() -> CanonicalJson {
    CanonicalJson::Array(Vec::new())
}

/// The common head of the dialog records.
fn dialog_head(dialog_id: &str, action: i32) -> Obj {
    let mut rec = Obj::new();
    rec.str("type", "dialog");
    rec.str("dialog_id", dialog_id);
    rec.str("agent_mode", AGENT_MODE);
    rec.int("action", action);
    rec.add("scores", empty_array());
    rec
}

fn build_block_roll_properties_record(game: &Game, report: &ReplayReport) -> R<Option<CanonicalJson>> {
    let roll = report.int_array("blockRoll")?;
    let chosen = report.int("diceIndex")?;
    let nr_of_dice = report.int("nrOfDice")?;
    let defender = report.defender_id()?;
    let Some(roll) = roll else { return Ok(None) };
    if roll.is_empty() || chosen < 0 || chosen as usize >= roll.len() {
        return Ok(None);
    }
    // try { .. } catch -> BLOCK_CHOICE_TEAM: nothing below can throw.
    let mut choosing_team_id = String::new();
    if nr_of_dice < 0 {
        if let Some(d) = defender.as_deref().and_then(|id| game.player(id)) {
            let def_home = game.team_home.player(&d.id).is_some();
            choosing_team_id = if def_home { game.team_home.id.clone() } else { game.team_away.id.clone() };
        }
    } else if let Some(a) = game.acting_player.player_id.as_deref().and_then(|id| game.player(id)) {
        let att_home = game.team_home.player(&a.id).is_some();
        choosing_team_id = if att_home { game.team_home.id.clone() } else { game.team_away.id.clone() };
    }
    let mut rec = dialog_head("BLOCK_ROLL_PROPERTIES", chosen);
    let mut param = Obj::new();
    param.int("num_dice", nr_of_dice);
    param.str("choosing_team_id", &choosing_team_id);
    if let Some(d) = &defender {
        param.str("target_id", d);
    }
    rec.add("dialog_param", param.build());
    rec.add("state", gss::serialize(game)?);
    Ok(Some(rec.build()))
}

fn build_skill_use_record(game: &Game, report: &ReplayReport) -> R<Option<CanonicalJson>> {
    let player_id = report.string("playerId")?;
    let skill = report.skill_name("skill")?;
    let used = report.boolean("used")?;
    let Some(skill) = skill else { return Ok(None) };
    let mut rec = dialog_head("SKILL_USE", if used { 0 } else { 1 });
    let mut param = Obj::new();
    if let Some(p) = &player_id {
        param.str("player_id", p);
    }
    param.str("skill", skill);
    param.int("min_roll", 0);
    rec.add("dialog_param", param.build());
    rec.add("state", gss::serialize(game)?);
    Ok(Some(rec.build()))
}

/// buildReceiveChoiceRecord / buildCoinChoiceRecord: action 0 when the flag is set.
fn build_flag_dialog_record(game: &Game, dialog_id: &str, flag: bool) -> R<CanonicalJson> {
    let mut rec = dialog_head(dialog_id, if flag { 0 } else { 1 });
    rec.add("dialog_param", Obj::new().build());
    rec.add("state", gss::serialize(game)?);
    Ok(rec.build())
}

fn build_re_roll_record(
    player_id: Option<&str>,
    action_name: Option<&str>,
    min_roll: i32,
    is_fumble: bool,
    state: Option<CanonicalJson>,
    action: i32,
) -> Option<CanonicalJson> {
    let state = state?;
    let mut rec = dialog_head("RE_ROLL", action);
    let mut param = Obj::new();
    if let Some(p) = player_id {
        param.str("player_id", p);
    }
    param.str("rerolled_action", action_name.unwrap_or("UNKNOWN"));
    param.int("min_roll", min_roll);
    param.bool("is_team_reroll", true);
    param.bool("is_pro_reroll", false);
    param.bool("is_fumble", is_fumble);
    rec.add("dialog_param", param.build());
    rec.add("state", state);
    Some(rec.build())
}

fn build_followup_record(state: Option<CanonicalJson>, action: i32) -> Option<CanonicalJson> {
    let state = state?;
    let mut rec = dialog_head("FOLLOWUP_CHOICE", action);
    rec.add("dialog_param", Obj::new().build());
    rec.add("state", state);
    Some(rec.build())
}

fn build_use_apothecary_record(
    player_id: Option<&str>,
    inj_state_base: i32,
    serious_injury: Option<&str>,
    state: Option<CanonicalJson>,
    action: i32,
) -> Option<CanonicalJson> {
    let (state, player_id) = (state?, player_id?);
    let mut rec = dialog_head("USE_APOTHECARY", action);
    let mut param = Obj::new();
    param.str("player_id", player_id);
    param.str("injury_state", player_state_base_to_string(inj_state_base));
    // JsonValue.NULL.toString(): the STRING "null".
    param.str("serious_injury", serious_injury.unwrap_or("null"));
    rec.add("dialog_param", param.build());
    rec.add("state", state);
    Some(rec.build())
}

fn build_apothecary_choice_record(
    game: &Game,
    report: &ReplayReport,
    old_base: i32,
    old_si: Option<&str>,
    new_base: i32,
    new_si: Option<&str>,
) -> R<Option<CanonicalJson>> {
    let player_id = report.player_id()?;
    let player_state = report.player_state("playerState")?;
    let (Some(player_id), Some(player_state)) = (player_id, player_state) else { return Ok(None) };
    let chosen_base = player_state.base() as i32;
    // chosenSI is computed in Java but not used.
    report.serious_injury_name("seriousInjury")?;
    let action = if chosen_base == new_base {
        0
    } else if chosen_base == old_base {
        1
    } else {
        return Ok(None);
    };
    let mut rec = dialog_head("APOTHECARY_CHOICE", action);
    let mut param = Obj::new();
    param.str("player_id", &player_id);
    param.str("old_player_state", player_state_base_to_string(old_base));
    param.str("old_serious_injury", old_si.unwrap_or("null"));
    param.str("new_player_state", player_state_base_to_string(new_base));
    param.str("new_serious_injury", new_si.unwrap_or("null"));
    rec.add("dialog_param", param.build());
    rec.add("state", gss::serialize(game)?);
    Ok(Some(rec.build()))
}

fn build_piling_on_record(game: &Game, report: &ReplayReport) -> R<Option<CanonicalJson>> {
    let player_id = report.string("playerId")?;
    let used = report.boolean("used")?;
    let Some(player_id) = player_id else { return Ok(None) };
    let mut rec = dialog_head("PILING_ON", if used { 0 } else { 1 });
    rec.add("dialog_param", pid_param(Some(&player_id)));
    rec.add("state", gss::serialize(game)?);
    Ok(Some(rec.build()))
}

/// The common tail of foul/handoff/gaze target records.
fn target_record(game: &Game, kind: &str, target_id: &str, mut targets: Vec<String>) -> R<CanonicalJson> {
    let mut chosen_idx = targets.iter().position(|t| t == target_id).map_or(-1, |i| i as i32);
    if chosen_idx < 0 {
        targets.clear();
        targets.push(target_id.to_string());
        chosen_idx = 0;
    }
    let mut rec = Obj::new();
    rec.str("type", kind);
    rec.str("agent_mode", AGENT_MODE);
    rec.int("action", chosen_idx);
    rec.str("target_id", target_id);
    rec.add("candidates", CanonicalJson::Array(targets.iter().map(|t| gss::string(t)).collect()));
    rec.add("state", gss::serialize(game)?);
    Ok(rec.build())
}

fn adjacent(a: FieldCoordinate, b: FieldCoordinate) -> bool {
    (a.x - b.x).abs() <= 1 && (a.y - b.y).abs() <= 1
}

fn build_foul_target_record(game: &Game, report: &ReplayReport) -> R<Option<CanonicalJson>> {
    let Some(defender_id) = report.defender_id()? else { return Ok(None) };
    let Some(fouler) = game.acting_player.player_id.as_deref().and_then(|id| game.player(id)) else { return Ok(None) };
    let Some(fc) = game.field_model.player_coordinate(&fouler.id) else { return Ok(None) };
    let is_home = game.team_home.player(&fouler.id).is_some();
    let op_team = if is_home { &game.team_away } else { &game.team_home };
    let mut targets = Vec::new();
    for op in team_players(op_team) {
        let Some(oc) = game.field_model.player_coordinate(&op.id) else { continue };
        let base = field_player_state(game, &op.id).base();
        if base != PS_PRONE && base != PS_STUNNED {
            continue;
        }
        if adjacent(oc, fc) {
            targets.push(op.id.clone());
        }
    }
    Ok(Some(target_record(game, "foul_target", &defender_id, targets)?))
}

fn build_handoff_target_record(game: &Game, report: &ReplayReport) -> R<Option<CanonicalJson>> {
    let Some(catcher_id) = report.string("catcherId")? else { return Ok(None) };
    let ap_id = game.acting_player.player_id.clone();
    let Some(passer) = ap_id.as_deref().and_then(|id| game.player(id)) else { return Ok(None) };
    let Some(fc) = game.field_model.player_coordinate(&passer.id) else { return Ok(None) };
    let is_home = game.team_home.player(&passer.id).is_some();
    let my_team = if is_home { &game.team_home } else { &game.team_away };
    let mut targets = Vec::new();
    for mate in team_players(my_team) {
        if Some(mate.id.as_str()) == ap_id.as_deref() {
            continue;
        }
        let Some(mc) = game.field_model.player_coordinate(&mate.id) else { continue };
        if field_player_state(game, &mate.id).base() != PS_STANDING {
            continue;
        }
        if adjacent(mc, fc) {
            targets.push(mate.id.clone());
        }
    }
    Ok(Some(target_record(game, "handoff_target", &catcher_id, targets)?))
}

fn build_gaze_target_record(game: &Game, report: &ReplayReport) -> R<Option<CanonicalJson>> {
    let attacker_id = report.string("attackerId")?;
    let Some(defender_id) = report.defender_id()? else { return Ok(None) };
    let att = attacker_id.as_deref().and_then(|id| game.player(id));
    let fc = att.and_then(|a| game.field_model.player_coordinate(&a.id));
    let is_home = att.is_some_and(|a| game.team_home.player(&a.id).is_some());
    let op_team = if is_home { &game.team_away } else { &game.team_home };
    let mut targets = Vec::new();
    if let Some(fc) = fc {
        for op in team_players(op_team) {
            let Some(oc) = game.field_model.player_coordinate(&op.id) else { continue };
            if adjacent(oc, fc) {
                targets.push(op.id.clone());
            }
        }
    }
    Ok(Some(target_record(game, "gaze_target", &defender_id, targets)?))
}

/// `scatterInverse(end, dir, distance)`.
fn scatter_inverse(end: Option<FieldCoordinate>, dir: Option<&str>, distance: i32) -> Option<FieldCoordinate> {
    let end = end?;
    let Some(dir) = dir else { return Some(end) };
    if distance <= 0 {
        return Some(end);
    }
    let (dx, dy) = match dir {
        "NORTH" => (0, -1),
        "NORTHEAST" => (1, -1),
        "EAST" => (1, 0),
        "SOUTHEAST" => (1, 1),
        "SOUTH" => (0, 1),
        "SOUTHWEST" => (-1, 1),
        "WEST" => (-1, 0),
        "NORTHWEST" => (-1, -1),
        _ => (0, 0),
    };
    Some(FieldCoordinate::new(end.x - dx * distance, end.y - dy * distance))
}

fn build_pass_target_record(
    report: &ReplayReport,
    captured: Option<FieldCoordinate>,
    state: Option<CanonicalJson>,
) -> R<Option<CanonicalJson>> {
    if state.is_none() {
        return Ok(None);
    }
    let mut target = captured;
    if target.is_none() {
        let (end, dir, distance) = report.scatter()?;
        if end.is_some() {
            target = scatter_inverse(end, dir, distance);
        }
    }
    if target.is_none() {
        return Ok(None);
    }
    Ok(build_coord_target_record("pass_target", target, state, false))
}

fn build_ttm_target_record(report: &ReplayReport, state: Option<CanonicalJson>) -> R<Option<CanonicalJson>> {
    if state.is_none() {
        return Ok(None);
    }
    let (end, dir, distance) = report.scatter()?;
    let target = if end.is_some() { scatter_inverse(end, dir, distance) } else { None };
    if target.is_none() {
        return Ok(None);
    }
    Ok(build_coord_target_record("ttm_target", target, state, false))
}

fn build_kickoff_target_record(game: &Game, report: &ReplayReport) -> R<Option<CanonicalJson>> {
    let (end, dir, distance) = report.scatter()?;
    if end.is_none() {
        return Ok(None);
    }
    let target = scatter_inverse(end, dir, distance);
    if target.is_none() {
        return Ok(None);
    }
    Ok(build_coord_target_record("kickoff_target", target, Some(gss::serialize(game)?), false))
}

fn build_coord_target_record(
    kind: &str,
    target: Option<FieldCoordinate>,
    state: Option<CanonicalJson>,
    partial: bool,
) -> Option<CanonicalJson> {
    let (target, state) = (target?, state?);
    let mut rec = Obj::new();
    rec.str("type", kind);
    rec.str("agent_mode", AGENT_MODE);
    rec.int("action", target.x * 15 + target.y);
    rec.int("target_x", target.x);
    rec.int("target_y", target.y);
    if partial {
        rec.bool("partial", true);
    }
    rec.add("state", state);
    Some(rec.build())
}

fn build_chainsaw_target_record(game: &Game, report: &ReplayReport) -> R<Option<CanonicalJson>> {
    let Some(defender_id) = report.defender_id()? else { return Ok(None) };
    let mut rec = Obj::new();
    rec.str("type", "chainsaw_target");
    rec.str("agent_mode", AGENT_MODE);
    rec.str("target_id", &defender_id);
    rec.add("action", int(0));
    rec.add("state", gss::serialize(game)?);
    Ok(Some(rec.build()))
}

fn build_bribery_corruption_record(game: &Game) -> R<CanonicalJson> {
    let mut rec = dialog_head("BRIBERY_AND_CORRUPTION_RE_ROLL", 0);
    rec.add("dialog_param", Obj::new().build());
    rec.add("state", gss::serialize(game)?);
    Ok(rec.build())
}

fn build_simple_dialog_record(game: &Game, dialog_id: Option<&str>, action: i32, param: CanonicalJson) -> R<Option<CanonicalJson>> {
    let Some(dialog_id) = dialog_id else { return Ok(None) };
    Ok(build_dialog_record_with_state(dialog_id, action, param, Some(gss::serialize(game)?)))
}

fn build_dialog_record_with_state(dialog_id: &str, action: i32, param: CanonicalJson, state: Option<CanonicalJson>) -> Option<CanonicalJson> {
    let state = state?;
    let mut rec = dialog_head(dialog_id, action);
    rec.add("dialog_param", param);
    rec.add("state", state);
    Some(rec.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_roll_action_names() {
        assert_eq!(report_id_to_re_roll_action("DODGE_ROLL"), "DODGE");
        assert_eq!(report_id_to_re_roll_action("ALWAYS_HUNGRY_ROLL"), "ALWAYS_HUNGRY");
        assert_eq!(report_id_to_re_roll_action("ALL_YOU_CAN_EAT"), "ALL_YOU_CAN_EAT");
        assert_eq!(report_id_to_re_roll_action("JUMP_ROLL"), "JUMP");
    }

    #[test]
    fn scatter_inverse_walks_back_along_the_direction() {
        let end = Some(FieldCoordinate::new(10, 7));
        assert_eq!(scatter_inverse(end, Some("NORTHEAST"), 3), Some(FieldCoordinate::new(7, 10)));
        assert_eq!(scatter_inverse(end, None, 3), end);
        assert_eq!(scatter_inverse(end, Some("WEST"), 0), end);
        assert_eq!(scatter_inverse(None, Some("WEST"), 2), None);
    }

    #[test]
    fn records_print_in_java_member_order() {
        let rec = build_dialog_record_with_state("TOUCHBACK", 0, pid_param(Some("p1")), Some(Obj::new().build())).unwrap();
        assert_eq!(
            rec.to_json_string(),
            "{\"type\":\"dialog\",\"dialog_id\":\"TOUCHBACK\",\"agent_mode\":\"fumbbl_replay\",\"action\":0,\"scores\":[],\"dialog_param\":{\"player_id\":\"p1\"},\"state\":{}}"
        );
        let apo = build_use_apothecary_record(Some("p"), 6, None, Some(Obj::new().build()), 1).unwrap();
        assert!(apo.to_json_string().contains("\"injury_state\":\"BH\",\"serious_injury\":\"null\""));
        assert!(build_followup_record(None, 0).is_none());
    }
}
