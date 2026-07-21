package com.fumbbl.ffb.server.step.generator.bb2016;

import com.fumbbl.ffb.RulesCollection;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/throw_team_mate.rs}.
 */
public class ThrowTeamMateFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build() {
		new ThrowTeamMate().pushSequence(new ThrowTeamMate.SequenceParams(gameState));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: throw_team_mate_starts_with_init
	@Test
	public void throwTeamMateStartsWithInit() {
		assertEquals(StepId.INIT_THROW_TEAM_MATE, build()[0].getId());
	}

	// Rust: throw_team_mate_ends_with_end_throw_team_mate_labelled
	@Test
	public void throwTeamMateEndsWithEndThrowTeamMateLabelled() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_THROW_TEAM_MATE, last.getId());
		assertEquals(IStepLabel.END_THROW_TEAM_MATE, last.getLabel());
	}

	// Rust: throw_team_mate_fumble_ttm_pass_is_labelled
	@Test
	public void throwTeamMateFumbleTtmPassIsLabelled() {
		assertEquals(StepId.FUMBLE_TTM_PASS, GeneratorTestSupport.findLabelled(build(),
			StepId.FUMBLE_TTM_PASS, IStepLabel.FUMBLE_TTM_PASS).getId());
	}

	// Rust: throw_team_mate_eat_team_mate_is_labelled
	@Test
	public void throwTeamMateEatTeamMateIsLabelled() {
		assertEquals(StepId.EAT_TEAM_MATE, GeneratorTestSupport.findLabelled(build(),
			StepId.EAT_TEAM_MATE, IStepLabel.EAT_TEAM_MATE).getId());
	}

	// Rust: throw_team_mate_apothecary_thrown_player_is_labelled
	@Test
	public void throwTeamMateApothecaryThrownPlayerIsLabelled() {
		assertEquals(StepId.APOTHECARY, GeneratorTestSupport.findLabelled(build(),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_THROWN_PLAYER).getId());
	}
}
