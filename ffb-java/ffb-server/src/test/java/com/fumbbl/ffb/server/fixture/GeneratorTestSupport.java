package com.fumbbl.ffb.server.fixture;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;

import java.lang.reflect.Field;

/**
 * Shared helpers for porting the Rust generator push-order tests
 * ({@code ffb-rust/crates/ffb-engine/src/step/generator/**}).
 *
 * <p>Lives in the {@code fixture} package (alongside {@link GameFixture}) so any
 * {@code step/generator/**} test package can reuse it without a cross-package
 * dependency on {@code step.generator}.
 *
 * <p>A Rust generator test builds the sequence with {@code Xxx::build_sequence(&params)}
 * and inspects the returned {@code Vec<SequenceStep>} (each with {@code step_id},
 * {@code label} and {@code params}). The Java generators instead push
 * {@code IStep}s onto {@code gameState.getStepStack()}. Because
 * {@link com.fumbbl.ffb.server.step.StepStack#push(java.util.List)} reverses the
 * authored list so the first-authored step ends on top, {@code stepStack.toArray()}
 * returns the steps in the same authored order the Rust vector uses. This class
 * exposes that array plus the ordering / label / reflection helpers the ported
 * assertions need.
 *
 * <p>Usage:
 * <pre>{@code
 * GameState gs = GameFixture.createGameState(3);
 * new EndGame().pushSequence(new EndGame.SequenceParams(gs, false));
 * IStep[] steps = GeneratorTestSupport.sequence(gs);
 * assertEquals(StepId.INIT_END_GAME, steps[0].getId());
 * }</pre>
 */
public final class GeneratorTestSupport {

	private GeneratorTestSupport() {
	}

	/** The steps a generator pushed, in authored order (Rust {@code Vec<SequenceStep>} order). */
	public static IStep[] sequence(GameState gameState) {
		return gameState.getStepStack().toArray();
	}

	/** Index of the first step with the given id, or -1 (Rust {@code position(...)}). */
	public static int indexOf(IStep[] steps, StepId id) {
		for (int i = 0; i < steps.length; i++) {
			if (steps[i].getId() == id) {
				return i;
			}
		}
		return -1;
	}

	/** Whether any step has the given id (Rust {@code any(|s| s.step_id == id)}). */
	public static boolean contains(IStep[] steps, StepId id) {
		return indexOf(steps, id) >= 0;
	}

	/** Number of steps with the given id. */
	public static int count(IStep[] steps, StepId id) {
		int n = 0;
		for (IStep step : steps) {
			if (step.getId() == id) {
				n++;
			}
		}
		return n;
	}

	/** First step matching id + label, or null (Rust {@code find(|s| id && label)}). */
	public static IStep findLabelled(IStep[] steps, StepId id, String label) {
		for (IStep step : steps) {
			if (step.getId() == id && label.equals(step.getLabel())) {
				return step;
			}
		}
		return null;
	}

	/** First step with the given id, or null. */
	public static IStep find(IStep[] steps, StepId id) {
		int i = indexOf(steps, id);
		return i >= 0 ? steps[i] : null;
	}

	/** Index of a specific step instance in the array. */
	public static int indexOfInstance(IStep[] steps, IStep target) {
		for (int i = 0; i < steps.length; i++) {
			if (steps[i] == target) {
				return i;
			}
		}
		return -1;
	}

	/**
	 * Read a boolean field (declared on the step or a superclass) that a step
	 * stored from a {@code StepParameter} during {@code init(...)}. Mirrors a Rust
	 * assertion like {@code s.params.iter().any(|p| matches!(p, ResetForFailedBlock(true)))}.
	 */
	public static boolean booleanField(Object target, String fieldName) {
		return (Boolean) readField(target, fieldName);
	}

	/** Read an arbitrary field (declared on the step or a superclass) via reflection. */
	public static Object readField(Object target, String fieldName) {
		Class<?> cls = target.getClass();
		while (cls != null) {
			try {
				Field field = cls.getDeclaredField(fieldName);
				field.setAccessible(true);
				return field.get(target);
			} catch (NoSuchFieldException e) {
				cls = cls.getSuperclass();
			} catch (IllegalAccessException e) {
				throw new IllegalStateException("Cannot read field " + fieldName, e);
			}
		}
		throw new IllegalStateException("No field " + fieldName + " on " + target.getClass());
	}
}
