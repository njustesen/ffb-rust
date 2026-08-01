package com.fumbbl.ffb.server;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/replay_state.rs tests. A ReplayState starts with the
 * given name and zeroed command/speed and false running/forward flags; it tracks the set of coaches
 * prevented from sketching (prevent/allow/query). (The command-driven mutation path handleCommand
 * is not part of the Rust unit tests.)
 */
public class ReplayStateTest {

	// rust: new_stores_name_and_defaults
	@Test
	public void newStoresNameAndDefaults() {
		ReplayState rs = new ReplayState("my-replay");
		assertEquals("my-replay", rs.getName());
		assertEquals(0, rs.getCommandNr());
		assertEquals(0, rs.getSpeed());
		assertFalse(rs.isRunning());
		assertFalse(rs.isForward());
	}

	// rust: prevent_and_allow_coach_sketching
	@Test
	public void preventAndAllowCoachSketching() {
		ReplayState rs = new ReplayState("r");
		rs.preventCoachFromSketching("coachA");
		assertTrue(rs.isCoachPreventedFromSketching("coachA"));
		assertFalse(rs.isCoachPreventedFromSketching("coachB"));
		rs.allowCoachToSketch("coachA");
		assertFalse(rs.isCoachPreventedFromSketching("coachA"));
	}

	// rust: multiple_coaches_tracked_independently
	@Test
	public void multipleCoachesTrackedIndependently() {
		ReplayState rs = new ReplayState("r");
		rs.preventCoachFromSketching("c1");
		rs.preventCoachFromSketching("c2");
		rs.allowCoachToSketch("c1");
		assertFalse(rs.isCoachPreventedFromSketching("c1"));
		assertTrue(rs.isCoachPreventedFromSketching("c2"));
	}

	// rust: allow_not_prevented_coach_is_noop
	@Test
	public void allowNotPreventedCoachIsNoop() {
		ReplayState rs = new ReplayState("r");
		rs.allowCoachToSketch("ghost");
		assertFalse(rs.isCoachPreventedFromSketching("ghost"));
	}

	// rust: prevent_same_coach_twice_is_idempotent
	@Test
	public void preventSameCoachTwiceIsIdempotent() {
		ReplayState rs = new ReplayState("r");
		rs.preventCoachFromSketching("dup");
		rs.preventCoachFromSketching("dup");
		assertTrue(rs.isCoachPreventedFromSketching("dup"));
		rs.allowCoachToSketch("dup");
		assertFalse(rs.isCoachPreventedFromSketching("dup"));
	}

	// rust: unknown_coach_is_not_prevented
	@Test
	public void unknownCoachIsNotPrevented() {
		assertFalse(new ReplayState("r").isCoachPreventedFromSketching("nobody"));
	}

	// rust: name_preserved_exactly
	@Test
	public void namePreservedExactly() {
		String name = "test-replay-2025";
		assertEquals(name, new ReplayState(name).getName());
	}
}
