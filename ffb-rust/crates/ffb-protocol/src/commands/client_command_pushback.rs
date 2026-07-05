use ffb_model::model::Pushback;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandPushback`.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPushback {
    pub pushback: Option<Pushback>,
}

impl ClientCommandPushback {
    pub fn new() -> Self { Self::default() }

    pub fn with_pushback(pushback: Pushback) -> Self {
        Self { pushback: Some(pushback) }
    }

    pub fn get_pushback(&self) -> Option<&Pushback> { self.pushback.as_ref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn pushback_stored() {
        let p = Pushback::new("p1", FieldCoordinate::new(3, 5));
        let cmd = ClientCommandPushback::with_pushback(p);
        assert_eq!(cmd.get_pushback().and_then(|p| p.get_player_id()), Some("p1"));
    }

    #[test]
    fn default_none() {
        let cmd = ClientCommandPushback::new();
        assert!(cmd.pushback.is_none());
    }
}
