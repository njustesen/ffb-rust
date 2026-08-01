package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
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
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_bitten.rs tests.
 */
public class InjuryTypeBittenTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3));
	}

	private void init(GameState gs) {
		gameState = gs;
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

	private void handleInjury(InjuryTypeBitten injuryType) {
		injuryType.injuryContext().setAttackerId("home1");
		injuryType.injuryContext().setDefenderId("away1");
		injuryType.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeBitten injuryType, String namePart) {
		return Arrays.stream(injuryType.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_always_broken
	@Test
	public void armorAlwaysBroken() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1); // injury dice only, armour bypassed
		InjuryTypeBitten bitten = new InjuryTypeBitten();
		handleInjury(bitten);
		assertTrue(bitten.injuryContext().isArmorBroken());
	}

	// rust: injury_is_set_and_not_prone
	@Test
	public void injuryIsSetAndNotProne() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBitten bitten = new InjuryTypeBitten();
		handleInjury(bitten);
		assertNotNull(bitten.injuryContext().getInjury());
		assertNotEquals(PlayerState.PRONE, bitten.injuryContext().getInjury().getBase());
	}

	// rust: turnover_default_true (flag audit: no Java override)
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypeBitten().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_bitten
	@Test
	public void sendToBoxReasonIsBitten() {
		assertEquals(SendToBoxReason.BITTEN, new InjuryTypeBitten().sendToBoxReason());
	}

	// rust: niggling_injured_defender_gets_niggling_injury_modifier
	@Test
	public void nigglingInjuredDefenderGetsNigglingInjuryModifier() {
		init(GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBitten bitten = new InjuryTypeBitten();
		handleInjury(bitten);
		assertTrue(hasInjuryModifier(bitten, "Niggling"));
	}

	// rust: casualty_range_total_caps_at_badly_hurt_without_casualty_roll
	@Test
	public void casualtyRangeTotalCapsAtBadlyHurtWithoutCasualtyRoll() {
		GameFixture.installScriptedDice(gameState, 5, 5); // total 10 -> casualty range
		InjuryTypeBitten bitten = new InjuryTypeBitten();
		handleInjury(bitten);
		assertEquals(PlayerState.BADLY_HURT, bitten.injuryContext().getInjury().getBase());
		assertNull(bitten.injuryContext().getCasualtyRoll(), "Bitten must never roll casualty dice");
	}

	// rust: non_niggling_defender_gets_no_niggling_injury_modifier
	@Test
	public void nonNigglingDefenderGetsNoNigglingInjuryModifier() {
		init(GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016));
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBitten bitten = new InjuryTypeBitten();
		handleInjury(bitten);
		assertFalse(hasInjuryModifier(bitten, "Niggling"));
	}
}
