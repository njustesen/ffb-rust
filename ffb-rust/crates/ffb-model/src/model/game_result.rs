use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::enums::{SendToBoxReason, SeriousInjuryKind, PlayerState, PS_BADLY_HURT, PS_SERIOUS_INJURY, PS_RIP};
use crate::model::player::{Player, PlayerId};
use crate::enums::Rules;
use crate::factory::serious_injury_factory;
use crate::model::keyword::Keyword;
use crate::model::team::Team;
use crate::util::java_json;
use serde_json::Value;

/// Per-player game result (accumulated during the game).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerResult {
    pub touchdowns: i32,
    pub completions: i32,
    pub casualties: i32,
    pub interceptions: i32,
    pub deflections: i32,
    pub landings: i32,
    pub catches_with_additional_spp: i32,
    pub completions_with_additional_spp: i32,
    pub casualties_with_additional_spp: i32,
    pub mvp: bool,
    pub spp_gained: i32,
    pub fouls: i32,
    /// Java: fBlocks — number of blocks thrown this game.
    pub blocks: i32,
    /// Set during end-game player loss check when player defects on illegal concession.
    pub defecting: bool,
    /// Java: PlayerResult.fRushing — rushing yards accumulated (ball-carrying moves).
    pub rushing: i32,
    /// Java: fPassing — passing yards accumulated.
    pub passing: i32,
    /// Java: fPlayerAwards — tally of awards (e.g. MVP nominations).
    pub player_awards: i32,
    /// Java: fTurnsPlayed — number of drive turns in which this player was active.
    pub turns_played: i32,
    /// Java: fCurrentSpps — the player's SPP total at game start (used for MVP tie-breaking).
    pub current_spps: i32,
    /// Java: fSeriousInjury — serious injury received this game (if any).
    pub serious_injury: Option<SeriousInjuryKind>,
    /// Java: fSeriousInjuryDecay — niggling decay applied this game (if any).
    pub serious_injury_decay: Option<SeriousInjuryKind>,
    /// Java: fSendToBoxReason — why this player was sent to the injury box this drive.
    pub send_to_box_reason: Option<SendToBoxReason>,
    /// Java: fSendToBoxTurn — turn on which the player was sent to the box.
    pub send_to_box_turn: i32,
    /// Java: fSendToBoxHalf — half in which the player was sent to the box.
    pub send_to_box_half: i32,
    /// Java: fSendToBoxByPlayerId — id of the player that caused the send-to-box.
    pub send_to_box_by_player_id: Option<String>,
    /// Java: fHasUsedSecretWeapon — whether this player fired a secret weapon this drive.
    pub has_used_secret_weapon: bool,
    /// Java: `PlayerResult.gainedHatred` (a `List<Keyword>`, "not synced to clients" but part of
    /// the JSON; BACKLOG §O replay state).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gained_hatred: Vec<Keyword>,
}

