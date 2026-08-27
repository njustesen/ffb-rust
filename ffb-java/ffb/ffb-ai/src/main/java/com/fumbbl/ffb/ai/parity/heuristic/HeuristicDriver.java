package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.IDialogParameter;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.ReRolledAction;
import com.fumbbl.ffb.ReRolledActions;
import com.fumbbl.ffb.model.Player;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import com.fumbbl.ffb.ai.simulation.MatchRunner;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.commands.ClientCommandCoinChoice;
import com.fumbbl.ffb.net.commands.ClientCommandReceiveChoice;
import com.fumbbl.ffb.server.GameState;

/**
 * The Java half of the heuristic agent, and the adapter between {@code ParityRunner}'s reactive
 * step/dialog loop and the Rust agent's {@code prompt -> score -> sample -> action} shape.
 *
 * <p>Mirror of {@code crates/ffb-engine/src/agent/heuristic_agent.rs}; contract in
 * {@code AGENT_CONTRACT_HEURISTIC.md}; campaign log in
 * {@code docs/PARITY_HEURISTIC_CAMPAIGN.md}.
 *
 * <h2>How to extend it</h2>
 *
 * One {@link PromptClass} per rung. For each: enumerate the options <b>in exactly the order the
 * Rust arm pushes them</b>, push the same weights, call {@link Sampler#pick} with the same
 * temperature, and map the returned index back to a client command. The index is the contract —
 * the agent never names an action, it names a position in an agreed enumeration.
 *
 * <p>A class that is not in the {@link ClassMask} must be left alone entirely, so
 * {@code ParityRunner}'s existing random policy answers it and its {@code decisionRng} /
 * {@code actionRng} draws happen exactly as {@code AGENT_CONTRACT.md} describes. Returning
 * {@code false} from {@link #tryDialog} does precisely that.
 *
 * <p><b>Do not consume the random agent's RNG here.</b> When the heuristic answers a prompt, the
 * draw the random contract would have spent is NOT spent — on either side. That asymmetry is
 * intentional and is why both engines must be given the same {@code --heur-classes}.
 */
public final class HeuristicDriver {

    private final Sampler sampler;
    private final ClassMask classes;

    public HeuristicDriver(long seed, float tempScale, ClassMask classes) {
        this.sampler = new Sampler(seed, tempScale);
        this.classes = classes;
    }

    public ClassMask classes() {
        return classes;
    }

    /** Whether this driver answers prompts of the given class. */
    public boolean handles(PromptClass c) {
        return classes.has(c);
    }

    /**
     * Answer {@code dialog} if its class is switched on.
     *
     * @return true when this driver injected a command and {@code ParityRunner} should do nothing
     *     further for this iteration; false to fall through to the random policy.
     */
    public boolean tryDialog(IDialogParameter dialog, Game game, GameState gameState) {
        if (dialog == null || dialog.getId() == null) {
            return false;
        }
        switch (dialog.getId()) {
            case COIN_CHOICE:
                if (!classes.has(PromptClass.COIN_CHOICE)) {
                    return false;
                }
                MatchRunner.inject(gameState, new ClientCommandCoinChoice(coinChoice()));
                return true;

            case RECEIVE_CHOICE:
                if (!classes.has(PromptClass.RECEIVE_CHOICE)) {
                    return false;
                }
                MatchRunner.inject(gameState, new ClientCommandReceiveChoice(receiveChoice(game)));
                return true;

            default:
                return false;
        }
    }

