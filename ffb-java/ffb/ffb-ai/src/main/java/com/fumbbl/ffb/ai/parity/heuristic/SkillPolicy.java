package com.fumbbl.ffb.ai.parity.heuristic;

import com.eclipsesource.json.Json;
import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.JsonValue;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;

/**
 * BACKLOG §M phase 3: the skill-use policy table, mirror of Rust {@code SkillPolicy}.
 *
 * <p>Both engines parse the SAME file: the Rust CLI exports its absolute path as
 * {@code FFB_SKILL_POLICY} and this JVM inherits it. Without the variable (a bare unit test)
 * the file is found by walking up from the working directory to
 * {@code ffb-rust/data/agent/skill_policy.json}. Integer milli-weights, so both sides compute
 * the identical f32 ({@code use / 1000f}).
 */
public final class SkillPolicy {

    /** One row: base use weight and coverage-bias strength, both in thousandths. */
    public static final class Row {
        public final int useMilli;
        public final int coverageMilli;

        Row(int useMilli, int coverageMilli) {
            this.useMilli = useMilli;
            this.coverageMilli = coverageMilli;
        }
    }

    private static volatile SkillPolicy instance;

    private final Row defaultRow;
    private final Map<String, Row> rows;

    private SkillPolicy(Row defaultRow, Map<String, Row> rows) {
        this.defaultRow = defaultRow;
        this.rows = rows;
    }

    /** The process-wide table. A missing or malformed file is fatal, never a silent default. */
    public static SkillPolicy get() {
        SkillPolicy p = instance;
        if (p == null) {
            synchronized (SkillPolicy.class) {
                p = instance;
                if (p == null) {
                    p = parse(read());
                    instance = p;
                }
            }
        }
        return p;
    }

    public Row row(String skillName) {
        Row r = rows.get(skillName);
        return r != null ? r : defaultRow;
    }

    private static String read() {
        String env = System.getenv("FFB_SKILL_POLICY");
        Path path = null;
        if (env != null) {
            path = Paths.get(env);
        } else {
            for (Path dir = Paths.get("").toAbsolutePath(); dir != null; dir = dir.getParent()) {
                Path cand = dir.resolve("ffb-rust").resolve("data").resolve("agent").resolve("skill_policy.json");
                if (Files.exists(cand)) {
                    path = cand;
                    break;
                }
            }
        }
        if (path == null) {
            throw new IllegalStateException("skill_policy: FFB_SKILL_POLICY unset and no ffb-rust/data/agent/skill_policy.json above "
                + Paths.get("").toAbsolutePath());
        }
        try {
            return new String(Files.readAllBytes(path), StandardCharsets.UTF_8);
        } catch (IOException e) {
            throw new IllegalStateException("skill_policy: " + path + ": " + e, e);
        }
    }

    static SkillPolicy parse(String text) {
        JsonObject root = Json.parse(text).asObject();
        Row base = new Row(500, 1000);
        JsonValue d = root.get("default");
        Row def = d == null ? base : row(d.asObject(), base);
        Map<String, Row> rows = new HashMap<>();
        JsonValue skills = root.get("skills");
        if (skills != null) {
            for (JsonObject.Member m : skills.asObject()) {
                rows.put(m.getName(), row(m.getValue().asObject(), def));
            }
        }
        return new SkillPolicy(def, rows);
    }

    private static Row row(JsonObject o, Row d) {
        return new Row(milli(o, "use", d.useMilli), milli(o, "coverage", d.coverageMilli));
    }

    private static int milli(JsonObject o, String key, int dv) {
        JsonValue v = o.get(key);
        if (v == null) {
            return dv;
        }
        int n = v.asInt();
        if (n < 0 || n > 1000) {
            throw new IllegalStateException("skill_policy: " + key + " must be an integer in 0..1000, got " + n);
        }
        return n;
    }
}
