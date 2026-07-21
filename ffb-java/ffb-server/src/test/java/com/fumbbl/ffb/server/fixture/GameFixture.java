package com.fumbbl.ffb.server.fixture;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.GameStatus;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.PlayerType;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.Weather;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Roster;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.RosterPosition;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommand;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.net.ReceivedCommand;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepCommandStatus;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.util.UtilServerStartGame;
import com.fumbbl.ffb.server.util.UtilSkillBehaviours;
import com.fumbbl.ffb.server.util.rng.Fortuna;
import com.fumbbl.ffb.util.UtilActingPlayer;
import com.fumbbl.ffb.util.UtilBox;
import com.fumbbl.ffb.util.UtilTeamValue;

import java.lang.reflect.Method;

/**
 * Foundation fixture for Java step tests, mirroring the Rust engine's test
 * helpers ({@code framework::test_team} / {@code add_player_with_skills} in
 * ffb-rust's {@code ffb-engine}).
 *
 * <p>Typical usage:
 *
 * <pre>{@code
 * GameState gameState = GameFixture.createGameState();          // 2 x 11 linemen
 * IStep step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
 * StepAction action = GameFixture.startStep(step);              // run start()
 * GameFixture.handleCommand(step, new ClientCommandStartGame(), true); // home coach
 * }</pre>
 *
 * <p>Construction order (mirrors {@code ServerCommandHandlerJoinApproved}'s
 * STANDALONE path, via the ffb-ai {@code HeadlessGameSetup} recipe): default
 * game options (BB2025) -> rules-dependent members -> initializeRules ->
 * skill behaviours -> teams. Teams are built programmatically on a synthetic
 * one-position "lineman" roster instead of loading team/roster XML from disk.
 */
public final class GameFixture {

	/** Position id of the synthetic roster position every fixture player uses. */
	public static final String LINEMAN_POSITION_ID = "lineman";

	// Default stats mirror the Rust test helper add_player_with_skills:
	// MV 6 / ST 3 / AG 3 / PA 4 / AV 8.
	public static final int DEFAULT_MOVEMENT = 6;
	public static final int DEFAULT_STRENGTH = 3;
	public static final int DEFAULT_AGILITY = 3;
	public static final int DEFAULT_PASSING = 4;
	public static final int DEFAULT_ARMOUR = 8;

	private GameFixture() {
	}

	// ── GameState construction ───────────────────────────────────────────────

	/** A GameState with two teams ("home"/"away") of 11 generic linemen each. */
	public static GameState createGameState() {
		return createGameState(11);
	}

	/** A GameState with two teams of {@code playersPerTeam} generic linemen. */
	public static GameState createGameState(int playersPerTeam) {
		return createGameState(new TestFantasyFootballServer(), playersPerTeam);
	}

	/**
	 * Create a fully initialized {@link GameState} backed by the given test
	 * server. Player ids are {@code home1..homeN} / {@code away1..awayN} with
	 * jersey numbers 1..N; all players start as reserves in the box.
	 */
	public static GameState createGameState(TestFantasyFootballServer server, int playersPerTeam) {
		GameState gameState = new GameState(server);

		// 1. Default game options (selects the BB2025 ruleset in STANDALONE mode)
		UtilServerStartGame.addDefaultGameOptions(gameState);

		// 2. Rules-dependent members and rules initialization
		Game game = gameState.getGame();
		game.setHomePlaying(true);
		game.setTurnMode(TurnMode.START_GAME);
		game.setTesting(true);
		game.getFieldModel().setWeather(Weather.NICE);
		gameState.initRulesDependentMembers(); // creates the StepFactory
		game.initializeRules();
		UtilSkillBehaviours.registerBehaviours(game, server.getDebugLog());

		// 3. Programmatic teams (no XML, no TeamCache/RosterCache)
		Team homeTeam = createTeam(game, "home", playersPerTeam);
		Team awayTeam = createTeam(game, "away", playersPerTeam);
		server.getGameCache().addTeamToGame(gameState, homeTeam, true);
		server.getGameCache().addTeamToGame(gameState, awayTeam, false);

		// 4. Same initial status a freshly scheduled game has before
		//    StepInitStartGame runs (Rust: GameStatus::Starting).
		gameState.setStatus(GameStatus.STARTING);

		return gameState;
	}

	// ── Team / player construction ───────────────────────────────────────────

