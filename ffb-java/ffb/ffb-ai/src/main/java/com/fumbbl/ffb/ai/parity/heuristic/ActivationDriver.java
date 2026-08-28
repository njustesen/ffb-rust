package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.SkillMechanic;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Team;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/**
 * The live half of the heuristic agent: it owns an activation from the moment it is declared until
 * the plan it made is spent.
 *
 * <p>Everything it decides is computed by classes that already have cross-language fixtures —
 * {@link Features}, {@link Reach}, {@link ValueModel}, {@link Arrival}, {@link Plans},
 * {@link BallMoves}, {@link PlanBuilder}, {@link Activation}, {@link ActivationChoice} and
 * {@link MoveReplay}. What lives here is the STATE those decisions are threaded through, and the
 * adapters that answer the eligibility questions from a real {@link Game}.
 *
 * <p><b>Activation and movement are one unit.</b> The plan is created by
 * {@link #chooseActivation} and consumed by {@link #replayMove}, and the harness's RNG
 * choreography for a prone move is split across both — so the two prompt classes cannot be switched
 * on separately without desynchronising the stream.
 */
public final class ActivationDriver {

    /** What the activation is FOR; the follow-up prompts replay this instead of re-deciding. */
    public static final class Plan {
        public final String player;
        public final MoveReplay.Kind kind;
        /** Remaining squares of the planned path, in order. */
        public final List<FieldCoordinate> path;
        /** The victim or receiver, for a terminal action. */
        public final String target;
        public boolean delivered;
        public boolean fired;

        Plan(String player, MoveReplay.Kind kind, List<FieldCoordinate> path, String target) {
            this.player = player;
            this.kind = kind;
            this.path = path;
            this.target = target;
        }
    }

    private final Sampler sampler;
    private Plan plan;
    private final Set<String> usedThisTurn = new HashSet<>();
    private String awaitingRun;
    private String lastTurnKey;

    public ActivationDriver(Sampler sampler) {
        this.sampler = sampler;
    }

    public Plan plan() {
        return plan;
    }

    public Set<String> usedThisTurn() {
        return usedThisTurn;
    }

    /** Rust {@code refresh_turn}: a new team turn clears the per-turn memory. */
    public void refreshTurn(Game game) {
        String key = game.getHalf() + ":" + turnOf(game) + ":" + game.isHomePlaying();
        if (!key.equals(lastTurnKey)) {
            lastTurnKey = key;
            usedThisTurn.clear();
            awaitingRun = null;
            plan = null;
        }
    }

    /**
     * BB2016 computes the dodge target from the old {@code 7 - AG} scale, which the reach search
     * needs. Read off the ruleset name rather than a version number, because that is what the
     * harness has to hand.
     */
    private static boolean editionIsBb2016(Game game) {
        com.fumbbl.ffb.mechanics.Mechanic m = (com.fumbbl.ffb.mechanics.Mechanic) game
            .getFactory(com.fumbbl.ffb.FactoryType.Factory.MECHANIC)
            .forName(com.fumbbl.ffb.mechanics.Mechanic.Type.AGILITY.name());
        return m != null && m.getClass().getName().contains("bb2016");
    }

    /** The enumeration's kind, as the replay state machine names it. */
    private static MoveReplay.Kind kindOf(PlanBuilder.Kind k) {
        if (k == null) {
            return MoveReplay.Kind.MOVE;
        }
        switch (k) {
            case PICKUP: return MoveReplay.Kind.PICKUP;
            case BLITZ: return MoveReplay.Kind.BLITZ;
            case FOUL: return MoveReplay.Kind.FOUL;
            case PASS: return MoveReplay.Kind.PASS;
            case HAND_OFF: return MoveReplay.Kind.HAND_OFF;
            case IMMEDIATE: return MoveReplay.Kind.IMMEDIATE;
            default: return MoveReplay.Kind.MOVE;
        }
    }

    private static int turnOf(Game game) {
        return game.isHomePlaying() ? game.getTurnDataHome().getTurnNr()
            : game.getTurnDataAway().getTurnNr();
    }