    /**
     * Rust {@code AgentPrompt::ReRollOffer}. {@code T = 0.20}.
     *
     * <p>Two options, in this order: index 0 = USE the re-roll, index 1 = decline. The weight is
     * {@code clamp(consequence * 0.833 * scarcity, 0, 1)}, and declining takes {@code 1 - that}.
     *
     * <p>The random parity contract ALWAYS declines, so every re-roll path in the engine is
     * unexercised until this arm is switched on -- 0 re-rolls consumed across the whole 100-seed
     * lineman baseline against 501 under the full heuristic agent.
     *
     * @return true to use the re-roll.
     */
    public boolean useReRoll(Game game, ReRolledAction action) {
        boolean home = game.isHomePlaying();
        com.fumbbl.ffb.model.TurnData td = home ? game.getTurnDataHome() : game.getTurnDataAway();
        boolean weCarry = weCarryTheBall(game, home);

        float consequence = consequenceOf(action, weCarry);

        // Rust: scarcity is 0 when the team has no re-rolls left, so w_use is 0 and the agent
        // always declines -- including a SKILL re-roll offered with an empty team-re-roll bank.
        // Mirrored exactly rather than "fixed".
        float scarcity;
        if (td.getReRolls() > 0) {
            float base = 0.45f + 0.55f * Math.min(td.getReRolls() / 3.0f, 1.0f);
            scarcity = td.getTurnNr() >= 7 ? base * 1.35f : base;
        } else {
            scarcity = 0.0f;
        }

        float wUse = consequence * 0.833f * scarcity;
        wUse = Math.max(0.0f, Math.min(1.0f, wUse));
        sampler.clear();
        sampler.push(wUse);          // index 0: use it
        sampler.push(1.0f - wUse);   // index 1: decline
        return sampler.pick(0.20f) == 0;
    }

    /**
     * The consequence of failing the roll being offered, mirroring Rust's match on the rerolled
     * action NAME. Rust spells these SCREAMING_SNAKE ("GFI", "PICKUP"); Java's are display strings
     * ("Go For It", "Pick Up"), so this compares the ReRolledActions CONSTANTS instead of names --
     * identity is what both engines actually mean, and it cannot drift on a spelling.
     *
     * <p>Rust's "GFI" is what its StepGoForIt passes; Java's rush passes ReRolledActions.RUSH.
     * GO_FOR_IT is folded in with it: both sit in the same bucket, so the pairing cannot matter.
     */
    private static float consequenceOf(ReRolledAction a, boolean weCarry) {
        if (a == ReRolledActions.RUSH || a == ReRolledActions.GO_FOR_IT
            || a == ReRolledActions.DODGE || a == ReRolledActions.PICK_UP
            || a == ReRolledActions.CATCH || a == ReRolledActions.JUMP
            || a == ReRolledActions.ESCAPE) {
            return weCarry ? 0.85f : 0.55f;
        }
        if (a == ReRolledActions.STAND_UP || a == ReRolledActions.TENTACLES
            || a == ReRolledActions.ALWAYS_HUNGRY || a == ReRolledActions.RIGHT_STUFF) {
            return 0.35f;
        }
        if (a == ReRolledActions.FOUL_APPEARANCE || a == ReRolledActions.HYPNOTIC_GAZE) {
            return 0.20f;
        }
        return 0.45f;
    }

    /**
     * Rust's {@code Features.carrier}, reduced to the one question this arm asks: is the ball
     * carried by the team that is currently acting?
     *
     * <p>Carried means in play, NOT moving (loose on the ground), and with a player standing on it.
     */
    private static boolean weCarryTheBall(Game game, boolean home) {
        com.fumbbl.ffb.model.FieldModel fm = game.getFieldModel();
        FieldCoordinate ball = fm.getBallCoordinate();
        // Rust `Features::ball_carried`: in play, ON the pitch, and not loose on the ground.
        if (ball == null || !fm.isBallInPlay() || !onPitch(ball) || fm.isBallMoving()) {
            return false;
        }
        Player<?> carrier = fm.getPlayer(ball);
        if (carrier == null) {
            return false;
        }
        return game.getTeamHome().hasPlayer(carrier) == home;
    }

