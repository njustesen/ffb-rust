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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_fumbled_ktm.rs tests.
 * A fumbled kicked-team-mate landing: armour bypassed; injury modifiers are filtered to those
 * registered to affectsEitherArmourOrInjuryOnBlock — keeping Mighty Blow, excluding niggling.
 */
public class InjuryTypeFumbledKtmTest {

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

	private void handleInjury(InjuryTypeFumbledKtm fumbledKtm) {
		fumbledKtm.injuryContext().setAttackerId("home1");
		fumbledKtm.injuryContext().setDefenderId("away1");
		fumbledKtm.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(),
			defender(), new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypeFumbledKtm fumbledKtm, String namePart) {
		return Arrays.stream(fumbledKtm.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_always_broken_and_injury_set
	@Test
	public void armorAlwaysBrokenAndInjurySet() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFumbledKtm fumbledKtm = new InjuryTypeFumbledKtm();
		handleInjury(fumbledKtm);
		assertTrue(fumbledKtm.injuryContext().isArmorBroken());
		assertNotNull(fumbledKtm.injuryContext().getInjury());
		assertNotEquals(PlayerState.PRONE, fumbledKtm.injuryContext().getInjury().getBase());
	}

	// rust: stun_is_ko
	@Test
	public void stunIsKo() {
		assertTrue(new InjuryTypeFumbledKtm().stunIsTreatedAsKo());
	}

	// rust: send_to_box_reason_is_kicked
	@Test
	public void sendToBoxReasonIsKicked() {
		assertEquals(SendToBoxReason.KICKED, new InjuryTypeFumbledKtm().sendToBoxReason());
	}

	// rust: cannot_apo_ko_into_stun
	@Test
	public void cannotApoKoIntoStun() {
		assertFalse(new InjuryTypeFumbledKtm().injuryType().canApoKoIntoStun());
	}

	// rust: causes_turnover_by_default
	@Test
	public void causesTurnoverByDefault() {
		assertTrue(new InjuryTypeFumbledKtm().injuryType().fallingDownCausesTurnover());
	}

	// rust: niggling_injury_modifier_reachable_but_filtered_by_block_property
	@Test
	public void nigglingInjuryModifierReachableButFilteredByBlockProperty() {
		init(GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFumbledKtm fumbledKtm = new InjuryTypeFumbledKtm();
		handleInjury(fumbledKtm);
		assertTrue(fumbledKtm.injuryContext().isArmorBroken());
		assertFalse(hasInjuryModifier(fumbledKtm, "Niggling"),
			"niggling modifiers are not registered to affectsEitherArmourOrInjuryOnBlock");
	}

	// rust: mighty_blow_kept_by_block_property_filter (bug #9: MightyBlow registers
	// affectsEitherArmourOrInjuryOnBlock, so the filter KEEPS it)
	@Test
	public void mightyBlowKeptByBlockPropertyFilter() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeFumbledKtm fumbledKtm = new InjuryTypeFumbledKtm();
		handleInjury(fumbledKtm);
		assertTrue(fumbledKtm.injuryContext().isArmorBroken());
		assertTrue(hasInjuryModifier(fumbledKtm, "Mighty Blow"));
	}
}