    /**
     * Choose which player to activate and what to declare.
     *
     * @param remaining the eligible players the harness computed, each with its still-live actions.
     * @return the decision; {@code player == null} means EndTurn.
     */
    public ActivationChoice.Decision chooseActivation(Game game,
            List<ActivationChoice.Eligible> remaining) {
        Features f = Features.build(game);
        boolean home = game.isHomePlaying();
        com.fumbbl.ffb.model.TurnData td =
            home ? game.getTurnDataHome() : game.getTurnDataAway();
        boolean teamReRoll = td.getReRolls() > 0 && !td.isReRollUsed();

        boolean bb2016 = game.getOptions() != null
            && com.fumbbl.ffb.util.StringTool.isProvided(String.valueOf(game.getRules()))
            && false;
        boolean blizzard = game.getFieldModel().getWeather() == com.fumbbl.ffb.Weather.BLIZZARD;
        ActivationChoice.Decision d = ActivationChoice.choose(f, sampler, new GameBoard(game, f),
            remaining, turnOf(game), teamReRoll, awaitingRun, usedThisTurn, home,
            editionIsBb2016(game), blizzard);
        if (d.player == null) {
            return d;
        }
        usedThisTurn.add(d.player);
        // A ball move is the reason the receiver must be the next one activated.
        if ("HandOffMove".equals(d.action) || "PassMove".equals(d.action)) {
            awaitingRun = d.target;
        }
        // Record what the activation is FOR, so the movement prompts replay it instead of
        // re-deciding. Without this the plan is always null and every move re-plans from scratch.
        ValueModel.Mover m = moverOf(game, f, d.player);
        if (m != null) {
            recordPlan(game, d.player, kindOf(d.kind), d.dest, d.target, m, teamReRoll);
        }
        return d;
    }

    /**
     * Record the plan the chosen candidate implies. Called after the declaration is accepted, so
     * the path is computed against the board the engine will actually walk.
     */
    public void recordPlan(Game game, String player, MoveReplay.Kind kind, Integer dest,
            String target, ValueModel.Mover m, boolean teamReRoll) {
        List<FieldCoordinate> path = new ArrayList<>();
        FieldCoordinate here = coordOf(game, player);
        if (dest != null && here != null) {
            Features f = Features.build(game);
            Reach r = Reach.search(f,
                Reach.budgetOf(here, m.ma, isProne(game, player), spentBy(game, player)),
                new Reach.MoverSpec(m.home, m.ag, false, false), false, false, teamReRoll);
            if (r != null) {
                path = r.pathTo(dest);
            }
        }
        plan = new Plan(player, kind, path, target);
    }

    /** Rust {@code handle_move}: what to do with the squares the engine is offering. */
    public MoveReplay.Verdict replayMove(Game game, String playerId,
            List<FieldCoordinate> squares) {
        MoveReplay.Kind kind = plan != null ? plan.kind : MoveReplay.Kind.MOVE;
        boolean isMine = plan != null && plan.player.equals(playerId);
        boolean pathEmpty = plan == null || plan.path.isEmpty();

        ActingPlayer ap = game.getActingPlayer();
        FieldCoordinate here = coordOf(game, playerId);
        FieldCoordinate tc = (plan != null && plan.target != null)
            ? coordOf(game, plan.target) : null;
        MoveReplay.Facts facts = new MoveReplay.Facts(
            ap != null && ap.getPlayerAction() != null ? ap.getPlayerAction().name() : null,
            ap != null && ap.hasBlocked(),
            ap != null && ap.hasFouled(),
            here != null && tc != null
                && Math.max(Math.abs(here.getX() - tc.getX()), Math.abs(here.getY() - tc.getY()))
                    == 1,
            tc != null,
            plan != null && !plan.path.isEmpty() && squares.contains(plan.path.get(0)),
            squares.isEmpty());

        MoveReplay.Verdict v = MoveReplay.decide(kind, isMine, pathEmpty,
            plan != null && plan.delivered, plan != null && plan.fired, facts);
        if (v == MoveReplay.Verdict.END_PLAYER_ACTION) {
            plan = null;
        }
        return v;
    }

