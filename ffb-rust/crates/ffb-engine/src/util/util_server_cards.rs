/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerCards.
///
/// Static utility methods for playing and deactivating inducement cards:
/// - findAllowedPlayersForCard: filter players eligible for a card's target
/// - activateCard: play animation, add reports, activate in InducementSet, run CardHandler
/// - deactivateCard: deactivate in InducementSet, remove from field model, run CardHandler
///
/// DEFERRED: all methods require GameState/IStep and CardHandlerFactory which are not yet ported.
pub struct UtilServerCards;

impl UtilServerCards {
    pub fn new() -> Self { Self }
}

impl Default for UtilServerCards {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_can_be_created() {
        let _ = UtilServerCards::new();
    }

    #[test]
    fn default_creates_instance() {
        let _ = UtilServerCards::default();
    }
}
