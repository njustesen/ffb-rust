package com.fumbbl.ffb.server.util.rng;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/rng/entropy_pool.rs tests. The SHA-256 entropy
 * pool has "enough" entropy once 32 bytes have been added; draining it via getEntropy resets the
 * count. (The Rust byte_count() accessor has no Java getter — that assertion is Rust-only.)
 */
public class EntropyPoolTest {

	// rust: test_not_enough_entropy_initially
	@Test
	public void notEnoughEntropyInitially() {
		assertFalse(new EntropyPool().hasEnoughEntropy());
	}

	// rust: test_add_entropy_increments_count (32 bytes -> enough entropy)
	@Test
	public void addingThirtyTwoBytesGivesEnoughEntropy() {
		EntropyPool pool = new EntropyPool();
		for (int b = 0; b < 32; b++) {
			pool.addEntropy((byte) b);
		}
		assertTrue(pool.hasEnoughEntropy());
	}

	// getEntropy drains the pool and resets the count below the threshold
	@Test
	public void getEntropyResetsCount() {
		EntropyPool pool = new EntropyPool();
		for (int b = 0; b < 32; b++) {
			pool.addEntropy((byte) b);
		}
		pool.getEntropy();
		assertFalse(pool.hasEnoughEntropy());
	}
}
