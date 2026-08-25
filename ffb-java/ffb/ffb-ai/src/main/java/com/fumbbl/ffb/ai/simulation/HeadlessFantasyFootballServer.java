package com.fumbbl.ffb.ai.simulation;

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
 * A minimal {@link FantasyFootballServer} that runs without a database, Jetty,
 * or any network I/O. Designed for headless game simulation and benchmarking.
 *
 * <p>All network and persistence calls are no-ops. The step-stack game loop
 * can still be driven by injecting {@link com.fumbbl.ffb.server.net.ReceivedCommand}
 * objects directly into a {@link GameState}.
 */
public class HeadlessFantasyFootballServer extends FantasyFootballServer {

    /**
     * Sentinel session objects used by {@link HeadlessSessionManager} so that
     * {@code checkCommandIsFromHomePlayer()} / {@code ...AwayPlayer()} can
     * distinguish home vs away commands without real WebSocket sessions.
     */
    public static final Session HOME_SESSION = makeSession();
    public static final Session AWAY_SESSION = makeSession();

    private static Session makeSession() {
        return (Session) Proxy.newProxyInstance(
            Session.class.getClassLoader(),
            new Class<?>[]{Session.class},
            (proxy, method, args) -> {
                if ("isOpen".equals(method.getName())) return Boolean.TRUE;
                return null;
            });
    }

    private DebugLog debugLog;
    private final ServerCommunication communication;
    private final DbUpdater dbUpdater;
    private final GameCache gameCache;
    private final SessionManager sessionManager;
    private final Fortuna fortuna = new Fortuna();

    public HeadlessFantasyFootballServer() {
        super(ServerMode.STANDALONE, new Properties());
        // debugLog is lazily created on first getDebugLog() call
        this.communication = new ServerCommunication(this) {
            @Override
            public void send(Session pSession, com.fumbbl.ffb.net.NetCommand command, boolean pLog) {
                // No-op: no network in headless mode
            }
            @Override
            protected void send(Session[] pSessions, com.fumbbl.ffb.net.NetCommand command, boolean pLog) {
                // No-op: no network in headless mode
            }
        };
        this.dbUpdater = new DbUpdater(this);
        this.sessionManager = new HeadlessSessionManager();
        this.gameCache = new HeadlessGameCache(this);
    }

    @Override
    public DebugLog getDebugLog() {
        if (debugLog == null) {
            // Anonymous subclass: override isLogging() so nothing is ever written.
            // The DebugLog constructor is safe with a null logFile and /tmp as base path.
            //
            // FFB_SERVER_DEBUG=1 turns it back ON. `GameState.executeStep` already calls
            // `logCurrentStep(IServerLogLevel.DEBUG, this)` for EVERY step the server runs, which is
            // the only way to see the steps inside a single action — ParityRunner's own probes only
            // fire at loop tops (steps that wait for a client command), so internally-chained steps
            // are invisible to them.
            final boolean serverDebug = System.getenv("FFB_SERVER_DEBUG") != null;
            if (serverDebug) {
                System.err.println("JSTEPALL debugLog created");
            }
            debugLog = new DebugLog(
                this,
                null,
                new File(System.getProperty("java.io.tmpdir")),
                serverDebug ? IServerLogLevel.DEBUG : IServerLogLevel.NO_LOGGING
            ) {
                @Override
                public boolean isLogging(int pLogLevel) {
                    return serverDebug && pLogLevel <= IServerLogLevel.DEBUG;
                }

                // DebugLog's own writer needs a log FILE; with none configured its output goes
                // nowhere. Send just the per-step line to stderr, where the parity harness's other
                // probes already go, so it can be diffed against Rust's FFB_DRIVE_TRACE.
                @Override
                public void logCurrentStep(int pLogLevel, com.fumbbl.ffb.server.GameState pGameState) {
                    if (serverDebug && pGameState != null && pGameState.getCurrentStep() != null) {
                        System.err.println("JSTEPALL step=" + pGameState.getCurrentStep().getId()
                            + " dice=" + pGameState.getDiceRoller().getCallCount());
                    }
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

    // Direct fDebugLog field accesses in FantasyFootballServer — override to be safe
    @Override
    public void logError(long gameId, String message) {}

    @Override
    public void logDebug(long gameId, String message) {}

    @Override
    public void logWithOutGameId(Throwable throwable) {}

    @Override
    public void closeResources(long id) {}

    /**
     * A {@link GameCache} that skips all database persistence. The in-memory
     * maps for game-name lookup still work; only DB writes are suppressed.
     */
    static class HeadlessGameCache extends GameCache {

        HeadlessGameCache(FantasyFootballServer server) {
            super(server);
        }

        @Override
        public void queueDbUpdate(GameState pGameState, boolean pWithSerialization) {
            // No-op: no database in headless mode
        }
    }

    /**
     * A {@link SessionManager} that returns the sentinel {@link #HOME_SESSION} and
     * {@link #AWAY_SESSION} objects for all games so that session-based ownership
     * checks in steps work correctly without real WebSocket connections.
     */
    static class HeadlessSessionManager extends SessionManager {
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
