use ffb_model::enums::NetCommandId;
use ffb_model::model::factory_type::FactoryContext;

/// 1:1 translation of `com.fumbbl.ffb.net.NetCommand`.
/// Abstract base for every command that flows over the WebSocket. Java models
/// `getId()`/`getContext()` as abstract methods overridden by each concrete
/// command class; Rust models the same shape as a trait each command struct
/// implements.
pub trait NetCommand {
    fn get_id(&self) -> NetCommandId;

    /// Java: `getContext()` — most commands are looked up against the
    /// per-game factory source; a handful (ClientCommand base, and specific
    /// ServerCommand subclasses like `ServerCommandGameTime`) use the
    /// application-wide source instead and override this.
    fn get_context(&self) -> FactoryContext {
        FactoryContext::GAME
    }

    /// Java: `isInternal()` — defaults to false, never overridden in-tree.
    fn is_internal(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;
    impl NetCommand for Dummy {
        fn get_id(&self) -> NetCommandId {
            NetCommandId::ClientJoin
        }
    }

    #[test]
    fn default_context_is_game() {
        assert_eq!(Dummy.get_context(), FactoryContext::GAME);
    }

    #[test]
    fn default_is_internal_false() {
        assert!(!Dummy.is_internal());
    }

    #[test]
    fn get_id_returns_overridden_value() {
        assert_eq!(Dummy.get_id(), NetCommandId::ClientJoin);
    }

    struct AppScoped;
    impl NetCommand for AppScoped {
        fn get_id(&self) -> NetCommandId {
            NetCommandId::ServerGameTime
        }
        fn get_context(&self) -> FactoryContext {
            FactoryContext::APPLICATION
        }
    }

    #[test]
    fn context_can_be_overridden() {
        assert_eq!(AppScoped.get_context(), FactoryContext::APPLICATION);
    }
}
