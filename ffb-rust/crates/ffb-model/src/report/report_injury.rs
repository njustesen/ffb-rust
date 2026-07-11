use crate::injury::context::injury_context::InjuryContext;
use crate::report::i_report::IReport;
use crate::report::skip_injury_parts::SkipInjuryParts;

/// 1:1 translation of `ReportInjury.java` — an `IReport` that can be initialized
/// from an `InjuryContext` and a `SkipInjuryParts` value.
pub trait ReportInjury: IReport {
    fn init(&mut self, injury_context: &InjuryContext, skip: SkipInjuryParts) -> &mut Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::report_id::ReportId;

    struct DummyReportInjury {
        skip: SkipInjuryParts,
        defender_id: Option<String>,
    }

    impl IReport for DummyReportInjury {
        fn get_id(&self) -> ReportId {
            ReportId::INJURY
        }
    }

    impl ReportInjury for DummyReportInjury {
        fn init(&mut self, injury_context: &InjuryContext, skip: SkipInjuryParts) -> &mut Self {
            self.skip = skip;
            self.defender_id = injury_context.defender_id.clone();
            self
        }
    }

    #[test]
    fn init_copies_context() {
        let mut report = DummyReportInjury { skip: SkipInjuryParts::None, defender_id: None };
        let mut ctx = InjuryContext::default();
        ctx.defender_id = Some("p1".into());
        report.init(&ctx, SkipInjuryParts::ArmourAndCas);
        assert_eq!(report.defender_id, Some("p1".to_string()));
        assert_eq!(report.skip, SkipInjuryParts::ArmourAndCas);
    }

    #[test]
    fn get_id_is_injury() {
        let report = DummyReportInjury { skip: SkipInjuryParts::None, defender_id: None };
        assert_eq!(report.get_id(), ReportId::INJURY);
    }
}
