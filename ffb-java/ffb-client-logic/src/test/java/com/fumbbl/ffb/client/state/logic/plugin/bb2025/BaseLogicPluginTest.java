package com.fumbbl.ffb.client.state.logic.plugin.bb2025;

import com.fumbbl.ffb.PlayerState;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/plugin/bb2025/base_logic_plugin.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
class BaseLogicPluginTest {

	@Test
	void playerCanNotMoveWhenRooted() {
		BaseLogicPlugin plugin = new BaseLogicPlugin();
		PlayerState rooted = new PlayerState(PlayerState.STANDING).changeRooted(true);
		assertTrue(plugin.playerCanNotMove(rooted));
	}

	@Test
	void playerCanMoveWhenNotPinned() {
		BaseLogicPlugin plugin = new BaseLogicPlugin();
		PlayerState standing = new PlayerState(PlayerState.STANDING);
		assertFalse(plugin.playerCanNotMove(standing));
	}
}
