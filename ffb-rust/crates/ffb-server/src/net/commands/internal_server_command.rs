/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommand.
/// Java abstract base class for all internal server commands.
pub trait InternalServerCommand {
    fn get_id(&self) -> &'static str;
    fn get_game_id(&self) -> i64;
    fn is_internal(&self) -> bool {
        true
    }
}
