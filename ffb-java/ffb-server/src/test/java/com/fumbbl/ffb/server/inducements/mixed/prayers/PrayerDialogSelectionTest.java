package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/mixed/prayers/prayer_dialog_selection.rs
 * tests (portable subset — Rust's builder/clone/Default plumbing tests were pruned; Java is a
 * two-field immutable value class).
 */
public class PrayerDialogSelectionTest {

	// rust: get_player_id_returns_set_value
	@Test
	public void gettersReturnCtorValues() {
		GameState gameState = GameFixture.createGameState(1, RulesCollection.Rules.BB2020);
		Game game = gameState.getGame();
		Skill dodge = GameFixture.skill(game, "Dodge");
		PrayerDialogSelection selection = new PrayerDialogSelection("player1", dodge);
		assertEquals("player1", selection.getPlayerId());
		assertEquals(dodge, selection.getSkill());
	}
}
