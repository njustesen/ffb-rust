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
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.model.ZappedPlayer;
import com.fumbbl.ffb.net.commands.ServerCommandUnzapPlayer;
import com.fumbbl.ffb.option.GameOptionId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;

/**
 * Mirrors the Rust tests in
 * crates/ffb-client/src/client/handler/client_command_handler_unzap_player.rs.
 *
 * <p>The Rust tests operate on a plain {@code &mut Game} model and assert
 * Rust-only state ({@code game.zapped_players}, {@code Player::is_zapped()}).
 * The real Java handler ({@link ClientCommandHandlerUnzapPlayer}) instead
 * mutates a mocked {@link FantasyFootballClient}'s {@code Team}/{@code
 * FieldModel} collaborators, so those Rust assertions are translated into
 * Mockito verifications of {@code team.addPlayer(...)} and {@code
 * fieldModel.sendPosition(...)} calls.</p>
 */
@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerUnzapPlayerTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private Game game;

	@Mock
	private Team team;

	@Mock
	private FieldModel fieldModel;

	private ClientCommandHandlerUnzapPlayer handler;

	@BeforeEach
	void setUp() {
		handler = new ClientCommandHandlerUnzapPlayer(client);
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
	 * Builds a real (non-mocked) {@link GameRules} so {@code ZappedPlayer.init(...)}
	 * can resolve real skills and per-edition zapped-player stats from
	 * {@code GameMechanic}; mocking that factory chain is impractical (six
	 * real skill lookups + a mechanic cast). Mirrors ffb-common's test-only
	 * {@code NetCommandTestUtil}, unreachable from this module (no test-jar
	 * dependency wired up).
	 */
	@SuppressWarnings({"rawtypes", "unchecked"})
	private static GameRules realRules() {
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
				realGame.getOptions().getFactory().createGameOption(GameOptionId.RULESVERSION).setValue("BB2020"));
		realGame.initializeRules();
		return realGame.getRules();
	}

	private static ZappedPlayer makeZappedPlayer(RosterPlayer original) {
		ZappedPlayer zapped = new ZappedPlayer();
		zapped.init(original, realRules());
		return zapped;
	}

	@Test
	void unzapsACurrentlyZappedPlayer() {
		RosterPlayer original = makeRosterPlayer("p1");
		ZappedPlayer zapped = makeZappedPlayer(original);

		given(game.getTeamById("home")).willReturn(team);
		given(team.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) zapped);
		given(game.getFieldModel()).willReturn(fieldModel);

		ServerCommandUnzapPlayer command = new ServerCommandUnzapPlayer("p1", "home");
		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(team).addPlayer(original);
		verify(fieldModel).sendPosition(original);
		// The restored player is the original RosterPlayer, with its own (non-zap) stats.
		assertEquals(3, original.getAgility());
	}

	@Test
	void doesNothingForAPlayerThatIsNotZapped() {
		RosterPlayer player = makeRosterPlayer("p1");
		given(game.getTeamById("home")).willReturn(team);
		given(team.getPlayerById("p1")).willReturn((com.fumbbl.ffb.model.Player) player);

		ServerCommandUnzapPlayer command = new ServerCommandUnzapPlayer("p1", "home");
		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(team, never()).addPlayer(any());
	}

	@Test
	void unknownPlayerIdIsANoOpButStillReturnsTrue() {
		given(game.getTeamById("home")).willReturn(team);
		given(team.getPlayerById("ghost")).willReturn(null);

		ServerCommandUnzapPlayer command = new ServerCommandUnzapPlayer("ghost", "home");
		assertTrue(handler.handleNetCommand(command, ClientCommandHandlerMode.PLAYING));

		verify(team, never()).addPlayer(any());
	}

	// wrong_command_type_is_ignored_but_returns_true: SKIPPED -- Java casts
	// `(ServerCommandUnzapPlayer) pNetCommand` unconditionally, so a wrong-type
	// command throws ClassCastException rather than no-op-ing like the Rust
	// enum match does.
}