impl PlayerResult {
    /// Java: `PlayerResult.initFrom(IFactorySource, JsonValue)` on a `new PlayerResult(teamResult)`.
    /// Returns the result's player id (`fPlayer = team.getPlayerById(playerId)`; the caller keys
    /// the map by `getPlayer().getId()`, which NPEs for a player the team does not hold -> `Err`).
    pub fn init_from(&mut self, rules: Rules, team: &Team, json_value: &Value) -> Result<String, String> {
        let o = java_json::to_json_object(json_value)?;
        let player_id = java_json::get_string(o, "playerId")?;
        let player_id = match player_id.as_deref().and_then(|id| team.player(id)) {
            Some(player) => player.id.clone(),
            None => return Err(format!("PlayerResult: player {player_id:?} not on team {} (Java NPE)", team.id)),
        };
        self.completions = java_json::get_int(o, "completions")?;
        self.completions_with_additional_spp = java_json::get_int(o, "completionsWithAdditionalSpp")?;
        self.touchdowns = java_json::get_int(o, "touchdowns")?;
        self.interceptions = java_json::get_int(o, "interceptions")?;
        self.casualties = java_json::get_int(o, "casualties")?;
        self.casualties_with_additional_spp = java_json::get_int(o, "casualtiesWithAdditionalSpp")?;
        self.player_awards = java_json::get_int(o, "playerAwards")?;
        self.blocks = java_json::get_int(o, "blocks")?;
        self.fouls = java_json::get_int(o, "fouls")?;
        self.rushing = java_json::get_int(o, "rushing")?;
        self.passing = java_json::get_int(o, "passing")?;
        self.current_spps = java_json::get_int(o, "currentSpps")?;
        let injury = |key: &str| -> Result<Option<SeriousInjuryKind>, String> {
            Ok(java_json::get_enum_name(o, key)?
                .and_then(|name| serious_injury_factory::for_name_in(rules, name))
                .map(|i| i.to_kind()))
        };
        self.serious_injury = injury("seriousInjury")?;
        self.serious_injury_decay = injury("seriousInjuryDecay")?;
        self.send_to_box_reason = java_json::get_enum_name(o, "sendToBoxReason")?.and_then(SendToBoxReason::from_name);
        self.send_to_box_turn = java_json::get_int(o, "sendToBoxTurn")?;
        self.send_to_box_half = java_json::get_int(o, "sendToBoxHalf")?;
        self.send_to_box_by_player_id = java_json::get_string(o, "sendToBoxByPlayerId")?;
        self.turns_played = java_json::get_int(o, "turnsPlayed")?;
        self.has_used_secret_weapon = java_json::get_boolean(o, "hasUsedSecretWeapon")?;
        self.defecting = java_json::get_boolean(o, "defecting")?;
        if java_json::is_defined_in(o, "catchesWithAdditionalSpp") {
            self.catches_with_additional_spp = java_json::get_int(o, "catchesWithAdditionalSpp")?;
        }
        if java_json::is_defined_in(o, "gainedHatred") {
            // Arrays.stream(null) NPEs; Keyword.forName(null) is UNKNOWN.
            let names = java_json::get_string_array(o, "gainedHatred")?
                .ok_or("PlayerResult: gainedHatred is null (Java NPE)")?;
            self.gained_hatred
                .extend(names.iter().map(|n| n.as_deref().map_or(Keyword::UNKNOWN, Keyword::for_name)));
        }
        self.landings = java_json::get_int(o, "landings")?;
        Ok(player_id)
    }

    /// Java: `PlayerResult.toJsonValue()`; `player_id` = `getPlayerId()`.
    pub fn to_json_value(&self, rules: Rules, player_id: &str) -> Result<Value, String> {
        let injury = |kind: Option<SeriousInjuryKind>| -> Result<Value, String> {
            match kind {
                None => Ok(Value::Null),
                Some(kind) => serious_injury_factory::name_of_kind_in(rules, kind)
                    .map(Value::from)
                    .ok_or_else(|| format!("PlayerResult {player_id}: {kind:?} is not a {} injury", rules.name())),
            }
        };
        let mut o = java_json::JsonObject::new();
        o.insert("playerId".into(), Value::from(player_id));
        o.insert("completions".into(), Value::from(self.completions));
        o.insert("completionsWithAdditionalSpp".into(), Value::from(self.completions_with_additional_spp));
        o.insert("touchdowns".into(), Value::from(self.touchdowns));
        o.insert("interceptions".into(), Value::from(self.interceptions));
        o.insert("casualties".into(), Value::from(self.casualties));
        o.insert("casualtiesWithAdditionalSpp".into(), Value::from(self.casualties_with_additional_spp));
        o.insert("playerAwards".into(), Value::from(self.player_awards));
        o.insert("blocks".into(), Value::from(self.blocks));
        o.insert("fouls".into(), Value::from(self.fouls));
        o.insert("rushing".into(), Value::from(self.rushing));
        o.insert("passing".into(), Value::from(self.passing));
        o.insert("currentSpps".into(), Value::from(self.current_spps));
        o.insert("seriousInjury".into(), injury(self.serious_injury)?);
        o.insert("seriousInjuryDecay".into(), injury(self.serious_injury_decay)?);
        o.insert(
            "sendToBoxReason".into(),
            java_json::string_or_null(self.send_to_box_reason.map(SendToBoxReason::name)),
        );
        o.insert("sendToBoxTurn".into(), Value::from(self.send_to_box_turn));
        o.insert("sendToBoxHalf".into(), Value::from(self.send_to_box_half));
        o.insert("sendToBoxByPlayerId".into(), java_json::string_or_null(self.send_to_box_by_player_id.as_deref()));
        o.insert("turnsPlayed".into(), Value::from(self.turns_played));
        o.insert("hasUsedSecretWeapon".into(), Value::from(self.has_used_secret_weapon));
        o.insert("defecting".into(), Value::from(self.defecting));
        o.insert("catchesWithAdditionalSpp".into(), Value::from(self.catches_with_additional_spp));
        o.insert(
            "gainedHatred".into(),
            Value::Array(self.gained_hatred.iter().map(|k| Value::from(k.get_name())).collect()),
        );
        o.insert("landings".into(), Value::from(self.landings));
        Ok(Value::Object(o))
    }
}

