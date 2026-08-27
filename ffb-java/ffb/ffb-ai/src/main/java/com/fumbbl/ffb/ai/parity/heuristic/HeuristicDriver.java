package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.IDialogParameter;
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
