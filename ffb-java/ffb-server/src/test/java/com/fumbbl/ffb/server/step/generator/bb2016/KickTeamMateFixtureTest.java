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
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/kick_team_mate.rs}.
 */
public class KickTeamMateFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build() {
		new KickTeamMate().pushSequence(new KickTeamMate.SequenceParams(gameState, 1, null));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: kick_team_mate_starts_with_init
	@Test
	public void kickTeamMateStartsWithInit() {
		assertEquals(StepId.INIT_KICK_TEAM_MATE, build()[0].getId());
	}

	// Rust: kick_team_mate_ends_with_end_labelled
	@Test
	public void kickTeamMateEndsWithEndLabelled() {
		IStep[] steps = build();
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_KICK_TEAM_MATE, last.getId());
		assertEquals(IStepLabel.END_KICK_TEAM_MATE, last.getLabel());
	}

	// Rust: kick_team_mate_has_kick_tm_double_rolled_labelled
	@Test
	public void kickTeamMateHasKickTmDoubleRolledLabelled() {
		assertEquals(StepId.KICK_TM_DOUBLE_ROLLED, GeneratorTestSupport.findLabelled(build(),
			StepId.KICK_TM_DOUBLE_ROLLED, IStepLabel.KICK_TM_DOUBLE_ROLLED).getId());
	}

	// Rust: kick_team_mate_has_apothecary_kicked_player_labelled
	@Test
	public void kickTeamMateHasApothecaryKickedPlayerLabelled() {
		assertEquals(StepId.APOTHECARY, GeneratorTestSupport.findLabelled(build(),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_KICKED_PLAYER).getId());
	}

	// Rust: kick_team_mate_right_stuff_is_labelled
	@Test
	public void kickTeamMateRightStuffIsLabelled() {
		assertEquals(IStepLabel.RIGHT_STUFF,
			GeneratorTestSupport.findLabelled(build(), StepId.RIGHT_STUFF, IStepLabel.RIGHT_STUFF).getLabel());
	}
}
