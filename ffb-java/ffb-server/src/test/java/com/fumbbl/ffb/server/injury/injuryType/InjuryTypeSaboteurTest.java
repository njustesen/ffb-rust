package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.SoundId;
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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_saboteur.rs tests.
 * The saboteur reveal KOs the player directly — no dice at all.
 */
public class InjuryTypeSaboteurTest {

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

	private InjuryTypeSaboteur handled() {
		InjuryTypeSaboteur saboteur = new InjuryTypeSaboteur();
		saboteur.injuryContext().setDefenderId("away1");
		saboteur.handleInjury(step, game, gameState, gameState.getDiceRoller(), null,
			game.getPlayerById("away1"), new FieldCoordinate(5, 5), null, null,
			ApothecaryMode.DEFENDER);
		return saboteur;
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

	// rust: does_not_cause_turnover
	@Test
	public void doesNotCauseTurnover() {
		assertFalse(new InjuryTypeSaboteur().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_saboteur
	@Test
	public void sendToBoxReasonIsSaboteur() {
		assertEquals(SendToBoxReason.SABOTEUR, new InjuryTypeSaboteur().sendToBoxReason());
	}

	// rust: cannot_use_apo
	@Test
	public void cannotUseApo() {
		assertFalse(new InjuryTypeSaboteur().canUseApo());
	}

	// rust: sets_ko_sound
	@Test
	public void setsKoSound() {
		assertEquals(SoundId.KO, handled().injuryContext().getSound());
	}
}
