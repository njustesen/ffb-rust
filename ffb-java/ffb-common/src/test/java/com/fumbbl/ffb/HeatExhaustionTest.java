package com.fumbbl.ffb;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/heat_exhaustion.rs for {@link HeatExhaustion}.
 */
public class HeatExhaustionTest {

	@Test
	public void serdeRoundTrip() {
		HeatExhaustion h = new HeatExhaustion("p1", true, 3);
		HeatExhaustion back = new HeatExhaustion().initFrom(NetCommandTestUtil.applicationSource(), h.toJsonValue());
		assertEquals("p1", back.getPlayerId());
		assertTrue(back.isExhausted());
		assertEquals(3, back.getRoll());
	}

	@Test
	public void rollBoundaryValues() {
		HeatExhaustion hMin = new HeatExhaustion("p", false, 1);
		HeatExhaustion hMax = new HeatExhaustion("p", false, 6);
		assertEquals(1, hMin.getRoll());
		assertEquals(6, hMax.getRoll());
	}

}
