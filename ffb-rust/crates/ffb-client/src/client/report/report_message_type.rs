use ffb_model::report::report_id::ReportId;

/// 1:1 translation of the `@ReportMessageType` annotation. Java attaches this via
/// reflection (`getClass().getAnnotation(ReportMessageType.class).value()`); Rust has no
/// annotations, so each concrete `*Message` struct instead implements `report_id()`
/// directly (see `ReportMessage` trait in `report_message_base.rs`).
pub struct ReportMessageType;

impl ReportMessageType {
    pub fn value_default() -> ReportId {
        ReportId::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_value_is_none() {
        assert_eq!(ReportMessageType::value_default(), ReportId::NONE);
    }
}
