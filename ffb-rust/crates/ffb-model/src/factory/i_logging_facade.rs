/// 1:1 translation of com.fumbbl.ffb.factory.ILoggingFacade.
pub trait ILoggingFacade {
    fn log_debug(&self, msg: &str);
    fn log_info(&self, msg: &str);
    fn log_warn(&self, msg: &str);
    fn log_error(&self, msg: &str);
}
