package com.fumbbl.ffb.server.mechanic.bb2020;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.bb2020.SeriousInjury;
import com.fumbbl.ffb.injury.context.InjuryContext;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/mechanic/bb2020/roll_mechanic.rs tests.
 * SI detail-table tests use a default-stat defender (every stat reduceable), matching the
 * Rust twins — Java dereferences the defender in the reduceable-stat filter.
 */
public class RollMechanicTest {

	private GameState gameState;
	private Game game;
	private RollMechanic mechanic;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		mechanic = new RollMechanic();
	}

	// rust: roll_casualty_bb2020_range
	@Test
	public void rollCasualtyBb2020Range() {
		for (int i = 0; i < 20; i++) {
			int[] roll = mechanic.rollCasualty(gameState.getDiceRoller());
			assertTrue(roll[0] >= 1 && roll[0] <= 16, "d16 in range, got " + roll[0]);
			assertTrue(roll[1] >= 1 && roll[1] <= 6, "d6 in range, got " + roll[1]);
		}
	}

	// rust: multi_block_modifiers_bb2020
	@Test
	public void multiBlockModifiersBb2020() {
		assertEquals(-2, mechanic.multiBlockAttackerModifier());
		assertEquals(0, mechanic.multiBlockDefenderModifier());
	}

	// rust: minimum_pro_roll_is_3_bb2020
	@Test
	public void minimumProRollIs3Bb2020() {
		assertEquals(3, mechanic.minimumProRoll());
	}

	// rust: allows_re_roll_bb2020_prohibited
	@Test
	public void allowsReRollBb2020Prohibited() {
		for (TurnMode mode : new TurnMode[]{TurnMode.KICKOFF, TurnMode.PASS_BLOCK, TurnMode.DUMP_OFF,
			TurnMode.BLITZ, TurnMode.QUICK_SNAP, TurnMode.BETWEEN_TURNS}) {
			assertFalse(mechanic.allowsTeamReRoll(mode), mode + " should be prohibited");
		}
	}

	// rust: allows_re_roll_bb2020_allowed
	@Test
	public void allowsReRollBb2020Allowed() {
		assertTrue(mechanic.allowsTeamReRoll(TurnMode.REGULAR));
	}

	// rust: no_additional_reroll_bb2020
	@Test
	public void noAdditionalRerollBb2020() {
		assertEquals(Optional.empty(), mechanic.findAdditionalReRollProperty(game.getTurnDataHome()));
	}

	// rust: casualty_badly_hurt_bb2020
	@Test
	public void casualtyBadlyHurtBb2020() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{6, 1});
		PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
			game, context, game.getPlayerById("home1"), false);
		assertEquals(PlayerState.BADLY_HURT, result.getBase());
	}

	// rust: casualty_serious_injury_at_7_bb2020
	@Test
	public void casualtySeriousInjuryAt7Bb2020() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{7, 1});
		PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
			game, context, game.getPlayerById("home1"), false);
		assertEquals(PlayerState.SERIOUS_INJURY, result.getBase());
	}

	// rust: casualty_rip_at_15_bb2020
	@Test
	public void casualtyRipAt15Bb2020() {
		InjuryContext context = new InjuryContext();
		context.setCasualtyRoll(new int[]{15, 2});
		PlayerState result = mechanic.interpretCasualtyRollAndAddModifiers(
			game, context, game.getPlayerById("home1"), false);
		assertEquals(PlayerState.RIP, result.getBase());
	}

	// rust: si_roll_seriously_hurt_7_to_9_bb2020
	@Test
	public void siRollSeriouslyHurt7To9Bb2020() {
		InjuryContext context = new InjuryContext();
		for (int cas : new int[]{7, 8, 9}) {
			context.setCasualtyRoll(new int[]{cas, 1});
			assertEquals(SeriousInjury.SERIOUSLY_HURT, mechanic.interpretSeriousInjuryRoll(game, context),
				"cas=" + cas);
		}
	}

	// rust: si_roll_serious_injury_ni_10_to_12_bb2020
	@Test
	public void siRollSeriousInjuryNi10To12Bb2020() {
		InjuryContext context = new InjuryContext();
		for (int cas : new int[]{10, 11, 12}) {
			context.setCasualtyRoll(new int[]{cas, 1});
			assertEquals(SeriousInjury.SERIOUS_INJURY, mechanic.interpretSeriousInjuryRoll(game, context),
				"cas=" + cas);
		}
	}

	// rust: si_roll_detail_table_d6_1_is_head_injury_bb2020
	@Test
	public void siRollDetailTableD61IsHeadInjuryBb2020() {
		InjuryContext context = new InjuryContext();
		context.setDefenderId("away1");
		context.setCasualtyRoll(new int[]{13, 1});
		assertEquals(SeriousInjury.HEAD_INJURY, mechanic.interpretSeriousInjuryRoll(game, context));
	}

	// rust: si_roll_detail_table_d6_6_is_dislocated_shoulder_bb2020
	@Test
	public void siRollDetailTableD66IsDislocatedShoulderBb2020() {
		InjuryContext context = new InjuryContext();
		context.setDefenderId("away1");
		context.setCasualtyRoll(new int[]{14, 6});
		assertEquals(SeriousInjury.DISLOCATED_SHOULDER, mechanic.interpretSeriousInjuryRoll(game, context));
	}

	// rust: injury_roll_forced_returns_preset_bb2020
	@Test
	public void injuryRollForcedReturnsPresetBb2020() {
		InjuryContext context = new InjuryContext();
		context.setInjury(new PlayerState(PlayerState.STUNNED));
		assertEquals(PlayerState.STUNNED, mechanic.interpretInjuryRoll(game, context).getBase());
	}

	// rust: injury_roll_total_2_stunned_bb2020
	@Test
	public void injuryRollTotal2StunnedBb2020() {
		InjuryContext context = new InjuryContext();
		context.setInjuryRoll(new int[]{1, 1});
		assertEquals(PlayerState.STUNNED, mechanic.interpretInjuryRoll(game, context).getBase());
	}

	// rust: injury_roll_total_10_casualty_bb2020
	@Test
	public void injuryRollTotal10CasualtyBb2020() {
		InjuryContext context = new InjuryContext();
		context.setInjuryRoll(new int[]{5, 5});
		assertNull(mechanic.interpretInjuryRoll(game, context));
	}
}
