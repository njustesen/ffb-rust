package com.fumbbl.ffb.ai.simulation;

import com.eclipsesource.json.JsonArray;
import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.json.UtilJson;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.NetCommand;
import com.fumbbl.ffb.net.NetCommandFactory;
import com.fumbbl.ffb.net.commands.ServerCommandGameState;

import java.io.File;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.TreeSet;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * BACKLOG §O diagnostic: finds every value in the replay corpus that the in-repo engine cannot
 * represent. Every net command of every replay is decoded with the engine (exactly as
 * FumbblReplayParser does, unknown model-change ids dropped first) and re-encoded with
 * {@code toJsonValue()}; any leaf that does not survive the round trip is counted under
 * {@code <netCommandId> <path> <kind>} with its original values. A newer-server enum value
 * decodes to null and shows up here as "lost"; an exception while encoding shows up as "throws".
 *
 * Usage: ReplayRoundTripCensus <out.txt> <dir|file.ffbr>...
 */
public class ReplayRoundTripCensus {

    static final int VALUE_CAP = 12;
    /** signature -> count */
    static final Map<String, Integer> COUNTS = new TreeMap<>();
    /** signature -> some original values */
    static final Map<String, TreeSet<String>> VALUES = new TreeMap<>();
    /** signature -> some replay ids */
    static final Map<String, TreeSet<String>> REPLAYS = new TreeMap<>();

