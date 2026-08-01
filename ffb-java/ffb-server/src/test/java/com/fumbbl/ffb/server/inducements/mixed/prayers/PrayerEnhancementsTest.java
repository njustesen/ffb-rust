package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2020.Prayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/prayer_player_effect.rs
 * tests. Java side: Prayer.enhancements(mechanic) applied via
 * FieldModel.addPrayerEnhancements / removed via removePrayerEnhancements.
 */
public class PrayerEnhancementsTest {

	private GameState gameState;
	private Game game;
	private RosterPlayer player;
	private int baseMovement;
	private int baseArmour;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		player = (RosterPlayer) game.getPlayerById("home1");
		baseMovement = player.getMovementWithModifiers();
		baseArmour = player.getArmourWithModifiers();
	}

	private boolean hasTempSkill(String skillName) {
		return player.getSkillsIncludingTemporaryOnes().stream()
			.anyMatch(s -> skillName.equals(s.getName()));
	}

	// rust: greasy_cleats_reduces_movement_by_one
	@Test
	public void greasyCleatsReducesMovementByOne() {
		game.getFieldModel().addPrayerEnhancements(player, Prayer.GREASY_CLEATS);
		assertEquals(baseMovement - 1, player.getMovementWithModifiers());
		assertEquals(baseArmour, player.getArmourWithModifiers());
	}

	// rust: iron_man_increases_armour_by_one
	@Test
	public void ironManIncreasesArmourByOne() {
		game.getFieldModel().addPrayerEnhancements(player, Prayer.IRON_MAN);
		assertEquals(baseArmour + 1, player.getArmourWithModifiers());
		assertEquals(baseMovement, player.getMovementWithModifiers());
	}

	// rust: stiletto_grants_stab_skill
	@Test
	public void stilettoGrantsStabSkill() {
		game.getFieldModel().addPrayerEnhancements(player, Prayer.STILETTO);
		assertTrue(hasTempSkill("Stab"));
	}

	// rust: bad_habits_grants_loner_with_2_value
	@Test
	public void badHabitsGrantsLonerWith2Value() {
		game.getFieldModel().addPrayerEnhancements(player, Prayer.BAD_HABITS);
		assertTrue(hasTempSkill("Loner"));
		assertTrue(player.temporarySkillValues(GameFixture.skill(game, "Loner")).contains("2"));
	}

	// rust: knuckle_dusters_grants_mighty_blow
	@Test
	public void knuckleDustersGrantsMightyBlow() {
		game.getFieldModel().addPrayerEnhancements(player, Prayer.KNUCKLE_DUSTERS);
		assertTrue(hasTempSkill("Mighty Blow"));
	}

	// rust: remove_prayer_player_effect_undoes_stat_mod
	@Test
	public void removePrayerEnhancementsUndoesStatMod() {
		game.getFieldModel().addPrayerEnhancements(player, Prayer.GREASY_CLEATS);
		assertEquals(baseMovement - 1, player.getMovementWithModifiers());
		game.getFieldModel().removePrayerEnhancements(player, Prayer.GREASY_CLEATS);
		assertEquals(baseMovement, player.getMovementWithModifiers());
	}

	// rust: remove_prayer_player_effect_undoes_skill_grant
	@Test
	public void removePrayerEnhancementsUndoesSkillGrant() {
		game.getFieldModel().addPrayerEnhancements(player, Prayer.STILETTO);
		assertTrue(hasTempSkill("Stab"));
		game.getFieldModel().removePrayerEnhancements(player, Prayer.STILETTO);
		assertFalse(hasTempSkill("Stab"));
	}

	// rust: unknown_prayer_is_noop (prayers without player enhancements change nothing)
	@Test
	public void prayerWithoutEnhancementsIsNoop() {
		game.getFieldModel().addPrayerEnhancements(player, Prayer.TREACHEROUS_TRAPDOOR);
		assertEquals(baseMovement, player.getMovementWithModifiers());
		assertEquals(baseArmour, player.getArmourWithModifiers());
	}
}
