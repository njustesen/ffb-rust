package com.fumbbl.ffb.util.pathfinding;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.FieldCoordinateBounds;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/util/pathfinding/path_finder_with_multi_jump.rs.
 * find_adjacent tests exercise FieldModel.findAdjacentCoordinates (same neighbour generation).
 */
public class PathFinderWithMultiJumpTest {

	private Game game;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
		game.getTeamHome().setId("home");
		game.getTeamAway().setId("away");
	}

	// rust: find_adjacent_distance_1_returns_up_to_8
	@Test
	public void findAdjacentDistance1ReturnsUpTo8() {
		assertEquals(8, game.getFieldModel()
			.findAdjacentCoordinates(new FieldCoordinate(5, 5), FieldCoordinateBounds.FIELD, 1, false).length);
	}

	// rust: find_adjacent_distance_2_returns_up_to_24
	@Test
	public void findAdjacentDistance2ReturnsUpTo24() {
		assertEquals(24, game.getFieldModel()
			.findAdjacentCoordinates(new FieldCoordinate(5, 5), FieldCoordinateBounds.FIELD, 2, false).length);
	}

	// rust: find_adjacent_clips_at_field_edge
	@Test
	public void findAdjacentClipsAtFieldEdge() {
		assertEquals(3, game.getFieldModel()
			.findAdjacentCoordinates(new FieldCoordinate(0, 0), FieldCoordinateBounds.FIELD, 1, false).length);
	}

	// rust: get_path_to_blitz_target_returns_none_without_acting_player
	@Test
	public void getPathToBlitzTargetReturnsNullWithoutActingPlayer() {
		RosterPlayer target = new RosterPlayer();
		target.setId("a1");
		game.getTeamAway().addPlayer(target);
		game.getFieldModel().setPlayerCoordinate(target, new FieldCoordinate(5, 5));
		game.getFieldModel().setPlayerState(target, new PlayerState(0x101));
		assertNull(PathFinderWithMultiJump.INSTANCE.getPathToBlitzTarget(game, target));
	}
}
