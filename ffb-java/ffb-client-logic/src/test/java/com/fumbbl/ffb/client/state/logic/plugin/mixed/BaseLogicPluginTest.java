package com.fumbbl.ffb.client.state.logic.plugin.mixed;

import com.fumbbl.ffb.PlayerState;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/plugin/mixed/base_logic_plugin.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
class BaseLogicPluginTest {

	@Test
	void playerCanNotMoveWhenRooted() {
		BaseLogicPlugin plugin = new BaseLogicPlugin();
		PlayerState rooted = new PlayerState(PlayerState.STANDING).changeRooted(true);
		assertTrue(plugin.playerCanNotMove(rooted));
	}

	@Test
	void playerCanMoveWhenChompedButNotRooted() {
		// mixed edition checks isRooted() only (unlike bb2025's isPinned()), so a chomped (but
		// not rooted) player state should NOT report playerCanNotMove.
		BaseLogicPlugin plugin = new BaseLogicPlugin();
		PlayerState chomped = new PlayerState(PlayerState.STANDING).changeChomped(true);
		assertFalse(plugin.playerCanNotMove(chomped));
	}
}
