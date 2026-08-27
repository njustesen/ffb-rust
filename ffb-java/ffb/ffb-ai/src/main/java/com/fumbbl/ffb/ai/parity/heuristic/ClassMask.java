package com.fumbbl.ffb.ai.parity.heuristic;

/**
 * Which {@link PromptClass}es the heuristic agent scores. Everything else is answered by
 * {@code ParityRunner}'s existing random policy, byte-for-byte as {@code AGENT_CONTRACT.md}
 * specifies.
 *
 * <p>Mirror of the Rust {@code ClassMask}. The two sides must be given the <b>same</b>
 * {@code --heur-classes} value on every run, or they will disagree about who answers what — which
 * is a divergence caused by the harness rather than by either engine.
 */
public final class ClassMask {

    /** Delegate everything — rung 0. Must reproduce the random-agent gate exactly. */
    public static final ClassMask NONE = new ClassMask(0);
    /** Score everything the agent knows how to score. */
    public static final ClassMask ALL = new ClassMask(0xffffffff);

    private final int bits;

    private ClassMask(int bits) {
        this.bits = bits;
    }

    public boolean has(PromptClass c) {
        return (bits & (1 << c.bit())) != 0;
    }

    public ClassMask with(PromptClass c) {
        return new ClassMask(bits | (1 << c.bit()));
    }

    /** {@code all}, {@code none}, or a comma-separated list of {@link PromptClass#className()}s. */
    public static ClassMask parse(String spec) {
        if (spec == null) {
            return NONE;
        }
        String s = spec.trim();
        if (s.equalsIgnoreCase("all")) {
            return ALL;
        }
        if (s.isEmpty() || s.equalsIgnoreCase("none")) {
            return NONE;
        }
        ClassMask m = NONE;
        for (String tok : s.split(",")) {
            String t = tok.trim();
            if (t.isEmpty()) {
                continue;
            }
            PromptClass c = PromptClass.byName(t);
            if (c == null) {
                StringBuilder known = new StringBuilder();
                for (PromptClass k : PromptClass.values()) {
                    if (known.length() > 0) {
                        known.append(',');
                    }
                    known.append(k.className());
                }
                throw new IllegalArgumentException(
                    "unknown prompt class '" + t + "'; known: " + known);
            }
            m = m.with(c);
        }
        return m;
    }

    @Override
    public String toString() {
        if (bits == 0) {
            return "none";
        }
        StringBuilder sb = new StringBuilder();
        for (PromptClass c : PromptClass.values()) {
            if (has(c)) {
                if (sb.length() > 0) {
                    sb.append(',');
                }
                sb.append(c.className());
            }
        }
        return sb.toString();
    }
}
