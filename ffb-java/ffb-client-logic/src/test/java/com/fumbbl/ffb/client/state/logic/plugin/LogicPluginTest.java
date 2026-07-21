package com.fumbbl.ffb.client.state.logic.plugin;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/plugin/logic_plugin.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// base_logic_plugin.rs / block_logic_extension_plugin.rs / move_logic_plugin.rs (this same
// `plugin` package) have no `#[cfg(test)]` block in Rust at all -- nothing to port for those
// three files.
class LogicPluginTest {

	private static final class DummyPlugin implements LogicPlugin {
		private final Type type;

		private DummyPlugin(Type type) {
			this.type = type;
		}

		@Override
		public Type getType() {
			return type;
		}
	}

	@Test
	void typeNameMatchesJavaEnumName() {
		assertEquals("MOVE", LogicPlugin.Type.MOVE.name());
		assertEquals("BLOCK", LogicPlugin.Type.BLOCK.name());
		assertEquals("BASE", LogicPlugin.Type.BASE.name());
	}

	@Test
	void defaultGetNameDelegatesToTypeName() {
		assertEquals("MOVE", new DummyPlugin(LogicPlugin.Type.MOVE).getName());
		assertEquals("BLOCK", new DummyPlugin(LogicPlugin.Type.BLOCK).getName());
		assertEquals("BASE", new DummyPlugin(LogicPlugin.Type.BASE).getName());
	}
}
