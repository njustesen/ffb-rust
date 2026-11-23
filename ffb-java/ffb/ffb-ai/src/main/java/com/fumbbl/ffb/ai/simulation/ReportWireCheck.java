package com.fumbbl.ffb.ai.simulation;

import com.eclipsesource.json.Json;
import com.eclipsesource.json.JsonArray;
import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.factory.GameOptionFactory;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.option.GameOptionId;
import com.fumbbl.ffb.option.GameOptionString;
import com.fumbbl.ffb.report.ReportList;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.util.UtilServerStartGame;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * BACKLOG §P: can the Java client decode the Rust server's reports? Every line of the Rust parity
 * report streams ({@code seed_N_rust_reports.jsonl}, the same {@code to_json} the server sends) is
 * decoded exactly as the client does ({@code ReportList.initFrom} with the edition's rules as the
 * factory source) and re-encoded with {@code toJsonValue()}; an exception, or a leaf that does not
 * survive the round trip, is counted like {@link ReplayRoundTripCensus} does. At most
 * {@code perId} lines per report id are checked.
 *
 * Usage: ReportWireCheck <out.txt> <BB2016|BB2020|BB2025> <perId> <dir>...
 * (only files under a {@code /<edition lowercase>/} directory are read).
 */
public class ReportWireCheck {

    public static void main(String[] args) throws Exception {
        Logger.getLogger("").setLevel(Level.SEVERE);
        File out = new File(args[0]);
        String edition = args[1];
        int perId = Integer.parseInt(args[2]);
        List<File> files = new ArrayList<>();
        for (int i = 3; i < args.length; i++) collect(new File(args[i]), edition.toLowerCase(), files);

        HeadlessFantasyFootballServer server = new HeadlessFantasyFootballServer();
        GameState gameState = new GameState(server);
        UtilServerStartGame.addDefaultGameOptions(gameState);
        GameOptionString ruleSet = (GameOptionString) new GameOptionFactory().createGameOption(GameOptionId.RULESVERSION);
        ruleSet.setValue(RulesCollection.Rules.valueOf(edition).name());
        gameState.getGame().getOptions().addOption(ruleSet);
        Game game = gameState.getGame();
        gameState.initRulesDependentMembers();
        game.initializeRules();

        Map<String, Integer> seen = new HashMap<>();
        int checked = 0;
        for (File f : files) {
            for (String line : Files.readAllLines(f.toPath(), StandardCharsets.UTF_8)) {
                if (line.trim().isEmpty()) continue;
                JsonObject report = Json.parse(line).asObject();
                report.remove("i");
                String rid = report.getString("reportId", "?");
                int n = seen.merge(rid, 1, Integer::sum);
                if (n > perId) continue;
                checked++;
                JsonObject list = new JsonObject();
                list.add("reports", new JsonArray().add(report));
                ReportList decoded;
                try {
                    decoded = new ReportList().initFrom(game.getRules(), list);
                } catch (Exception e) {
                    StackTraceElement[] st = e.getStackTrace();
                    ReplayRoundTripCensus.note(f.getName(), "decode", rid, "throws",
                        e + (st.length > 0 ? " @ " + st[0] : "") + " :: " + line);
                    continue;
                }
                JsonValue again;
                try {
                    again = decoded.toJsonValue();
                } catch (Exception e) {
                    ReplayRoundTripCensus.note(f.getName(), "encode", rid, "throws", String.valueOf(e));
                    continue;
                }
                ReplayRoundTripCensus.diff(f.getName(), "report", "", rid, list, again);
            }
        }
        StringBuilder sb = new StringBuilder();
        sb.append("edition ").append(edition).append(" files ").append(files.size()).append(" checked ")
            .append(checked).append(" ids ").append(seen.size()).append('\n');
        for (Map.Entry<String, Integer> e : ReplayRoundTripCensus.COUNTS.entrySet()) {
            sb.append(e.getValue()).append('\t').append(e.getKey()).append("\t values=")
                .append(ReplayRoundTripCensus.VALUES.get(e.getKey())).append('\n');
        }
        sb.append("PROBLEMS ").append(ReplayRoundTripCensus.COUNTS.size()).append('\n');
        Files.write(out.toPath(), sb.toString().getBytes(StandardCharsets.UTF_8));
        System.out.print(sb);
    }

    static void collect(File f, String edition, List<File> out) {
        if (f.isDirectory()) {
            File[] list = f.listFiles();
            if (list == null) return;
            java.util.Arrays.sort(list);
            for (File x : list) collect(x, edition, out);
        } else if (f.getName().endsWith("_rust_reports.jsonl")
            && f.getPath().replace('\\', '/').contains("/" + edition + "/")) {
            out.add(f);
        }
    }
}