    /**
     * Rust {@code best_move} plus the re-plan tail of {@code handle_move}: no usable plan, so
     * decide a destination from scratch.
     *
     * <p>A re-plan that ends ON the loose ball becomes a PICKUP rather than a plain move — the
     * value model changes the moment the player has the ball, so the activation must be allowed to
     * continue rather than end.
     *
     * @return the path to send, or an empty list to deselect.
     */
    public List<FieldCoordinate> replan(Game game, String playerId,
            List<FieldCoordinate> squares) {
        Features f = Features.build(game);
        ValueModel.Mover m = moverOf(game, f, playerId);
        FieldCoordinate here = coordOf(game, playerId);
        if (m == null || here == null) {
            plan = null;
            return new ArrayList<>();
        }
        com.fumbbl.ffb.model.TurnData td =
            m.home ? game.getTurnDataHome() : game.getTurnDataAway();
        boolean teamReRoll = td.getReRolls() > 0 && !td.isReRollUsed();
        Reach r = Reach.search(f,
            Reach.budgetOf(here, m.ma, isProne(game, playerId), spentBy(game, playerId)),
            new Reach.MoverSpec(m.home, m.ag, false, false), false, false, teamReRoll);
        if (r == null) {
            plan = null;
            return new ArrayList<>();
        }
        // best_move: the highest arrival weight over the REACHED set, strict `>` so the first
        // maximum wins.
        float bestW = -Float.MAX_VALUE;
        Integer bestI = null;
        for (int i : r.order) {
            float w = Arrival.weight(f, r, i, m);
            if (w > bestW) {
                bestW = w;
                bestI = i;
            }
        }
        List<FieldCoordinate> path = new ArrayList<>();
        if (bestI != null && bestW > 0.0f) {
            path = r.pathTo(bestI);
        }
        if (path.isEmpty() || !squares.contains(path.get(0))) {
            plan = null;
            return new ArrayList<>();
        }
        FieldCoordinate last = path.get(path.size() - 1);
        boolean endsOnBall = f.ballLoose && f.ball != null && f.ball.equals(last);
        plan = new Plan(playerId, endsOnBall ? MoveReplay.Kind.PICKUP : MoveReplay.Kind.MOVE,
            new ArrayList<>(), null);
        plan.delivered = true;
        return path;
    }

    /** Take the whole remaining path; the engine walks it. Marks the plan delivered. */
    public List<FieldCoordinate> takePath() {
        List<FieldCoordinate> p = new ArrayList<>(plan.path);
        plan.path.clear();
        plan.delivered = true;
        return p;
    }

    /** Latch the terminal action so it is attempted at most once per activation. */
    public void markFired() {
        plan.fired = true;
    }

    /**
     * Rust {@code budget_of}'s {@code spent}: how much movement this player has already used.
     *
     * <p>Only the ACTING player has spent anything — for anyone else the activation has not started
     * — and it is what stops a re-plan from handing a blitzer who has already blocked a full fresh
     * move allowance. Passing 0 here gave the Java blitzer a longer reach than the Rust one and
     * landed him a square further on.
     */
    private static int spentBy(Game game, String playerId) {
        ActingPlayer ap = game.getActingPlayer();
        if (ap == null || !playerId.equals(ap.getPlayerId())) {
            return 0;
        }
        return Math.max(ap.getCurrentMove(), 0);
    }

    private static boolean isProne(Game game, String playerId) {
        Player<?> p = game.getPlayerById(playerId);
        PlayerState st = (p != null) ? game.getFieldModel().getPlayerState(p) : null;
        // PlayerState has no isProne(); prone-or-stunned minus stunned is the same
        // predicate the reach budget cares about (a stunned player is not activating).
        return st != null && st.isProneOrStunned() && !st.isStunned();
    }

    private static FieldCoordinate coordOf(Game game, String playerId) {
        Player<?> p = game.getPlayerById(playerId);
        return (p != null) ? game.getFieldModel().getPlayerCoordinate(p) : null;
    }

    /** Answers the eligibility questions {@link ActivationChoice} asks, from a real game. */
    private static final class GameBoard implements ActivationChoice.Board {
        private final Game game;
        private final Features f;

        GameBoard(Game game, Features f) {
            this.game = game;
            this.f = f;
        }

        private Team opponents() {
            return game.isHomePlaying() ? game.getTeamAway() : game.getTeamHome();
        }

