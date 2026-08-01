package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_piling_on_knocked_out.rs
 * tests. Piling on onto a knocked-out player: direct KO, no dice; apothecary NOT usable
 * (the one PilingOn variant where Java overrides canUseApo).
 */
public class InjuryTypePilingOnKnockedOutTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "away1", 5, 5);
	}

	private InjuryTypePilingOnKnockedOut handled() {
		InjuryTypePilingOnKnockedOut pilingOn = new InjuryTypePilingOnKnockedOut(step);
		pilingOn.injuryContext().setDefenderId("away1");
		pilingOn.handleInjury(step, game, gameState, gameState.getDiceRoller(), null,
			game.getPlayerById("away1"), new FieldCoordinate(5, 5), null, null,
			ApothecaryMode.DEFENDER);
		return pilingOn;
	}

	// rust: armor_always_broken
	@Test
	public void armorAlwaysBroken() {
		assertTrue(handled().injuryContext().isArmorBroken());
	}

	// rust: injury_is_ps_knocked_out
	@Test
	public void injuryIsPsKnockedOut() {
		assertEquals(PlayerState.KNOCKED_OUT, handled().injuryContext().getInjury().getBase());
	}

	// rust: turnover_default_true (flag audit: no Java override)
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypePilingOnKnockedOut(step).injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_ko_on_piling_on
	@Test
	public void sendToBoxReasonIsKoOnPilingOn() {
		assertEquals(SendToBoxReason.KO_ON_PILING_ON, new InjuryTypePilingOnKnockedOut(step).sendToBoxReason());
	}

	// rust: cannot_use_apo
	@Test
	public void cannotUseApo() {
		assertFalse(new InjuryTypePilingOnKnockedOut(step).canUseApo());
	}

	// rust: is_caused_by_opponent
	@Test
	public void isCausedByOpponent() {
		assertTrue(new InjuryTypePilingOnKnockedOut(step).injuryType().isCausedByOpponent());
	}
}
