package com.fumbbl.ffb.server.fixture;

import com.fumbbl.ffb.net.NetCommand;
import com.fumbbl.ffb.server.DbUpdater;
import com.fumbbl.ffb.server.DebugLog;
import com.fumbbl.ffb.server.FantasyFootballServer;
import com.fumbbl.ffb.server.GameCache;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.IServerLogLevel;
import com.fumbbl.ffb.server.ServerMode;
import com.fumbbl.ffb.server.net.ServerCommunication;
import com.fumbbl.ffb.server.net.SessionManager;
import com.fumbbl.ffb.server.util.rng.Fortuna;

import org.eclipse.jetty.websocket.api.Session;

import java.io.File;
import java.lang.reflect.Proxy;
import java.util.Properties;

/**
 * A minimal {@link FantasyFootballServer} for step unit tests: no database, no
 * Jetty, no network I/O, no logging. All persistence and network calls are
 * no-ops; everything a step dereferences during {@code start()} /
 * {@code handleCommand()} is backed by a real object.
 *
 * <p>Modelled on {@code HeadlessFantasyFootballServer} from the ffb-ai
 * simulation harness, but self-contained in the ffb-server test tree.
 *
 * <p>What is real vs stubbed:
 * <ul>
 *   <li><b>Real:</b> {@link com.fumbbl.ffb.FactoryManager} + application-scope
 *       factories (built by the super constructor), {@link ServerMode#STANDALONE},
 *       {@link Fortuna} dice source, {@link SessionManager} lookup tables.</li>
 *   <li><b>No-op:</b> {@link ServerCommunication#send} (both variants),
 *       {@link GameCache#queueDbUpdate}, {@link DebugLog#isLogging} (always false).</li>
 *   <li><b>Sentinel:</b> {@link #HOME_SESSION} / {@link #AWAY_SESSION} dynamic
 *       proxies stand in for WebSocket sessions so session-identity checks in
 *       steps (e.g. {@code StepInitStartGame}'s home/away coach detection) work.</li>
 * </ul>
 */
public class TestFantasyFootballServer extends FantasyFootballServer {

	/** Sentinel session identifying commands sent by the home coach. */
	public static final Session HOME_SESSION = makeSession();
	/** Sentinel session identifying commands sent by the away coach. */
	public static final Session AWAY_SESSION = makeSession();

	private static Session makeSession() {
		return (Session) Proxy.newProxyInstance(
			Session.class.getClassLoader(),
			new Class<?>[] { Session.class },
			(proxy, method, args) -> {
				if ("isOpen".equals(method.getName())) {
					return Boolean.TRUE;
				}
				return null;
			});
	}

	private DebugLog debugLog;
	private final ServerCommunication communication;
	private final DbUpdater dbUpdater;
	private final GameCache gameCache;
	private final SessionManager sessionManager;
	// A ScriptedFortuna (subclass of Fortuna) so tests can install a deterministic
	// roll sequence via GameFixture.installScriptedDice(...). With no script loaded
	// it behaves exactly like the real random Fortuna.
	private final ScriptedFortuna fortuna = new ScriptedFortuna();

	public TestFantasyFootballServer() {
		super(ServerMode.STANDALONE, new Properties());
		this.communication = new ServerCommunication(this) {
			@Override
			public void send(Session pSession, NetCommand command, boolean pLog) {
				// no-op: no network in tests
			}

			@Override
			protected void send(Session[] pSessions, NetCommand command, boolean pLog) {
				// no-op: no network in tests
			}
		};
		this.dbUpdater = new DbUpdater(this);
		this.sessionManager = new TestSessionManager();
		this.gameCache = new TestGameCache(this);
	}

	@Override
	public DebugLog getDebugLog() {
		if (debugLog == null) {
			// isLogging() == false means nothing is ever formatted or written.
			debugLog = new DebugLog(this, null, new File(System.getProperty("java.io.tmpdir")),
				IServerLogLevel.NO_LOGGING) {
				@Override
				public boolean isLogging(int pLogLevel) {
					return false;
				}
			};
		}
		return debugLog;
	}

	@Override
	public ServerCommunication getCommunication() {
		return communication;
	}

	@Override
	public DbUpdater getDbUpdater() {
		return dbUpdater;
	}

	@Override
	public GameCache getGameCache() {
		return gameCache;
	}

	@Override
	public SessionManager getSessionManager() {
		return sessionManager;
	}

	@Override
	public Fortuna getFortuna() {
		return fortuna;
	}

	// FantasyFootballServer accesses its private fDebugLog field directly in
	// these methods; fDebugLog is only set by run(), which tests never call.

	@Override
	public void logError(long gameId, String message) {
	}

	@Override
	public void logDebug(long gameId, String message) {
	}

	@Override
	public void logWithOutGameId(Throwable throwable) {
	}

	@Override
	public void closeResources(long id) {
	}

	/** A {@link GameCache} whose database writes are suppressed. */
	static class TestGameCache extends GameCache {

		TestGameCache(FantasyFootballServer server) {
			super(server);
			// note: init() is NOT called - it would load rosters/ and teams/
			// from the working directory. Fixture teams are built in code.
		}

		@Override
		public void queueDbUpdate(GameState pGameState, boolean pWithSerialization) {
			// no-op: no database in tests
		}
	}

	/**
	 * A {@link SessionManager} that answers the sentinel {@link #HOME_SESSION} /
	 * {@link #AWAY_SESSION} for every game id, so steps can attribute injected
	 * commands to the correct coach without real WebSocket connections.
	 */
	static class TestSessionManager extends SessionManager {

		@Override
		public synchronized Session getSessionOfHomeCoach(long gameId) {
			return HOME_SESSION;
		}

		@Override
		public synchronized Session getSessionOfAwayCoach(long gameId) {
			return AWAY_SESSION;
		}
	}
}
