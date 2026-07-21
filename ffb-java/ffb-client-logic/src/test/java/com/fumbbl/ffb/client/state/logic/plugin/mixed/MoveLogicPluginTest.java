package com.fumbbl.ffb.client.state.logic.plugin.mixed;

import com.fumbbl.ffb.client.state.logic.ClientAction;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/plugin/mixed/move_logic_plugin.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
class MoveLogicPluginTest {

	@Test
	void availableActionsIsThenIStartedBlastinOnly() {
		MoveLogicPlugin plugin = new MoveLogicPlugin();
		Set<ClientAction> actions = plugin.availableActions();
		assertEquals(Collections.singleton(ClientAction.THEN_I_STARTED_BLASTIN), actions);
	}
}
