package com.fumbbl.ffb.client.state.logic.plugin.bb2025;

import com.fumbbl.ffb.client.state.logic.ClientAction;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/plugin/bb2025/move_logic_plugin.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reason):
// - perform_available_action_does_not_panic_for_move_and_incorporeal: the real Java
//   performAvailableAction takes a live MoveLogicModule and, for the MOVE/PUNT branch, calls
//   `actingPlayer.getPlayer()` to send the acting-player command -- ActingPlayer.getPlayer()
//   resolves via its owning Game, which would be null for a bare `new ActingPlayer(null)`,
//   risking an NPE rather than exercising real behavior. The Rust version this mirrors is
//   itself a documented no-op placeholder (MoveLogicModule isn't wired into the Rust crate
//   yet), so there is no faithful, safely-mockable equivalent to port here.
class MoveLogicPluginTest {

	@Test
	void availableActionsIsIncorporealOnly() {
		MoveLogicPlugin plugin = new MoveLogicPlugin();
		Set<ClientAction> actions = plugin.availableActions();
		assertEquals(Collections.singleton(ClientAction.INCORPOREAL), actions);
	}
}