/// Per-team final game result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamResult {
    pub score: i32,
    pub winnings: i32,
    pub fan_factor_modifier: i32,
    pub dedicated_fans_modifier: i32,
    pub fan_factor: i32,
    pub fame: i32,
    pub player_results: HashMap<PlayerId, PlayerResult>,
    /// Set when this team conceded the game.
    pub conceded: bool,
    /// Set when this team was considered stalling (reduces winnings by 1).
    pub stalled: bool,
    pub raised_dead: i32,
    /// Java: `TeamResult.teamValue` — the team's current TV at game start.
    pub team_value: i32,
    /// Java: `TeamResult.pettyCashFromTvDiff` — petty cash available to the underdog.
    pub petty_cash_from_tv_diff: i32,
    /// Java: `TeamResult.penaltyScore` — penalty-shootout goals scored. Default -1 in Java
    /// (unset); represented as 0 here since Rust uses 0-defaults and -1 has no semantic use.
    pub penalty_score: i32,
    /// Java: `TeamResult.fPettyCashTransferred` — amount of petty cash the team chose to spend.
    pub petty_cash_transferred: i32,
    /// Java: `TeamResult.fPettyCashUsed` — petty cash actually spent on inducements.
    pub petty_cash_used: i32,
    /// Java: `TeamResult.fSpectators` — spectators attending this game.
    pub spectators: i32,
    /// Java: `TeamResult.fSpirallingExpenses` — spiralling expenses penalty applied.
    pub spiralling_expenses: i32,
    /// Java: `TeamResult.fBadlyHurtSuffered` — badly hurt casualties count.
    pub badly_hurt_suffered: i32,
    /// Java: `TeamResult.fSeriousInjurySuffered` — serious injury casualties count.
    pub serious_injury_suffered: i32,
    /// Java: `TeamResult.fRipSuffered` — RIP casualties count.
    pub rip_suffered: i32,
    /// Java: `TeamResult.treasurySpentOnInducements` — gold spent on inducements this game.
    pub treasury_spent_on_inducements: i32,
}

impl TeamResult {
    /// Java: `TeamResult.sufferInjury(PlayerState)` — increments the appropriate injury counter.
    pub fn suffer_injury(&mut self, player_state: PlayerState) {
        match player_state.base() {
            PS_BADLY_HURT => self.badly_hurt_suffered += 1,
            PS_SERIOUS_INJURY => self.serious_injury_suffered += 1,
            PS_RIP => self.rip_suffered += 1,
            _ => {}
        }
    }

    /// Java: `GameResult.getPlayerResult(player)` — returns (creating if needed) the player's result.
    pub fn player_result_mut(&mut self, player_id: &str) -> &mut PlayerResult {
        self.player_results.entry(player_id.to_string()).or_default()
    }

    /// Java: `GameResult.getPlayerResult(player)` — read-only access to a player's result.
    pub fn player_result(&self, player_id: &str) -> Option<&PlayerResult> {
        self.player_results.get(player_id)
    }

    /// Java: `TeamResult.initFrom(IFactorySource, JsonValue)`; `team` = `getTeam()` (the game's
    /// team of the same side, which `Game.initFrom` has already decoded in place).
    pub fn init_from(&mut self, rules: Rules, team: &Team, json_value: &Value) -> Result<(), String> {
        let o = java_json::to_json_object(json_value)?;
        self.score = java_json::get_int(o, "score")?;
        self.conceded = java_json::get_boolean(o, "conceded")?;
        self.raised_dead = java_json::get_int(o, "raisedDead")?;
        self.spectators = java_json::get_int(o, "spectators")?;
        self.fame = java_json::get_int(o, "fame")?;
        self.winnings = java_json::get_int(o, "winnings")?;
        self.fan_factor_modifier = java_json::get_int(o, "fanFactorModifier")?;
        self.badly_hurt_suffered = java_json::get_int(o, "badlyHurtSuffered")?;
        self.serious_injury_suffered = java_json::get_int(o, "seriousInjurySuffered")?;
        self.rip_suffered = java_json::get_int(o, "ripSuffered")?;
        self.spiralling_expenses = java_json::get_int(o, "spirallingExpenses")?;
        self.player_results.clear();
        if let Some(player_result_array) = java_json::get_array(o, "playerResults")? {
            for element in player_result_array {
                let mut player_result = PlayerResult::default();
                let player_id = player_result.init_from(rules, team, element)?;
                // HashMap.put: a later result for the same player replaces the earlier one.
                self.player_results.insert(player_id, player_result);
            }
        }
        self.petty_cash_from_tv_diff = java_json::get_int(o, "pettyCashFromTvDiff")?;
        self.petty_cash_transferred = java_json::get_int(o, "pettyCashTransferred")?;
        self.petty_cash_used = java_json::get_int(o, "pettyCashUsed")?;
        self.team_value = java_json::get_int(o, "teamValue")?;
        self.treasury_spent_on_inducements = java_json::get_int(o, "treasuryUsedOnInducements")?;
        self.fan_factor = java_json::get_int(o, "fanFactor")?;
        self.dedicated_fans_modifier = java_json::get_int(o, "dedicatedFans")?;
        self.penalty_score = java_json::get_int(o, "penaltyScore")?;
        if java_json::is_defined_in(o, "stalling") {
            self.stalled = java_json::get_boolean(o, "stalling")?;
        }
        Ok(())
    }

