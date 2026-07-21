package com.fumbbl.ffb.model;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/model/block_kind.rs for {@link BlockKind}.
 */
public class BlockKindTest {

	@Test
	public void serdeRoundTrip() {
		assertEquals(BlockKind.BLOCK, BlockKind.valueOf(BlockKind.BLOCK.name()));
	}

	@Test
	public void allVariantsSerdeRoundTrip() {
		for (BlockKind v : new BlockKind[] { BlockKind.BLOCK, BlockKind.STAB, BlockKind.VOMIT, BlockKind.CHAINSAW }) {
			assertEquals(v, BlockKind.valueOf(v.name()));
		}
	}
}
