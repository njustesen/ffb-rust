package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.commands.ClientCommandMove;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_player_move.rs fetchMoveStack /
 * fetchFromSquare tests. Home commands pass coordinates through unchanged; away commands
 * transform each coordinate to the home perspective. (updateMoveSquares / isValidMove require
 * full game state and are a separate deferred batch.)
 */
public class UtilServerPlayerMoveTest {

	private ClientCommandMove moveCommand(FieldCoordinate from, FieldCoordinate[] to) {
		return new ClientCommandMove("p1", from, to, null);
	}

	// rust: fetch_move_stack_home_command_unchanged
	@Test
	public void fetchMoveStackHomeUnchanged() {
		FieldCoordinate[] to = {new FieldCoordinate(5, 7), new FieldCoordinate(6, 7)};
		assertArrayEquals(to, UtilServerPlayerMove.fetchMoveStack(moveCommand(null, to), true));
	}

	// rust: fetch_move_stack_away_command_transforms
	@Test
	public void fetchMoveStackAwayTransforms() {
		FieldCoordinate[] to = {new FieldCoordinate(5, 7), new FieldCoordinate(6, 7)};
		FieldCoordinate[] expected = {to[0].transform(), to[1].transform()};
		assertArrayEquals(expected, UtilServerPlayerMove.fetchMoveStack(moveCommand(null, to), false));
	}

	// rust: fetch_move_stack_empty_input
	@Test
	public void fetchMoveStackEmptyInput() {
		assertEquals(0, UtilServerPlayerMove.fetchMoveStack(moveCommand(null, new FieldCoordinate[0]), true).length);
	}

	// rust: fetch_from_square_home_unchanged
	@Test
	public void fetchFromSquareHomeUnchanged() {
		FieldCoordinate from = new FieldCoordinate(10, 7);
		assertEquals(from, UtilServerPlayerMove.fetchFromSquare(moveCommand(from, new FieldCoordinate[0]), true));
	}

	// rust: fetch_from_square_away_transforms
	@Test
	public void fetchFromSquareAwayTransforms() {
		FieldCoordinate from = new FieldCoordinate(10, 7);
		assertEquals(from.transform(),
			UtilServerPlayerMove.fetchFromSquare(moveCommand(from, new FieldCoordinate[0]), false));
	}
}
