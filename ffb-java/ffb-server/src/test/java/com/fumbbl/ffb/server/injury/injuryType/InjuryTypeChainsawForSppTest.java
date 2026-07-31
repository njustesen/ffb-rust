package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_chainsaw_for_spp.rs tests.
 * The chainsaw kickback has no attacker (null is legal); the +3 chainsaw armour modifier is
 * sourced from the skill factory inside armourRoll.
 */
public class InjuryTypeChainsawForSppTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "away1", 5, 5); // defender
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeChainsawForSpp chainsaw) {
		chainsaw.injuryContext().setDefenderId("away1");
		chainsaw.handleInjury(step, game, gameState, gameState.getDiceRoller(), null, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private void armourRoll(InjuryTypeChainsawForSpp chainsaw) {
		chainsaw.injuryContext().setDefenderId("away1");
		chainsaw.armourRoll(game, gameState, gameState.getDiceRoller(), null, defender(),
			DiceInterpreter.getInstance(), chainsaw.injuryContext(), true);
	}

	private void injuryRoll(InjuryTypeChainsawForSpp chainsaw) {
		chainsaw.injuryContext().setDefenderId("away1");
		chainsaw.injuryContext().setArmorBroken(true);
		chainsaw.injuryRoll(game, gameState, gameState.getDiceRoller(), null, defender(),
			chainsaw.injuryContext());
	}

	private long chainsawModifierCount(InjuryContext context) {
		return Arrays.stream(context.getArmorModifiers())
			.filter(m -> "Chainsaw".equals(m.getName())).count();
	}

	// rust: armor_save_leaves_no_injury
	@Test
	public void armorSaveLeavesNoInjury() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1); // 2 + 3 chainsaw < 13 -> save
		InjuryTypeChainsawForSpp chainsaw = new InjuryTypeChainsawForSpp();
		handleInjury(chainsaw);
		assertFalse(chainsaw.injuryContext().isArmorBroken());
		assertNull(chainsaw.injuryContext().getInjury());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1); // break; injury 2 -> stunned
		InjuryTypeChainsawForSpp chainsaw = new InjuryTypeChainsawForSpp();
		handleInjury(chainsaw);
		assertTrue(chainsaw.injuryContext().isArmorBroken());
		assertNotNull(chainsaw.injuryContext().getInjury());
	}

	// rust: chainsaw_armor_modifier_added_to_roll
	@Test
	public void chainsawArmorModifierAddedToRoll() {
		defender().setArmour(7);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeChainsawForSpp chainsaw = new InjuryTypeChainsawForSpp();
		armourRoll(chainsaw);
		assertEquals(1, chainsawModifierCount(chainsaw.injuryContext()));
	}

	// rust: chainsaw_modifier_not_duplicated_if_already_present
	@Test
	public void chainsawModifierNotDuplicatedIfAlreadyPresent() {
		defender().setArmour(7);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeChainsawForSpp chainsaw = new InjuryTypeChainsawForSpp();
		GameFixture.skill(game, "Chainsaw").getArmorModifiers()
			.forEach(chainsaw.injuryContext()::addArmorModifier);
		armourRoll(chainsaw);
		assertEquals(1, chainsawModifierCount(chainsaw.injuryContext()));
	}

	// rust: stunty_defender_uses_stunty_injury_table
	// (deterministic Java version: scripted injury roll 3+4 = 7 with a Stunty defender -> KO)
	@Test
	public void stuntyDefenderUsesStuntyInjuryTable() {
		defender().setArmour(2);
		defender().addSkill(GameFixture.skill(game, "Stunty"));
		GameFixture.installScriptedDice(gameState, 3, 4); // total 7 + Stunty -> KO in BB2025
		InjuryTypeChainsawForSpp chainsaw = new InjuryTypeChainsawForSpp();
		injuryRoll(chainsaw);
		assertEquals(PlayerState.KNOCKED_OUT, chainsaw.injuryContext().getInjury().getBase());
	}

	// rust: is_caused_by_opponent_is_true
	@Test
	public void isCausedByOpponentIsTrue() {
		assertTrue(new InjuryTypeChainsawForSpp().injuryType().isCausedByOpponent());
	}

	// rust: falling_down_causes_turnover_defaults_true
	@Test
	public void fallingDownCausesTurnoverDefaultsTrue() {
		assertTrue(new InjuryTypeChainsawForSpp().injuryType().fallingDownCausesTurnover());
	}

	// rust: is_worth_spps_is_true_for_spp_variant
	@Test
	public void isWorthSppsIsTrueForSppVariant() {
		assertTrue(new InjuryTypeChainsawForSpp().injuryType().isWorthSpps());
	}

	// rust: failed_armour_places_prone_is_false
	@Test
	public void failedArmourPlacesProneIsFalse() {
		assertFalse(new InjuryTypeChainsawForSpp().injuryType().failedArmourPlacesProne());
	}

	// rust: send_to_box_reason_is_chainsaw
	@Test
	public void sendToBoxReasonIsChainsaw() {
		assertEquals(SendToBoxReason.CHAINSAW, new InjuryTypeChainsawForSpp().sendToBoxReason());
	}

	// rust: iron_hard_skin_defender_suppresses_chainsaw_modifier
	@Test
	public void ironHardSkinDefenderSuppressesChainsawModifier() {
		defender().setArmour(7);
		defender().addSkill(GameFixture.skill(game, "Iron Hard Skin"));
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeChainsawForSpp chainsaw = new InjuryTypeChainsawForSpp();
		armourRoll(chainsaw);
		assertEquals(0, chainsawModifierCount(chainsaw.injuryContext()));
	}
}
