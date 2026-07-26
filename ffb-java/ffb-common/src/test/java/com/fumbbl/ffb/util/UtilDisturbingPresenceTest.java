package com.fumbbl.ffb.util;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-model/src/util/util_disturbing_presence.rs tests.
 * Disturbing Presence skill loaded via NetCommandTestUtil.gameSource()'s SkillFactory.
 */
public class UtilDisturbingPresenceTest {

	private static final int ACTIVE_STANDING = 0x101;

	private Game game;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
		game.getTeamHome().setId("home");
		game.getTeamAway().setId("away");
	}

	private RosterPlayer addPlayer(boolean home, String id, FieldCoordinate coord) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		(home ? game.getTeamHome() : game.getTeamAway()).addPlayer(p);
		game.getFieldModel().setPlayerCoordinate(p, coord);
		game.getFieldModel().setPlayerState(p, new PlayerState(ACTIVE_STANDING));
		return p;
	}

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	// rust: no_opposing_players_returns_zero
	@Test
	public void noOpposingPlayersReturnsZero() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5));
		assertEquals(0, UtilDisturbingPresence.findOpposingDisturbingPresences(game, h1));
	}

	// rust: opposing_player_without_skill_not_counted
	@Test
	public void opposingPlayerWithoutSkillNotCounted() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5));
		addPlayer(false, "a1", new FieldCoordinate(6, 5));
		assertEquals(0, UtilDisturbingPresence.findOpposingDisturbingPresences(game, h1));
	}

	// rust: opposing_player_with_skill_within_3_steps_counted
	@Test
	public void opposingPlayerWithSkillWithin3StepsCounted() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5));
		RosterPlayer a1 = addPlayer(false, "a1", new FieldCoordinate(7, 5));
		a1.addSkill(skill("Disturbing Presence"));
		assertEquals(1, UtilDisturbingPresence.findOpposingDisturbingPresences(game, h1));
	}

	// rust: opposing_player_with_skill_beyond_3_steps_not_counted
	@Test
	public void opposingPlayerWithSkillBeyond3StepsNotCounted() {
		Player<?> h1 = addPlayer(true, "h1", new FieldCoordinate(5, 5));
		RosterPlayer a1 = addPlayer(false, "a1", new FieldCoordinate(10, 5));
		a1.addSkill(skill("Disturbing Presence"));
		assertEquals(0, UtilDisturbingPresence.findOpposingDisturbingPresences(game, h1));
	}

	// NOTE (test equalization): rust player_without_skill_not_counted passes an empty/unfindable
	// player id; Rust defends nulls and returns 0, but Java's findOtherTeam(game, player) derefs
	// player.getTeam() and would NPE on a teamless player — Rust-only defensive-null case.
}
