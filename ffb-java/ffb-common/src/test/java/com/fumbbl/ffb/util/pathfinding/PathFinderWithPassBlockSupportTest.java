package com.fumbbl.ffb.util.pathfinding;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.FieldCoordinateBounds;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/util/pathfinding/path_finder_with_pass_block_support.rs.
 * The Rust find_adjacent tests exercise the PathFinder's find_adjacent_coordinates wrapper; in Java
 * that neighbour generation is FieldModel.findAdjacentCoordinates (same behavior).
 */
public class PathFinderWithPassBlockSupportTest {

	private Game game;

	@BeforeEach
	void setUp() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		game = new Game(app, app.getFactoryManager());
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

	// rust: get_shortest_path_to_coord_returns_none_without_acting_player
	@Test
	public void getShortestPathToCoordReturnsNullWithoutActingPlayer() {
		assertNull(PathFinderWithPassBlockSupport.INSTANCE.getShortestPath(game, new FieldCoordinate(5, 5)));
	}

	// rust: allow_pass_block_move_returns_empty_without_skill
	@Test
	public void allowPassBlockMoveReturnsEmptyWithoutSkill() {
		RosterPlayer player = new RosterPlayer();
		player.setId("p1");
		player.setMovement(6);
		FieldCoordinate[] result = PathFinderWithPassBlockSupport.INSTANCE.allowPassBlockMove(
			game, player, new FieldCoordinate(5, 5), 3, false, Collections.emptySet());
		assertEquals(0, result.length);
	}
}