        /**
         * @param canonicalOrder BLITZ enumerates its foes in canonical {@code (side, nr)} order,
         *     while BLOCK and FOUL take theirs from {@code legal_block_targets} /
         *     {@code legal_foul_targets}, which sort by COORDINATE. The two are deliberately
         *     different in Rust and sorting both the same way swaps the victims: seed 1 had
         *     away_01 blocking home_01 in Java and home_02 in Rust from the identical board.
         */
        private List<PlanBuilder.BlockTarget> foes(String playerId, boolean adjacentOnly,
                boolean fouls, boolean canonicalOrder) {
            FieldModel fm = game.getFieldModel();
            FieldCoordinate here = coordOf(game, playerId);
            List<PlanBuilder.BlockTarget> out = new ArrayList<>();
            if (here == null) {
                return out;
            }
            Player<?> att = game.getPlayerById(playerId);
            int attStr = att != null ? att.getStrengthWithModifiers() : 3;
            for (Player<?> o : opponents().getPlayers()) {
                FieldCoordinate oc = fm.getPlayerCoordinate(o);
                PlayerState os = fm.getPlayerState(o);
                if (oc == null || os == null || !Features.onPitch(oc.getX(), oc.getY())) {
                    continue;
                }
                int d = Math.max(Math.abs(here.getX() - oc.getX()),
                    Math.abs(here.getY() - oc.getY()));
                boolean eligible = fouls
                    ? os.isProneOrStunned() && d == 1
                    : os.hasTacklezones() && (!adjacentOnly || d == 1);
                if (!eligible) {
                    continue;
                }
                float w = fouls ? foulWeight(playerId, o) : blockWeight(playerId, o.getId(), attStr);
                out.add(new PlanBuilder.BlockTarget(o.getId(), oc, w));
            }
            if (canonicalOrder) {
                out.sort((a, b) -> {
                    Player<?> pa = game.getPlayerById(a.id);
                    Player<?> pb = game.getPlayerById(b.id);
                    int sa = game.getTeamHome().hasPlayer(pa) ? 0 : 1;
                    int sb = game.getTeamHome().hasPlayer(pb) ? 0 : 1;
                    return sa != sb ? Integer.compare(sa, sb)
                        : Integer.compare(pa.getNr(), pb.getNr());
                });
            } else {
                out.sort((a, b) -> a.at.getX() != b.at.getX()
                    ? Integer.compare(a.at.getX(), b.at.getX())
                    : Integer.compare(a.at.getY(), b.at.getY()));
            }
            return out;
        }

        /**
         * Rust {@code foul_weight}. The arithmetic itself lives in {@link BallMoves#foulWeight};
         * this supplies the four board facts it reads.
         *
         * <p>Fouls were scored as a flat ZERO until ITER45, which made every foul candidate tie at
         * `wPlayer * 0` and lose to any move — so the Java agent never fouled, while Rust weighed
         * the armour break against the ejection risk and regularly did. It is not a scoring
         * difference the state hash can see directly: it shows up one step later, as a victim who
         * is Prone on one side and KO'd on the other.
         *
         * @param attId the fouling player
         * @param def the prone or stunned victim
         */
        private float foulWeight(String attId, Player<?> def) {
            Player<?> att = game.getPlayerById(attId);
            FieldCoordinate dc = game.getFieldModel().getPlayerCoordinate(def);
            if (att == null || dc == null) {
                return 0.0f;
            }
            SkillMechanic mechanic = (SkillMechanic) game.getFactory(FactoryType.Factory.MECHANIC)
                .forName(Mechanic.Type.SKILL.name());
            int off = com.fumbbl.ffb.util.UtilPlayer.findOffensiveFoulAssists(game, att, def,
                mechanic);
            int dfn = com.fumbbl.ffb.util.UtilPlayer.findDefensiveFoulAssists(game, att, def);
            ValueModel.Mover m = moverOf(game, f, attId);
            if (m == null) {
                return 0.0f;
            }
            return BallMoves.foulWeight(f, def.getArmourWithModifiers(), off, dfn, dc,
                bribesOf(game.isHomePlaying()), m);
        }

        /**
         * The team's remaining Bribe the Ref inducements. Rust keeps this as a plain count on the
         * team; the engine keeps it in the turn's inducement set, so read it back out by type name.
         */
        private int bribesOf(boolean home) {
            com.fumbbl.ffb.model.TurnData td =
                home ? game.getTurnDataHome() : game.getTurnDataAway();
            if (td == null || td.getInducementSet() == null) {
                return 0;
            }
            for (com.fumbbl.ffb.inducement.Inducement i : td.getInducementSet().getInducements()) {
                if (i != null && i.getType() != null
                    && "bribes".equalsIgnoreCase(i.getType().getName())) {
                    return i.getValue() - i.getUses();
                }
            }
            return 0;
        }

