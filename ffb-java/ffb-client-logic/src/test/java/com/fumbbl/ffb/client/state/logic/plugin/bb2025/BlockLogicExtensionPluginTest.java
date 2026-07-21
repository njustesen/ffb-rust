package com.fumbbl.ffb.client.state.logic.plugin.bb2025;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/plugin/bb2025/block_logic_extension_plugin.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// No performAvailableAction/actionContext tests exist in the Rust source for this file (the
// Rust versions are documented placeholders pending BlockLogicExtension being wired into the
// crate), so there is nothing analogous to port for those methods here either.
class BlockLogicExtensionPluginTest {

	@Test
	void availableActionsIsChompOnly() {
		BlockLogicExtensionPlugin plugin = new BlockLogicExtensionPlugin();
		Set<ClientAction> actions = plugin.availableActions();
		assertEquals(Collections.singleton(ClientAction.CHOMP), actions);
	}

	@Test
	void playerCanNotMoveWhenPinned() {
		BlockLogicExtensionPlugin plugin = new BlockLogicExtensionPlugin();
		PlayerState chomped = new PlayerState(PlayerState.STANDING).changeChomped(true);
		assertTrue(plugin.playerCanNotMove(chomped));
	}

	@Test
	void playerCanMoveWhenNotPinned() {
		BlockLogicExtensionPlugin plugin = new BlockLogicExtensionPlugin();
		PlayerState standing = new PlayerState(PlayerState.STANDING);
		assertFalse(plugin.playerCanNotMove(standing));
	}
}
