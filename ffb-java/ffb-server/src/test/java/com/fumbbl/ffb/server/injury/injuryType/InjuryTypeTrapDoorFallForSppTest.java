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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_trap_door_fall_for_spp.rs
 * tests. Extends the crowd base: armour always broken; a non-KO/non-casualty injury result
 * becomes RESERVE (fell through the trapdoor).
 */
public class InjuryTypeTrapDoorFallForSppTest {

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

	private InjuryTypeTrapDoorFallForSpp handled() {
		InjuryTypeTrapDoorFallForSpp trapDoor = new InjuryTypeTrapDoorFallForSpp();
		trapDoor.injuryContext().setDefenderId("away1");
		trapDoor.handleInjury(step, game, gameState, gameState.getDiceRoller(), null,
			game.getPlayerById("away1"), new FieldCoordinate(5, 5), null, null,
			ApothecaryMode.DEFENDER);
		return trapDoor;
	}

	// rust: armor_always_broken
	@Test
	public void armorAlwaysBroken() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		assertTrue(handled().injuryContext().isArmorBroken());
	}

	// rust: injury_is_reserve_or_ko_or_casualty (deterministic Java version: stunned -> RESERVE)
	@Test
	public void injuryIsReserveWhenNotKoOrCasualty() {
		GameFixture.installScriptedDice(gameState, 1, 1); // total 2 -> stunned -> RESERVE
		assertEquals(PlayerState.RESERVE, handled().injuryContext().getInjury().getBase());
	}

	// rust: does_not_cause_turnover
	@Test
	public void doesNotCauseTurnover() {
		assertFalse(new InjuryTypeTrapDoorFallForSpp().injuryType().fallingDownCausesTurnover());
	}

	// rust: is_caused_by_opponent_and_worth_spps
	@Test
	public void isCausedByOpponentAndWorthSpps() {
		InjuryTypeTrapDoorFallForSpp trapDoor = new InjuryTypeTrapDoorFallForSpp();
		assertTrue(trapDoor.injuryType().isCausedByOpponent());
		assertTrue(trapDoor.injuryType().isWorthSpps());
	}

	// rust: send_to_box_reason_is_trap_door_fall
	@Test
	public void sendToBoxReasonIsTrapDoorFall() {
		assertEquals(SendToBoxReason.TRAP_DOOR_FALL, new InjuryTypeTrapDoorFallForSpp().sendToBoxReason());
	}
}