    /**
     * Rust {@code AgentPrompt::Touchback}. {@code T = 0.20}.
     *
     * <p>Who picks the ball up after a touchback. Fast is good, Sure Hands is better, and standing
     * ON the line of scrimmage is bad enough to outweigh both -- that player is about to be
     * blocked.
     *
     * <p>The eligible set is the RECEIVING team's on-pitch players WITH TACKLE ZONES, which is not
     * the same predicate as the random contract's {@code isStanding()}; it is what
     * {@code StepTouchback} builds, and the two sides must enumerate the same set. Order is
     * {@code (side, nr)}, as everywhere else.
     *
     * @return the chosen player, or null when nobody is eligible.
     */
    public Player<?> touchback(Game game) {
        com.fumbbl.ffb.model.FieldModel fm = game.getFieldModel();
        com.fumbbl.ffb.model.Team recv = game.isHomePlaying() ? game.getTeamAway() : game.getTeamHome();
        List<Player<?>> eligible = new ArrayList<>();
        for (Player<?> p : recv.getPlayers()) {
            FieldCoordinate c = fm.getPlayerCoordinate(p);
            com.fumbbl.ffb.PlayerState ps = fm.getPlayerState(p);
            if (c == null || ps == null || !onPitch(c) || !ps.hasTacklezones()) {
                continue;
            }
            eligible.add(p);
        }
        if (eligible.isEmpty()) {
            return null;
        }
        eligible.sort((a, b) -> Long.compare(canonKey(game, a.getId()), canonKey(game, b.getId())));
        int los = game.isHomePlaying() ? 12 : 13;
        sampler.clear();
        for (Player<?> p : eligible) {
            int ma = p.getMovementWithModifiers();
            float w = 0.3f + 0.4f * Math.min(ma / 9.0f, 1.0f);
            if (hasSkillNamed(p, "Sure Hands")) {
                w += 0.3f;
            }
            FieldCoordinate c = fm.getPlayerCoordinate(p);
            if (Math.abs(c.getX() - los) <= 1) {
                w -= 0.5f;
            }
            sampler.push(w);
        }
        return eligible.get(sampler.pick(0.20f));
    }

    /**
     * Rust {@code canon_key}: {@code (side, jersey nr)}. Player IDS must never enter an ordering —
     * they are generated differently by the two engines. Home sorts before away.
     */
    private static long canonKey(Game game, String playerId) {
        Player<?> p = game.getPlayerById(playerId);
        long side = (p != null && game.getTeamHome().hasPlayer(p)) ? 0L : 1L;
        long nr = (p != null) ? p.getNr() : Integer.MAX_VALUE;
        return (side << 32) | (nr & 0xffffffffL);
    }

    /**
     * Rust {@code block_weight}: how much this defender is worth blocking, as a weight in
     * {@code [0.01, 1.0]}.
     *
     * <p>The die count comes from {@code ServerUtilPlayer.findBlockStrength} on BOTH sides —
     * Rust's {@code find_block_strength} is a translation of that same method, so the assist
     * arithmetic is not re-derived here.
     */
    float blockWeight(Game game, String attId, String defId, int attStr) {
        com.fumbbl.ffb.model.FieldModel fm = game.getFieldModel();
        Player<?> att = game.getPlayerById(attId);
        Player<?> def = game.getPlayerById(defId);
        if (att == null || def == null) {
            return 0.05f;
        }
        FieldCoordinate ac = fm.getPlayerCoordinate(att);
        FieldCoordinate dc = fm.getPlayerCoordinate(def);
        if (ac == null || dc == null) {
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
            case 3:
                w = 0.90f;
                break;
            case 2:
                w = 0.60f;
                break;
            case 1:
                w = hasSkillNamed(att, "Block") ? 0.40f : 0.25f;
                break;
            case -2:
                w = 0.10f;
                break;
            default:
                w = 0.025f;
                break;
        }
        boolean defHasBall = hasBall(game, defId);
        if (defHasBall) {
            w *= 1.35f;
        }
        if (canSurf(game, attId, defId)) {
            w *= defHasBall ? 1.9f : 1.5f;
        }
        if (hasSkillNamed(def, "Block")
            && !hasSkillNamed(att, "Block")
            && !hasSkillNamed(att, "Wrestle")) {
            w *= 0.70f;
        }
        return Math.min(Math.max(w, 0.01f), 1.0f);
    }

