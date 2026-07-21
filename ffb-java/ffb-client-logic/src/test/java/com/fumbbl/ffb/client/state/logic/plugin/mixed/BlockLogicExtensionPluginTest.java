package com.fumbbl.ffb.client.state.logic.plugin.mixed;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/plugin/mixed/block_logic_extension_plugin.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
class BlockLogicExtensionPluginTest {

	@Test
	void availableActionsIsThenIStartedBlastinOnly() {
		BlockLogicExtensionPlugin plugin = new BlockLogicExtensionPlugin();
		Set<ClientAction> actions = plugin.availableActions();
		assertEquals(Collections.singleton(ClientAction.THEN_I_STARTED_BLASTIN), actions);
	}

	@Test
	void playerCanNotMoveWhenRooted() {
		BlockLogicExtensionPlugin plugin = new BlockLogicExtensionPlugin();
		PlayerState rooted = new PlayerState(PlayerState.STANDING).changeRooted(true);
		assertTrue(plugin.playerCanNotMove(rooted));
	}

	@Test
	void playerCanMoveWhenNotRooted() {
		BlockLogicExtensionPlugin plugin = new BlockLogicExtensionPlugin();
		PlayerState standing = new PlayerState(PlayerState.STANDING);
		assertFalse(plugin.playerCanNotMove(standing));
	}
}
