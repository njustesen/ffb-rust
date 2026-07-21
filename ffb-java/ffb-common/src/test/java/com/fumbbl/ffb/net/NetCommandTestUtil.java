package com.fumbbl.ffb.net;

import com.fumbbl.ffb.FactoryManager;
import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.factory.INamedObjectFactory;
import com.fumbbl.ffb.model.Game;

import java.util.Map;

/**
 * Shared helpers for net command serialization tests (mirrors the Rust
 * ffb-protocol test suite). Provides minimal {@link IFactorySource}
 * instances the way production code wires them up: an application-context
 * source (as FantasyFootballServer does) and a game-context source obtained
 * from an initialized {@link Game}'s {@link com.fumbbl.ffb.model.GameRules}.
 */
public final class NetCommandTestUtil {

	private static IFactorySource applicationSource;
	private static IFactorySource gameSource;

	private NetCommandTestUtil() {
	}

	public static synchronized IFactorySource applicationSource() {
		if (applicationSource == null) {
			applicationSource = new ApplicationFactorySource(new FactoryManager());
		}
		return applicationSource;
	}

	public static synchronized IFactorySource gameSource() {
		if (gameSource == null) {
			IFactorySource app = applicationSource();
			Game game = new Game(app, app.getFactoryManager());
			game.initializeRules();
			gameSource = game.getRules();
		}
		return gameSource;
	}

	private static class ApplicationFactorySource implements IFactorySource {

		private final FactoryManager manager;
		@SuppressWarnings("rawtypes")
		private final Map<Factory, INamedObjectFactory> factories;

		ApplicationFactorySource(FactoryManager manager) {
			this.manager = manager;
			this.factories = manager.getFactoriesForContext(getContext(), this);
		}

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
			if (context == getContext()) {
				return this;
			}
			return gameSource();
		}

		@SuppressWarnings({ "rawtypes", "unchecked" })
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
	}
}
