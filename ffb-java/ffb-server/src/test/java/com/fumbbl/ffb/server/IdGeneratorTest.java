package com.fumbbl.ffb.server;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/id_generator.rs tests. IdGenerator is a sequential
 * counter: generateId() pre-increments and returns the new value; lastId() returns the current
 * value without incrementing. (Rust's Default::default() == new(0); Java has no no-arg constructor,
 * so the default-start test uses new IdGenerator(0).)
 */
public class IdGeneratorTest {

	// rust: generate_id_increments_from_initial
	@Test
	public void generateIdIncrementsFromInitial() {
		IdGenerator gen = new IdGenerator(0);
		assertEquals(1, gen.generateId());
		assertEquals(2, gen.generateId());
	}

	// rust: last_id_returns_current_without_incrementing
	@Test
	public void lastIdReturnsCurrentWithoutIncrementing() {
		IdGenerator gen = new IdGenerator(5);
		assertEquals(5, gen.lastId());
		gen.generateId();
		assertEquals(6, gen.lastId());
	}

	// rust: generate_id_with_nonzero_start
	@Test
	public void generateIdWithNonzeroStart() {
		assertEquals(101, new IdGenerator(100).generateId());
	}

	// rust: default_starts_at_zero (Java has no no-arg ctor; new(0) mirrors Rust default())
	@Test
	public void defaultStartsAtZero() {
		IdGenerator gen = new IdGenerator(0);
		assertEquals(0, gen.lastId());
		assertEquals(1, gen.generateId());
	}

	// rust: negative_start_increments_correctly
	@Test
	public void negativeStartIncrementsCorrectly() {
		IdGenerator gen = new IdGenerator(-3);
		assertEquals(-2, gen.generateId());
		assertEquals(-1, gen.generateId());
		assertEquals(0, gen.generateId());
	}

	// rust: many_sequential_ids_are_strictly_increasing
	@Test
	public void manySequentialIdsAreStrictlyIncreasing() {
		IdGenerator gen = new IdGenerator(0);
		long previous = gen.generateId();
		for (int i = 0; i < 9; i++) {
			long next = gen.generateId();
			assertEquals(previous + 1, next);
			previous = next;
		}
	}

	// rust: last_id_tracks_generate_id
	@Test
	public void lastIdTracksGenerateId() {
		IdGenerator gen = new IdGenerator(0);
		for (long expected = 1; expected <= 5; expected++) {
			gen.generateId();
			assertEquals(expected, gen.lastId());
		}
	}
}
