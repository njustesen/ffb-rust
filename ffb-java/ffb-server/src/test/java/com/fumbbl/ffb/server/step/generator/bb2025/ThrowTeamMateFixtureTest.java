package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/throw_team_mate.rs}.
 */
public class ThrowTeamMateFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build() {
		new ThrowTeamMate().pushSequence(new ThrowTeamMate.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	private IStep[] buildWithThrownPlayer(String thrownPlayerId) {
		new ThrowTeamMate().pushSequence(new ThrowTeamMate.SequenceParams(gameState, thrownPlayerId, false));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: throw_team_mate_has_25_steps_with_activation
	@Test
	public void throwTeamMateHas25StepsWithActivation() {
		IStep[] steps = build();
		assertEquals(25, steps.length);
		assertEquals(StepId.INIT_ACTIVATION, steps[1].getId());
	}

	// Rust: throw_team_mate_sets_eventual_defender_when_thrown_player_id_present
	@Test
	public void throwTeamMateSetsEventualDefenderWhenThrownPlayerIdPresent() {
		IStep[] steps = buildWithThrownPlayer("p9");
		assertEquals(26, steps.length);
		IStep setDefender = GeneratorTestSupport.find(steps, StepId.SET_DEFENDER);
		assertEquals("p9", GeneratorTestSupport.readField(setDefender, "defenderId"));
	}

	// Rust: throw_team_mate_ends_with_end_throw_team_mate
	@Test
	public void throwTeamMateEndsWithEndThrowTeamMate() {
		IStep[] steps = build();
		assertEquals(StepId.END_THROW_TEAM_MATE, steps[steps.length - 1].getId());
	}

	// Rust: throw_team_mate_eat_team_mate_is_labelled
	@Test
	public void throwTeamMateEatTeamMateIsLabelled() {
		IStep etm = GeneratorTestSupport.find(build(), StepId.EAT_TEAM_MATE);
		assertEquals(IStepLabel.EAT_TEAM_MATE, etm.getLabel());
	}

	// Rust: throw_team_mate_apothecary_thrown_player_is_labelled
	@Test
	public void throwTeamMateApothecaryThrownPlayerIsLabelled() {
		assertEquals(StepId.APOTHECARY, GeneratorTestSupport.findLabelled(build(),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_THROWN_PLAYER).getId());
	}

	// Rust: throw_team_mate_resolve_pass_is_labelled
	@Test
	public void throwTeamMateResolvePassIsLabelled() {
		IStep dsp = GeneratorTestSupport.find(build(), StepId.DISPATCH_SCATTER_PLAYER);
		assertEquals(IStepLabel.RESOLVE_PASS, dsp.getLabel());
	}
}
