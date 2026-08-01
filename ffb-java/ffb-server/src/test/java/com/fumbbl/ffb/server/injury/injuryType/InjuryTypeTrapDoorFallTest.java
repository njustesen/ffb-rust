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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_trap_door_fall.rs tests.
 * The crowd-base trap-door injury: armour always broken; a non-KO/non-casualty injury result
 * becomes RESERVE (fell through the trapdoor). No turnover, canApoKoIntoStun false, SendToBoxReason
 * TRAP_DOOR_FALL. (The Rust default_equivalent_to_new plumbing test is Rust-structural — exempt;
 * the Rust java_class_name test keys off can_apo_ko_into_stun, mirrored here as canApoKoIntoStun().)
 */
public class InjuryTypeTrapDoorFallTest {

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

	private InjuryTypeTrapDoorFall handled() {
		InjuryTypeTrapDoorFall trapDoor = new InjuryTypeTrapDoorFall();
		trapDoor.injuryContext().setDefenderId("away1");
		trapDoor.handleInjury(step, game, gameState, gameState.getDiceRoller(), null,
			game.getPlayerById("away1"), new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
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
		assertFalse(new InjuryTypeTrapDoorFall().injuryType().fallingDownCausesTurnover());
	}

	// rust: java_class_name_matches_can_apo_ko_into_stun_lookup (a trap-door KO cannot be revived
	// into stun — mirrored via the canApoKoIntoStun() override the Rust lookup keys off)
	@Test
	public void cannotApoKoIntoStun() {
		assertFalse(new InjuryTypeTrapDoorFall().injuryType().canApoKoIntoStun());
	}

	// rust: send_to_box_reason_is_trap_door_fall
	@Test
	public void sendToBoxReasonIsTrapDoorFall() {
		assertEquals(SendToBoxReason.TRAP_DOOR_FALL, new InjuryTypeTrapDoorFall().sendToBoxReason());
	}

	// rust: context_stores_defender_id
	@Test
	public void contextStoresDefender() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		assertEquals("away1", handled().injuryContext().getDefenderId());
	}
}
