use ffb_model::enums::NetCommandId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.UtilNetCommand`.
/// Utility helpers for validating command identity during deserialization.
pub struct UtilNetCommand;

impl UtilNetCommand {
    /// Asserts that `received_id` matches `expected_id`.
    /// Java: `validateCommandId(NetCommand, NetCommandId)`.
    pub fn validate_command_id(
        expected_id: NetCommandId,
        received_id: Option<NetCommandId>,
    ) -> Result<(), String> {
        match received_id {
            None => Err("netCommandId must not be null/missing".to_string()),
            Some(id) if id != expected_id => Err(format!(
                "Wrong netCommand id. Expected {:?} received {:?}",
                expected_id, id
            )),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_ids_ok() {
        assert!(UtilNetCommand::validate_command_id(
            NetCommandId::ClientJoin,
            Some(NetCommandId::ClientJoin)
        )
        .is_ok());
    }

    #[test]
    fn mismatched_ids_err() {
        let result = UtilNetCommand::validate_command_id(
            NetCommandId::ClientJoin,
            Some(NetCommandId::ClientTalk),
        );
        assert!(result.is_err());
    }

    #[test]
    fn none_received_id_err() {
        let result = UtilNetCommand::validate_command_id(NetCommandId::ClientJoin, None);
        assert!(result.is_err());
    }
}
