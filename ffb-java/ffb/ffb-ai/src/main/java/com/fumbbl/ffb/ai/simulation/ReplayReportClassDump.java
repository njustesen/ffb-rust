package com.fumbbl.ffb.ai.simulation;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.ReportFactory;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.report.IReport;
import com.fumbbl.ffb.report.ReportId;

import java.io.File;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * BACKLOG §O O3 oracle: which report CLASS each ruleset's {@link ReportFactory} decodes every
 * {@link ReportId} into (the action records test {@code instanceof ReportSkillRoll},
 * {@code mixed.ReportInjury} vs {@code bb2016.ReportInjury} ...). One replay per ruleset supplies
 * the game (its rules = its factory source), exactly as {@code FumbblReplayParser} decodes syncs.
 * Output (stdout): {@code <ruleset>\t<ReportId name>\t<class canonical name | NONE>\t<superclass chain>}.
 * Usage: {@code ReplayReportClassDump <dir|file.ffbr>...}
 */
public final class ReplayReportClassDump {

    private ReplayReportClassDump() {
    }

    public static void main(String[] args) throws Exception {
        Logger.getLogger("").setLevel(Level.WARNING);
        Logger.getLogger("org.eclipse.jetty").setLevel(Level.OFF);
        List<File> files = new ArrayList<>();
        for (String a : args) {
            File in = new File(a);
            if (in.isDirectory()) {
                File[] found = in.listFiles(f -> f.isFile() && f.getName().endsWith(".ffbr"));
                if (found != null) {
                    List<File> sorted = new ArrayList<>();
                    Collections.addAll(sorted, found);
                    sorted.sort(java.util.Comparator.comparing(File::getName));
                    files.addAll(sorted);
                }
            } else {
                files.add(in);
            }
        }
        HeadlessFantasyFootballServer server = new HeadlessFantasyFootballServer();
        Set<String> done = new TreeSet<>();
        for (File f : files) {
            FumbblReplayParser.CUR = new FumbblReplayParser.PerReplay();
            Game game = ReplayStartDump.decodeGameState(f, server);
            if (game == null) {
                continue;
            }
            String ruleset = String.valueOf(game.getOptions().getRulesVersion());
            if (!done.add(ruleset)) {
                continue;
            }
            ReportFactory factory = game.getFactory(FactoryType.Factory.REPORT);
            for (ReportId id : ReportId.values()) {
                String cls;
                StringBuilder chain = new StringBuilder();
                try {
                    IReport report = factory.forId(id);
                    cls = report.getClass().getCanonicalName();
                    for (Class<?> c = report.getClass().getSuperclass(); c != null && c != Object.class; c = c.getSuperclass()) {
                        if (chain.length() > 0) {
                            chain.append(',');
                        }
                        chain.append(c.getCanonicalName());
                    }
                } catch (Exception e) {
                    cls = "NONE";
                }
                System.out.println(ruleset + "\t" + id.name() + "\t" + cls + "\t" + chain);
            }
            if (done.size() == 3) {
                break;
            }
        }
    }
}
