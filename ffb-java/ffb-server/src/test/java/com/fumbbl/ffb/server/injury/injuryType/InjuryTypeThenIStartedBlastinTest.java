package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_then_i_started_blastin.rs
 * tests.
 */
public class InjuryTypeThenIStartedBlastinTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "home1", 2, 2); // attacker
		GameFixture.placePlayer(gameState, "away1", 5, 5); // defender
	}

	private RosterPlayer attacker() {
		return (RosterPlayer) game.getPlayerById("home1");
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeThenIStartedBlastin blastin) {
		blastin.injuryContext().setAttackerId("home1");
		blastin.injuryContext().setDefenderId("away1");
		blastin.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeThenIStartedBlastin blastin, String name) {
		return Arrays.stream(blastin.injuryContext().getInjuryModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	// rust: armor_save_leaves_no_injury
	@Test
	public void armorSaveLeavesNoInjury() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeThenIStartedBlastin blastin = new InjuryTypeThenIStartedBlastin();
		handleInjury(blastin);
		assertFalse(blastin.injuryContext().isArmorBroken());
		assertNull(blastin.injuryContext().getInjury());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeThenIStartedBlastin blastin = new InjuryTypeThenIStartedBlastin();
		handleInjury(blastin);
		assertTrue(blastin.injuryContext().isArmorBroken());
		assertNotNull(blastin.injuryContext().getInjury());
	}

	// rust: causes_turnover_by_default
	@Test
	public void causesTurnoverByDefault() {
		assertTrue(new InjuryTypeThenIStartedBlastin().injuryType().fallingDownCausesTurnover());
	}

	// rust: is_caused_by_opponent_is_true
	@Test
	public void isCausedByOpponentIsTrue() {
		assertTrue(new InjuryTypeThenIStartedBlastin().injuryType().isCausedByOpponent());
	}

	// rust: send_to_box_reason_is_then_i_started_blastin
	@Test
	public void sendToBoxReasonIsThenIStartedBlastin() {
		assertEquals(SendToBoxReason.THEN_I_STARTED_BLASTIN,
			new InjuryTypeThenIStartedBlastin().sendToBoxReason());
	}

	// rust: failed_armour_places_prone_is_false
	@Test
	public void failedArmourPlacesProneIsFalse() {
		assertFalse(new InjuryTypeThenIStartedBlastin().injuryType().failedArmourPlacesProne());
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeThenIStartedBlastin blastin = new InjuryTypeThenIStartedBlastin();
		handleInjury(blastin);
		assertTrue(hasInjuryModifier(blastin, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeThenIStartedBlastin blastin = new InjuryTypeThenIStartedBlastin();
		handleInjury(blastin);
		assertFalse(hasInjuryModifier(blastin, "Mighty Blow"));
	}
}
