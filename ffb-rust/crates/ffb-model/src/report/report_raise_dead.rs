use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportRaiseDead.java`.
#[derive(Debug, Clone)]
pub struct ReportRaiseDead {
    pub player_id: String,
    pub position: Option<String>,
    pub nurgles_rot: bool,
}

impl ReportRaiseDead {
    pub fn new(player_id: String, position: Option<String>, nurgles_rot: bool) -> Self {
        Self { player_id, position, nurgles_rot }
    }

    pub fn get_player_id(&self) -> &str { &self.player_id }
    pub fn get_position(&self) -> Option<&str> { self.position.as_deref() }
    pub fn is_nurgles_rot(&self) -> bool { self.nurgles_rot }
}

impl IReport for ReportRaiseDead {
    fn get_id(&self) -> ReportId { ReportId::RAISE_DEAD }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportRaiseDead {
        ReportRaiseDead::new("p1".into(), Some("Zombie".into()), true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::RAISE_DEAD);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "raiseDead");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_player_id(), "p1");
        assert_eq!(r.get_position(), Some("Zombie"));
        assert!(r.is_nurgles_rot());
    }
}
