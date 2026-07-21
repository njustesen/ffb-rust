package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.model.ActingPlayer;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/abstract_block_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - get_id is not exercised here as a standalone test in the Rust source (it has no
//   #[test] of its own; only is_suffering_blood_lust is unit-tested there), so there is
//   nothing to port for getId()/endTurn() beyond what is already covered by concrete
//   subclass tests.
class AbstractBlockLogicModuleTest {

	@Test
	void isSufferingBloodLustReadsFlag() {
		ActingPlayer actingPlayer = new ActingPlayer(null);

		assertFalse(new Probe().isSufferingBloodLust(actingPlayer));

		actingPlayer.setSufferingBloodLust(true);

		assertTrue(new Probe().isSufferingBloodLust(actingPlayer));
	}

	// AbstractBlockLogicModule is abstract; a minimal concrete subclass is needed to call
	// its instance method isSufferingBloodLust(ActingPlayer) without constructing a real
	// FantasyFootballClient/perform()/endTurn() dependency graph.
	private static final class Probe extends AbstractBlockLogicModule {
		Probe() {
			super(null);
		}

		@Override
		protected void performAvailableAction(com.fumbbl.ffb.model.Player<?> player,
				com.fumbbl.ffb.client.state.logic.ClientAction action) {
		}

		@Override
		protected com.fumbbl.ffb.client.state.logic.interaction.ActionContext actionContext(
				com.fumbbl.ffb.model.ActingPlayer actingPlayer) {
			return null;
		}

		@Override
		public com.fumbbl.ffb.ClientStateId getId() {
			return null;
		}

		@Override
		public java.util.Set<com.fumbbl.ffb.client.state.logic.ClientAction> availableActions() {
			return java.util.Collections.emptySet();
		}
	}
}
