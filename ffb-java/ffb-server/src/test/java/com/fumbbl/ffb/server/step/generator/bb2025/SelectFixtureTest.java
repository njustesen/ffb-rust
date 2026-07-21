package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.model.BlockTarget;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.generator.GeneratorTestSupport;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/select.rs}.
 *
 * <p>Rust threads {@code is_blitz_move} through {@code SelectParams}; the Java
 * generator instead derives it from the acting player's {@code PlayerAction}
 * ({@code playerAction.isBlitzMove()}), so the blitz-move test activates a player
 * with {@link PlayerAction#BLITZ_MOVE} first. Rust omits an empty {@code BlockTargets}
 * param entirely; Java always threads {@code getBlockTargets()} (an empty list),
 * so the behavioral equivalent is asserting the target list's contents.
 */
public class SelectFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(com.fumbbl.ffb.server.step.generator.Select.SequenceParams params) {
		new Select().pushSequence(params);
		return GeneratorTestSupport.sequence(gameState);
	}

	private com.fumbbl.ffb.server.step.generator.Select.SequenceParams defaultParams() {
		return new com.fumbbl.ffb.server.step.generator.Select.SequenceParams(gameState, false);
	}

	// Rust: select_sequence_starts_with_init_selecting
	@Test
	public void selectSequenceStartsWithInitSelecting() {
		assertEquals(StepId.INIT_SELECTING, build(defaultParams())[0].getId());
	}

	// Rust: select_sequence_ends_with_end_selecting
	@Test
	public void selectSequenceEndsWithEndSelecting() {
		IStep[] steps = build(defaultParams());
		assertEquals(StepId.END_SELECTING, steps[steps.length - 1].getId());
	}

	// Rust: select_sequence_has_reset_fumblerooskie_labelled_end_selecting
	@Test
	public void selectSequenceHasResetFumblerooskieLabelledEndSelecting() {
		IStep rfr = GeneratorTestSupport.find(build(defaultParams()), StepId.RESET_FUMBLEROOSKIE);
		assertNotNull(rfr);
		assertEquals(IStepLabel.END_SELECTING, rfr.getLabel());
	}

	// Rust: update_persistence_param_passed_to_init_selecting
	@Test
	public void updatePersistenceParamPassedToInitSelecting() {
		com.fumbbl.ffb.server.step.generator.Select.SequenceParams params =
			new com.fumbbl.ffb.server.step.generator.Select.SequenceParams(gameState, true);
		IStep init = build(params)[0];
		assertTrue(GeneratorTestSupport.booleanField(init, "fUpdatePersistence"));
	}

	// Rust: is_blitz_move_sets_reset_for_failed_block
	@Test
	public void isBlitzMoveSetsResetForFailedBlock() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLITZ_MOVE);
		IStep rfr = GeneratorTestSupport.find(build(defaultParams()), StepId.RESET_FUMBLEROOSKIE);
		assertNotNull(rfr);
		assertTrue(GeneratorTestSupport.booleanField(rfr, "resetForFailedBlock"));
	}

	// Rust: block_targets_wired_to_end_selecting_when_present
	@Test
	public void blockTargetsWiredToEndSelectingWhenPresent() {
		List<BlockTarget> targets = new ArrayList<>();
		targets.add(new BlockTarget("p1", null, null));
		com.fumbbl.ffb.server.step.generator.Select.SequenceParams params =
			new com.fumbbl.ffb.server.step.generator.Select.SequenceParams(gameState, false, targets);
		IStep end = GeneratorTestSupport.find(build(params), StepId.END_SELECTING);
		assertNotNull(end);
		@SuppressWarnings("unchecked")
		List<BlockTarget> wired = (List<BlockTarget>) GeneratorTestSupport.readField(end, "blockTargets");
		assertEquals(1, wired.size());
		assertEquals("p1", wired.get(0).getPlayerId());
	}

	// Rust: block_targets_omitted_when_empty
	@Test
	public void blockTargetsOmittedWhenEmpty() {
		IStep end = GeneratorTestSupport.find(build(defaultParams()), StepId.END_SELECTING);
		assertNotNull(end);
		@SuppressWarnings("unchecked")
		List<BlockTarget> wired = (List<BlockTarget>) GeneratorTestSupport.readField(end, "blockTargets");
		assertTrue(wired == null || wired.isEmpty());
	}
}
