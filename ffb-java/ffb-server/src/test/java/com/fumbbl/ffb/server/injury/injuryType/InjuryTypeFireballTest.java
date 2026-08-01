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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_fireball.rs tests.
 * Same bonus semantics as Lightning: the Fireball +1 goes to the armour roll only when the raw
 * roll saves; when armour is already broken the bonus carries to the injury roll instead.
 */
public class InjuryTypeFireballTest {

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

	private void handleInjury(InjuryTypeFireball fireball) {
		fireball.injuryContext().setAttackerId("home1");
		fireball.injuryContext().setDefenderId("away1");
		fireball.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasArmorModifier(InjuryTypeFireball fireball, String name) {
		return Arrays.stream(fireball.injuryContext().getArmorModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	private boolean hasInjuryModifier(InjuryTypeFireball fireball, String name) {
		return Arrays.stream(fireball.injuryContext().getInjuryModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFireball fireball = new InjuryTypeFireball();
		handleInjury(fireball);
		assertEquals(PlayerState.PRONE, fireball.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeFireball fireball = new InjuryTypeFireball();
		handleInjury(fireball);
		assertTrue(fireball.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, fireball.injuryContext().getInjury().getBase());
	}

	// rust: falling_down_causes_turnover_defaults_true
	@Test
	public void fallingDownCausesTurnoverDefaultsTrue() {
		assertTrue(new InjuryTypeFireball().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_fireball
	@Test
	public void sendToBoxReasonIsFireball() {
		assertEquals(SendToBoxReason.FIREBALL, new InjuryTypeFireball().sendToBoxReason());
	}

	// rust: fireball_bonus_applies_to_armor_only_when_raw_roll_saves
	@Test
	public void fireballBonusAppliesToArmorOnlyWhenRawRollSaves() {
		defender().setArmour(7);
		GameFixture.installScriptedDice(gameState, 3, 3, 1, 1); // 6 < 7; +1 fireball -> break
		InjuryTypeFireball fireball = new InjuryTypeFireball();
		handleInjury(fireball);
		assertTrue(fireball.injuryContext().isArmorBroken());
		assertTrue(hasArmorModifier(fireball, "Fireball"));
		assertFalse(hasInjuryModifier(fireball, "Fireball"),
			"bonus consumed by the armor roll must not also boost the injury roll");
	}

	// rust: fireball_bonus_applies_to_injury_when_armor_already_broken
	@Test
	public void fireballBonusAppliesToInjuryWhenArmorAlreadyBroken() {
		defender().setArmour(7);
		GameFixture.installScriptedDice(gameState, 1, 1); // injury dice only
		InjuryTypeFireball fireball = new InjuryTypeFireball();
		fireball.injuryContext().setArmorBroken(true);
		handleInjury(fireball);
		assertFalse(hasArmorModifier(fireball, "Fireball"));
		assertTrue(hasInjuryModifier(fireball, "Fireball"));
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeFireball fireball = new InjuryTypeFireball();
		handleInjury(fireball);
		assertTrue(fireball.injuryContext().isArmorBroken());
		assertTrue(hasInjuryModifier(fireball, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeFireball fireball = new InjuryTypeFireball();
		handleInjury(fireball);
		assertFalse(hasInjuryModifier(fireball, "Mighty Blow"));
	}
}
