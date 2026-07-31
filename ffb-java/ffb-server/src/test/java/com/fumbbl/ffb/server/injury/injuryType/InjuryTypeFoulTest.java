package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.injury.Foul;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.DiceInterpreter;
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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_foul.rs tests.
 * Same package as InjuryTypeServer to reach the protected armourRoll/injuryRoll.
 * Dice are scripted: armour and injury rolls are 2d6 each.
 */
public class InjuryTypeFoulTest {

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

	private Player<?> attacker() {
		return game.getPlayerById("home1");
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void addSkill(Player<?> player, String skillName) {
		((RosterPlayer) player).addSkill(GameFixture.skill(game, skillName));
	}

	private void setContextIds(InjuryContext context) {
		context.setAttackerId("home1");
		context.setDefenderId("away1");
	}

	private boolean hasArmorModifier(InjuryContext context, String name) {
		return Arrays.stream(context.getArmorModifiers()).anyMatch(m -> name.equals(m.getName()));
	}

	private boolean hasInjuryModifier(InjuryContext context, String name) {
		return Arrays.stream(context.getInjuryModifiers()).anyMatch(m -> name.equals(m.getName()));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 6, 6); // 12 <= 13 -> save
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
		assertEquals(PlayerState.PRONE, foul.injuryContext().getInjury().getBase());
	}

	// rust: should_play_fall_sound_is_false
	@Test
	public void shouldPlayFallSoundIsFalse() {
		assertFalse(new Foul().shouldPlayFallSound());
	}

	// rust: send_to_box_reason_is_fouled
	@Test
	public void sendToBoxReasonIsFouled() {
		assertEquals(SendToBoxReason.FOULED, new InjuryTypeFoul(false).sendToBoxReason());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1); // armour 12 > 2 -> break; injury 2 -> stunned
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
		assertTrue(foul.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, foul.injuryContext().getInjury().getBase());
	}

	// rust: dirty_player_adds_armor_modifier
	@Test
	public void dirtyPlayerAddsArmorModifier() {
		addSkill(attacker(), "Dirty Player");
		defender().setArmour(9);
		GameFixture.installScriptedDice(gameState, 1, 1); // 2 < 9 -> not broken, reach findArmorModifiers
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.armourRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			DiceInterpreter.getInstance(), foul.injuryContext(), true);
		assertTrue(hasArmorModifier(foul.injuryContext(), "Dirty Player"));
	}

	// rust: no_dirty_player_no_armor_modifier
	@Test
	public void noDirtyPlayerNoArmorModifier() {
		defender().setArmour(9);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.armourRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			DiceInterpreter.getInstance(), foul.injuryContext(), true);
		assertFalse(hasArmorModifier(foul.injuryContext(), "Dirty Player"));
	}

	// rust: dirty_player_adds_injury_modifier
	@Test
	public void dirtyPlayerAddsInjuryModifier() {
		addSkill(attacker(), "Dirty Player");
		GameFixture.installScriptedDice(gameState, 1, 1); // injury 2 -> stunned, no casualty dice needed
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.injuryContext().setArmorBroken(true);
		foul.injuryRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			foul.injuryContext());
		assertTrue(hasInjuryModifier(foul.injuryContext(), "Dirty Player"));
	}

	// rust: no_dirty_player_no_injury_modifier
	@Test
	public void noDirtyPlayerNoInjuryModifier() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.injuryContext().setArmorBroken(true);
		foul.injuryRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			foul.injuryContext());
		assertFalse(hasInjuryModifier(foul.injuryContext(), "Dirty Player"));
	}

	// rust: blatant_foul_card_sets_armor_broken
	@Test
	public void blatantFoulCardSetsArmorBroken() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		defender().setArmour(13);
		Card blatantFoul = (Card) game.getFactory(FactoryType.Factory.CARD).forName("Blatant Foul");
		game.getTurnDataHome().getInducementSet().addAvailableCard(blatantFoul);
		game.getTurnDataHome().getInducementSet().activateCard(blatantFoul);
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.armourRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			DiceInterpreter.getInstance(), foul.injuryContext(), true);
		assertTrue(foul.injuryContext().isArmorBroken());
	}

	// rust: no_blatant_foul_card_no_forced_armor_broken
	@Test
	public void noBlatantFoulCardNoForcedArmorBroken() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1); // 2 never breaks armour 13
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.armourRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			DiceInterpreter.getInstance(), foul.injuryContext(), true);
		assertFalse(foul.injuryContext().isArmorBroken());
	}

	// rust: chainsaw_foul_adds_chainsaw_modifier
	@Test
	public void chainsawFoulAddsChainsawModifier() {
		addSkill(attacker(), "Chainsaw");
		defender().setArmour(9);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFoul foul = new InjuryTypeFoul(true);
		setContextIds(foul.injuryContext());
		foul.armourRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			DiceInterpreter.getInstance(), foul.injuryContext(), true);
		assertTrue(hasArmorModifier(foul.injuryContext(), "Chainsaw"));
	}

	// rust: chainsaw_foul_no_chainsaw_on_attacker_no_modifier
	@Test
	public void chainsawFoulNoChainsawOnAttackerNoModifier() {
		defender().setArmour(9);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFoul foul = new InjuryTypeFoul(true);
		setContextIds(foul.injuryContext());
		foul.armourRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			DiceInterpreter.getInstance(), foul.injuryContext(), true);
		assertFalse(hasArmorModifier(foul.injuryContext(), "Chainsaw"));
	}

	// rust: non_chainsaw_foul_no_chainsaw_modifier
	@Test
	public void nonChainsawFoulNoChainsawModifier() {
		addSkill(attacker(), "Chainsaw");
		defender().setArmour(9);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFoul foul = new InjuryTypeFoul(false);
		setContextIds(foul.injuryContext());
		foul.armourRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			DiceInterpreter.getInstance(), foul.injuryContext(), true);
		assertFalse(hasArmorModifier(foul.injuryContext(), "Chainsaw"));
	}

	// rust: iron_hard_skin_defender_blocks_chainsaw_modifier
	@Test
	public void ironHardSkinDefenderBlocksChainsawModifier() {
		addSkill(attacker(), "Chainsaw");
		addSkill(defender(), "Iron Hard Skin");
		defender().setArmour(9);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFoul foul = new InjuryTypeFoul(true);
		setContextIds(foul.injuryContext());
		foul.armourRoll(game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			DiceInterpreter.getInstance(), foul.injuryContext(), true);
		assertFalse(hasArmorModifier(foul.injuryContext(), "Chainsaw"));
	}
}