        /** Rust {@code block_weight}, via the engine's own assist arithmetic. */
        private float blockWeight(String attId, String defId, int attStr) {
            Player<?> att = game.getPlayerById(attId);
            Player<?> def = game.getPlayerById(defId);
            if (att == null || def == null) {
                return 0.05f;
            }
            int aStr = com.fumbbl.ffb.server.util.ServerUtilPlayer.findBlockStrength(
                game, att, attStr, def, false);
            int dStr = com.fumbbl.ffb.server.util.ServerUtilPlayer.findBlockStrength(
                game, def, def.getStrengthWithModifiers(), att, false);
            int n;
            if (aStr > 2 * dStr) {
                n = 3;
            } else if (aStr > dStr) {
                n = 2;
            } else if (2 * aStr < dStr) {
                n = -3;
            } else if (aStr < dStr) {
                n = -2;
            } else {
                n = 1;
            }
            float w;
            switch (n) {
                case 3: w = 0.90f; break;
                case 2: w = 0.60f; break;
                case 1: w = hasSkill(att, "Block") ? 0.40f : 0.25f; break;
                case -2: w = 0.10f; break;
                default: w = 0.025f; break;
            }
            FieldCoordinate dc = game.getFieldModel().getPlayerCoordinate(def);
            boolean defHasBall = f.ballCarried && f.ball != null && f.ball.equals(dc);
            if (defHasBall) {
                w *= 1.35f;
            }
            if (hasSkill(def, "Block") && !hasSkill(att, "Block") && !hasSkill(att, "Wrestle")) {
                w *= 0.70f;
            }
            return Math.min(Math.max(w, 0.01f), 1.0f);
        }

        private static boolean hasSkill(Player<?> p, String name) {
            if (p == null) {
                return false;
            }
            for (com.fumbbl.ffb.model.skill.Skill s : p.getSkillsIncludingTemporaryOnes()) {
                if (s != null && name.equals(s.getName())) {
                    return true;
                }
            }
            return false;
        }

        @Override
        public List<PlanBuilder.BlockTarget> blockTargets(String playerId) {
            return foes(playerId, true, false, false);
        }

        @Override
        public List<PlanBuilder.BlockTarget> blitzFoes(String playerId) {
            return foes(playerId, false, false, true);
        }

        @Override
        public List<PlanBuilder.BlockTarget> foulTargets(String playerId) {
            return foes(playerId, true, true, false);
        }

        @Override
        public List<PlanBuilder.Receiver> receivers(String playerId, boolean forPass) {
            // Rust: team-mates on the pitch, coordinate-sorted for a pass and canonically for a
            // give. Both lists exclude the thrower himself.
            FieldModel fm = game.getFieldModel();
            Team mine = game.isHomePlaying() ? game.getTeamHome() : game.getTeamAway();
            List<Player<?>> mates = new ArrayList<>();
            for (Player<?> p : mine.getPlayers()) {
                if (p.getId().equals(playerId)) {
                    continue;
                }
                FieldCoordinate c = fm.getPlayerCoordinate(p);
                if (c != null && Features.onPitch(c.getX(), c.getY())) {
                    mates.add(p);
                }
            }
            if (forPass) {
                mates.sort((a, b) -> {
                    FieldCoordinate ca = fm.getPlayerCoordinate(a);
                    FieldCoordinate cb = fm.getPlayerCoordinate(b);
                    return ca.getX() != cb.getX() ? Integer.compare(ca.getX(), cb.getX())
                        : Integer.compare(ca.getY(), cb.getY());
                });
            } else {
                mates.sort((a, b) -> Integer.compare(a.getNr(), b.getNr()));
            }

            Player<?> thrower = game.getPlayerById(playerId);
            ValueModel.Mover m = moverOf(game, f, playerId);
            BallMoves.Ctx ctx = new BallMoves.Ctx(turnOf(game),
                game.getFieldModel().getWeather() == com.fumbbl.ffb.Weather.BLIZZARD);
            List<PlanBuilder.Receiver> out = new ArrayList<>();
            for (Player<?> p : mates) {
                FieldCoordinate at = fm.getPlayerCoordinate(p);
                PlayerState ps = fm.getPlayerState(p);
                final BallMoves.RcvSpec spec = new BallMoves.RcvSpec(at,
                    p.getMovementWithModifiers(), p.getAgilityWithModifiers(),
                    p.getStrengthWithModifiers(), ps != null && ps.isActive(),
                    hasSkill(p, "Catch"), hasSkill(p, "Sure Hands"), hasSkill(p, "Side Step"));
                if (forPass) {
                    out.add(new PassReceiver(p.getId(), at, f, ctx, spec, m, game, thrower));
                } else {
                    out.add(new GiveReceiver(p.getId(), at, f, ctx, spec, m));
                }
            }
            return out;
        }
    }

