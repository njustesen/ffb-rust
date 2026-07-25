package com.fumbbl.ffb.marking;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-model/src/marking/transient_player_marker.rs tests.
 */
public class TransientPlayerMarkerTest {

	// rust: mode_display_text
	@Test
	public void modeDisplayText() {
		assertEquals("Append", TransientPlayerMarker.Mode.APPEND.getDisplayText());
		assertEquals("Add", TransientPlayerMarker.Mode.ADD.getDisplayText());
	}

	// rust: new_sets_player_id_and_mode
	@Test
	public void newSetsPlayerIdAndMode() {
		TransientPlayerMarker m = new TransientPlayerMarker("p1", TransientPlayerMarker.Mode.PREPEND);
		assertEquals("p1", m.getPlayerId());
		assertEquals(TransientPlayerMarker.Mode.PREPEND, m.getMode());
	}
}
