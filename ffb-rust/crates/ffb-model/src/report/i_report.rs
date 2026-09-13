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
    /// Java: `IReport extends IJsonSerializable` — every report serialises itself with the same
    /// camelCase keys the Java client reads. Rust reports carry that as an inherent
    /// `to_json_value`; this trait hook exposes it through `dyn IReport` so the parity harness can
    /// write the whole report stream of a game (`seed_N_rust_reports.jsonl`) and compare it with
    /// the Java side's. `None` means the report has no JSON form yet — the harness then writes
    /// its `reportId` alone.
    fn to_json(&self) -> Option<serde_json::Value> { None }
}
