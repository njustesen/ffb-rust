package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.injury.StabForSpp;
import com.fumbbl.ffb.injury.context.InjuryContext;
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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_stab_for_spp.rs tests.
 * Stab has no foul assists, so a null attacker is legal (matching the Rust None attacker).
 */
public class InjuryTypeStabForSppTest {

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

	private void initBb2016() {
		gameState = GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "away1", 5, 5);
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeStabForSpp stab) {
		stab.injuryContext().setDefenderId("away1");
		stab.handleInjury(step, game, gameState, gameState.getDiceRoller(), null, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifierContaining(InjuryContext context, String namePart) {
		return Arrays.stream(context.getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	private boolean hasArmorModifier(InjuryContext context, String name) {
		return Arrays.stream(context.getArmorModifiers()).anyMatch(m -> name.equals(m.getName()));
	}

	// rust: armor_save_leaves_no_injury
	@Test
	public void armorSaveLeavesNoInjury() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 6, 6); // 12 < 13 -> save
		InjuryTypeStabForSpp stab = new InjuryTypeStabForSpp(true);
		handleInjury(stab);
		assertFalse(stab.injuryContext().isArmorBroken());
		assertNull(stab.injuryContext().getInjury());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1); // break; injury 2 -> stunned
		InjuryTypeStabForSpp stab = new InjuryTypeStabForSpp(true);
		handleInjury(stab);
		assertTrue(stab.injuryContext().isArmorBroken());
		assertNotNull(stab.injuryContext().getInjury());
	}

	// rust: initial_context_has_no_injury
	@Test
	public void initialContextHasNoInjury() {
		InjuryTypeStabForSpp stab = new InjuryTypeStabForSpp(true);
		assertFalse(stab.injuryContext().isArmorBroken());
		assertNull(stab.injuryContext().getInjury());
	}

	// rust: niggling_injury_modifier_applied_when_armor_breaks
	@Test
	public void nigglingInjuryModifierAppliedWhenArmorBreaks() {
		// BB2016 game: only the bb2016 InjuryModifiers collection carries niggling modifiers
		// (bb2020/bb2025 have none) — mirrors the Rust helper's explicit Bb2016 rules.
		initBb2016();
		defender().setArmour(2);
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE); // NI attribute
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeStabForSpp stab = new InjuryTypeStabForSpp(true);
		handleInjury(stab);
		assertTrue(stab.injuryContext().isArmorBroken());
		assertTrue(hasInjuryModifierContaining(stab.injuryContext(), "Niggling"),
			"expected a niggling injury modifier");
	}

	// rust: no_niggling_injury_no_modifier
	@Test
	public void noNigglingInjuryNoModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeStabForSpp stab = new InjuryTypeStabForSpp(true);
		handleInjury(stab);
		assertTrue(stab.injuryContext().isArmorBroken());
		assertFalse(hasInjuryModifierContaining(stab.injuryContext(), "Niggling"));
	}

	// rust: use_injury_modifiers_false_skips_niggling_modifier_even_when_present
	@Test
	public void useInjuryModifiersFalseSkipsNigglingModifierEvenWhenPresent() {
		initBb2016();
		defender().setArmour(2);
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeStabForSpp stab = new InjuryTypeStabForSpp(false); // bb2016 StabBehaviour variant
		handleInjury(stab);
		assertTrue(stab.injuryContext().isArmorBroken());
		assertEquals(0, stab.injuryContext().getInjuryModifiers().length,
			"no injury modifiers with useInjuryModifiers=false");
	}

	// rust: add_defender_chainsaw_true_applies_chainsaw_armor_modifier
	@Test
	public void addDefenderChainsawTrueAppliesChainsawArmorModifier() {
		defender().setArmour(8);
		defender().addSkill(GameFixture.skill(game, "Chainsaw"));
		GameFixture.installScriptedDice(gameState, 3, 3, 1, 1); // 6+3 >= 8 -> break
		InjuryTypeStabForSpp stab = new InjuryTypeStabForSpp(true, true); // StepTreacherous variant
		handleInjury(stab);
		assertTrue(hasArmorModifier(stab.injuryContext(), "Chainsaw"));
	}

	// rust: add_defender_chainsaw_false_does_not_apply_chainsaw_modifier
	@Test
	public void addDefenderChainsawFalseDoesNotApplyChainsawModifier() {
		defender().setArmour(8);
		defender().addSkill(GameFixture.skill(game, "Chainsaw"));
		GameFixture.installScriptedDice(gameState, 3, 3); // 6 < 8 -> save (no injury dice needed)
		InjuryTypeStabForSpp stab = new InjuryTypeStabForSpp(true);
		handleInjury(stab);
		assertFalse(hasArmorModifier(stab.injuryContext(), "Chainsaw"));
	}

	// rust: send_to_box_reason_is_stabbed
	@Test
	public void sendToBoxReasonIsStabbed() {
		assertEquals(SendToBoxReason.STABBED, new InjuryTypeStabForSpp(true).sendToBoxReason());
	}

	// rust: falling_down_causes_turnover_defaults_true
	@Test
	public void fallingDownCausesTurnoverDefaultsTrue() {
		assertTrue(new StabForSpp().fallingDownCausesTurnover());
	}

	// rust: failed_armour_places_prone_is_false
	@Test
	public void failedArmourPlacesProneIsFalse() {
		// The InjuryTypeStab constructor calls setFailedArmourPlacesProne(false) on its Stab.
		StabForSpp stab = new StabForSpp();
		assertTrue(stab.failedArmourPlacesProne(), "base default is true");
		new InjuryTypeStabForSpp(true); // constructor side effect applies to its own instance
		InjuryTypeStabForSpp server = new InjuryTypeStabForSpp(true);
		assertFalse(server.injuryType().failedArmourPlacesProne());
	}
}
