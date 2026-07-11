//! 1:1 translation of `com.fumbbl.ffb.client.handler` (the *incoming*
//! `ServerCommand*` handlers, confusingly named `ClientCommandHandler*` in
//! Java — see module docs on each file). Only the modules translated so far
//! are declared here; the remaining PascalCase stub files
//! (`ClientCommandHandler*.rs`) in this directory are placeholders for
//! future batches — see `TRANSLATION_TRACKER.md`.

pub mod abstract_client_command_handler_sketch;
pub mod client_command_handler;
pub mod client_command_handler_add_player;
pub mod client_command_handler_add_sketches;
pub mod client_command_handler_admin_message;
pub mod client_command_handler_clear_sketches;
pub mod client_command_handler_factory;
pub mod client_command_handler_game_state;
pub mod client_command_handler_game_time;
pub mod client_command_handler_join;
pub mod client_command_handler_leave;
pub mod client_command_handler_mode;
pub mod client_command_handler_model_sync;
pub mod client_command_handler_remove_player;
pub mod client_command_handler_remove_sketches;
pub mod client_command_handler_set_prevent_sketching;
pub mod client_command_handler_sketch_add_coordinate;
pub mod client_command_handler_sketch_set_color;
pub mod client_command_handler_sketch_set_label;
pub mod client_command_handler_socket_closed;
pub mod client_command_handler_sound;
pub mod client_command_handler_talk;
pub mod client_command_handler_unzap_player;
pub mod client_command_handler_update_local_player_markers;
pub mod client_command_handler_user_settings;
pub mod client_command_handler_zap_player;
pub mod sub_handler_game_state_marking;
