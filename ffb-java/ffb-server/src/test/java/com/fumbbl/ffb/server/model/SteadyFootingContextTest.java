package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.server.InjuryResult;
import com.fumbbl.ffb.server.injury.injuryType.InjuryTypeBlock;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/model/steady_footing_context.rs tests.
 * SteadyFootingContext wraps one of DropPlayerContext / InjuryResult / InjuryTypeServer (Java's
 * three-ctor pattern; Rust's three-variant inner). The Rust injury_type_name string maps to Java's
 * getInjuryType() object (asserted via its class). getApothecaryMode falls back through
 * dropPlayerContext -> injuryResult -> ATTACKER default.
 */
public class SteadyFootingContextTest {

	// rust: from_drop_player_context
	@Test
	public void fromDropPlayerContext() {
		SteadyFootingContext sfc = new SteadyFootingContext(new DropPlayerContext());
		assertNotNull(sfc.getDropPlayerContext());
		assertNull(sfc.getInjuryResult());
		assertNull(sfc.getInjuryType());
	}

	// rust: from_injury_result
	@Test
	public void fromInjuryResult() {
		SteadyFootingContext sfc = new SteadyFootingContext(new InjuryResult());
		assertNotNull(sfc.getInjuryResult());
		assertNull(sfc.getDropPlayerContext());
	}

	// rust: from_injury_type_name
	@Test
	public void fromInjuryTypeName() {
		SteadyFootingContext sfc = new SteadyFootingContext(new InjuryTypeBlock());
		assertNotNull(sfc.getInjuryType());
		assertEquals("InjuryTypeBlock", sfc.getInjuryType().getClass().getSimpleName());
		assertNull(sfc.getDropPlayerContext());
		assertNull(sfc.getInjuryResult());
	}

	// rust: get_apothecary_mode_from_drop_player_context
	@Test
	public void getApothecaryModeFromDropPlayerContext() {
		DropPlayerContext ctx = new DropPlayerContext(new InjuryResult(), false, false, null, "p",
			ApothecaryMode.DEFENDER, false);
		SteadyFootingContext sfc = new SteadyFootingContext(ctx);
		assertEquals(ApothecaryMode.DEFENDER, sfc.getApothecaryMode());
	}

	// rust: get_apothecary_mode_default_for_injury_type_name
	@Test
	public void getApothecaryModeDefaultForInjuryTypeName() {
		SteadyFootingContext sfc = new SteadyFootingContext(new InjuryTypeBlock());
		assertEquals(ApothecaryMode.ATTACKER, sfc.getApothecaryMode());
	}

	// rust: get_apothecary_mode_from_injury_result
	@Test
	public void getApothecaryModeFromInjuryResult() {
		InjuryResult ir = new InjuryResult();
		ir.injuryContext().setApothecaryMode(ApothecaryMode.ATTACKER);
		SteadyFootingContext sfc = new SteadyFootingContext(ir);
		assertEquals(ApothecaryMode.ATTACKER, sfc.getApothecaryMode());
	}

	// rust: from_injury_type_name_injury_result_is_none
	@Test
	public void fromInjuryTypeNameInjuryResultIsNone() {
		SteadyFootingContext sfc = new SteadyFootingContext(new InjuryTypeBlock());
		assertNull(sfc.getInjuryResult());
	}

	// rust: from_drop_player_injury_type_name_is_none
	@Test
	public void fromDropPlayerInjuryTypeNameIsNone() {
		SteadyFootingContext sfc = new SteadyFootingContext(new DropPlayerContext());
		assertNull(sfc.getInjuryType());
	}
}