    /** Prices a give from a square with the fixture-pinned {@code handoffWeight}. */
    private static final class GiveReceiver extends PlanBuilder.Receiver {
        private final Features f;
        private final BallMoves.Ctx ctx;
        private final BallMoves.RcvSpec spec;
        private final ValueModel.Mover m;

        GiveReceiver(String id, FieldCoordinate at, Features f, BallMoves.Ctx ctx,
                BallMoves.RcvSpec spec, ValueModel.Mover m) {
            super(id, at);
            this.f = f;
            this.ctx = ctx;
            this.spec = spec;
            this.m = m;
        }

        @Override
        public Float weightFrom(FieldCoordinate from) {
            return BallMoves.handoffWeight(f, ctx, spec, from, m);
        }
    }

    /** Prices a throw from a square, asking the ENGINE which faces are accurate and which fumble. */
    private static final class PassReceiver extends PlanBuilder.Receiver {
        private final Features f;
        private final BallMoves.Ctx ctx;
        private final BallMoves.RcvSpec spec;
        private final ValueModel.Mover m;
        private final Game game;
        private final Player<?> thrower;

        PassReceiver(String id, FieldCoordinate at, Features f, BallMoves.Ctx ctx,
                BallMoves.RcvSpec spec, ValueModel.Mover m, Game game, Player<?> thrower) {
            super(id, at);
            this.f = f;
            this.ctx = ctx;
            this.spec = spec;
            this.m = m;
            this.game = game;
            this.thrower = thrower;
        }

        @Override
        public Float weightFrom(FieldCoordinate from) {
            // The engine's own lookup, so the edition's table is the one that answers.
            com.fumbbl.ffb.mechanics.PassMechanic mech =
                (com.fumbbl.ffb.mechanics.PassMechanic) game
                    .getFactory(com.fumbbl.ffb.FactoryType.Factory.MECHANIC)
                    .forName(com.fumbbl.ffb.mechanics.Mechanic.Type.PASS.name());
            int tz = f.tz[Features.sideIdx(m.home)][Features.ix(from.getX(), from.getY())] & 0xff;
            int[] faces = BallMoves.gradeFaces(mech, game, thrower, from, at, tz);
            if (faces == null) {
                // Not a legal throw at all, so the option must not exist.
                return null;
            }
            return BallMoves.passWeight(f, ctx, spec, from, m, faces[0], faces[1]);
        }
    }

    /** Rust {@code mover_of}. */
    public static ValueModel.Mover moverOf(Game game, Features f, String playerId) {
        Player<?> p = game.getPlayerById(playerId);
        FieldCoordinate c = (p != null) ? game.getFieldModel().getPlayerCoordinate(p) : null;
        if (p == null || c == null) {
            return null;
        }
        boolean home = game.getTeamHome().hasPlayer(p);
        com.fumbbl.ffb.model.TurnData td =
            home ? game.getTurnDataHome() : game.getTurnDataAway();
        boolean isCarrier = f.ballCarried && f.ball != null && f.ball.equals(c);
        return new ValueModel.Mover(home, isCarrier, p.getMovementWithModifiers(),
            p.getAgilityWithModifiers(), p.getStrengthWithModifiers(),
            GameBoard.hasSkill(p, "Sure Hands"), GameBoard.hasSkill(p, "Side Step"),
            GameBoard.hasSkill(p, "Catch"),
            ValueModel.endzoneDistance(c.getX(), home),
            Math.max(8 - td.getTurnNr(), 0),
            f.unactivated[Features.sideIdx(home)]);
    }
}
