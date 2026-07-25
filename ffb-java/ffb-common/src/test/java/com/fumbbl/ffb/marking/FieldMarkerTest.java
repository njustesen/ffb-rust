package com.fumbbl.ffb.marking;

import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/marking/field_marker.rs tests.
 * Rust's transform_opt(None) maps to Java's static null-safe transform(FieldMarker).
 */
public class FieldMarkerTest {

	// rust: transform_swaps_texts
	@Test
	public void transformSwapsTexts() {
		FieldMarker m = new FieldMarker(new FieldCoordinate(3, 4), "Home", "Away");
		FieldMarker t = m.transform();
		assertEquals("Away", t.getHomeText());
		assertEquals("Home", t.getAwayText());
	}

	// rust: transform_opt_none_returns_none
	@Test
	public void transformOptNoneReturnsNone() {
		assertNull(FieldMarker.transform(null));
	}
}
