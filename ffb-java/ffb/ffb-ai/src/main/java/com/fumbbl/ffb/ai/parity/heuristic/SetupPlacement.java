package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.option.GameOptionId;
import com.fumbbl.ffb.option.UtilGameOption;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Kick-off setup scorer — mirror of Rust {@code agent/setup_heuristic.rs} (docs/HEURISTIC_AGENT.md
 * §6.21).
 *
 * <p>One placement per decision: every {@code (reserve, legal square)} pair is scored, the shared
 * {@link Sampler} picks one at {@code T = 0.30}, the harness places it, and the loop repeats until
 * {@link #enumerate} returns nothing. Legality is by construction — the line of scrimmage is
 * filled first, a full wide zone is not offered, the field cap stops the loop, and a
 * {@code needsToBeSetUp} player is forced in before the slots run out — so the engine's
 * {@code SetupMechanic.checkSetup} never fires.
 *
 * <p><b>Every arithmetic statement below is in the same order as the Rust line it mirrors</b>:
 * {@code float} constants times {@code (float)} integer terms, summed left to right, no
 * transcendental functions. Java {@code float} arithmetic is IEEE-754 single precision without
 * FMA contraction, exactly as Rust's {@code f32}, so the two agree bit for bit — which
 * {@code SetupPlacementTest} pins against the golden Rust emits.
 *
 * <p>Options are enumerated players-by-jersey, then squares column-major in the HOME frame
 * ({@code x 0..=12} outer, {@code y 0..=14} inner). Player ids never enter an ordering.
 */
public final class SetupPlacement {

    /** Rust {@code SETUP_T}. */
    public static final float SETUP_T = 0.30f;
    /** Columns of the own half in the home frame. */
    public static final int HW = 13;
    /** Rows of the pitch. */
    public static final int HH = 15;

    private static final int DEFAULT_MAX_FIELD = 11;
    private static final int DEFAULT_MAX_WIDE = 2;
    private static final int DEFAULT_MIN_LOS = 3;

    private SetupPlacement() {
    }

    /** Rust {@code SetupPlayer}. */
    public static final class SPlayer {
        public String id;
        public int nr;
        public int st;
        public int av;
        public int ma;
        /** Agility on the BB2016 scale: raw AG in BB2016, {@code 7 - target} otherwise. */
        public int agg;
        /** Position cost in thousands of gold. */
        public int costK;
        public boolean block;
        public boolean guard;
        public boolean standFirm;
        public boolean sideStep;
        public boolean fend;
        public boolean thickSkull;
        public boolean mighty;
        public boolean frenzy;
        public boolean stunty;
        public boolean sureHands;
        public boolean passer;
        public boolean catcher;
        public boolean nerves;
        public boolean dodge;
        public boolean sprint;
        public boolean leap;
        public boolean bigHand;
        public boolean kick;
        public boolean kor;
        public boolean negatrait;
        public boolean noHands;
        public boolean loner;
        public boolean ballAndChain;
        public boolean mustField;

        boolean slow() {
            return ma <= 5;
        }

        /** Rust {@code SetupPlayer::from_player}. */
        public static SPlayer of(Player<?> p, boolean bb2016) {
            SPlayer s = new SPlayer();
            s.id = p.getId();
            s.nr = p.getNr();
            s.st = p.getStrengthWithModifiers();
            s.av = p.getArmourWithModifiers();
            s.ma = p.getMovementWithModifiers();
            int ag = p.getAgilityWithModifiers();
            s.agg = bb2016 ? ag : 7 - ag;
            s.costK = (p.getPosition() == null ? 0 : p.getPosition().getCost()) / 1000;
            s.block = has(p, "block") || has(p, "wrestle");
            s.guard = has(p, "guard");
            s.standFirm = has(p, "standfirm");
            s.sideStep = has(p, "sidestep");
            s.fend = has(p, "fend");
            s.thickSkull = has(p, "thickskull");
            s.mighty = has(p, "mightyblow") || has(p, "claw") || has(p, "claws");
            s.frenzy = has(p, "frenzy");
            s.stunty = has(p, "stunty") || has(p, "titchy");
            s.sureHands = has(p, "surehands");
            s.passer = has(p, "pass") || has(p, "accurate");
            s.catcher = has(p, "catch") || has(p, "divingcatch");
            s.nerves = has(p, "nervesofsteel");
            s.dodge = has(p, "dodge");
            s.sprint = has(p, "sprint") || has(p, "surefeet");
            s.leap = has(p, "leap");
            s.bigHand = has(p, "bighand") || has(p, "extraarms");
            s.kick = has(p, "kick");
            s.kor = has(p, "kickoffreturn");
            s.negatrait = has(p, "bonehead") || has(p, "reallystupid") || has(p, "wildanimal")
                || has(p, "takeroot") || has(p, "bloodlust") || has(p, "animalsavagery")
                || has(p, "unchannelledfury");
            s.noHands = has(p, "nohands");
            s.loner = has(p, "loner");
            s.ballAndChain = has(p, "ballandchain");
            s.mustField = p.hasSkillProperty(NamedProperties.needsToBeSetUp);
            return s;
        }
    }

    /**
     * Rust {@code Player::has_skill(SkillId::X)}. Rust's {@code SkillId} is derived from the Java
     * skill CLASS name by {@code from_class_name}, which lower-cases the alphanumerics — so the
     * faithful mirror compares the class simple name the same way, not the display name, which is
     * spelled differently per edition ({@code Bone-Head} / {@code Bone Head}).
     */
    static boolean has(Player<?> p, String key) {
        for (com.fumbbl.ffb.model.skill.Skill s : p.getSkillsIncludingTemporaryOnes()) {
            if (s != null && key.equals(normalise(s.getClass().getSimpleName()))) {
                return true;
            }
        }
        return false;
    }

    private static String normalise(String s) {
        StringBuilder b = new StringBuilder(s.length());
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (Character.isLetterOrDigit(c)) {
                b.append(Character.toLowerCase(c));
            }
        }
        return b.toString();
    }

    static float clamp(float v, float lo, float hi) {
        return Math.min(Math.max(v, lo), hi);
    }

    /** Rust {@code cheapness}. */
    public static float cheapness(SPlayer p) {
        if (p.stunty) {
            return 0.0f;
        }
        return 0.35f * clamp((float) (70 - p.costK) / 30.0f, -1.5f, 1.0f);
    }

    /** Rust {@code line_score}. */
    public static float lineScore(SPlayer p) {
        float w = 0.55f * (float) (p.st - 3) + 0.30f * (float) (p.av - 8);
        if (p.av <= 7) {
            w -= 0.6f;
        }
        if (p.block) {
            w += 0.5f;
        }
        if (p.guard) {
            w += 0.4f;
        }
        if (p.standFirm) {
            w += 0.3f;
        }
        if (p.fend || p.sideStep) {
            w += 0.2f;
        }
        if (p.thickSkull) {
            w += 0.2f;
        }
        if (p.mighty) {
            w += 0.15f;
        }
        if (p.stunty) {
            w -= 1.0f;
        } else {
            w += cheapness(p);
        }
        if (p.negatrait) {
            w -= 0.15f;
        }
        if (p.sureHands || p.passer) {
            w -= 0.35f;
        }
        if (p.catcher && p.ma >= 7) {
            w -= 0.25f;
        }
        return w;
    }

    /** Rust {@code handler_score}. */
    public static float handlerScore(SPlayer p) {
        float w = 0.35f * (float) (p.agg - 3) + 0.12f * (float) (p.ma - 6);
        if (p.sureHands) {
            w += 0.6f;
        }
        if (p.passer) {
            w += 0.35f;
        }
        if (p.bigHand) {
            w += 0.2f;
        }
        if (p.kor) {
            w += 0.25f;
        }
        if (p.dodge) {
            w += 0.1f;
        }
        if (p.sprint) {
            w += 0.1f;
        }
        if (p.leap) {
            w += 0.1f;
        }
        if (p.noHands || p.ballAndChain) {
            w -= 3.0f;
        }
        if (p.negatrait) {
            w -= 0.8f;
        }
        if (p.loner) {
            w -= 0.3f;
        }
        if (p.stunty) {
            w -= 0.3f;
        }
        if (p.st >= 5) {
            w -= 0.4f;
        }
        return w;
    }

    /** Rust {@code receiver_score}. */
    public static float receiverScore(SPlayer p) {
        float w = 0.35f * (float) (p.ma - 6) + 0.25f * (float) (p.agg - 3);
        if (p.catcher) {
            w += 0.5f;
        }
        if (p.nerves) {
            w += 0.2f;
        }
        if (p.dodge) {
            w += 0.25f;
        }
        if (p.sprint) {
            w += 0.15f;
        }
        if (p.leap) {
            w += 0.1f;
        }
        if (p.noHands || p.ballAndChain) {
            w -= 3.0f;
        }
        if (p.negatrait) {
            w -= 0.6f;
        }
        if (p.st >= 5) {
            w -= 0.4f;
        }
        return w;
    }

    /** Rust {@code fragile}. */
    public static float fragile(SPlayer p) {
        float f = clamp((float) (p.costK - 70) / 40.0f, 0.0f, 1.0f)
            * clamp((float) (9 - p.av) / 2.0f, 0.0f, 1.0f);
        if (p.stunty) {
            f += 0.5f;
        }
        return Math.min(f, 1.0f);
    }

    /** Rust {@code power}. */
    public static float power(SPlayer p) {
        float w = (float) p.st + 0.5f * (float) (p.av - 8);
        if (p.block) {
            w += 0.5f;
        }
        if (p.guard) {
            w += 0.4f;
        }
        if (p.mighty) {
            w += 0.3f;
        }
        if (p.standFirm || p.sideStep) {
            w += 0.3f;
        }
        if (p.frenzy) {
            w += 0.2f;
        }
        if (p.stunty) {
            w -= 0.4f;
        }
        if (p.negatrait) {
            w -= 0.3f;
        }
        return w;
    }

    /** Rust {@code team_power}: the eleven strongest, ties by jersey. */
    public static float teamPower(List<SPlayer> players) {
        List<float[]> ps = new ArrayList<>();
        for (SPlayer p : players) {
            ps.add(new float[] {power(p), (float) p.nr});
        }
        ps.sort((a, b) -> {
            int c = Float.compare(b[0], a[0]);
            return c != 0 ? c : Float.compare(a[1], b[1]);
        });
        float acc = 0.0f;
        for (int i = 0; i < ps.size() && i < 11; i++) {
            acc += ps.get(i)[0];
        }
        return acc;
    }

    /** Rust {@code mean_ma}. */
    public static float meanMa(List<SPlayer> players) {
        if (players.isEmpty()) {
            return 0.0f;
        }
        int sum = 0;
        for (SPlayer p : players) {
            sum += p.ma;
        }
        return (float) sum / (float) players.size();
    }

    /** Rust {@code Counts}. */
    public static final class Counts {
        public int placed;
        public int losMine;
        public int wingUp;
        public int wingLow;
        public int deepHave;
        public int deepest;
        public int up;
        public int low;
    }

    /** Rust {@code SetupBoard}: the own half in the HOME frame of the team setting up. */
    public static final class Board {
        public boolean home;
        public boolean offence;
        public boolean weaker;
        public boolean stronger;
        public boolean fast;
        public boolean oppFrenzy;
        public int maxField = DEFAULT_MAX_FIELD;
        public int maxWide = DEFAULT_MAX_WIDE;
        public int minLos = DEFAULT_MIN_LOS;
        public final boolean[][] occupied = new boolean[HW][HH];
        public final boolean[][] mine = new boolean[HW][HH];
        public final int[][] oppAdj = new int[HW][HH];
        public int losOpp;

        public Board(boolean home, boolean offence) {
            this.home = home;
            this.offence = offence;
        }

        /** Raw (home-frame) square → the coordinate the field model stores for this team. */
        public FieldCoordinate stored(int x, int y) {
            FieldCoordinate raw = new FieldCoordinate(x, y);
            return home ? raw : raw.transform();
        }

        public void placeMine(int x, int y) {
            if (x >= 0 && x < HW && y >= 0 && y < HH) {
                mine[x][y] = true;
                occupied[x][y] = true;
            }
        }

        /** Rust {@code add_opponent}: a STORED coordinate. */
        public void addOpponent(FieldCoordinate storedC) {
            FieldCoordinate raw = home ? storedC : storedC.transform();
            if (raw.getX() >= 0 && raw.getX() < HW && raw.getY() >= 0 && raw.getY() < HH) {
                occupied[raw.getX()][raw.getY()] = true;
            }
            int theirLosX = home ? 13 : 12;
            if (storedC.getX() == theirLosX && storedC.getY() >= 4 && storedC.getY() <= 10) {
                losOpp++;
            }
            for (int x = 0; x < HW; x++) {
                for (int y = 0; y < HH; y++) {
                    FieldCoordinate s = stored(x, y);
                    int dx = Math.abs(s.getX() - storedC.getX());
                    int dy = Math.abs(s.getY() - storedC.getY());
                    if (dx <= 1 && dy <= 1 && (dx | dy) != 0) {
                        oppAdj[x][y]++;
                    }
                }
            }
        }

        public Counts counts() {
            Counts c = new Counts();
            for (int x = 0; x < HW; x++) {
                for (int y = 0; y < HH; y++) {
                    if (!mine[x][y]) {
                        continue;
                    }
                    c.placed++;
                    int d = 12 - x;
                    if (x == 12 && y >= 4 && y <= 10) {
                        c.losMine++;
                    }
                    if (y <= 3) {
                        c.wingUp++;
                    }
                    if (y >= 11) {
                        c.wingLow++;
                    }
                    if (d >= 5) {
                        c.deepHave++;
                    }
                    if (d > c.deepest) {
                        c.deepest = d;
                    }
                    if (y < 7) {
                        c.up++;
                    }
                    if (y > 7) {
                        c.low++;
                    }
                }
            }
            return c;
        }

        int friendsAdj(int x, int y) {
            int n = 0;
            for (int dx = -1; dx <= 1; dx++) {
                for (int dy = -1; dy <= 1; dy++) {
                    if (dx == 0 && dy == 0) {
                        continue;
                    }
                    int nx = x + dx;
                    int ny = y + dy;
                    if (nx >= 0 && nx < HW && ny >= 0 && ny < HH && mine[nx][ny]) {
                        n++;
                    }
                }
            }
            return n;
        }

        float contactTerm(SPlayer p, int x, int y) {
            if (!offence) {
                return 0.0f;
            }
            int c = oppAdj[x][y];
            float w = 0.0f;
            if (c == 1) {
                w += p.block ? 0.5f : 0.1f;
            } else if (c >= 2) {
                w -= 0.45f * (float) (c - 1);
                if (!p.block) {
                    w -= 0.2f;
                }
            }
            w -= 0.7f * fragile(p) * (float) c;
            return w;
        }

        float symmetryTerm(Counts c, int x, int y) {
            if (offence) {
                return 0.0f;
            }
            float w = 0.0f;
            if (y != 7 && mine[x][14 - y]) {
                w += 0.6f;
            }
            int diff = c.low - c.up;
            if (y < 7) {
                w += 0.25f * (float) Math.min(Math.max(diff, -2), 2);
            } else if (y > 7) {
                w += 0.25f * (float) Math.min(Math.max(-diff, -2), 2);
            }
            return w;
        }

        /** Rust {@code score_los}. */
        public float scoreLos(SPlayer p, Counts c, int y) {
            float w = lineScore(p);
            w += y == 7 ? 0.15f : (y == 6 || y == 8) ? 0.10f : 0.0f;
            w += contactTerm(p, 12, y);
            w += symmetryTerm(c, 12, y);
            return w;
        }

        /** Rust {@code score_square}. */
        public float scoreSquare(SPlayer p, Counts c, int x, int y) {
            int d = 12 - x;
            int mid = Math.abs(y - 7);
            boolean wide = y <= 3 || y >= 11;
            boolean sideline = y == 0 || y == 14;
            boolean edge = y == 1 || y == 13;
            boolean onLos = x == 12 && y >= 4 && y <= 10;
            int wingCount = y <= 3 ? c.wingUp : c.wingLow;

            float w = 0.0f;
            if (sideline) {
                w -= 3.0f;
            } else if (edge) {
                w -= 0.6f;
            }
            if (x == 0) {
                w -= 1.0f;
            } else if (x == 1) {
                w -= 0.4f;
            }
            if (oppFrenzy && !(p.sideStep || p.standFirm)) {
                if (edge) {
                    w -= 0.6f;
                } else if (y == 2 || y == 12) {
                    w -= 0.3f;
                }
            }
            if (p.slow()) {
                w += d <= 2 ? 0.25f : -0.15f;
                w += 0.05f * (float) (3 - Math.min(mid, 3));
            }
            float fr = fragile(p);

            if (offence) {
                switch (d) {
                    case 0:
                        break;
                    case 1:
                    case 2:
                        w += 0.30f;
                        break;
                    case 3:
                        w += 0.10f;
                        break;
                    case 4:
                        w += -0.20f;
                        break;
                    default:
                        w += -0.50f;
                        break;
                }
                if (d >= 5 && d <= 9 && mid <= 3) {
                    w += 0.9f * handlerScore(p);
                    w += c.deepHave == 0 ? 0.9f : c.deepHave == 1 ? 0.35f : -1.0f;
                    if (d >= 6 && d <= 8) {
                        w += 0.15f;
                    }
                }
                if (onLos) {
                    w += c.losMine <= losOpp ? 0.5f : -0.3f;
                    w += 0.5f * lineScore(p);
                }
                w += contactTerm(p, x, y);
                if ((y == 2 || y == 3 || y == 11 || y == 12) && d >= 1 && d <= 3) {
                    w += 0.7f * receiverScore(p);
                    if (wingCount == 0) {
                        w += 0.25f;
                    }
                }
                int f = Math.min(friendsAdj(x, y), 2);
                w += fr * (0.45f * (float) f - (d <= 1 ? 0.3f : 0.0f));
            } else {
                int targetLos = weaker ? 3 : stronger ? 6 : 4;
                if (onLos) {
                    w += c.losMine < targetLos ? 0.45f + 0.5f * lineScore(p) : -0.8f;
                }
                switch (d) {
                    case 0:
                        break;
                    case 1:
                        w += 0.10f;
                        break;
                    case 2:
                        w += 0.35f;
                        break;
                    case 3:
                        w += 0.30f;
                        break;
                    case 4:
                        w += 0.10f;
                        break;
                    case 5:
                        w += -0.10f;
                        break;
                    default:
                        w += -0.40f;
                        break;
                }
                if (weaker && fast) {
                    if (d >= 3 && d <= 6) {
                        w += 0.5f;
                    }
                    if (d <= 1) {
                        w -= 0.6f;
                    }
                    if (y <= 2 || y >= 12) {
                        w -= 0.4f;
                    }
                }
                if (c.deepest < 5 && d >= 5 && d <= 7 && mid <= 2) {
                    w += 0.5f + 0.4f * receiverScore(p);
                }
                w += symmetryTerm(c, x, y);
                if (wide && d >= 1 && d <= 3) {
                    w += wingCount == 0 ? 0.4f : 0.15f;
                }
                if (p.kick && d >= 2 && y >= 4 && y <= 10) {
                    w += 0.6f;
                }
                w += fr * (d <= 1 ? -0.5f : 0.2f);
            }
            return w;
        }
    }

    /** One chosen placement: the player and the RAW home-frame square to send. */
    public static final class Placement {
        public final String playerId;
        public final int x;
        public final int y;

        public Placement(String playerId, int x, int y) {
            this.playerId = playerId;
            this.x = x;
            this.y = y;
        }
    }

    /** Rust {@code SetupOption}: candidate index, raw square, weight. */
    public static final class Option {
        public final int player;
        public final int x;
        public final int y;
        public final float w;

        Option(int player, int x, int y, float w) {
            this.player = player;
            this.x = x;
            this.y = y;
            this.w = w;
        }
    }

    /** Rust {@code enumerate}. Empty when the setup is complete. */
    public static List<Option> enumerate(Board board, List<SPlayer> candidates) {
        Counts c = board.counts();
        List<Option> out = new ArrayList<>();
        if (candidates.isEmpty() || c.placed >= board.maxField) {
            return out;
        }
        int remaining = Math.min(c.placed + candidates.size(), board.maxField) - c.placed;
        if (remaining <= 0) {
            return out;
        }
        List<Integer> must = new ArrayList<>();
        for (int i = 0; i < candidates.size(); i++) {
            if (candidates.get(i).mustField) {
                must.add(i);
            }
        }
        List<Integer> idxs;
        if (!must.isEmpty() && must.size() >= remaining) {
            idxs = must;
        } else {
            idxs = new ArrayList<>();
            for (int i = 0; i < candidates.size(); i++) {
                idxs.add(i);
            }
        }
        boolean needLos = c.losMine < Math.min(board.minLos, c.losMine + remaining);
        for (int i : idxs) {
            SPlayer p = candidates.get(i);
            if (needLos) {
                for (int y = 4; y <= 10; y++) {
                    if (board.occupied[12][y]) {
                        continue;
                    }
                    out.add(new Option(i, 12, y, board.scoreLos(p, c, y)));
                }
            } else {
                for (int x = 0; x < HW; x++) {
                    for (int y = 0; y < HH; y++) {
                        if (board.occupied[x][y]) {
                            continue;
                        }
                        if (y <= 3 && c.wingUp >= board.maxWide) {
                            continue;
                        }
                        if (y >= 11 && c.wingLow >= board.maxWide) {
                            continue;
                        }
                        out.add(new Option(i, x, y, board.scoreSquare(p, c, x, y)));
                    }
                }
            }
        }
        return out;
    }

    /** The candidates and the options for one placement — Rust {@code setup_options}. */
    public static final class Decision {
        public final List<SPlayer> candidates;
        public final List<Option> options;

        Decision(List<SPlayer> candidates, List<Option> options) {
            this.candidates = candidates;
            this.options = options;
        }
    }

    /**
     * Rust {@code setup_options}: the reserves of the team setting up, in jersey order, and the
     * scored placements. {@code null} when nothing is left to place.
     */
    public static Decision setupOptions(Game game, boolean bb2016) {
        boolean home = game.isHomePlaying();
        Team team = home ? game.getTeamHome() : game.getTeamAway();
        Team opp = home ? game.getTeamAway() : game.getTeamHome();
        FieldModel fm = game.getFieldModel();

        Board board = new Board(home, game.isSetupOffense());
        board.maxField = opt(game, GameOptionId.MAX_PLAYERS_ON_FIELD, DEFAULT_MAX_FIELD);
        board.maxWide = opt(game, GameOptionId.MAX_PLAYERS_IN_WIDE_ZONE, DEFAULT_MAX_WIDE);
        board.minLos = opt(game, GameOptionId.MIN_PLAYERS_ON_LOS, DEFAULT_MIN_LOS);

        List<Player<?>> mine = new ArrayList<>(java.util.Arrays.asList(team.getPlayers()));
        mine.sort(Comparator.comparingInt(Player::getNr));
        List<SPlayer> candidates = new ArrayList<>();
        List<SPlayer> availMine = new ArrayList<>();
        for (Player<?> p : mine) {
            PlayerState st = fm.getPlayerState(p);
            boolean settable = st == null || st.canBeSetUpNextDrive();
            if (settable) {
                availMine.add(SPlayer.of(p, bb2016));
            }
            FieldCoordinate c = fm.getPlayerCoordinate(p);
            if (c != null && onPitch(c)) {
                FieldCoordinate raw = home ? c : c.transform();
                board.placeMine(raw.getX(), raw.getY());
                continue;
            }
            boolean reserve = st == null || st.getBase() == PlayerState.RESERVE;
            if (reserve) {
                candidates.add(SPlayer.of(p, bb2016));
            }
        }

        List<SPlayer> availOpp = new ArrayList<>();
        List<Player<?>> oppSorted = new ArrayList<>(java.util.Arrays.asList(opp.getPlayers()));
        oppSorted.sort(Comparator.comparingInt(Player::getNr));
        for (Player<?> p : oppSorted) {
            PlayerState st = fm.getPlayerState(p);
            boolean settable = st == null || st.canBeSetUpNextDrive();
            if (settable) {
                availOpp.add(SPlayer.of(p, bb2016));
            }
            FieldCoordinate c = fm.getPlayerCoordinate(p);
            if (c != null && onPitch(c)) {
                board.addOpponent(c);
            }
        }
        float myPower = teamPower(availMine);
        float oppPower = teamPower(availOpp);
        board.weaker = myPower < oppPower - 1.0f;
        board.stronger = myPower > oppPower + 1.0f;
        board.fast = meanMa(availMine) >= 6.5f;
        for (SPlayer p : availOpp) {
            if (p.frenzy) {
                board.oppFrenzy = true;
            }
        }

        List<Option> options = enumerate(board, candidates);
        if (options.isEmpty()) {
            return null;
        }
        return new Decision(candidates, options);
    }

    private static int opt(Game game, GameOptionId id, int dflt) {
        int v = UtilGameOption.getIntOption(game, id);
        return v <= 0 ? dflt : v;
    }

    static boolean onPitch(FieldCoordinate c) {
        return c.getX() >= 0 && c.getX() <= 25 && c.getY() >= 0 && c.getY() <= 14;
    }
}
