use super::report_id::ReportId;

/// 1:1 translation of `IReport.java`.
///
/// `Any` is included as a supertrait (auto-satisfied by every `'static` implementor, no
/// per-type changes required) solely so tests can `downcast_ref` a boxed `dyn IReport`
/// back to its concrete type via trait-upcasting coercion.
pub trait IReport: Send + Sync + std::any::Any {
    fn get_id(&self) -> ReportId;
    fn get_name(&self) -> &str {
        self.get_id().get_name()
    }
}
