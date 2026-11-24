"""Paper figure: the setup policy's architecture, one placement step.

    python ml/setup_bc/figures/plot_architecture.py

Writes setup_policy_architecture.{pdf,svg,png} next to this script. Drawn at 11 x 5.6 in for a
full-width figure (7 in): at that size the smallest text is about 5.5 pt. Style conventions
(palette, sans fonts, white ground, editable vector text, 300 dpi) follow the
"scientific-figure-making" notes of github.com/ChenLiu-1996/figures4papers.
"""
from __future__ import annotations

import os

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
from matplotlib.patches import Circle, FancyArrowPatch, FancyBboxPatch, Rectangle  # noqa: E402

PALETTE = {
    "blue_main": "#0F4D92", "blue_secondary": "#3775BA", "blue_soft": "#DCE8F5",
    "green_1": "#DDF3DE", "green_3": "#8BCF8B", "green_dark": "#3E8E41",
    "red_1": "#F6CFCB", "red_strong": "#B64342",
    "neutral": "#CFCECE", "neutral_soft": "#F2F2F2", "grey": "#767676", "ink": "#272727",
    "highlight": "#FFD700", "gold_dark": "#B8860B", "heat": "#E08A00",
}
plt.rcParams.update({
    "font.family": ["Arial", "DejaVu Sans", "sans-serif"], "font.size": 9,
    "svg.fonttype": "none", "pdf.fonttype": 42, "ps.fonttype": 42,
})
T, S = 8.6, 7.4     # title / body point sizes
P = PALETTE


def box(ax, x0, y0, x1, y1, fill, edge, lw=1.3, z=2):
    ax.add_patch(FancyBboxPatch((x0, y0), x1 - x0, y1 - y0, boxstyle="round,pad=0,rounding_size=0.9",
                                fc=fill, ec=edge, lw=lw, zorder=z))


def title(ax, x, y, s, **kw):
    ax.text(x, y, s, fontsize=T, fontweight="bold", color=P["ink"], va="center", zorder=5, **kw)


def body(ax, x, y, s, color=None, **kw):
    ax.text(x, y, s, fontsize=S, color=color or P["ink"], va="center", zorder=5, **kw)


def wire(ax, pts, color=None, lw=1.2, arrow=True, ls="-", z=3):
    """A right-angled polyline with an arrow head on its last segment."""
    color = color or P["ink"]
    xs, ys = zip(*pts)
    if len(pts) > 2 or not arrow:
        n = len(pts) - (1 if arrow else 0)
        ax.plot(xs[:n], ys[:n], color=color, lw=lw, ls=ls, solid_capstyle="round", solid_joinstyle="round", zorder=z)
    if arrow:
        ax.add_patch(FancyArrowPatch(pts[-2], pts[-1], arrowstyle="-|>", mutation_scale=9, color=color, lw=lw,
                                     ls=ls, shrinkA=0, shrinkB=0, zorder=z))


def pitch(ax, x0, y0, w, own=(), opp=(), heat=None, current=None):
    """A 26 x 15 pitch; the side setting up is on the left. Squares are (x, y), y from the top."""
    c = w / 26.0
    h = 15 * c
    ax.add_patch(Rectangle((x0, y0), w, h, fc="white", ec="none", zorder=2))
    ax.add_patch(Rectangle((x0, y0), 13 * c, h, fc="#EEF5EC", ec="none", zorder=2))
    for band in (0, 11):                                   # wide zones of our half
        ax.add_patch(Rectangle((x0, y0 + h - (band + 4) * c), 13 * c, 4 * c, fc=P["green_1"], ec="none", zorder=2))
    if heat:
        top = max(heat.values())
        for (x, y), p in heat.items():
            ax.add_patch(Rectangle((x0 + x * c, y0 + h - (y + 1) * c), c, c, fc=P["heat"], ec="none",
                                   alpha=0.15 + 0.85 * p / top, zorder=3))
    for i in range(27):
        ax.plot([x0 + i * c] * 2, [y0, y0 + h], color=P["neutral"], lw=0.3, zorder=3)
    for j in range(16):
        ax.plot([x0, x0 + w], [y0 + j * c] * 2, color=P["neutral"], lw=0.3, zorder=3)
    ax.add_patch(Rectangle((x0, y0), w, h, fc="none", ec=P["ink"], lw=1.0, zorder=4))
    ax.plot([x0 + 13 * c] * 2, [y0, y0 + h], color=P["ink"], lw=1.0, zorder=4)
    ax.add_patch(Rectangle((x0 + 12 * c, y0 + h - 11 * c), c, 7 * c, fc="none", ec=P["green_dark"], lw=1.2, zorder=4))
    for squares, colour in ((own, P["blue_main"]), (opp, P["red_strong"])):
        for x, y in squares:
            ax.add_patch(Circle((x0 + (x + .5) * c, y0 + h - (y + .5) * c), 0.36 * c, fc=colour, ec="white", lw=0.3, zorder=5))
    if current:
        x, y = current
        ax.add_patch(Rectangle((x0 + x * c, y0 + h - (y + 1) * c), c, c, fc="none", ec=P["ink"], lw=1.2, zorder=6))
    return h


