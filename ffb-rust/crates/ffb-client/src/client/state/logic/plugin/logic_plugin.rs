//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin`.
//!
//! Java's `LogicPlugin` is an interface extending `INamedObject`, with a nested
//! `Type` enum (`MOVE`, `BLOCK`, `BASE`). The nested enum is translated as a
//! top-level `LogicPluginType` here since Rust has no nested-type equivalent.

/// java: `LogicPlugin.Type`
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicPluginType {
    MOVE,
    BLOCK,
    BASE,
}

impl LogicPluginType {
    /// java: `Type.name()` (implicit `Enum.name()`)
    pub fn name(self) -> &'static str {
        match self {
            LogicPluginType::MOVE => "MOVE",
            LogicPluginType::BLOCK => "BLOCK",
            LogicPluginType::BASE => "BASE",
        }
    }
}

/// 1:1 translation of the `LogicPlugin` interface (`extends INamedObject`).
pub trait LogicPlugin {
    fn get_type(&self) -> LogicPluginType;

    /// java: `default String getName() { return getType().name(); }`
    fn get_name(&self) -> String {
        self.get_type().name().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyPlugin(LogicPluginType);
    impl LogicPlugin for DummyPlugin {
        fn get_type(&self) -> LogicPluginType {
            self.0
        }
    }

    #[test]
    fn type_name_matches_java_enum_name() {
        assert_eq!(LogicPluginType::MOVE.name(), "MOVE");
        assert_eq!(LogicPluginType::BLOCK.name(), "BLOCK");
        assert_eq!(LogicPluginType::BASE.name(), "BASE");
    }

    #[test]
    fn default_get_name_delegates_to_type_name() {
        assert_eq!(DummyPlugin(LogicPluginType::MOVE).get_name(), "MOVE");
        assert_eq!(DummyPlugin(LogicPluginType::BLOCK).get_name(), "BLOCK");
        assert_eq!(DummyPlugin(LogicPluginType::BASE).get_name(), "BASE");
    }
}
