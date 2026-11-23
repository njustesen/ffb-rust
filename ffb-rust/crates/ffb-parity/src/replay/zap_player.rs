//! `serverZapPlayer` / `serverUnzapPlayer` in a replay -- 1:1 ports of the harness's
//! `FumbblReplayParser.handleZapPlayer` / `handleUnzapPlayer` (= `ClientCommandHandlerZapPlayer` /
//! `ClientCommandHandlerUnzapPlayer`), with `ServerCommandZapPlayer` / `ServerCommandUnzapPlayer
//! .initFrom` as the decoder (both: `teamId`, `playerId`).
//!
//! Representation: the Rust game keeps a zapped player as the engine does (`Game.add_zapped_player`:
//! the team holds a frog-stat copy, `Game.zapped_players` the original); `Team.to_json_value_with`
//! writes the Java `ZappedPlayer` JSON and the model-change processor sends a zapped player's
//! mutators to the original, as Java's delegation does.

use ffb_model::enums::Rules;
use ffb_model::model::game::Game;
use ffb_model::model::zapped_player::ZappedPlayer;
use ffb_model::model::zapped_position::ZappedPosition;
use ffb_model::util::java_json;
use serde_json::Value;

/// `teamId` + `playerId` of a zap / unzap command.
pub struct ZapCommand {
    pub team_id: Option<String>,
    pub player_id: Option<String>,
}

impl ZapCommand {
    pub fn init_from(json: &Value) -> Result<Self, String> {
        let o = java_json::to_json_object(json)?;
        Ok(ZapCommand { team_id: java_json::get_string(o, "teamId")?, player_id: java_json::get_string(o, "playerId")? })
    }
}

/// Java `Game.getTeamById(id)`: home when the id matches, then away when it matches (away wins a
/// tie); null id or no match -> null. Returns `Some(home)`.
fn team_side(game: &Game, team_id: Option<&str>) -> Option<bool> {
    let id = team_id?;
    let mut side = None;
    if id == game.team_home.id {
        side = Some(true);
    }
    if id == game.team_away.id {
        side = Some(false);
    }
    side
}

fn is_zapped(game: &Game, player_id: &str) -> bool {
    game.zapped_players.iter().any(|z| z.original_player.id == player_id)
}

/// `handleZapPlayer`: a RosterPlayer of the team becomes a `ZappedPlayer` (`init`: original +
/// `new ZappedPosition(original.getPosition(), rules)`), `team.addPlayer(zapped)`; `sendPosition`
/// only notifies.
pub fn handle_zap_player(game: &mut Game, command: &ZapCommand) -> Result<(), String> {
    let home = team_side(game, command.team_id.as_deref()).ok_or("serverZapPlayer: getTeamById is null (NPE)")?;
    let Some(player_id) = command.player_id.as_deref() else { return Ok(()) };
    let team = if home { &game.team_home } else { &game.team_away };
    let Some(player) = team.player(player_id) else { return Ok(()) };
    if is_zapped(game, player_id) {
        return Ok(()); // already a ZappedPlayer: not instanceof RosterPlayer
    }
    let position_name = team
        .roster
        .as_ref()
        .and_then(|r| r.positions.iter().find(|p| p.id == player.position_id))
        .map(|p| p.name.clone())
        .unwrap_or_default();
    // GameMechanic.zappedPlayerStats(): bb2016 MA5 ST1 AG4 PA0 AV4, bb2020/bb2025 MA5 ST1 AG2 PA0 AV5.
    let position = match game.rules {
        Rules::Bb2016 => ZappedPosition::new_bb2016(player.position_id.clone(), position_name),
        _ => ZappedPosition::new_bb2020(player.position_id.clone(), position_name),
    };
    let original = player.clone();
    game.add_zapped_player(ZappedPlayer::new(original, position));
    Ok(())
}

/// `handleUnzapPlayer`: a `ZappedPlayer` of the team is replaced by its original
/// (`team.addPlayer(originalPlayer)`); `sendPosition` only notifies.
pub fn handle_unzap_player(game: &mut Game, command: &ZapCommand) -> Result<(), String> {
    let home = team_side(game, command.team_id.as_deref()).ok_or("serverUnzapPlayer: getTeamById is null (NPE)")?;
    let Some(player_id) = command.player_id.as_deref() else { return Ok(()) };
    let team = if home { &game.team_home } else { &game.team_away };
    if !team.has_player(player_id) || !is_zapped(game, player_id) {
        return Ok(());
    }
    let index = game.zapped_players.iter().position(|z| z.original_player.id == player_id).ok_or("unzap: vanished")?;
    let zapped = game.zapped_players.remove(index);
    let team = if home { &mut game.team_home } else { &mut game.team_away };
    let live = team.player_mut(player_id).ok_or("unzap: player vanished")?;
    *live = zapped.original_player;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::player::Player;
    use ffb_model::model::team::Team;

    fn game() -> Game {
        let mut game = Game::new(Team::new(), Team::new(), Rules::Bb2025);
        game.team_home.id = "home".into();
        game.team_away.id = "away".into();
        game.team_home.players.push(Player { id: "h1".into(), nr: 1, strength: 4, ..Default::default() });
        game
    }

    fn cmd(team: &str, player: &str) -> ZapCommand {
        ZapCommand { team_id: Some(team.into()), player_id: Some(player.into()) }
    }

    /// Expectations read off `ClientCommandHandlerZapPlayer` / `Unzap` + `ZappedPlayer.toJsonValue`.
    #[test]
    fn zap_writes_zapped_player_json_and_unzap_restores() {
        let mut game = game();
        let before = game.team_home.to_json_value_with(Rules::Bb2025, &game.zapped_players).unwrap();
        handle_zap_player(&mut game, &cmd("away", "h1")).unwrap();
        assert!(game.zapped_players.is_empty(), "not a player of the away team: no-op");
        handle_zap_player(&mut game, &cmd("home", "h1")).unwrap();
        assert_eq!(game.zapped_players.len(), 1);
        assert_eq!(game.team_home.player("h1").unwrap().strength, 1, "the team holds the frog");
        let json = game.team_home.to_json_value_with(Rules::Bb2025, &game.zapped_players).unwrap();
        let entry = &json["playerArray"][0];
        assert_eq!(entry["playerKind"], "zappedPlayer");
        assert_eq!(entry["player"], before["playerArray"][0], "the original player's JSON");
        handle_zap_player(&mut game, &cmd("home", "h1")).unwrap();
        assert_eq!(game.zapped_players.len(), 1, "a ZappedPlayer is not zapped again");
        handle_unzap_player(&mut game, &cmd("home", "h1")).unwrap();
        assert!(game.zapped_players.is_empty());
        assert_eq!(game.team_home.player("h1").unwrap().strength, 4);
    }
}