def main():
    fig, ax = plt.subplots(figsize=(11, 5.6))
    ax.set_xlim(0, 124)
    ax.set_ylim(-3, 58)
    ax.set_aspect("equal")
    ax.axis("off")
    grey, blue, gold, green, red = P["grey"], P["blue_main"], P["gold_dark"], P["green_dark"], P["red_strong"]

    # ---------------------------------------------------------------- inputs (left column)
    box(ax, 2, 47, 24, 56.5, P["neutral_soft"], grey)
    title(ax, 3.2, 54.3, "Player table (48 rows)")
    body(ax, 3.2, 51.4, "MA ST AG PA AV, cost, injuries")
    body(ax, 3.2, 48.9, "+ skills (up to 16 per player)")

    own = [(12, 6), (12, 7), (12, 8), (11, 2), (11, 12), (10, 5)]
    opp = [(13, 5), (13, 7), (13, 9), (14, 2), (14, 12), (15, 6), (15, 8), (17, 4), (17, 10), (18, 7), (21, 7)]
    title(ax, 2, 43.7, "Board (26 × 15 squares)")
    pitch(ax, 2, 29.6, 22, own, opp)

    title(ax, 2, 23.8, "Benches (2 × 16 slots)")
    for row, (y, filled, colour) in enumerate(((21.0, 7, P["blue_main"]), (19.0, 3, P["red_strong"]))):
        for i in range(16):
            fc = P["highlight"] if (row == 0 and i == 0) else (colour if i < filled else "white")
            ax.add_patch(Rectangle((2 + i * 1.375, y), 1.2, 1.2, fc=fc, ec=P["ink"], lw=0.6, zorder=4))
    ax.add_patch(Rectangle((2, 16.5), 1.2, 1.2, fc=P["highlight"], ec=P["ink"], lw=0.6, zorder=4))
    body(ax, 3.9, 17.1, "player to place (most valuable first)")

    box(ax, 2, 3, 24, 12, P["neutral_soft"], grey)
    title(ax, 3.2, 9.9, "Match and team state (37)")
    body(ax, 3.2, 7.2, "kick / receive, half, turn, score,")
    body(ax, 3.2, 4.8, "weather, re-rolls, team value, …")

    # ---------------------------------------------------------------- player encoder + embeddings
    box(ax, 30, 46.5, 46, 56.5, P["blue_soft"], blue)
    title(ax, 31.2, 54.3, "Player encoder")
    body(ax, 31.2, 51.4, "Σ skill emb. ⊕ stats")
    body(ax, 31.2, 48.9, "MLP → 16-d vector")
    wire(ax, [(24, 51.5), (30, 51.5)])
    wire(ax, [(46, 51.5), (50, 51.5), (50, 20.5)], blue, 2.0, arrow=False)
    body(ax, 51.0, 47.5, "E", color=blue, fontweight="bold")
    body(ax, 51.0, 45.2, "48 × 16", color=blue)

    wire(ax, [(24, 35.5), (50, 35.5)], arrow=False)
    body(ax, 37, 36.6, "who stands where", color=grey, ha="center", style="italic")
    wire(ax, [(24, 20.5), (50, 20.5)], arrow=False)
    body(ax, 37, 21.6, "who is left", color=grey, ha="center", style="italic")
    for y in (35.5, 20.5):
        ax.add_patch(Circle((50, y), 0.75, fc=blue, ec="white", lw=0.6, zorder=6))
    wire(ax, [(50, 35.5), (60, 35.5)], blue, 2.0)
    wire(ax, [(50, 20.5), (56, 20.5), (56, 30), (60, 30)], gold, 2.0)
    wire(ax, [(56, 20.5), (70, 20.5), (70, 10), (79, 10)], blue, 2.0)
    body(ax, 63, 21.7, "2 × 16 × 16", color=blue, ha="center")
    wire(ax, [(24, 7), (79, 7)])

    # ---------------------------------------------------------------- rules + geometry, the input stack
    box(ax, 55, 49, 77, 56.5, P["neutral_soft"], grey)
    title(ax, 56.2, 54.3, "Setup rules + geometry")
    body(ax, 56.2, 51.2, "legal squares, x / y, zones")
    wire(ax, [(66, 49), (66, 45)])

    segments = [(41.5, 45, "6 geometry", P["neutral"]), (39, 41.5, "1 legal", P["red_1"]),
                (33, 39, "16 board", P["blue_soft"]), (27, 33, "16 current", "#FFF1A8")]
    for lo, hi, label, colour in segments:
        ax.add_patch(Rectangle((60, lo), 12, hi - lo, fc=colour, ec=P["ink"], lw=0.9, zorder=4))
        body(ax, 66, (lo + hi) / 2, label, ha="center")
    body(ax, 66, 25.5, "39 × 26 × 15", ha="center")

    # ---------------------------------------------------------------- trunk
    box(ax, 75, 33, 83, 39, P["blue_soft"], blue)
    body(ax, 79, 37.2, "Conv 3×3", ha="center", fontweight="bold")
    body(ax, 79, 34.7, "39 → 64", ha="center")
    wire(ax, [(72, 36), (75, 36)])
    wire(ax, [(83, 36), (86, 36)])

    box(ax, 86, 27, 102, 45, P["blue_soft"], blue)
    title(ax, 94, 43.0, "6 × residual block", ha="center")
    body(ax, 94, 40.2, "Norm · ReLU · Conv", ha="center")
    box(ax, 88.5, 35.7, 99.5, 38.5, P["green_1"], green, lw=1.1, z=4)
    body(ax, 94, 37.1, "FiLM(g)", ha="center", fontweight="bold")
    body(ax, 94, 34.0, "Norm · ReLU · Conv", ha="center")
    body(ax, 94, 31.2, "+ skip connection", ha="center")
    body(ax, 94, 28.6, "64 ch, 26 × 15", ha="center", color=grey)

    box(ax, 79, 3, 97, 13, P["green_1"], green)
    title(ax, 80.2, 10.8, "Context MLP")
    body(ax, 80.2, 8.0, "565 → 256 → 128")
    body(ax, 80.2, 5.3, "one vector g per step")
    wire(ax, [(94, 13), (94, 27)], green, 2.0)
    body(ax, 95.0, 20, "g", color=green, fontweight="bold")

    # ---------------------------------------------------------------- heads, mask, output
    box(ax, 105.9, 32.1, 120.9, 44.1, P["blue_soft"], blue, z=1)       # the second head, behind
    box(ax, 105, 33, 120, 45, P["blue_soft"], blue)
    title(ax, 106.2, 43.0, "Policy heads × 2")
    body(ax, 106.2, 40.3, "defense | offense")
    body(ax, 106.2, 37.6, "squares: 1×1 conv")
    body(ax, 106.2, 35.0, "skip: pool ⊕ g, MLP")
    wire(ax, [(102, 36), (105, 36)])
    wire(ax, [(112.5, 32.1), (112.5, 29)])

    box(ax, 105, 23, 120, 29, P["red_1"], red)
    title(ax, 112.5, 27.2, "Rule mask", ha="center")
    body(ax, 112.5, 24.7, "+ softmax (392)", ha="center")
    wire(ax, [(77, 53), (122.6, 53), (122.6, 26), (121.6, 26)], red, 1.2, arrow=False, ls=(0, (4, 2)))
    wire(ax, [(121.8, 26), (120, 26)], red, 1.2)
    body(ax, 98, 54.3, "legal mask", color=red, ha="center", style="italic")
    wire(ax, [(112.5, 23), (112.5, 18.6)])

    heat = {(10, 7): 1.0, (9, 6): .55, (9, 8): .55, (11, 5): .3, (11, 9): .3, (10, 6): .25, (10, 8): .25,
            (7, 7): .2, (8, 4): .12, (8, 10): .12, (6, 7): .08}
    pitch(ax, 100, 5, 22, own, opp, heat=heat, current=(10, 7))
    title(ax, 111, 3.3, "Placement probabilities", ha="center")

    # ---------------------------------------------------------------- the auto-regressive loop
    wire(ax, [(100, 11), (98.5, 11), (98.5, 0.6), (0.8, 0.6), (0.8, 35.5), (2, 35.5)], gold, 1.6)
    body(ax, 49.5, -1.3, "place the player (or leave it out)  →  board, bench and mask update  →  next player, "
         "until 11 are on the pitch or the bench is empty", color=gold, ha="center", style="italic")

    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "setup_policy_architecture")
    for ext in ("pdf", "svg", "png"):
        fig.savefig(f"{out}.{ext}", dpi=300, bbox_inches="tight", pad_inches=0.05, facecolor="white")
        print(f"wrote {out}.{ext}")
    plt.close(fig)


if __name__ == "__main__":
    main()
