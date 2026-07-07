use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandKickoff`.
/// Sent by the kicking team to place the ball for kickoff.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandKickoff {
    /// Java: `fBallCoordinate`
    pub ball_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandKickoff {
    pub fn new(ball_coordinate: FieldCoordinate) -> Self {
        Self { ball_coordinate: Some(ball_coordinate) }
    }

    /// Java: `getBallCoordinate()`
    pub fn get_ball_coordinate(&self) -> Option<FieldCoordinate> {
        self.ball_coordinate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinate_stored() {
        let coord = FieldCoordinate::new(7, 3);
        let cmd = ClientCommandKickoff::new(coord);
        assert_eq!(cmd.get_ball_coordinate(), Some(coord));
    }

    #[test]
    fn default_has_no_coordinate() {
        let cmd = ClientCommandKickoff::default();
        assert!(cmd.ball_coordinate.is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandKickoff::default()).is_empty());
    }

}
