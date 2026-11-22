package com.fumbbl.ffb.ai.simulation;

import com.eclipsesource.json.JsonArray;
import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.IDialogParameter;
import com.fumbbl.ffb.dialog.DialogParameterFactory;
import com.fumbbl.ffb.json.UtilJson;
import com.fumbbl.ffb.model.Game;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * BACKLOG §O O2(a)(6) oracle: every DISTINCT dialog parameter of the replay corpus, decoded and
 * re-encoded by the engine. Sources: the {@code serverGameState}'s {@code game.dialogParameter}
 * and every {@code gameSetDialogParameter} model change (after the parser's drop rules, exactly as
 * {@link FumbblReplayParser#parseFile} applies them). Decode = {@code ModelChangeDataType
 * .DIALOG_PARAMETER} = {@code new DialogParameterFactory().forJsonValue(rules, value)}; encode =
 * {@code toJsonValue()}.
 * <p>
 * Output {@code <out>}: one line per distinct input,
 * {@code <ruleset>\t<start|change>\t<canonical RAW input>\t<canonical output | null | ERROR <exception>>}
 * (raw = before {@link FumbblReplayParser#dropUnknownValues}, which applies to model changes only), canonical =
 * {@link FumbblReplayParser#canonicalCopy} under the path {@code dialogParameter} (the same
 * unordered-array rule the game state uses). The Rust port ({@code ffb-replay dialog-check}) must
 * reproduce every output from its input.
 * Usage: {@code ReplayDialogDump <out.tsv> <dir|file.ffbr>...}
 */
public final class ReplayDialogDump {

    private ReplayDialogDump() {
    }

    public static void main(String[] args) throws Exception {
        Logger.getLogger("").setLevel(Level.WARNING);
        Logger.getLogger("org.eclipse.jetty").setLevel(Level.OFF);
        if (args.length < 2) {
            System.err.println("Usage: ReplayDialogDump <out.tsv> <dir|file.ffbr>...");
            System.exit(1);
        }
        File out = new File(args[0]);
        LinkedHashMap<String, File> byId = new LinkedHashMap<>();
        for (int i = 1; i < args.length; i++) {
            File in = new File(args[i]);
            List<File> files = new ArrayList<>();
            if (in.isDirectory()) {
                File[] found = in.listFiles(f -> f.isFile() && f.getName().endsWith(".ffbr"));
                if (found != null) {
                    Collections.addAll(files, found);
                }
                files.sort(java.util.Comparator.comparing(File::getName));
            } else {
                files.add(in);
            }
            for (File f : files) {
                byId.putIfAbsent(f.getName().replaceFirst("\\.ffbr$", ""), f);
            }
        }
        HeadlessFantasyFootballServer server = new HeadlessFantasyFootballServer();
        // "<ruleset>\t<canonical input>" -> output text, first seen order
        Map<String, String> distinct = new LinkedHashMap<>();
        long total = 0;
        int done = 0;
        long t0 = System.currentTimeMillis();
        for (Map.Entry<String, File> e : byId.entrySet()) {
            FumbblReplayParser.CUR = new FumbblReplayParser.PerReplay();
            Game game = ReplayStartDump.decodeGameState(e.getValue(), server);
            if (game == null) {
                continue;
            }
            String ruleset = String.valueOf(game.getOptions().getRulesVersion());
            JsonObject root = UtilJson.gunzip(Files.readAllBytes(e.getValue().toPath())).asObject();
            JsonArray commands = root.get("commandArray").asArray();
            boolean first = true;
            for (JsonValue v : commands) {
                JsonObject cmd = v.asObject();
                String nid = cmd.getString("netCommandId", "");
                if (first && "serverGameState".equals(nid)) {
                    first = false;
                    JsonValue g = cmd.get("game");
                    JsonValue dp = g != null && g.isObject() ? g.asObject().get("dialogParameter") : null;
                    if (dp != null) {
                        total++;
                        note(distinct, game, ruleset + "\tstart\t" + canonical(dp), dp);
                    }
                    continue;
                }
                if (!"serverModelSync".equals(nid)) {
                    continue;
                }
                JsonValue mcl = cmd.get("modelChangeList");
                if (mcl == null || !mcl.isObject() || mcl.asObject().get("modelChangeArray") == null) {
                    continue;
                }
                for (JsonValue mc : mcl.asObject().get("modelChangeArray").asArray()) {
                    if (!mc.isObject() || !"gameSetDialogParameter".equals(mc.asObject().getString("modelChangeId", ""))) {
                        continue;
                    }
                    JsonValue before = mc.asObject().get("modelChangeValue");
                    String rawKey = canonical(before == null ? JsonValue.NULL : before);
                    if (FumbblReplayParser.dropUnknownValues(game, mc.asObject())) {
                        continue;
                    }
                    JsonValue value = mc.asObject().get("modelChangeValue");
                    total++;
                    note(distinct, game, ruleset + "\tchange\t" + rawKey, value == null ? JsonValue.NULL : value);
                }
            }
            done++;
            if (done % 200 == 0) {
                System.out.printf("  [%d/%d] %d values, %d distinct, %.1f s%n", done, byId.size(), total,
                    distinct.size(), (System.currentTimeMillis() - t0) / 1000.0);
            }
        }
        StringBuilder sb = new StringBuilder();
        for (Map.Entry<String, String> d : distinct.entrySet()) {
            sb.append(d.getKey()).append('\t').append(d.getValue()).append('\n');
        }
        Files.write(out.toPath(), sb.toString().getBytes(StandardCharsets.UTF_8));
        System.out.printf("Done. %d replays, %d values, %d distinct, %.1f s%n", done, total, distinct.size(),
            (System.currentTimeMillis() - t0) / 1000.0);
    }

    static String canonical(JsonValue raw) {
        return FumbblReplayParser.canonicalCopy(raw, "dialogParameter").toString();
    }

    /** {@code key} = ruleset, source (start|change) and the canonical RAW input (before the drop rules). */
    static void note(Map<String, String> distinct, Game game, String key, JsonValue raw) {
        if (distinct.containsKey(key)) {
            return;
        }
        String text;
        try {
            IDialogParameter p = new DialogParameterFactory().forJsonValue(game.getRules(), raw);
            text = p == null ? "null" : FumbblReplayParser.canonicalCopy(p.toJsonValue(), "dialogParameter").toString();
        } catch (Throwable t) {
            text = "ERROR " + String.valueOf(t).replace('\n', ' ').replace('\t', ' ');
        }
        distinct.put(key, text);
    }
}
