package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.BlockResult;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/block_result_factory.rs
 * for {@link BlockResultFactory}.
 */
public class BlockResultFactoryTest {

	@Test
	public void forRollSkull() {
		BlockResultFactory f = new BlockResultFactory();
		assertEquals(BlockResult.SKULL, f.forRoll(1));
	}

	@Test
	public void forRollBothDown() {
		assertEquals(BlockResult.BOTH_DOWN, new BlockResultFactory().forRoll(2));
	}

	@Test
	public void forRollPushbackDefault() {
		assertEquals(BlockResult.PUSHBACK, new BlockResultFactory().forRoll(3));
		assertEquals(BlockResult.PUSHBACK, new BlockResultFactory().forRoll(4));
	}

	@Test
	public void forRollPowPushback() {
		assertEquals(BlockResult.POW_PUSHBACK, new BlockResultFactory().forRoll(5));
	}

	@Test
	public void forRollPow() {
		assertEquals(BlockResult.POW, new BlockResultFactory().forRoll(6));
	}

	@Test
	public void forNameRoundTrip() {
		BlockResultFactory f = new BlockResultFactory();
		assertEquals(BlockResult.SKULL, f.forName("SKULL"));
		assertNull(f.forName("NONEXISTENT"));
	}
}
