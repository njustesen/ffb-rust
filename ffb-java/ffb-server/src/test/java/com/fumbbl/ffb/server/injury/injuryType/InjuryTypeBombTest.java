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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_bomb.rs tests.
 * A bomb hit adds no Bomb modifier of its own; a chainsaw-carrying defender takes the +3
 * kickback modifier unless Iron Hard Skin suppresses it. Armour save leaves the injury unset.
 */
public class InjuryTypeBombTest {

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

	private void handleInjury(InjuryTypeBomb bomb) {
		bomb.injuryContext().setAttackerId("home1");
		bomb.injuryContext().setDefenderId("away1");
		bomb.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasArmorModifier(InjuryTypeBomb bomb, String name) {
		return Arrays.stream(bomb.injuryContext().getArmorModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	private boolean hasInjuryModifier(InjuryTypeBomb bomb, String name) {
		return Arrays.stream(bomb.injuryContext().getInjuryModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	// rust: armor_not_broken_leaves_injury_unset
	@Test
	public void armorNotBrokenLeavesInjuryUnset() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBomb bomb = new InjuryTypeBomb();
		handleInjury(bomb);
		assertFalse(bomb.injuryContext().isArmorBroken());
		assertNull(bomb.injuryContext().getInjury());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBomb bomb = new InjuryTypeBomb();
		handleInjury(bomb);
		assertTrue(bomb.injuryContext().isArmorBroken());
		assertNotNull(bomb.injuryContext().getInjury());
	}

	// rust: turnover_default_true
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypeBomb().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_bomb
	@Test
	public void sendToBoxReasonIsBomb() {
		assertEquals(SendToBoxReason.BOMB, new InjuryTypeBomb().sendToBoxReason());
	}

	// rust: does_not_add_bomb_armor_or_injury_modifier
	@Test
	public void doesNotAddBombArmorOrInjuryModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBomb bomb = new InjuryTypeBomb();
		handleInjury(bomb);
		assertFalse(hasArmorModifier(bomb, "Bomb"));
		assertFalse(hasInjuryModifier(bomb, "Bomb"));
	}

	// rust: defender_with_chainsaw_gets_chainsaw_modifier
	@Test
	public void defenderWithChainsawGetsChainsawModifier() {
		defender().setArmour(7);
		defender().addSkill(GameFixture.skill(game, "Chainsaw"));
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1); // 12+3 breaks
		InjuryTypeBomb bomb = new InjuryTypeBomb();
		handleInjury(bomb);
		assertTrue(hasArmorModifier(bomb, "Chainsaw"));
	}

	// rust: defender_with_iron_hard_skin_blocks_chainsaw_modifier
	@Test
	public void defenderWithIronHardSkinBlocksChainsawModifier() {
		defender().setArmour(7);
		defender().addSkill(GameFixture.skill(game, "Chainsaw"));
		defender().addSkill(GameFixture.skill(game, "Iron Hard Skin"));
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBomb bomb = new InjuryTypeBomb();
		handleInjury(bomb);
		assertFalse(hasArmorModifier(bomb, "Chainsaw"));
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBomb bomb = new InjuryTypeBomb();
		handleInjury(bomb);
		assertTrue(hasInjuryModifier(bomb, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBomb bomb = new InjuryTypeBomb();
		handleInjury(bomb);
		assertFalse(hasInjuryModifier(bomb, "Mighty Blow"));
	}
}
