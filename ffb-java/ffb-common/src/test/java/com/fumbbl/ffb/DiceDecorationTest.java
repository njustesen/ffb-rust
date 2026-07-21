package com.fumbbl.ffb;

import com.fumbbl.ffb.model.BlockKind;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/dice_decoration.rs for {@link DiceDecoration}.
 * The Rust transform_of_default case relies on Rust's None-coordinate handling; the Java transform()
 * dereferences the coordinate, so that default case has no equivalent and is omitted.
 */
public class DiceDecorationTest {

	@Test
	public void transformPreservesDiceCountAndBlockKind() {
		DiceDecoration d = new DiceDecoration(new FieldCoordinate(5, 3), 3, BlockKind.STAB);
		DiceDecoration t = d.transform();
		assertEquals(3, t.getNrOfDice());
		assertEquals(BlockKind.STAB, t.getBlockKind());
	}

	@Test
	public void transformMirrorsCoordinate() {
		FieldCoordinate coord = new FieldCoordinate(3, 5);
		DiceDecoration d = new DiceDecoration(coord, 1, BlockKind.BLOCK);
		DiceDecoration t = d.transform();
		assertEquals(coord.transform(), t.getCoordinate());
		assertNotEquals(coord, t.getCoordinate());
	}

}
