package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.modifiers.ArmorModifier;
import com.fumbbl.ffb.modifiers.ArmorModifierContext;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/modifiers/armor_modifier_factory.rs tests (subset).
 * The factory is initialized against a GameFixture (BB2025) game.
 */
public class ArmorModifierFactoryTest {

	private Game game;
	private ArmorModifierFactory factory;

	@BeforeEach
	void setUp() {
		game = GameFixture.createGameState().getGame();
		factory = new ArmorModifierFactory();
		factory.initialize(game);
	}

	private RosterPlayer player(String id) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		return p;
	}

	// rust: for_name_finds_foul_assist
	@Test
	public void forNameFindsFoulAssist() {
		assertNotNull(factory.forName("1 Offensive Assist"));
	}

	// rust: for_name_returns_none_for_unknown
	@Test
	public void forNameReturnsNoneForUnknown() {
		assertNull(factory.forName("Unknown Modifier"));
	}

	// rust: get_foul_assist_returns_matching_modifier
	@Test
	public void getFoulAssistReturnsMatchingModifier() {
		ArmorModifierContext ctx =
			new ArmorModifierContext(game, player("a"), player("d"), false, true, 3);
		Set<ArmorModifier> mods = factory.getFoulAssist(ctx);
		assertEquals(1, mods.size());
	}

	// rust: get_foul_assist_defensive_assist
	@Test
	public void getFoulAssistDefensiveAssist() {
		ArmorModifierContext ctx =
			new ArmorModifierContext(game, player("a"), player("d"), false, true, -2);
		Set<ArmorModifier> mods = factory.getFoulAssist(ctx);
		assertEquals(1, mods.size());
	}
}
