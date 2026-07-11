/// 1:1 translation of com.fumbbl.ffb.server.handler.IReceivedCommandHandler.
pub trait IReceivedCommandHandler {
    /// Java: handleCommand(ReceivedCommand) — dispatches a received command.
    fn handle_command(&self) -> bool;
}
