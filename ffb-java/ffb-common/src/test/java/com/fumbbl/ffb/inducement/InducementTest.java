package com.fumbbl.ffb.inducement;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.InducementTypeFactory;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-model/src/inducement/inducement.rs tests.
 * The Rust port folds the usages onto the Inducement struct; in Java usages live
 * on the InducementType returned by {@link Inducement#getType()}.
 */
public class InducementTest {

	@Test
	public void usesLeftClamped() {
		Inducement i = new Inducement(null, 1);
		i.setUses(5);
		assertEquals(0, i.getUsesLeft());
	}

	@Test
	public void hasUsageReturnsTrueForMatching() {
		InducementType type = new InducementType("BRIBE", null, null, null, null, null, null, false, null,
			Usage.AVOID_BAN, Usage.REROLL);
		Inducement i = new Inducement(type, 1);
		assertTrue(i.getType().hasUsage(Usage.AVOID_BAN));
		assertTrue(i.getType().hasUsage(Usage.REROLL));
		assertFalse(i.getType().hasUsage(Usage.STAR));
	}

	@Test
	public void serdeRoundTrip() {
		InducementTypeFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.INDUCEMENT_TYPE);
		// "wanderingApothecaries" is a base inducement type carrying Usage.APOTHECARY.
		InducementType apo = factory.forName("wanderingApothecaries");
		assertNotNull(apo);
		Inducement original = new Inducement(apo, 1);
		JsonObject json = original.toJsonValue();
		Inducement back = new Inducement().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("wanderingApothecaries", back.getType().getName());
		assertEquals(1, back.getValue());
		assertTrue(back.getType().getUsages().contains(Usage.APOTHECARY));
	}
}