    /**
     * Rust {@code AgentPrompt::BlitzTarget}. {@code T = 0.15}.
     *
     * <p>The Rust arm consults the activation PLAN first, but a plan only exists once the
     * {@code activateplayer} class is ported; with it off, {@code self.plan} is {@code None} and
     * the arm falls straight through to this scored enumeration.
     *
     * <p>{@code candidates} arrives in the harness's coordinate order and is RE-SORTED by
     * {@code (side, nr)} here, exactly as the Rust arm does — the sampler returns an index, so
     * the two sides must enumerate identically.
     *
     * @return the chosen defender's id, or null when there is nothing to blitz.
     */
    public String blitzTarget(Game game, String attackerId, List<String> candidates) {
        if (candidates == null || candidates.isEmpty()) {
            return null;
        }
        List<String> ordered = new ArrayList<>(candidates);
        ordered.sort((a, b) -> Long.compare(canonKey(game, a), canonKey(game, b)));
        Player<?> att = game.getPlayerById(attackerId);
        int attStr = (att != null) ? att.getStrengthWithModifiers() : 3;
        sampler.clear();
        for (String defId : ordered) {
            sampler.push(blockWeight(game, attackerId, defId, attStr));
        }
        return ordered.get(sampler.pick(0.15f));
    }

    /**
     * Rust `push_squares`: the three squares a block can push the defender into — straight back
     * plus the two flanking it, derived purely from the attacker/defender geometry.
     */
    private static List<FieldCoordinate> pushSquares(Game game, String attId, String defId) {
        com.fumbbl.ffb.model.FieldModel fm = game.getFieldModel();
        Player<?> att = game.getPlayerById(attId);
        Player<?> def = game.getPlayerById(defId);
        FieldCoordinate a = (att != null) ? fm.getPlayerCoordinate(att) : null;
        FieldCoordinate d = (def != null) ? fm.getPlayerCoordinate(def) : null;
        List<FieldCoordinate> out = new ArrayList<>();
        if (a == null || d == null) {
            return out;
        }
        int dx = Integer.signum(d.getX() - a.getX());
        int dy = Integer.signum(d.getY() - a.getY());
        FieldCoordinate base = new FieldCoordinate(d.getX() + dx, d.getY() + dy);
        out.add(base);
        if (dx != 0 && dy != 0) {
            out.add(new FieldCoordinate(d.getX() + dx, d.getY()));
            out.add(new FieldCoordinate(d.getX(), d.getY() + dy));
        } else if (dx != 0) {
            out.add(new FieldCoordinate(base.getX(), base.getY() - 1));
            out.add(new FieldCoordinate(base.getX(), base.getY() + 1));
        } else {
            out.add(new FieldCoordinate(base.getX() - 1, base.getY()));
            out.add(new FieldCoordinate(base.getX() + 1, base.getY()));
        }
        return out;
    }

    /** Rust `can_surf`: any push square off the pitch means the defender can be crowd-surfed. */
    private static boolean canSurf(Game game, String attId, String defId) {
        for (FieldCoordinate c : pushSquares(game, attId, defId)) {
            if (!onPitch(c)) {
                return true;
            }
        }
        return false;
    }

