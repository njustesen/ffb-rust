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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_fumbled_ktm_apo_ko.rs
 * tests. The apo-KO variant: unfiltered injury modifiers, but the factory is called with a NULL
 * attacker — attacker skills never apply; niggling (defender-based) does.
 */
public class InjuryTypeFumbledKtmApoKoTest {

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

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeFumbledKtmApoKo fumbledKtm) {
		fumbledKtm.injuryContext().setAttackerId("home1");
		fumbledKtm.injuryContext().setDefenderId("away1");
		fumbledKtm.handleInjury(step, game, gameState, gameState.getDiceRoller(),
			game.getPlayerById("home1"), defender(), new FieldCoordinate(5, 5), null, null,
			ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeFumbledKtmApoKo fumbledKtm, String namePart) {
		return Arrays.stream(fumbledKtm.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_always_broken_and_injury_set
	@Test
	public void armorAlwaysBrokenAndInjurySet() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFumbledKtmApoKo fumbledKtm = new InjuryTypeFumbledKtmApoKo();
		handleInjury(fumbledKtm);
		assertTrue(fumbledKtm.injuryContext().isArmorBroken());
		assertNotNull(fumbledKtm.injuryContext().getInjury());
		assertNotEquals(PlayerState.PRONE, fumbledKtm.injuryContext().getInjury().getBase());
	}

	// rust: send_to_box_reason_is_kicked
	@Test
	public void sendToBoxReasonIsKicked() {
		assertEquals(SendToBoxReason.KICKED, new InjuryTypeFumbledKtmApoKo().sendToBoxReason());
	}

	// rust: causes_turnover_by_default
	@Test
	public void causesTurnoverByDefault() {
		assertTrue(new InjuryTypeFumbledKtmApoKo().injuryType().fallingDownCausesTurnover());
	}

	// rust: niggling_injury_modifier_applied
	@Test
	public void nigglingInjuryModifierApplied() {
		init(GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFumbledKtmApoKo fumbledKtm = new InjuryTypeFumbledKtmApoKo();
		handleInjury(fumbledKtm);
		assertTrue(hasInjuryModifier(fumbledKtm, "Niggling"));
	}

	// rust: attacker_skill_never_applies_because_attacker_passed_as_null
	@Test
	public void attackerSkillNeverAppliesBecauseAttackerPassedAsNull() {
		((RosterPlayer) game.getPlayerById("home1")).addSkill(GameFixture.skill(game, "Mighty Blow"));
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFumbledKtmApoKo fumbledKtm = new InjuryTypeFumbledKtmApoKo();
		handleInjury(fumbledKtm);
		assertFalse(hasInjuryModifier(fumbledKtm, "Mighty Blow"),
			"the factory is called with a null attacker inside handleInjury");
	}
}