	/**
	 * Create a team of {@code nrOfPlayers} generic linemen. Mirrors the Rust
	 * {@code test_team} helper (race "human", coach "coach", no rerolls).
	 * Player ids are {@code <teamId>1..<teamId>N}, jersey numbers 1..N.
	 */
	public static Team createTeam(Game game, String teamId, int nrOfPlayers) {
		Team team = new Team(game.getRules());
		team.setId(teamId);
		team.setName(teamId);
		team.setCoach("coach");
		team.setRace("human");

		Roster roster = createLinemanRoster();
		for (int nr = 1; nr <= nrOfPlayers; nr++) {
			RosterPlayer player = newPlayer(teamId + nr, nr);
			team.addPlayer(player);
		}
		// binds the roster, resolves each player's position and copies stats
		team.updateRoster(roster, game.getRules());
		team.setTeamValue(UtilTeamValue.findTeamValue(team));
		return team;
	}

	/**
	 * Add one player to a team of an already-built GameState (Java analogue of
	 * Rust's {@code add_player_with_skills}). The player is created on the
	 * lineman position with default stats, given the named skills, and placed
	 * in the reserve box.
	 */
	public static RosterPlayer addPlayer(GameState gameState, boolean homeTeam, String playerId, int nr,
										 String... skillNames) {
		return addPlayer(gameState, homeTeam, playerId, nr, DEFAULT_MOVEMENT, DEFAULT_STRENGTH, DEFAULT_AGILITY,
			DEFAULT_PASSING, DEFAULT_ARMOUR, skillNames);
	}

	/** Same as {@link #addPlayer(GameState, boolean, String, int, String...)} with explicit stats. */
	public static RosterPlayer addPlayer(GameState gameState, boolean homeTeam, String playerId, int nr,
										 int movement, int strength, int agility, int passing, int armour,
										 String... skillNames) {
		Game game = gameState.getGame();
		Team team = homeTeam ? game.getTeamHome() : game.getTeamAway();

		RosterPlayer player = newPlayer(playerId, nr);
		team.addPlayer(player);
		player.updatePosition(team.getRoster().getPositionById(LINEMAN_POSITION_ID), game.getRules(), game.getId());

		// custom stats override the position defaults
		player.setMovement(movement);
		player.setStrength(strength);
		player.setAgility(agility);
		player.setPassing(passing);
		player.setArmour(armour);
		for (String skillName : skillNames) {
			player.addSkill(skill(game, skillName));
		}

		game.getFieldModel().setPlayerState(player, new PlayerState(PlayerState.RESERVE));
		UtilBox.putPlayerIntoBox(game, player);
		return player;
	}

	/** Resolve a skill by display name (e.g. "Block", "Dodge") from the active ruleset. */
	public static Skill skill(Game game, String name) {
		Skill skill = game.getRules().getSkillFactory().forName(name);
		if (skill == null) {
			throw new IllegalArgumentException("Unknown skill: " + name);
		}
		return skill;
	}

	private static RosterPlayer newPlayer(String playerId, int nr) {
		RosterPlayer player = new RosterPlayer();
		player.setId(playerId);
		player.setName(playerId);
		player.setNr(nr);
		player.setPositionId(LINEMAN_POSITION_ID);
		player.setGender(PlayerGender.MALE);
		return player;
	}

	private static Roster createLinemanRoster() {
		Roster roster = new Roster();
		roster.setId("human");
		roster.setName("human");
		roster.setReRollCost(50000);
		roster.setMaxReRolls(8);

		RosterPosition lineman = new RosterPosition(LINEMAN_POSITION_ID);
		lineman.setName("Lineman");
		lineman.setShorthand("L");
		lineman.setType(PlayerType.REGULAR);
		lineman.setGender(PlayerGender.MALE);
		lineman.setQuantity(16);
		lineman.setCost(50000);
		lineman.setMovement(DEFAULT_MOVEMENT);
		lineman.setStrength(DEFAULT_STRENGTH);
		lineman.setAgility(DEFAULT_AGILITY);
		lineman.setPassing(DEFAULT_PASSING);
		lineman.setArmour(DEFAULT_ARMOUR);
		addPosition(roster, lineman);
		return roster;
	}

	/**
	 * {@code Roster.addPosition} is private (production code only fills rosters
	 * from XML/JSON), so the synthetic position is registered via reflection.
	 */
	private static void addPosition(Roster roster, RosterPosition position) {
		try {
			Method addPosition = Roster.class.getDeclaredMethod("addPosition", RosterPosition.class);
			addPosition.setAccessible(true);
			addPosition.invoke(roster, position);
		} catch (ReflectiveOperationException e) {
			throw new IllegalStateException("Cannot register roster position", e);
		}
	}

	// ── Game-situation helpers ───────────────────────────────────────────────

