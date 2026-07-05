use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportBiteSpectator.java`.
#[derive(Debug, Clone)]
pub struct ReportBiteSpectator {
    pub player_id: String,
}

impl ReportBiteSpectator {
    pub fn new(player_id: String) -> Self {
        Self { player_id }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
}

impl IReport for ReportBiteSpectator {
    fn get_id(&self) -> ReportId { ReportId::BITE_SPECTATOR }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportBiteSpectator {
        ReportBiteSpectator::new("p1".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::BITE_SPECTATOR);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "biteSpectator");
    }

    #[test]
    fn get_player_id() {
        assert_eq!(make().get_player_id(), "p1");
    }
}
