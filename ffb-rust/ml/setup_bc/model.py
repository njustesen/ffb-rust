"""The setup policy: a learned player encoder feeding a FiLM-conditioned ResNet with one policy
head for defense (kicking) and one for offense (receiving)."""
from __future__ import annotations

import torch
import torch.nn as nn
import torch.nn.functional as F

from features import A_SKIP, BENCH, H, LOS_X, N_ACT, N_FEATS, N_GLOBALS, N_SQ, W

EMB = 16


class PlayerEncoder(nn.Module):
    """Stats + skills of one player -> a 16-float embedding. Team-agnostic: the board says whose
    player it is (left half = ours, right half = theirs)."""

    def __init__(self, n_skills: int, skill_dim: int = 24, hidden: int = 64):
        super().__init__()
        self.skill = nn.Embedding(n_skills, skill_dim, padding_idx=0)
        self.mlp = nn.Sequential(
            nn.Linear(N_FEATS + skill_dim, hidden), nn.ReLU(),
            nn.Linear(hidden, hidden), nn.ReLU(),
            nn.Linear(hidden, EMB),
        )

    def forward(self, feats, skills):  # [B,P,F], [B,P,K] -> [B,P,EMB]
        return self.mlp(torch.cat([feats, self.skill(skills).sum(2)], -1))


class Block(nn.Module):
    def __init__(self, ch: int, gdim: int, drop: float):
        super().__init__()
        self.n1, self.n2 = nn.GroupNorm(8, ch), nn.GroupNorm(8, ch)
        self.c1 = nn.Conv2d(ch, ch, 3, padding=1)
        self.c2 = nn.Conv2d(ch, ch, 3, padding=1)
        self.film = nn.Linear(gdim, 2 * ch)
        self.drop = nn.Dropout2d(drop)

    def forward(self, x, g):
        h = self.c1(F.relu(self.n1(x)))
        gamma, beta = self.film(g).chunk(2, -1)
        h = h * (1 + gamma[..., None, None]) + beta[..., None, None]
        h = self.c2(self.drop(F.relu(self.n2(h))))
        return x + h


def fixed_planes() -> torch.Tensor:
    """[6, W, H]: x, y, own half, own line of scrimmage, opposing line of scrimmage, wide zones."""
    x = torch.arange(W).float()[:, None].expand(W, H)
    y = torch.arange(H).float()[None, :].expand(W, H)
    centre = (y >= 4) & (y <= 10)
    return torch.stack([
        x / (W - 1), y / (H - 1), (x <= LOS_X).float(),
        ((x == LOS_X) & centre).float(), ((x == LOS_X + 1) & centre).float(), (~centre).float(),
    ])


class SetupPolicy(nn.Module):
    def __init__(self, n_skills: int, ch: int = 64, blocks: int = 6, gdim: int = 128, drop: float = 0.1,
                 z: bool = False):
        """`z`: condition on how the game ended for the team setting up (1 won, 0.5 drew, 0 lost);
        at inference z is a dial: "set up like a team that goes on to win / lose"."""
        super().__init__()
        self.use_z = z
        self.encoder = PlayerEncoder(n_skills)
        self.register_buffer("planes", fixed_planes())
        self.gmlp = nn.Sequential(
            nn.Linear(N_GLOBALS + EMB + 2 * BENCH * EMB + int(z), 256), nn.ReLU(),
            nn.Linear(256, gdim), nn.ReLU(),
        )
        self.stem = nn.Conv2d(2 * EMB + 6 + 1, ch, 3, padding=1)
        self.blocks = nn.ModuleList(Block(ch, gdim, drop) for _ in range(blocks))
        self.norm = nn.GroupNorm(8, ch)
        # head 0 = defense (kicking side), head 1 = offense (receiving side)
        self.square = nn.ModuleList(
            nn.Sequential(nn.Conv2d(ch, 32, 1), nn.ReLU(), nn.Conv2d(32, 1, 1)) for _ in range(2))
        self.other = nn.ModuleList(
            nn.Sequential(nn.Linear(ch + gdim, 64), nn.ReLU(), nn.Linear(64, 2)) for _ in range(2))

    def forward(self, b: dict) -> torch.Tensor:
        """Batch of observations -> [B, N_ACT] logits, illegal actions at -inf."""
        B = b["feats"].shape[0]
        emb = self.encoder(b["feats"], b["skills"])
        emb = torch.cat([emb.new_zeros(B, 1, EMB), emb], 1)   # row 0 = nobody
        rows = torch.arange(B, device=emb.device)[:, None]
        board = emb[rows, b["board"]].view(B, W, H, EMB).permute(0, 3, 1, 2)
        cur = emb[rows[:, 0], b["cur"]]
        legal = b["legal"]

        context = [b["globals"], cur, emb[rows, b["own_bench"]].flatten(1), emb[rows, b["opp_bench"]].flatten(1)]
        if self.use_z:
            context.append(b["z"].float()[:, None])
        g = self.gmlp(torch.cat(context, -1))
        x = torch.cat([
            board, cur[:, :, None, None].expand(B, EMB, W, H),
            self.planes[None].expand(B, -1, -1, -1),
            legal[:, :N_SQ].view(B, 1, W, H).float(),
        ], 1)
        x = self.stem(x)
        for blk in self.blocks:
            x = blk(x, g)
        x = F.relu(self.norm(x))
        pooled = torch.cat([x.mean((2, 3)), g], -1)

        per_head = [torch.cat([sq(x).flatten(1), ot(pooled)], 1) for sq, ot in zip(self.square, self.other)]
        logits = torch.stack(per_head, 1)[rows[:, 0], b["head"]]
        return logits.masked_fill(~legal, float("-inf"))


def set_loss(logits: torch.Tensor, target: torch.Tensor) -> torch.Tensor:
    """-log of the probability mass on the actions that agree with the human formation."""
    logp = F.log_softmax(logits, -1)
    return -torch.logsumexp(logp.masked_fill(~target, float("-inf")), -1)


def to_torch(batch: dict, device) -> dict:
    return {k: torch.as_tensor(v).to(device, non_blocking=True) for k, v in batch.items()}


assert N_ACT == N_SQ + 2 and A_SKIP == N_SQ
