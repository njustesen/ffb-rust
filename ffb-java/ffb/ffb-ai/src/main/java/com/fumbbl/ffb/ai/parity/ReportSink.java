package com.fumbbl.ffb.ai.parity;

import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.report.IReport;
import com.fumbbl.ffb.report.ReportList;

import java.io.FileOutputStream;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;

/**
 * Captures the stock engine's report stream for one parity game.
 *
 * <p>Every {@code getResult().addReport(...)} the server makes reaches the clients through exactly
 * one place, {@code ServerCommunication.sendModelSync(...)}; the headless server's communication
 * object forwards the {@code ReportList} it receives here. One JSON line per report, the report's own
 * {@code toJsonValue()} (the same camelCase keys the client reads) plus {@code "i"}, the parity step
 * index the report was produced under. The Rust harness writes the twin file from
 * {@code game.report_list}, and {@code ffb-parity} compares the two — the coverage record that is
 * complete by construction, next to the state-hash verdict.
 *
 * <p>Static because the server (and its communication) is built once per JVM and reused across the
 * batch; {@link #open} is called per seed. Inactive (all no-ops) unless a path was opened.
 */
public final class ReportSink {
    private static PrintWriter out;
    /** The parity step index the harness is currently recording; stamped on each report line. */
    public static volatile int stepIndex = 0;

    private ReportSink() {}

    public static void open(String path) throws java.io.IOException {
        close();
        out = new PrintWriter(new java.io.OutputStreamWriter(new FileOutputStream(path), StandardCharsets.UTF_8), false);
    }

    public static boolean isOpen() {
        return out != null;
    }

    public static void record(ReportList reports) {
        if (out == null || reports == null) return;
        for (IReport report : reports.getReports()) {
            if (report == null) continue;
            JsonValue v = report.toJsonValue();
            JsonObject o = (v != null && v.isObject()) ? v.asObject() : new JsonObject();
            if (!o.names().contains("reportId") && report.getId() != null) {
                o.add("reportId", report.getId().getName());
            }
            o.add("i", stepIndex);
            out.println(o.toString());
        }
    }

    public static void close() {
        if (out != null) {
            out.flush();
            out.close();
            out = null;
        }
    }
}
