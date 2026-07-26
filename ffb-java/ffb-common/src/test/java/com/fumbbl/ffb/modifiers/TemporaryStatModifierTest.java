package com.fumbbl.ffb.modifiers;

import com.fumbbl.ffb.mechanics.StatsMechanic;
import org.junit.jupiter.api.Test;

import java.util.function.IntUnaryOperator;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/temporary_stat_modifier.rs tests.
 * TemporaryStatModifier is abstract in Java (Rust concretizes it with an apply-closure); the Rust
 * apply_fn closures map to an overridden apply() in the ApplyStub subclass below. The limit is
 * mechanic-derived (Rust fabricates it), so limit assertions use the mixed StatsMechanic bounds.
 */
public class TemporaryStatModifierTest {

	private final StatsMechanic mechanic = new com.fumbbl.ffb.mechanics.mixed.StatsMechanic();

	private static class ApplyStub extends TemporaryStatModifier {
		private final IntUnaryOperator fn;

		ApplyStub(PlayerStatKey key, StatsMechanic mechanic, IntUnaryOperator fn) {
			super(key, mechanic);
			this.fn = fn;
		}

		@Override
		public int apply(int value) {
			return fn.applyAsInt(value);
		}
	}

	private ApplyStub stub(PlayerStatKey key, IntUnaryOperator fn) {
		return new ApplyStub(key, mechanic, fn);
	}

	// rust: apply_fn_is_called_with_value
	@Test
	public void applyFnIsCalledWithValue() {
		ApplyStub m = stub(PlayerStatKey.MA, v -> v + 2);
		assertEquals(6, m.apply(4));
	}

	// rust: applies_to_matching_stat
	@Test
	public void appliesToMatchingStat() {
		ApplyStub m = stub(PlayerStatKey.ST, v -> v);
		assertTrue(m.appliesTo(PlayerStatKey.ST));
		assertFalse(m.appliesTo(PlayerStatKey.MA));
	}

	// rust: get_limit_returns_correct_bounds (Rust fabricates AV=(2,13); Java derives AV=(3,11))
	@Test
	public void getLimitReturnsCorrectBounds() {
		ApplyStub m = stub(PlayerStatKey.AV, v -> v);
		assertEquals(3, m.getLimit().getMin());
		assertEquals(11, m.getLimit().getMax());
	}

	// rust: apply_fn_can_clamp_to_limit
	@Test
	public void applyFnCanClampToLimit() {
		ApplyStub m = stub(PlayerStatKey.MA, v -> Math.max(1, Math.min(6, v)));
		assertEquals(1, m.apply(0));
		assertEquals(6, m.apply(10));
		assertEquals(3, m.apply(3));
	}

	// rust: applies_to_returns_false_for_other_stat
	@Test
	public void appliesToReturnsFalseForOtherStat() {
		ApplyStub m = stub(PlayerStatKey.AG, v -> v + 1);
		assertTrue(m.appliesTo(PlayerStatKey.AG));
		assertFalse(m.appliesTo(PlayerStatKey.PA));
		assertFalse(m.appliesTo(PlayerStatKey.AV));
	}
}
