package com.fumbbl.ffb.server;

import com.fumbbl.ffb.net.commands.ServerCommand;
import com.fumbbl.ffb.net.commands.ServerCommandGameTime;
import com.fumbbl.ffb.net.commands.ServerCommandRemovePlayer;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/game_log.rs tests.
 * ServerCommandRemovePlayer is replayable (base default); ServerCommandGameTime overrides
 * isReplayable() to false. (Java spellings: getUncommitedServerCommands / setLastCommitedCommandNr.)
 */
public class GameLogTest {

	private GameLog newLog() {
		return new GameLog(GameFixture.createGameState());
	}

	private ServerCommand replayable(int nr) {
		ServerCommandRemovePlayer c = new ServerCommandRemovePlayer("p");
		c.setCommandNr(nr);
		return c;
	}

	private ServerCommand gameTime(int nr) {
		ServerCommandGameTime c = new ServerCommandGameTime(1, 2);
		c.setCommandNr(nr);
		return c;
	}

	private int[] commandNrs(ServerCommand[] cmds) {
		int[] nrs = new int[cmds.length];
		for (int i = 0; i < cmds.length; i++) {
			nrs[i] = cmds[i].getCommandNr();
		}
		return nrs;
	}

	// rust: test_add_and_size
	@Test
	public void testAddAndSize() {
		GameLog log = newLog();
		assertEquals(0, log.size());
		log.add(replayable(1));
		log.add(replayable(2));
		assertEquals(2, log.size());
	}

	// rust: add_skips_non_replayable_commands
	@Test
	public void addSkipsNonReplayableCommands() {
		GameLog log = newLog();
		log.add(gameTime(1));
		assertEquals(0, log.size());
	}

	// rust: add_keeps_replayable_commands
	@Test
	public void addKeepsReplayableCommands() {
		GameLog log = newLog();
		log.add(replayable(1));
		assertEquals(1, log.size());
	}

	// rust: test_clear
	@Test
	public void testClear() {
		GameLog log = newLog();
		log.add(replayable(1));
		log.clear();
		assertEquals(0, log.size());
	}

	// rust: get_uncommitted_server_commands_filters_by_last_committed
	@Test
	public void getUncommittedServerCommandsFiltersByLastCommitted() {
		GameLog log = newLog();
		log.add(replayable(1));
		log.add(replayable(2));
		log.add(replayable(3));
		log.setLastCommitedCommandNr(1);
		assertArrayEquals(new int[]{2, 3}, commandNrs(log.getUncommitedServerCommands()));
	}

	// rust: get_uncommitted_server_commands_empty_when_all_committed
	@Test
	public void getUncommittedServerCommandsEmptyWhenAllCommitted() {
		GameLog log = newLog();
		log.add(replayable(1));
		log.setLastCommitedCommandNr(5);
		assertEquals(0, log.getUncommitedServerCommands().length);
	}

	// rust: find_max_command_nr_returns_zero_for_empty_log
	@Test
	public void findMaxCommandNrReturnsZeroForEmptyLog() {
		assertEquals(0, newLog().findMaxCommandNr());
	}

	// rust: find_max_command_nr_finds_the_highest_command_nr
	@Test
	public void findMaxCommandNrFindsTheHighestCommandNr() {
		GameLog log = newLog();
		log.add(replayable(3));
		log.add(replayable(7));
		log.add(replayable(2));
		assertEquals(7, log.findMaxCommandNr());
	}

	// rust: set_and_get_last_committed_command_nr
	@Test
	public void setAndGetLastCommittedCommandNr() {
		GameLog log = newLog();
		assertEquals(0, log.getLastCommitedCommandNr());
		log.setLastCommitedCommandNr(9);
		assertEquals(9, log.getLastCommitedCommandNr());
	}

	// rust: get_server_commands_exposes_stored_commands
	@Test
	public void getServerCommandsExposesStoredCommands() {
		GameLog log = newLog();
		log.add(replayable(1));
		log.add(replayable(2));
		assertArrayEquals(new int[]{1, 2}, commandNrs(log.getServerCommands()));
	}
}