    public static void main(String[] args) throws Exception {
        Logger.getLogger("").setLevel(Level.SEVERE);
        File out = new File(args[0]);
        List<File> files = new ArrayList<>();
        java.util.Set<String> seen = new java.util.HashSet<>();
        for (int i = 1; i < args.length; i++) {
            File f = new File(args[i]);
            File[] list = f.isDirectory() ? f.listFiles(x -> x.getName().endsWith(".ffbr")) : new File[]{f};
            if (list == null) continue;
            java.util.Arrays.sort(list, java.util.Comparator.comparing(File::getName));
            for (File x : list) if (seen.add(x.getName())) files.add(x);
        }
        HeadlessFantasyFootballServer server = new HeadlessFantasyFootballServer();
        int n = 0;
        for (File f : files) {
            String id = f.getName().replaceFirst("\\.ffbr$", "");
            FumbblReplayParser.CUR = new FumbblReplayParser.PerReplay();
            try {
                census(f, id, server);
            } catch (Exception e) {
                note(id, "replay", "", "exception", String.valueOf(e));
            }
            if (++n % 100 == 0) System.out.println("  " + n + "/" + files.size());
        }
        StringBuilder sb = new StringBuilder();
        sb.append("files ").append(files.size()).append('\n');
        for (Map.Entry<String, Integer> e : COUNTS.entrySet()) {
            sb.append(e.getValue()).append('\t').append(e.getKey()).append("\t values=").append(VALUES.get(e.getKey()))
                .append("\t replays=").append(REPLAYS.get(e.getKey())).append('\n');
        }
        Files.write(out.toPath(), sb.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8));
        System.out.println("signatures " + COUNTS.size() + " -> " + out);
    }

    static void census(File file, String id, HeadlessFantasyFootballServer server) throws Exception {
        JsonObject root = UtilJson.gunzip(Files.readAllBytes(file.toPath())).asObject();
        JsonArray commands = root.get("commandArray").asArray();
        NetCommandFactory factory = new NetCommandFactory(server);
        Game game = null;
        for (JsonValue v : commands) {
            JsonObject cmd = v.asObject();
            String nid = cmd.getString("netCommandId", "");
            if (game == null) {
                if (!"serverGameState".equals(nid)) continue;
                ServerCommandGameState gs = (ServerCommandGameState) factory.forJsonValue(server, v);
                if (FumbblReplayParser.dropUnknownInducements(gs.getGame(), cmd.get("game").asObject())) {
                    gs = (ServerCommandGameState) factory.forJsonValue(server, v);
                }
                game = gs.getGame();
                roundTrip(id, nid, v, gs);
                continue;
            }
            if ("serverModelSync".equals(nid)) {
                JsonValue mcl = cmd.get("modelChangeList");
                if (mcl != null && mcl.isObject() && mcl.asObject().get("modelChangeArray") != null) {
                    JsonArray kept = new JsonArray();
                    for (JsonValue mc : mcl.asObject().get("modelChangeArray").asArray()) {
                        String mid = mc.isObject() ? mc.asObject().getString("modelChangeId", "") : "";
                        // the parser's own rules first: what is left over is what they miss
                        if (FumbblReplayParser.KNOWN_MODEL_CHANGES.contains(mid)
                            && !FumbblReplayParser.dropUnknownValues(game, mc.asObject())) kept.add(mc);
                    }
                    mcl.asObject().set("modelChangeArray", kept);
                }
                FumbblReplayParser.dropUnknownReports(cmd);
            }
            NetCommand decoded;
            try {
                decoded = factory.forJsonValue(game.getRules(), v);
            } catch (Exception e) {
                StackTraceElement[] st = e.getStackTrace();
                note(id, nid, "", "decode-throws", e + (st.length > 1 ? " @ " + st[0] + " < " + st[1] : ""));
                continue;
            }
            if (decoded == null) {
                note(id, nid, "", "decode-null", "");
                continue;
            }
            roundTrip(id, nid, v, decoded);
        }
    }

    static void roundTrip(String id, String nid, JsonValue original, NetCommand decoded) {
        JsonValue again;
        try {
            again = decoded.toJsonValue();
        } catch (Exception e) {
            StackTraceElement[] st = e.getStackTrace();
            String where = "";
            for (StackTraceElement s : st) {
                if (s.getClassName().startsWith("com.fumbbl")) { where = " @ " + s; break; }
            }
            note(id, nid, "", "encode-throws", e + where);
            return;
        }
        diff(id, nid, "", "", original, again);
    }

    static void diff(String id, String nid, String path, String ctx, JsonValue a, JsonValue b) {
        if (a == null) a = JsonValue.NULL;
        if (b == null) b = JsonValue.NULL;
        if (a.isObject() && b.isObject()) {
            JsonObject ao = a.asObject(), bo = b.asObject();
            String c = ctx;
            String mid = ao.getString("modelChangeId", null);
            if (mid != null) c = mid;
            JsonValue rid = ao.get("reportId");
            if (rid != null && rid.isString()) c = rid.asString();
            TreeSet<String> keys = new TreeSet<>(ao.names());
            keys.addAll(bo.names());
            for (String k : keys) {
                diff(id, nid, path + "." + k, c, ao.get(k), bo.get(k));
            }
            return;
        }
        if (a.isArray() && b.isArray()) {
            JsonArray aa = a.asArray(), ba = b.asArray();
            if (aa.size() != ba.size()) {
                // unordered containers or dropped elements: compare as multisets of text
                List<String> as = new ArrayList<>(), bs = new ArrayList<>();
                for (JsonValue x : aa) as.add(x.toString());
                for (JsonValue x : ba) bs.add(x.toString());
                List<String> lost = new ArrayList<>(as);
                for (String x : bs) lost.remove(x);
                note(id, nid, ctx + " " + path + "[]", "array-size " + aa.size() + "->" + ba.size(), lost.toString());
                return;
            }
            if (!aa.equals(ba)) {
                // a hash container serialises in an arbitrary order: same multiset = no loss
                List<String> as = new ArrayList<>(), bs = new ArrayList<>();
                for (JsonValue x : aa) as.add(x.toString());
                for (JsonValue x : ba) bs.add(x.toString());
                java.util.Collections.sort(as);
                java.util.Collections.sort(bs);
                if (as.equals(bs)) return;
            }
            for (int i = 0; i < aa.size(); i++) {
                diff(id, nid, path + "[]", ctx, aa.get(i), ba.get(i));
            }
            return;
        }
        if (a.equals(b)) return;
        // numbers equal in value (e.g. 1 vs 1.0) are not a loss
        if (a.isNumber() && b.isNumber() && a.asDouble() == b.asDouble()) return;
        String kind;
        if (b.isNull()) kind = "lost";
        else if (a.isNull()) kind = "added";
        else kind = "changed";
        note(id, nid, ctx + " " + path, kind, a + (kind.equals("changed") ? " -> " + b : ""));
    }

    static void note(String id, String nid, String path, String kind, String value) {
        String sig = nid + " " + path.trim() + " " + kind;
        COUNTS.merge(sig, 1, Integer::sum);
        TreeSet<String> vs = VALUES.computeIfAbsent(sig, k -> new TreeSet<>());
        if (vs.size() < VALUE_CAP) vs.add(value.length() > 160 ? value.substring(0, 160) : value);
        TreeSet<String> rs = REPLAYS.computeIfAbsent(sig, k -> new TreeSet<>());
        if (rs.size() < 5) rs.add(id);
    }
}
