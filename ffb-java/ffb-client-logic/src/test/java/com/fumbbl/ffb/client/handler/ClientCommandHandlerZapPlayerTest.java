package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.FactoryManager;
import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerType;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.factory.INamedObjectFactory;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.GameRules;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.model.ZappedPlayer;
import com.fumbbl.ffb.net.commands.ServerCommandZapPlayer;
import com.fumbbl.ffb.option.GameOptionId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;

/**
 * Mirrors the Rust tests in
 * crates/ffb-client/src/client/handler/client_command_handler_zap_player.rs.
 *
 * <p>The Rust tests operate on a plain {@code &mut Game} model and assert
 * Rust-only state ({@code game.zapped_players}, {@code Player::is_zapped()}).
 * The real Java handler ({@link ClientCommandHandlerZapPlayer}) instead
 * mutates a mocked {@link FantasyFootballClient}'s {@code Team}/{@code
 * FieldModel} collaborators, so those Rust assertions are translated into
 * Mockito verifications of {@code team.addPlayer(...)} and {@code
 * fieldModel.sendPosition(...)} calls.</p>
 */
@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerZapPlayerTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private Game game;

	@Mock
	private Team team;

	@Mock
	private FieldModel fieldModel;

	private ClientCommandHandlerZapPlayer handler;

	@BeforeEach
	void setUp() {
		handler = new ClientCommandHandlerZapPlayer(client);
		given(client.getGame()).willReturn(game);
	}

	private static RosterPlayer makeRosterPlayer(String id) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		p.setName("Bob");
		p.setNr(1);
		p.setPositionId("lineman");
		p.setType(PlayerType.REGULAR);
		p.setGender(PlayerGender.MALE);
		p.setMovement(6);
		p.setStrength(3);
		p.setAgility(3);
		p.setPassing(4);
		p.setArmour(8);
		return p;
	}

	/**
	 * Builds a real (non-mocked) {@link GameRules} for the given rules version
	 * string ("BB2016"/"BB2020"/"BB2025"). {@code ZappedPlayer.init(...)} needs
	 * a working {@code IFactorySource} to resolve real skills and the
	 * per-edition zapped-player stats from {@code GameMechanic}; mocking that
	 * factory chain is impractical (six real skill lookups + a mechanic cast),
	 * so this mirrors ffb-common's test-only {@code NetCommandTestUtil}, which
	 * is not reachable from this module (no test-jar dependency wired up).
	 */
	@SuppressWarnings({"rawtypes", "unchecked"})
	private static GameRules realRules(String rulesVersion) {
		FactoryManager manager = new FactoryManager();
		IFactorySource appSource = new IFactorySource() {
			private final Map<Factory, INamedObjectFactory> factories =
					manager.getFactoriesForContext(FactoryContext.APPLICATION, this);

			@Override
			public FactoryManager getFactoryManager() {
				return manager;
			}

			@Override
			public FactoryContext getContext() {
				return FactoryContext.APPLICATION;
			}

			@Override
			public IFactorySource forContext(FactoryContext context) {
				return this;
			}

			@Override
			public <T extends INamedObjectFactory> T getFactory(Factory factory) {
				return (T) factories.get(factory);
			}

			@Override
			public void logError(long gameId, String message) {
				// no-op in tests
			}

			@Override
			public void logDebug(long gameId, String message) {
				// no-op in tests
			}

			@Override
			public void logWithOutGameId(Throwable throwable) {
				throw new IllegalStateException(throwable);
			}
		};
		Game realGame = new Game(appSource, manager);
		realGame.getOptions().addOption(
				realGame.getOptions().getFactory().createGameOption(GameOptionId.RULESVERSION).setValue(rulesVersion));
		realGame.initializeRules();
		return realGame.getRules();
	}

	@Test
	void zapsANotYetZappedPlayer() {
		RosterPlayer player = makeRosterPlayer("p1");
		given(game.getTeamById("home")).willReturn(team);
		given(team.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) player);
		given(game.getRules()).willReturn(realRules("BB2020"));
		given(game.getFieldModel()).willReturn(fieldModel);

		ServerCommandZapPlayer command = new ServerCommandZapPlayer("p1", "home");
		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(team).addPlayer(any(ZappedPlayer.class));
		verify(fieldModel).sendPosition(player);
	}

	@Test
	void doesNotZapAnAlreadyZappedPlayerAgain() {
		RosterPlayer player = makeRosterPlayer("p1");
		given(game.getTeamById("home")).willReturn(team);
		given(game.getRules()).willReturn(realRules("BB2020"));
		given(game.getFieldModel()).willReturn(fieldModel);

		ServerCommandZapPlayer command = new ServerCommandZapPlayer("p1", "home");

		given(team.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) player);
		handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING);

		ArgumentCaptor<Player> captor = ArgumentCaptor.forClass(Player.class);
		verify(team, times(1)).addPlayer(captor.capture());
		Player<?> zappedPlayer = captor.getValue();

		// Once zapped, team.getPlayerById would now return the ZappedPlayer
		// instead of the RosterPlayer -- the `instanceof RosterPlayer` check
		// in the handler then fails and it is not zapped again.
		given(team.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) zappedPlayer);
		handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING);

		verify(team, times(1)).addPlayer(any(Player.class));
	}

	@Test
	void unknownPlayerIdIsANoOpButStillReturnsTrue() {
		given(game.getTeamById("home")).willReturn(team);
		given(team.getPlayerById("ghost")).willReturn(null);

		ServerCommandZapPlayer command = new ServerCommandZapPlayer("ghost", "home");
		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(team, never()).addPlayer(any());
	}

	// wrong_command_type_is_ignored_but_returns_true: SKIPPED -- Java casts
	// `(ServerCommandZapPlayer) pNetCommand` unconditionally, so a wrong-type
	// command throws ClassCastException rather than no-op-ing like the Rust
	// enum match does.

	@Test
	void bb2016ZapUsesBb2016Stats() {
		RosterPlayer player = makeRosterPlayer("p1");
		given(game.getTeamById("home")).willReturn(team);
		given(team.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) player);
		given(game.getRules()).willReturn(realRules("BB2016"));
		given(game.getFieldModel()).willReturn(fieldModel);

		ServerCommandZapPlayer command = new ServerCommandZapPlayer("p1", "home");
		handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING);

		ArgumentCaptor<Player> captor = ArgumentCaptor.forClass(Player.class);
		verify(team).addPlayer(captor.capture());
		// BB2016 zap agility is 4 (vs BB2020's 2) -- see GameMechanic.zappedPlayerStats()
		// in mechanics/bb2016 vs mechanics/bb2020.
		assertEquals(4, captor.getValue().getAgility());
	}
}