    /// Java: `TeamResult.toJsonValue()`. `playerResults` = one entry per `getTeam().getPlayers()`
    /// (sorted by nr); `getPlayerResult` creates a default result for a team player that has
    /// none (written here without storing it -- the stored default would be identical).
    pub fn to_json_value(&self, rules: Rules, team: Option<&Team>) -> Result<Value, String> {
        let mut o = java_json::JsonObject::new();
        o.insert("score".into(), Value::from(self.score));
        o.insert("conceded".into(), Value::from(self.conceded));
        o.insert("raisedDead".into(), Value::from(self.raised_dead));
        o.insert("spectators".into(), Value::from(self.spectators));
        o.insert("fame".into(), Value::from(self.fame));
        o.insert("winnings".into(), Value::from(self.winnings));
        o.insert("fanFactorModifier".into(), Value::from(self.fan_factor_modifier));
        o.insert("badlyHurtSuffered".into(), Value::from(self.badly_hurt_suffered));
        o.insert("seriousInjurySuffered".into(), Value::from(self.serious_injury_suffered));
        o.insert("ripSuffered".into(), Value::from(self.rip_suffered));
        o.insert("spirallingExpenses".into(), Value::from(self.spiralling_expenses));
        if let Some(team) = team {
            let mut players: Vec<&Player> = team.players.iter().collect();
            players.sort_by_key(|p| p.nr);
            let default_result = PlayerResult::default();
            let mut player_result_array = Vec::with_capacity(players.len());
            for player in players {
                let result = self.player_results.get(&player.id).unwrap_or(&default_result);
                player_result_array.push(result.to_json_value(rules, &player.id)?);
            }
            o.insert("playerResults".into(), Value::Array(player_result_array));
        }
        o.insert("pettyCashFromTvDiff".into(), Value::from(self.petty_cash_from_tv_diff));
        o.insert("pettyCashTransferred".into(), Value::from(self.petty_cash_transferred));
        o.insert("pettyCashUsed".into(), Value::from(self.petty_cash_used));
        o.insert("teamValue".into(), Value::from(self.team_value));
        o.insert("treasuryUsedOnInducements".into(), Value::from(self.treasury_spent_on_inducements));
        o.insert("fanFactor".into(), Value::from(self.fan_factor));
        o.insert("dedicatedFans".into(), Value::from(self.dedicated_fans_modifier));
        o.insert("penaltyScore".into(), Value::from(self.penalty_score));
        o.insert("stalling".into(), Value::from(self.stalled));
        Ok(Value::Object(o))
    }
}

/// Full game result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameResult {
    pub home: TeamResult,
    pub away: TeamResult,
}

impl GameResult {
    /// Java: `GameResult.getTeamResult(team)` — returns the TeamResult for the given team id.
    pub fn team_result_mut(&mut self, is_home: bool) -> &mut TeamResult {
        if is_home { &mut self.home } else { &mut self.away }
    }

    pub fn team_result(&self, is_home: bool) -> &TeamResult {
        if is_home { &self.home } else { &self.away }
    }

