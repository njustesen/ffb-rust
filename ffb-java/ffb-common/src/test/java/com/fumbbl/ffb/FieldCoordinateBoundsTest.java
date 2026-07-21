package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/types/field_coordinate.rs that exercise
 * {@link FieldCoordinateBounds}.
 */
public class FieldCoordinateBoundsTest {

	@Test
	public void boundsInBounds() {
		assertTrue(FieldCoordinateBounds.HALF_HOME.isInBounds(new FieldCoordinate(0, 0)));
		assertFalse(FieldCoordinateBounds.HALF_HOME.isInBounds(new FieldCoordinate(13, 0)));
	}

	@Test
	public void boundsSize() {
		assertEquals(390, FieldCoordinateBounds.FIELD.size());
	}

	@Test
	public void boundsCoordinatesCount() {
		assertEquals(7, FieldCoordinateBounds.LOS_HOME.fieldCoordinates().length);
	}

	@Test
	public void fieldCoordinateBoundsFieldContainsCenter() {
		assertTrue(FieldCoordinateBounds.FIELD.isInBounds(new FieldCoordinate(13, 8)));
	}

	@Test
	public void fieldCoordinateBoundsHasTopLeftCorner() {
		assertEquals(new FieldCoordinate(0, 0), FieldCoordinateBounds.FIELD.getTopLeftCorner());
	}

	@Test
	public void fieldCoordinateBoundsHasBottomRightCorner() {
		assertEquals(new FieldCoordinate(25, 14), FieldCoordinateBounds.FIELD.getBottomRightCorner());
	}

	@Test
	public void fieldCoordinateBoundsEndzoneHomeInBounds() {
		assertTrue(FieldCoordinateBounds.ENDZONE_HOME.isInBounds(new FieldCoordinate(0, 5)));
	}

	@Test
	public void fieldCoordinateBoundsEndzoneAwayInBounds() {
		assertTrue(FieldCoordinateBounds.ENDZONE_AWAY.isInBounds(new FieldCoordinate(25, 5)));
	}
}
