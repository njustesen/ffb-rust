/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPuntToCrowd`.
/// Sent when a player punts the ball to the crowd.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPuntToCrowd {
    /// Java: `puntToCrowd`
    pub punt_to_crowd: bool,
}

impl ClientCommandPuntToCrowd {
    pub fn new(punt_to_crowd: bool) -> Self { Self { punt_to_crowd } }
    pub fn is_punt_to_crowd(&self) -> bool { self.punt_to_crowd }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn true_stored() {
        let cmd = ClientCommandPuntToCrowd::new(true);
        assert!(cmd.is_punt_to_crowd());
    }
    #[test]
    fn default_false() {
        let cmd = ClientCommandPuntToCrowd::default();
        assert!(!cmd.punt_to_crowd);
    }

    #[test]
    fn false_stored() {
        let cmd = ClientCommandPuntToCrowd::new(false);
        assert!(!cmd.is_punt_to_crowd());
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPuntToCrowd::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandPuntToCrowd::default().clone();
    }
}