    /** Is the ball carried by this player right now? */
    private static boolean hasBall(Game game, String playerId) {
        com.fumbbl.ffb.model.FieldModel fm = game.getFieldModel();
        FieldCoordinate ball = fm.getBallCoordinate();
        // Rust `Features::ball_carried`: in play, ON the pitch, and not loose on the ground.
        if (ball == null || !fm.isBallInPlay() || !onPitch(ball) || fm.isBallMoving()) {
            return false;
        }
        Player<?> p = game.getPlayerById(playerId);
        FieldCoordinate pc = (p != null) ? fm.getPlayerCoordinate(p) : null;
        return pc != null && ball.equals(pc);
    }

    /**
     * Rust {@code AgentPrompt::BlockChoice}. {@code T = 0.12}.
     *
     * <p>One option per die, IN THE ORDER THE DICE WERE ROLLED — the returned index is an index
     * into {@code dice}, which is the contract with the caller.
     *
     * <p>When the choice is the OPPONENT's ({@code ownChoice == false}) every weight is flipped to
     * {@code 1 - w}, so the same table serves both sides of the table.
     *
     * @param dice the block-die faces, in roll order.
     * @return index into {@code dice} of the chosen face.
     */
    public int blockChoice(Game game, String attackerId, String defenderId, int[] dice,
            boolean ownChoice) {
        boolean defHasBall = hasBall(game, defenderId);
        Player<?> att = game.getPlayerById(attackerId);
        Player<?> def = game.getPlayerById(defenderId);
        boolean attBlock = hasSkillNamed(att, "Block");
        boolean attWrestle = hasSkillNamed(att, "Wrestle");
        boolean attTackle = hasSkillNamed(att, "Tackle");
        boolean defBlock = hasSkillNamed(def, "Block");
        boolean defDodge = hasSkillNamed(def, "Dodge");
        boolean surf = canSurf(game, attackerId, defenderId);

        sampler.clear();
        for (int d : dice) {
            float w;
            switch (d) {
                case 6:
                    w = 0.90f;
                    break;
                case 5:
                    w = (defDodge && !attTackle) ? 0.30f : (surf ? 0.95f : 0.80f);
                    break;
                case 2: {
                    boolean attDown = !attBlock && !attWrestle;
                    boolean defDown = !defBlock;
                    if (!attDown && defDown) {
                        w = 0.70f;
                    } else if (attDown && defDown) {
                        w = defHasBall ? 0.50f : 0.30f;
                    } else if (attDown) {
                        w = 0.10f;
                    } else {
                        w = 0.35f;
                    }
                    break;
                }
                case 1:
                    w = 0.05f;
                    break;
                default:
                    w = surf ? 0.80f : 0.40f;
                    break;
            }
            if (!ownChoice) {
                w = 1.0f - w;
            }
            sampler.push(w);
        }
        return sampler.pick(0.12f);
    }

    /**
     * Rust checks `p.has_skill(SkillId::Block)` — the literal SKILL, not one of the properties a
     * skill happens to register. Match on the skill NAME so the two sides mean the same thing:
     * several properties are shared by more than one skill, and keying off a property would make
     * e.g. a Wrestle-only player read as having Block.
     */
    private static boolean hasSkillNamed(Player<?> p, String name) {
        if (p == null) {
            return false;
        }
        for (com.fumbbl.ffb.model.skill.Skill s : p.getSkills()) {
            if (s != null && name.equals(s.getName())) {
                return true;
            }
        }
        return false;
    }

    /** Rust `on_pitch`: 0..=25 by 0..=14. Off-pitch means the push crowd-surfs the defender. */
    private static boolean onPitch(FieldCoordinate c) {
        return c.getX() >= 0 && c.getX() <= 25 && c.getY() >= 0 && c.getY() <= 14;
    }

    /** Rust `endzone_x`: home attacks toward x=25, away toward x=0. */
    private static int endzoneX(boolean home) {
        return home ? 25 : 0;
    }

    /** Rust `endzone_distance`. */
    private static int endzoneDistance(FieldCoordinate c, boolean home) {
        return Math.abs(c.getX() - endzoneX(home));
    }

