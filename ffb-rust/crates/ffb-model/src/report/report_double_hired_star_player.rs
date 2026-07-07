use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportDoubleHiredStarPlayer.java`.
#[derive(Debug, Clone)]
pub struct ReportDoubleHiredStarPlayer {
    /// Translated from `fStarPlayerName`.
    pub star_player_name: String,
}

impl ReportDoubleHiredStarPlayer {
    pub fn new(star_player_name: String) -> Self {
        Self { star_player_name }
    }

    pub fn get_star_player_name(&self) -> &str {
        &self.star_player_name
    }
}

impl IReport for ReportDoubleHiredStarPlayer {
    fn get_id(&self) -> ReportId {
        ReportId::DOUBLE_HIRED_STAR_PLAYER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportDoubleHiredStarPlayer {
        ReportDoubleHiredStarPlayer::new("Griff Oberwald".into())
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::DOUBLE_HIRED_STAR_PLAYER);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "doubleHiredStarPlayer");
    }

    #[test]
    fn star_player_name_getter() {
        assert_eq!(make().get_star_player_name(), "Griff Oberwald");
    }

    #[test]
    fn different_star_player() {
        let r = ReportDoubleHiredStarPlayer::new("Morg 'n' Thorg".into());
        assert_eq!(r.get_star_player_name(), "Morg 'n' Thorg");
    }

    #[test]
    fn star_player_name_matches_field() {
        let r = make();
        assert_eq!(r.get_star_player_name(), r.star_player_name.as_str());
    }
}
