use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandTouchback`.
/// Sent when a touchback occurs — the receiving team places the ball.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandTouchback {
    /// Java: `fBallCoordinate`
    pub ball_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandTouchback {
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
        let coord = FieldCoordinate::new(12, 8);
        let cmd = ClientCommandTouchback::new(coord);
        assert_eq!(cmd.get_ball_coordinate(), Some(coord));
    }

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandTouchback::default();
        assert!(cmd.ball_coordinate.is_none());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandTouchback::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandTouchback::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandTouchback::default());
        assert!(s.contains("ClientCommandTouchback"));
    }
}
