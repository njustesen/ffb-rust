package com.fumbbl.ffb.server.injury.modification;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.injury.CrowdPush;
import com.fumbbl.ffb.injury.Stab;
import com.fumbbl.ffb.injury.context.ModifiedInjuryContext;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/modification/modification_params.rs tests.
 * Rust splits Java's GameState into game + rng and stores injury_type_name (a class simple name);
 * Java stores the GameState + ModifiedInjuryContext + InjuryType object. The Rust injury_type_name
 * strings map to real InjuryType instances (getInjuryType() round-trips the passed object).
 */
public class ModificationParamsTest {

	private GameState gameState() {
		return GameFixture.createGameState();
	}

	private ModifiedInjuryContext context(ApothecaryMode mode) {
		ModifiedInjuryContext ctx = new ModifiedInjuryContext();
		ctx.setApothecaryMode(mode);
		return ctx;
	}

	// rust: modification_params_new_stores_fields
	@Test
	public void newStoresFields() {
		Block injuryType = new Block();
		ModificationParams params = new ModificationParams(gameState(), context(ApothecaryMode.DEFENDER), injuryType);
		assertEquals(injuryType, params.getInjuryType());
		assertFalse(params.getNewContext().isArmorBroken());
	}

	// rust: different_injury_type_name_stored
	@Test
	public void differentInjuryTypeNameStored() {
		Stab injuryType = new Stab();
		ModificationParams params = new ModificationParams(gameState(), context(ApothecaryMode.ATTACKER), injuryType);
		assertEquals("Stab", params.getInjuryType().getClass().getSimpleName());
	}

	// rust: new_context_apo_mode_stored
	@Test
	public void newContextApoModeStored() {
		ModificationParams params = new ModificationParams(gameState(), context(ApothecaryMode.ATTACKER), new Block());
		assertEquals(ApothecaryMode.ATTACKER, params.getNewContext().getApothecaryMode());
	}

	// rust: injury_type_name_is_accessible
	@Test
	public void injuryTypeNameIsAccessible() {
		ModificationParams params = new ModificationParams(gameState(), context(ApothecaryMode.DEFENDER), new CrowdPush());
		assertNotNull(params.getInjuryType());
		assertEquals("CrowdPush", params.getInjuryType().getClass().getSimpleName());
	}

	// rust: new_context_starts_with_no_injury
	@Test
	public void newContextStartsWithNoInjury() {
		ModificationParams params = new ModificationParams(gameState(), context(ApothecaryMode.DEFENDER), new Block());
		assertNull(params.getNewContext().getInjury());
	}
}
