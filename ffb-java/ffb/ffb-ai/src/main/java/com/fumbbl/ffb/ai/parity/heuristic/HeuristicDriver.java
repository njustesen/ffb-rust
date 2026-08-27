package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.IDialogParameter;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.ReRolledAction;
import com.fumbbl.ffb.ReRolledActions;
import com.fumbbl.ffb.model.Player;
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
        if (ball == null || !fm.isBallInPlay() || fm.isBallMoving()) {
            return false;
        }
        Player<?> carrier = fm.getPlayer(ball);
        if (carrier == null) {
            return false;
        }
        return game.getTeamHome().hasPlayer(carrier) == home;
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
