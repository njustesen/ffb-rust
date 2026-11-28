package com.fumbbl.ffb.ai.defects;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.ai.simulation.HeadlessFantasyFootballServer;
import com.fumbbl.ffb.ai.simulation.HeadlessGameSetup;
import com.fumbbl.ffb.ai.simulation.MatchRunner;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.commands.ClientCommand;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.step.IStep;

import java.io.File;
import java.util.List;

/**
 * A headless stock-Java game for the JAVA_DEFECTS.md tests: two parity teams, every player in
 * reserve, an empty step stack and scripted dice. Tests place the players they need, push the real
 * step sequence of the defect and drive it with the client commands a real coach would send.
 */
final class DefectFixture {

	private static HeadlessFantasyFootballServer server;

	final GameState gameState;
	final Game game;

	private DefectFixture(GameState gameState) {
		this.gameState = gameState;
		this.game = gameState.getGame();
	}

	/** @param edition "BB2016", "BB2020" or "BB2025"; @param race e.g. "human" (parity team files) */
	static DefectFixture create(String edition, String race) throws Exception {
		if (server == null) {
			server = new HeadlessFantasyFootballServer();
		}
		String suffix = edition.equals("BB2016") ? "16" : edition.equals("BB2020") ? "20" : "25";
		String base = "team" + camel(race) + "Parity" + suffix;
		GameState gs = HeadlessGameSetup.create(server, base + "Home", base + "Away", serverDir(), edition);
		gs.getStepStack().clear();
		Game game = gs.getGame();
		game.setTurnMode(TurnMode.REGULAR);
		game.setHomePlaying(true);
		game.getTurnDataHome().setTurnNr(1);
		game.getTurnDataAway().setTurnNr(1);
		return new DefectFixture(gs);
	}

	private static File serverDir() {
		File cwd = new File(System.getProperty("user.dir"));
		File candidate = new File(cwd.getParentFile(), "ffb-server");
		return candidate.exists() ? candidate : new File(cwd, "ffb-server");
	}

	private static String camel(String race) {
		StringBuilder sb = new StringBuilder();
		for (String part : race.split("_")) {
			sb.append(Character.toUpperCase(part.charAt(0))).append(part.substring(1));
		}
		return sb.toString();
	}

	Player<?> home(int nr) {
		return game.getTeamHome().getPlayerByNr(nr);
	}

	Player<?> away(int nr) {
		return game.getTeamAway().getPlayerByNr(nr);
	}

	Skill skill(Class<? extends Skill> skillClass) {
		SkillFactory factory = game.getFactory(FactoryType.Factory.SKILL);
		return factory.forClass(skillClass);
	}

	void give(Player<?> player, Class<? extends Skill> skillClass) {
		((RosterPlayer) player).addSkill(skill(skillClass));
	}

	void place(Player<?> player, int x, int y, int base) {
		game.getFieldModel().setPlayerCoordinate(player, new FieldCoordinate(x, y));
		game.getFieldModel().setPlayerState(player, new PlayerState(base).changeActive(true));
	}

	void act(Player<?> player, PlayerAction action) {
		game.getActingPlayer().setPlayer(player);
		game.getActingPlayer().setPlayerAction(action);
		game.getActingPlayer().setOldPlayerState(game.getFieldModel().getPlayerState(player));
	}

	/**
	 * Script the next dice, in the order the engine asks for them. Every request is recorded in
	 * {@link #rolled} as "dN=v"; running past the script fails the test with the die that was asked
	 * for. (DiceRoller.addTestRoll cannot be used: its queue NPEs on its own entries.)
	 */
	void dice(int... rolls) {
		ScriptedDice scripted = new ScriptedDice(rolls);
		server.getFortuna().setDelegate(scripted);
		this.scripted = scripted;
	}

	private ScriptedDice scripted;

	List<String> rolled() {
		return scripted == null ? java.util.Collections.emptyList() : scripted.log;
	}

	static final class ScriptedDice implements com.fumbbl.ffb.server.util.rng.Fortuna.IDiceRoller {
		private final int[] rolls;
		private int next;
		final List<String> log = new java.util.ArrayList<>();

		ScriptedDice(int[] rolls) {
			this.rolls = rolls;
		}

		@Override
		public int getDieRoll(int sides) {
			if (next >= rolls.length) {
				throw new IllegalStateException("dice script exhausted: engine asked for d" + sides + " after " + log);
			}
			int value = rolls[next++];
			if (value < 1 || value > sides) {
				throw new IllegalStateException("scripted " + value + " for a d" + sides + " after " + log);
			}
			log.add("d" + sides + "=" + value);
			return value;
		}

		@Override
		public int getCallCount() {
			return next;
		}
	}

	void push(List<IStep> steps) {
		gameState.getStepStack().push(steps);
	}

	void startNextStep() {
		gameState.startNextStep();
	}

	void send(ClientCommand command, boolean homeTeam) {
		MatchRunner.injectForTeam(gameState, command, homeTeam);
	}

	IStep current() {
		return gameState.getCurrentStep();
	}

	/**
	 * A step that stands in for the rest of a sequence: it records every parameter published to the
	 * stack while it sits below the step under test, and parks (CONTINUE) when it is reached.
	 */
	static final class Recorder extends com.fumbbl.ffb.server.step.AbstractStep {
		final java.util.Map<com.fumbbl.ffb.server.step.StepParameterKey, Object> published = new java.util.LinkedHashMap<>();
		private final com.fumbbl.ffb.server.step.StepId id;
		boolean reached;

		Recorder(GameState gameState, com.fumbbl.ffb.server.step.StepId id) {
			super(gameState);
			this.id = id;
		}

		@Override
		public com.fumbbl.ffb.server.step.StepId getId() {
			return id;
		}

		@Override
		public boolean setParameter(com.fumbbl.ffb.server.step.StepParameter parameter) {
			published.put(parameter.getKey(), parameter.getValue());
			return false;
		}

		@Override
		public void start() {
			reached = true;
			getResult().setNextAction(com.fumbbl.ffb.server.step.StepAction.CONTINUE);
		}
	}

	Recorder recorder(com.fumbbl.ffb.server.step.StepId id) {
		Recorder recorder = new Recorder(gameState, id);
		gameState.getStepStack().push(recorder);
		return recorder;
	}

	void pushStep(IStep step) {
		gameState.getStepStack().push(step);
	}
}
