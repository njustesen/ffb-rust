package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.Inducement;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.InducementSet;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_inducement_use.rs tests. useInducement
 * consumes N charges when (value - uses) >= N, incrementing uses; otherwise it fails and consumes
 * nothing. (The Rust use_one/use_inducement bare-int helper form is Rust-only, exempt.)
 */
public class UtilServerInducementUseTest {

	private GameState gameState;
	private Game game;
	private InducementType type;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		type = (InducementType) game.getFactory(FactoryType.Factory.INDUCEMENT_TYPE).forName("bloodweiserBabes");
	}

	/** Builds an InducementSet holding one inducement of the given value with `uses` already spent. */
	private InducementSet setWith(int value, int uses) {
		InducementSet set = new InducementSet();
		Inducement inducement = new Inducement(type, value);
		inducement.setUses(uses);
		set.addInducement(inducement);
		return set;
	}

	private int usesOf(InducementSet set) {
		return set.get(type).getUses();
	}

	// rust: use_inducement_succeeds_when_charges_remain
	@Test
	public void succeedsWhenChargesRemain() {
		InducementSet set = setWith(3, 0);
		assertTrue(UtilServerInducementUse.useInducement(type, 1, set));
		assertEquals(1, usesOf(set));
	}

	// rust: use_inducement_fails_when_no_charges_remain
	@Test
	public void failsWhenNoChargesRemain() {
		InducementSet set = setWith(3, 3);
		assertFalse(UtilServerInducementUse.useInducement(type, 1, set));
		assertEquals(3, usesOf(set));
	}

	// rust: use_inducement_succeeds_for_multiple_charges
	@Test
	public void succeedsForMultipleCharges() {
		InducementSet set = setWith(5, 0);
		assertTrue(UtilServerInducementUse.useInducement(type, 3, set));
		assertEquals(3, usesOf(set));
	}

	// rust: use_inducement_fails_when_not_enough_for_multiple_charges
	@Test
	public void failsWhenNotEnoughForMultipleCharges() {
		InducementSet set = setWith(2, 0);
		assertFalse(UtilServerInducementUse.useInducement(type, 3, set));
		assertEquals(0, usesOf(set));
	}

	// rust: use_inducement_exactly_one_charge_left_succeeds
	@Test
	public void exactlyOneChargeLeftSucceeds() {
		InducementSet set = setWith(3, 2);
		assertTrue(UtilServerInducementUse.useInducement(type, 1, set));
		assertEquals(3, usesOf(set));
	}

	// rust: use_inducement_zero_value_always_fails
	@Test
	public void zeroValueAlwaysFails() {
		InducementSet set = setWith(0, 0);
		assertFalse(UtilServerInducementUse.useInducement(type, 1, set));
	}

	// rust: use_inducement_nr_of_uses_zero_always_succeeds
	@Test
	public void nrOfUsesZeroAlwaysSucceeds() {
		InducementSet set = setWith(5, 5);
		assertTrue(UtilServerInducementUse.useInducement(type, 0, set));
		assertEquals(5, usesOf(set));
	}
}
