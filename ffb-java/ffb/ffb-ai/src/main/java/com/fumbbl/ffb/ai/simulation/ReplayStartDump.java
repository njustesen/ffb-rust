package com.fumbbl.ffb.ai.simulation;

import com.eclipsesource.json.JsonArray;
import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.json.UtilJson;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.commands.ServerCommandGameState;
import com.fumbbl.ffb.net.NetCommandFactory;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * BACKLOG §O O2(a) diagnostic: the two states no {@code .states} line shows on its own.
 * Per replay (same input list rule as {@code FumbblReplayParser --per-replay}: dirs sorted by
 * name, deduplicated by replay id, first wins) it decodes the {@code serverGameState} exactly as
 * {@link FumbblReplayParser#parseFile} does (incl. {@code dropUnknownInducements}) and writes
 * <ul>
 *   <li>{@code start_hashes.txt}: {@code <id> <sha gs> <sha sog>} per replay, where gs = the
 *   canonical JSON of the decoded end-of-game {@link Game}, sog = that of
 *   {@link FumbblReplayParser#startOfGame}; {@code <id> ERROR <exception>} if either throws;</li>
 *   <li>with {@code --full}: {@code <id>.gs.json} and {@code <id>.sog.json} (the canonical texts).</li>
 * </ul>
 * The Rust loader ({@code ffb-replay start-check}) must reproduce both, byte for byte.
 * Usage: {@code ReplayStartDump <outDir> [--full] <dir|file.ffbr>...}
 */
public final class ReplayStartDump {

    private ReplayStartDump() {
    }

    public static void main(String[] args) throws Exception {
        Logger.getLogger("").setLevel(Level.WARNING);
        Logger.getLogger("org.eclipse.jetty").setLevel(Level.OFF);
        if (args.length < 2) {
            System.err.println("Usage: ReplayStartDump <outDir> [--full] <dir|file.ffbr>...");
            System.exit(1);
        }
        File outDir = new File(args[0]);
        boolean full = false;
        List<File> inputs = new ArrayList<>();
        for (int i = 1; i < args.length; i++) {
            if ("--full".equals(args[i])) {
                full = true;
            } else {
                inputs.add(new File(args[i]));
            }
        }
        java.util.LinkedHashMap<String, File> byId = new java.util.LinkedHashMap<>();
        for (File in : inputs) {
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
        outDir.mkdirs();
        HeadlessFantasyFootballServer server = new HeadlessFantasyFootballServer();
        StringBuilder hashes = new StringBuilder();
        int done = 0;
        int errors = 0;
        long t0 = System.currentTimeMillis();
        for (java.util.Map.Entry<String, File> e : byId.entrySet()) {
            String id = e.getKey();
            FumbblReplayParser.CUR = new FumbblReplayParser.PerReplay();
            try {
                Game game = decodeGameState(e.getValue(), server);
                if (game == null) {
                    hashes.append(id).append(" ERROR no serverGameState\n");
                    errors++;
                } else {
                    String gs = FumbblReplayParser.canonicalGameJson(game);
                    String sog = FumbblReplayParser.canonicalGameJson(FumbblReplayParser.startOfGame(game));
                    hashes.append(id).append(' ')
                        .append(FumbblReplayParser.sha256Hex(gs.getBytes(StandardCharsets.UTF_8))).append(' ')
                        .append(FumbblReplayParser.sha256Hex(sog.getBytes(StandardCharsets.UTF_8))).append('\n');
                    if (full) {
                        FumbblReplayParser.writeText(new File(outDir, id + ".gs.json"), gs);
                        FumbblReplayParser.writeText(new File(outDir, id + ".sog.json"), sog);
                    }
                }
            } catch (Throwable t) {
                hashes.append(id).append(" ERROR ").append(String.valueOf(t).replace('\n', ' ')).append('\n');
                errors++;
            }
            done++;
            if (done % 200 == 0) {
                System.out.printf("  [%d/%d] %d errors, %.1f s%n", done, byId.size(), errors,
                    (System.currentTimeMillis() - t0) / 1000.0);
            }
        }
        FumbblReplayParser.writeText(new File(outDir, "start_hashes.txt"), hashes.toString());
        System.out.printf("Done. %d replays, %d errors, %.1f s%n", done, errors,
            (System.currentTimeMillis() - t0) / 1000.0);
    }

    /** The serverGameState decode of {@link FumbblReplayParser#parseFile}, verbatim. */
    static Game decodeGameState(File file, HeadlessFantasyFootballServer server) throws Exception {
        byte[] raw = Files.readAllBytes(file.toPath());
        JsonObject root = UtilJson.gunzip(raw).asObject();
        JsonArray commandArray = root.get("commandArray").asArray();
        if (commandArray == null || commandArray.size() == 0) {
            return null;
        }
        NetCommandFactory factory = new NetCommandFactory(server);
        for (int i = 0; i < commandArray.size(); i++) {
            JsonObject cmdObj = commandArray.get(i).asObject();
            if ("serverGameState".equals(cmdObj.getString("netCommandId", ""))) {
                ServerCommandGameState gsCmd =
                    (ServerCommandGameState) factory.forJsonValue(server, commandArray.get(i));
                if (gsCmd != null && gsCmd.getGame() != null && cmdObj.get("game") != null
                    && FumbblReplayParser.dropUnknownInducements(gsCmd.getGame(), cmdObj.get("game").asObject())) {
                    gsCmd = (ServerCommandGameState) factory.forJsonValue(server, commandArray.get(i));
                }
                return gsCmd == null ? null : gsCmd.getGame();
            }
        }
        return null;
    }
}
