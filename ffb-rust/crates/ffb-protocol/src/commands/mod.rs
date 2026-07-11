pub mod any_client_command;
pub mod any_server_command;
pub mod client_command;
pub mod client_command_acting_player;
pub mod client_command_add_sketch;
pub mod client_command_apothecary_choice;
pub mod client_command_argue_the_call;
pub mod client_command_blitz_move;
pub mod client_command_block;
pub mod client_command_block_choice;
pub mod client_command_block_or_re_roll_choice_for_target;
pub mod client_command_bloodlust_action;
pub mod client_command_buy_card;
pub mod client_command_buy_inducements;
pub mod client_command_clear_sketches;
pub mod client_command_close_session;
pub mod client_command_coin_choice;
pub mod client_command_concede_game;
pub mod client_command_confirm;
pub mod client_command_debug_client_state;
pub mod client_command_end_turn;
pub mod client_command_field_coordinate;
pub mod client_command_followup_choice;
pub mod client_command_foul;
pub mod client_command_gaze;
pub mod client_command_hand_over;
pub mod client_command_illegal_procedure;
pub mod client_command_interceptor_choice;
pub mod client_command_join;
pub mod client_command_join_replay;
pub mod client_command_journeymen;
pub mod client_command_keyword_selection;
pub mod client_command_kick_off_result_choice;
pub mod client_command_kick_team_mate;
pub mod client_command_kickoff;
pub mod client_command_load_automatic_player_markings;
pub mod client_command_move;
pub mod client_command_pass;
pub mod client_command_password_challenge;
pub mod client_command_petty_cash;
pub mod client_command_pick_up_choice;
pub mod client_command_pile_driver;
pub mod client_command_ping;
pub mod client_command_player_choice;
pub mod client_command_position_selection;
pub mod client_command_punt_to_crowd;
pub mod client_command_pushback;
pub mod client_command_receive_choice;
pub mod client_command_remove_sketches;
pub mod client_command_replay;
pub mod client_command_replay_status;
pub mod client_command_request_version;
pub mod client_command_select_card_to_buy;
pub mod client_command_select_weather;
pub mod client_command_set_block_target_selection;
pub mod client_command_set_marker;
pub mod client_command_set_prevent_sketching;
pub mod client_command_setup_player;
pub mod client_command_sketch_add_coordinate;
pub mod client_command_sketch_set_color;
pub mod client_command_sketch_set_label;
pub mod client_command_skill_selection;
pub mod client_command_start_game;
pub mod client_command_swoop;
pub mod client_command_synchronous_multi_block;
pub mod client_command_talk;
pub mod client_command_target_selected;
pub mod client_command_team_setup_delete;
pub mod client_command_team_setup_load;
pub mod client_command_team_setup_save;
pub mod client_command_throw_keg;
pub mod client_command_throw_team_mate;
pub mod client_command_touchback;
pub mod client_command_transfer_replay_control;
pub mod client_command_unset_block_target_selection;
pub mod client_command_update_player_markings;
pub mod client_command_use_apothecaries;
pub mod client_command_use_apothecary;
pub mod client_command_use_brawler;
pub mod client_command_use_chainsaw;
pub mod client_command_use_consummate_re_roll_for_block;
pub mod client_command_use_fumblerooskie;
pub mod client_command_use_hatred;
pub mod client_command_use_igors;
pub mod client_command_use_inducement;
pub mod client_command_use_multi_block_dice_re_roll;
pub mod client_command_use_pro_re_roll_for_block;
pub mod client_command_use_re_roll;
pub mod client_command_use_re_roll_for_target;
pub mod client_command_use_single_block_die_re_roll;
pub mod client_command_use_skill;
pub mod client_command_use_team_mates_wisdom;
pub mod client_command_user_settings;
pub mod client_command_wizard_spell;
pub mod client_sketch_command;
pub mod i_command_with_acting_player;
pub mod server_command;
pub mod server_command_add_player;
pub mod server_command_add_sketches;
pub mod server_command_admin_message;
pub mod server_command_automatic_player_markings;
pub mod server_command_clear_sketches;
pub mod server_command_game_list;
pub mod server_command_game_state;
pub mod server_command_game_time;
pub mod server_command_join;
pub mod server_command_leave;
pub mod server_command_model_sync;
pub mod server_command_password_challenge;
pub mod server_command_pong;
pub mod server_command_remove_player;
pub mod server_command_remove_sketches;
pub mod server_command_replay;
pub mod server_command_replay_control;
pub mod server_command_replay_status;
pub mod server_command_set_prevent_sketching;
pub mod server_command_sketch_add_coordinate;
pub mod server_command_sketch_set_color;
pub mod server_command_sketch_set_label;
pub mod server_command_sound;
pub mod server_command_status;
pub mod server_command_talk;
pub mod server_command_team_list;
pub mod server_command_team_setup_list;
pub mod server_command_unzap_player;
pub mod server_command_update_local_player_markers;
pub mod server_command_user_settings;
pub mod server_command_version;
pub mod server_command_zap_player;
pub mod util_net_command;

use thiserror::Error;
use crate::client_commands::ClientCommand;
use crate::server_commands::ServerCommand;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown command id: {0}")]
    UnknownCommand(String),
}

/// Parse a raw JSON payload from the server into a `ServerCommand`.
pub fn parse_server_command(json: &str) -> Result<ServerCommand, ProtocolError> {
    Ok(serde_json::from_str(json)?)
}

/// Serialize a `ClientCommand` to JSON for sending to the server.
pub fn serialize_client_command(cmd: &ClientCommand) -> Result<String, ProtocolError> {
    Ok(serde_json::to_string(cmd)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_commands::{ClientCommand, ClientEndTurn, ClientBlock};
    use crate::server_commands::{ServerCommand, ServerPong};

    #[test]
    fn serialize_client_end_turn() {
        let cmd = ClientCommand::ClientEndTurn(ClientEndTurn);
        let json = serialize_client_command(&cmd).unwrap();
        assert!(json.contains("clientEndTurn"));
    }

    #[test]
    fn serialize_then_parse_server_pong() {
        let cmd = ServerCommand::ServerPong(ServerPong { timestamp: 9999 });
        let json = serde_json::to_string(&cmd).unwrap();
        let back = parse_server_command(&json).unwrap();
        assert!(matches!(back, ServerCommand::ServerPong(ServerPong { timestamp: 9999 })));
    }

    #[test]
    fn parse_server_command_returns_error_on_bad_json() {
        let result = parse_server_command("{not valid json}");
        assert!(result.is_err(), "invalid JSON must return Err");
    }

    #[test]
    fn serialize_client_block() {
        let cmd = ClientCommand::ClientBlock(ClientBlock { defender_id: "p7".into() });
        let json = serialize_client_command(&cmd).unwrap();
        assert!(json.contains("clientBlock"), "must contain command tag");
        assert!(json.contains("p7"), "must contain defender_id");
    }

    #[test]
    fn parse_server_command_returns_error_on_unknown_type() {
        let result = parse_server_command(r#"{"netCommandId":"unknownXyz"}"#);
        assert!(result.is_err(), "unknown command type must return Err");
    }
}
