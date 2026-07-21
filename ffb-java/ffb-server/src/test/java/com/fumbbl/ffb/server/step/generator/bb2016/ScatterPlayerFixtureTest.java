package com.fumbbl.ffb.server.step.generator.bb2016;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/scatter_player.rs}.
 */
public class ScatterPlayerFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build(String thrownPlayerId, boolean hasSwoop, boolean throwScatter) {
		new ScatterPlayer().pushSequence(new ScatterPlayer.SequenceParams(gameState, thrownPlayerId,
			new PlayerState(PlayerState.STANDING), false, new FieldCoordinate(1, 1), hasSwoop, throwScatter));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: scatter_player_without_swoop_starts_with_init_scatter_player
	@Test
	public void scatterPlayerWithoutSwoopStartsWithInitScatterPlayer() {
		assertEquals(StepId.INIT_SCATTER_PLAYER, build("p1", false, false)[0].getId());
	}

	// Rust: scatter_player_with_swoop_starts_with_swoop
	@Test
	public void scatterPlayerWithSwoopStartsWithSwoop() {
		assertEquals(StepId.SWOOP, build("p1", true, false)[0].getId());
	}

	// Rust: scatter_player_ends_with_end_scatter_player
	@Test
	public void scatterPlayerEndsWithEndScatterPlayer() {
		IStep[] steps = build("p1", false, false);
		assertEquals(StepId.END_SCATTER_PLAYER, steps[steps.length - 1].getId());
	}

	// Rust: scatter_player_apothecary_hit_player_is_labelled
	@Test
	public void scatterPlayerApothecaryHitPlayerIsLabelled() {
		assertNotNull(GeneratorTestSupport.findLabelled(build("p1", false, false),
			StepId.APOTHECARY, IStepLabel.APOTHECARY_HIT_PLAYER));
	}

	// Rust: thrown_player_id_passed_to_init_scatter_player
	@Test
	public void thrownPlayerIdPassedToInitScatterPlayer() {
		IStep init = GeneratorTestSupport.find(build("tpid", false, false), StepId.INIT_SCATTER_PLAYER);
		assertEquals("tpid", GeneratorTestSupport.readField(init, "fThrownPlayerId"));
	}

	// Rust: throw_scatter_param_passed
	@Test
	public void throwScatterParamPassed() {
		IStep init = GeneratorTestSupport.find(build("p1", false, true), StepId.INIT_SCATTER_PLAYER);
		assertTrue(GeneratorTestSupport.booleanField(init, "fThrowScatter"));
	}
}