    /// Java: `GameResult.initFrom(IFactorySource, JsonValue)`; the teams are the game's
    /// (`TeamResult.getTeam()`). A missing/non-object team result throws in `toJsonObject`.
    pub fn init_from(&mut self, rules: Rules, team_home: &Team, team_away: &Team, json_value: &Value) -> Result<(), String> {
        let o = java_json::to_json_object(json_value)?;
        let sub = |key: &str| o.get(key).unwrap_or(&Value::Null);
        self.home.init_from(rules, team_home, sub("teamResultHome"))?;
        self.away.init_from(rules, team_away, sub("teamResultAway"))?;
        Ok(())
    }

    /// Java: `GameResult.toJsonValue()`.
    pub fn to_json_value(&self, rules: Rules, team_home: Option<&Team>, team_away: Option<&Team>) -> Result<Value, String> {
        let mut o = java_json::JsonObject::new();
        o.insert("teamResultHome".into(), self.home.to_json_value(rules, team_home)?);
        o.insert("teamResultAway".into(), self.away.to_json_value(rules, team_away)?);
        Ok(Value::Object(o))
    }

    pub fn winner(&self) -> Option<bool> {
        match self.home.score.cmp(&self.away.score) {
            std::cmp::Ordering::Greater => Some(true),  // home wins
            std::cmp::Ordering::Less => Some(false),    // away wins
            std::cmp::Ordering::Equal => None,          // draw
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffer_injury_badly_hurt() {
        let mut tr = TeamResult::default();
        tr.suffer_injury(PlayerState::new(PS_BADLY_HURT));
        assert_eq!(tr.badly_hurt_suffered, 1);
        assert_eq!(tr.serious_injury_suffered, 0);
        assert_eq!(tr.rip_suffered, 0);
    }

    #[test]
    fn suffer_injury_serious_injury() {
        let mut tr = TeamResult::default();
        tr.suffer_injury(PlayerState::new(PS_SERIOUS_INJURY));
        assert_eq!(tr.serious_injury_suffered, 1);
    }

    #[test]
    fn suffer_injury_rip() {
        let mut tr = TeamResult::default();
        tr.suffer_injury(PlayerState::new(PS_RIP));
        assert_eq!(tr.rip_suffered, 1);
    }

    #[test]
    fn suffer_injury_stunned_is_noop() {
        use crate::enums::PS_STUNNED;
        let mut tr = TeamResult::default();
        tr.suffer_injury(PlayerState::new(PS_STUNNED));
        assert_eq!(tr.badly_hurt_suffered, 0);
        assert_eq!(tr.serious_injury_suffered, 0);
        assert_eq!(tr.rip_suffered, 0);
    }

    #[test]
    fn player_result_mut_creates_on_first_access() {
        let mut tr = TeamResult::default();
        {
            let pr = tr.player_result_mut("p1");
            pr.touchdowns = 2;
        }
        assert_eq!(tr.player_results["p1"].touchdowns, 2);
    }

    #[test]
    fn player_result_returns_none_for_unknown() {
        let tr = TeamResult::default();
        assert!(tr.player_result("nobody").is_none());
    }

    #[test]
    fn team_result_mut_selects_correct_side() {
        let mut gr = GameResult::default();
        gr.team_result_mut(true).score = 3;
        gr.team_result_mut(false).score = 1;
        assert_eq!(gr.home.score, 3);
        assert_eq!(gr.away.score, 1);
    }

    #[test]
    fn winner_logic() {
        let mut gr = GameResult::default();
        gr.home.score = 2;
        gr.away.score = 1;
        assert_eq!(gr.winner(), Some(true));

        gr.home.score = 1;
        assert_eq!(gr.winner(), None);

        gr.away.score = 2;
        assert_eq!(gr.winner(), Some(false));
    }

    #[test]
    fn serde_round_trip() {
        let gr = GameResult::default();
        let json = serde_json::to_string(&gr).unwrap();
        let back: GameResult = serde_json::from_str(&json).unwrap();
        assert_eq!(gr.home.score, back.home.score);
    }

    fn json_team(id: &str, players: &[(&str, i32)]) -> Team {
        let mut team = Team::new();
        team.id = id.to_string();
        for (pid, nr) in players {
            team.players.push(Player { id: pid.to_string(), nr: *nr, ..Default::default() });
        }
        team
    }

    fn json_player_result(player_id: &str) -> Value {
        serde_json::json!({
            "playerId": player_id, "completions": 1, "touchdowns": 2, "casualties": 3,
            "seriousInjury": "Smashed Knee (-MA)", "seriousInjuryDecay": "No Such Injury",
            "sendToBoxReason": "wrestle", "sendToBoxByPlayerId": null, "turnsPlayed": 7,
            "hasUsedSecretWeapon": false, "defecting": true,
            "gainedHatred": ["Dwarf", null, "Not A Keyword"], "landings": 4
        })
    }

    #[test]
    fn json_team_result_follows_the_java_decode() {
        let rules = Rules::Bb2025;
        // Players listed out of nr order: TeamResult.toJsonValue follows getPlayers() (by nr).
        let team = json_team("t1", &[("p2", 2), ("p1", 1)]);
        let json = serde_json::json!({
            "score": 2, "conceded": false, "penaltyScore": -1, "dedicatedFans": 1,
            "treasuryUsedOnInducements": 50000,
            "playerResults": [json_player_result("p2")]
        });
        let mut tr = TeamResult::default();
        tr.stalled = true;
        tr.init_from(rules, &team, &json).unwrap();
        // STALLING absent -> isDefinedIn false -> the old value is kept.
        assert!(tr.stalled);
        assert_eq!(tr.penalty_score, -1);
        let pr = &tr.player_results["p2"];
        // Unknown injury name -> forName null; "wrestle" -> BALL_AND_CHAIN (backward compat).
        assert!(pr.serious_injury.is_some());
        assert_eq!(pr.serious_injury_decay, None);
        assert_eq!(pr.send_to_box_reason, Some(SendToBoxReason::BallAndChain));
        // Keyword.forName: null and unknown names are UNKNOWN.
        assert_eq!(pr.gained_hatred, vec![Keyword::DWARF, Keyword::UNKNOWN, Keyword::UNKNOWN]);

        let out = tr.to_json_value(rules, Some(&team)).unwrap();
        let results = out["playerResults"].as_array().unwrap();
        // p1 has no result: getPlayerResult creates a default one; nr order p1, p2.
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["playerId"], "p1");
        assert_eq!(results[0]["seriousInjury"], Value::Null);
        assert_eq!(results[0]["gainedHatred"], serde_json::json!([]));
        assert_eq!(results[1]["playerId"], "p2");
        assert_eq!(results[1]["seriousInjury"], "Smashed Knee (-MA)");
        assert_eq!(results[1]["sendToBoxReason"], "ballAndChain");
        assert_eq!(results[1]["gainedHatred"], serde_json::json!(["Dwarf", "Unknown", "Unknown"]));
        assert_eq!(out["stalling"], true);
        assert_eq!(out["treasuryUsedOnInducements"], 50000);
        // getTeam() == null -> no playerResults key.
        assert!(tr.to_json_value(rules, None).unwrap().get("playerResults").is_none());
    }

    #[test]
    fn json_game_result_errors_where_java_throws() {
        let rules = Rules::Bb2025;
        let team = json_team("t1", &[("p1", 1)]);
        let ok = serde_json::json!({ "conceded": false, "playerResults": [json_player_result("p1")] });
        let mut gr = GameResult::default();
        gr.init_from(rules, &team, &team, &serde_json::json!({ "teamResultHome": ok, "teamResultAway": ok }))
            .unwrap();
        // UtilJson.toJsonObject(null) throws.
        assert!(GameResult::default().init_from(rules, &team, &team, &serde_json::json!({ "teamResultHome": ok })).is_err());
        // A primitive boolean from an absent Boolean: NPE.
        assert!(TeamResult::default().init_from(rules, &team, &serde_json::json!({})).is_err());
        // A result for a player the team does not hold: getPlayer().getId() NPEs.
        let stranger = serde_json::json!({ "conceded": false, "playerResults": [json_player_result("nobody")] });
        assert!(TeamResult::default().init_from(rules, &team, &stranger).is_err());
        // gainedHatred: null -> Arrays.stream(null) NPEs.
        let mut null_hatred = json_player_result("p1");
        null_hatred["gainedHatred"] = Value::Null;
        assert!(PlayerResult::default().init_from(rules, &team, &null_hatred).is_err());
    }

    #[test]
    fn draw_returns_none_winner() {
        let gr = GameResult::default();
        assert_eq!(gr.winner(), None);
    }

    #[test]
    fn team_result_accumulates_player_results() {
        let mut tr = TeamResult::default();
        let mut pr = PlayerResult::default();
        pr.touchdowns = 2;
        pr.spp_gained = 6;
        tr.player_results.insert("p1".to_string(), pr);
        assert_eq!(tr.player_results["p1"].touchdowns, 2);
        assert_eq!(tr.player_results.len(), 1);
    }
}