    /**
     * Rust {@code AgentPrompt::Pushback}. {@code T = 0.15}.
     *
     * <p>Options are the UNLOCKED pushback squares sorted by {@code (x, y)} — the same list and the
     * same order Rust's prompt carries (`step_pushback.rs` filters `!sq.locked` and the arm sorts).
     * Pushing a player off the pitch is worth most, and worth even more when he has the ball;
     * a sideline square is worth more than an interior one; and any square further from the
     * DEFENDER's own endzone gets a 1.3 multiplier, because that is the direction that hurts him.
     *
     * @param squares unlocked pushback squares, any order — this sorts them.
     * @return the chosen square.
     */
    public FieldCoordinate pushbackChoice(Game game, String attackerId, String defenderId,
            List<FieldCoordinate> squares) {
        List<FieldCoordinate> sorted = new ArrayList<>(squares);
        sorted.sort(Comparator.comparingInt(FieldCoordinate::getX)
            .thenComparingInt(FieldCoordinate::getY));

        Player<?> defender = game.getPlayerById(defenderId);
        boolean defHome = defender != null && game.getTeamHome().hasPlayer(defender);
        com.fumbbl.ffb.model.FieldModel fm = game.getFieldModel();
        FieldCoordinate ball = fm.getBallCoordinate();
        boolean ballCarried = ball != null && fm.isBallInPlay() && !fm.isBallMoving();
        FieldCoordinate defCoord = (defender != null) ? fm.getPlayerCoordinate(defender) : null;
        boolean defHasBall = ballCarried && defCoord != null && ball.equals(defCoord);

        Player<?> attacker = game.getPlayerById(attackerId);
        FieldCoordinate attCoord = (attacker != null) ? fm.getPlayerCoordinate(attacker) : null;

        sampler.clear();
        for (FieldCoordinate sq : sorted) {
            float w;
            if (!onPitch(sq)) {
                w = defHasBall ? 1.0f : 0.95f;
            } else if (sq.getY() == 0 || sq.getY() == 14) {
                w = 0.55f;
            } else {
                w = 0.20f;
            }
            if (attCoord != null
                && endzoneDistance(sq, !defHome) > endzoneDistance(attCoord, !defHome)) {
                w *= 1.3f;
            }
            sampler.push(w);
        }
        return sorted.get(sampler.pick(0.15f));
    }

    // ── scored arms ────────────────────────────────────────────────────────────
    //
    // Each mirrors one arm of the Rust `act_boardless` match, option for option.

    /**
     * Rust {@code AgentPrompt::CoinChoice}: two options at a flat 0.5 each, {@code T = 1.0}.
     *
     * <p>A genuine coin flip when sampled; under argmax the first strict maximum wins, so heads.
     */
    private boolean coinChoice() {
        sampler.clear();
        sampler.push(0.5f); // index 0: heads
        sampler.push(0.5f); // index 1: tails
        return sampler.pick(1.0f) == 0;
    }

    /**
     * Rust {@code AgentPrompt::ReceiveChoice}: receiving is worth more in the second half, when
     * there is no drive back. {@code T = 0.30}.
     */
    private boolean receiveChoice(Game game) {
        // Rust reads `g.half`, which is already 1 when this prompt fires. Java's Game.getHalf()
        // still returns 0 at the pre-kickoff receive choice, so reading it raw would score 0.85
        // where Rust scores 0.65 -- two different distributions that happen to agree about 82% of
        // the time, which showed up as 9 of 100 seeds diverging at the very first logged step.
        // `Math.max(1, ...)` is the same normalisation ParityRunner.stateString already applies to
        // this field for exactly this reason.
        float w = Math.max(1, game.getHalf()) == 1 ? 0.65f : 0.85f;
        sampler.clear();
        sampler.push(w);          // index 0: receive
        sampler.push(1.0f - w);   // index 1: kick
        return sampler.pick(0.30f) == 0;
    }
}