	/** Put a player on the pitch at (x, y), standing. */
	public static void placePlayer(GameState gameState, String playerId, int x, int y) {
		Game game = gameState.getGame();
		com.fumbbl.ffb.model.Player<?> player = game.getPlayerById(playerId);
		if (player == null) {
			throw new IllegalArgumentException("Unknown player: " + playerId);
		}
		game.getFieldModel().setPlayerState(player, new PlayerState(PlayerState.STANDING));
		game.getFieldModel().setPlayerCoordinate(player, new FieldCoordinate(x, y));
	}

	public static void setTurnMode(GameState gameState, TurnMode turnMode) {
		gameState.getGame().setTurnMode(turnMode);
	}

	public static void setHalf(GameState gameState, int half) {
		gameState.getGame().setHalf(half);
	}

	/** Make {@code playerId} the acting player with the given action. */
	public static void setActingPlayer(GameState gameState, String playerId, PlayerAction action) {
		UtilActingPlayer.changeActingPlayer(gameState.getGame(), playerId, action, false);
	}

	// ── Deterministic dice ───────────────────────────────────────────────────

	/**
	 * Install a deterministic roll sequence on the game's dice source, mirroring
	 * how the Rust step tests seed their RNG. The supplied faces are consumed, in
	 * order, by subsequent die draws made through
	 * {@code gameState.getDiceRoller()} (which resolves every roll via the test
	 * server's {@link ScriptedFortuna}). Covers all categories: d3/d4/d6/d8/d16,
	 * 2d6 rolls like weather/armour/injury (two faces), block dice (d6 faces
	 * 1..6 = skull..pow) and scatter/throw-in direction (d8).
	 *
	 * <p>Faces must be valid for the die they land on (e.g. 1..6 for a d6) or the
	 * draw throws {@link IllegalStateException}. Once the script is exhausted,
	 * draws fall back to the real random {@link ScriptedFortuna} behaviour.
	 *
	 * <p>Calling this repeatedly appends to the existing script; use
	 * {@link #clearScriptedDice(GameState)} to reset. Only works on a GameState
	 * created by this fixture (backed by {@link TestFantasyFootballServer}).
	 *
	 * @see ScriptedFortuna for the sequential-ordering / re-roll constraints
	 */
	public static void installScriptedDice(GameState gameState, int... rolls) {
		scriptedFortuna(gameState).script(rolls);
	}

	/** Discard any remaining scripted dice, reverting to real random draws. */
	public static void clearScriptedDice(GameState gameState) {
		scriptedFortuna(gameState).clearScript();
	}

	private static ScriptedFortuna scriptedFortuna(GameState gameState) {
		Fortuna fortuna = gameState.getServer().getFortuna();
		if (!(fortuna instanceof ScriptedFortuna)) {
			throw new IllegalStateException(
				"installScriptedDice requires a GameState from GameFixture (backed by "
					+ "TestFantasyFootballServer); found Fortuna of type " + fortuna.getClass().getName());
		}
		return (ScriptedFortuna) fortuna;
	}

	// ── Step execution helpers ───────────────────────────────────────────────

	/** Construct a step through the GameState's {@link com.fumbbl.ffb.server.step.StepFactory}. */
	public static IStep createStep(GameState gameState, StepId stepId) {
		return gameState.getStepFactory().forStepId(stepId);
	}

	/**
	 * Run a step's {@code start()} in isolation (without the GameState step
	 * loop) and return the action it decided on. {@link StepAction#CONTINUE}
	 * means the step is waiting for a client command - the same semantics as
	 * the Rust {@code StepOutcome::cont()}.
	 */
	public static StepAction startStep(IStep step) {
		step.start();
		return step.getResult().getNextAction();
	}

	/**
	 * Deliver a client command to a step as if the home or away coach had sent
	 * it over the wire. Uses the fixture's sentinel sessions, so steps that
	 * compare the command's session against
	 * {@code sessionManager.getSessionOfHomeCoach()/getSessionOfAwayCoach()}
	 * attribute it correctly.
	 */
	public static StepCommandStatus handleCommand(IStep step, NetCommand command, boolean fromHomeCoach) {
		return step.handleCommand(receivedCommand(command, fromHomeCoach));
	}

	/** Wrap a NetCommand in a {@link ReceivedCommand} from the home or away coach. */
	public static ReceivedCommand receivedCommand(NetCommand command, boolean fromHomeCoach) {
		return new ReceivedCommand(command,
			fromHomeCoach ? TestFantasyFootballServer.HOME_SESSION : TestFantasyFootballServer.AWAY_SESSION);
	}

	/** The action the step last decided on ({@code CONTINUE} = still waiting). */
	public static StepAction nextAction(IStep step) {
		return step.getResult().getNextAction();
	}
}
