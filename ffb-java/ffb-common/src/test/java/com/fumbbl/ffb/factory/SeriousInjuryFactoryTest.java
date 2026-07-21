package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.FactoryManager;
import com.fumbbl.ffb.InjuryAttribute;
import com.fumbbl.ffb.SeriousInjury;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.fumbbl.ffb.option.GameOptionId;
import com.fumbbl.ffb.option.GameOptionString;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/serious_injury_factory.rs
 * for {@link SeriousInjuryFactory}.
 * <p>
 * The Rust {@code SeriousInjuryKind} has no Java equivalent, so the concrete rules-edition enum
 * constants are compared directly. The value-count tests, the empty-factory attribute test, the
 * {@code to_kind} round-trip, and the BB2016 MA lookup (whose two matching values have
 * unspecified iteration order in the backing {@code HashSet}) are intentionally not ported.
 */
public class SeriousInjuryFactoryTest {

	private static SeriousInjuryFactory factory(String rulesVersion) {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		FactoryManager manager = app.getFactoryManager();
		Game game = new Game(app, manager);
		GameOptionString rules = new GameOptionString(GameOptionId.RULESVERSION);
		rules.setValue(rulesVersion);
		game.getOptions().addOption(rules);
		SeriousInjuryFactory factory = new SeriousInjuryFactory();
		factory.initialize(game);
		return factory;
	}

	@Test
	public void deadReturnsDeadVariantForEachEdition() {
		SeriousInjuryFactory bb2016 = factory("BB2016");
		assertEquals(com.fumbbl.ffb.bb2016.SeriousInjury.DEAD, bb2016.dead());
		assertTrue(bb2016.dead().isDead());

		SeriousInjuryFactory bb2020 = factory("BB2020");
		assertEquals(com.fumbbl.ffb.bb2020.SeriousInjury.DEAD, bb2020.dead());
		assertTrue(bb2020.dead().isDead());

		SeriousInjuryFactory bb2025 = factory("BB2025");
		assertEquals(com.fumbbl.ffb.bb2025.SeriousInjury.DEAD, bb2025.dead());
		assertTrue(bb2025.dead().isDead());
	}

	@Test
	public void poisonOnlyPresentInBb2016() {
		assertNotNull(factory("BB2016").poison());
		assertNull(factory("BB2025").poison());
	}

	@Test
	public void forNameFindsBb2025HeadInjury() {
		SeriousInjury found = factory("BB2025").forName("Head Injury (-AV)");
		assertEquals(com.fumbbl.ffb.bb2025.SeriousInjury.HEAD_INJURY, found);
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(factory("BB2025").forName("Not A Real Injury"));
	}

	@Test
	public void forAttributeBb2025AgIsDislocatedHip() {
		SeriousInjury found = factory("BB2025").forAttribute(InjuryAttribute.AG);
		assertEquals(com.fumbbl.ffb.bb2025.SeriousInjury.DISLOCATED_HIP, found);
	}

	@Test
	public void forAttributeBb2020AgIsNeckInjury() {
		SeriousInjury found = factory("BB2020").forAttribute(InjuryAttribute.AG);
		assertEquals(com.fumbbl.ffb.bb2020.SeriousInjury.NECK_INJURY, found);
	}
}
