# 40_network.md — full framework: server + client networking (no GUI)

Goal: translate the entire Java FFB framework except the Swing GUI, AND make the **existing
Java Swing GUI client connect to the Rust server** over the wire. Feasible because it's
WebSocket + JSON; the bar is **byte-for-byte wire fidelity**. Java ground truth under
`C:\Users\Admin\niels\ffb\ffb`.

## Transport
- Server: Jetty WebSocket, endpoint **`/command`**, port **22227** (`server.ini` `server.port`;
  no default). Class `FantasyFootballServer` → `CommandServlet`/`CommandSocket`
  (maxText 64KiB). Client: Tyrus (JSR-356) `ws://host:22227/command` (plaintext).
- One NetCommand = one WS message = one JSON document (UTF-8). Discriminator = `netCommandId`
  field; **no extra envelope**. Client sends **binary** frames; both ends accept text or binary.
- Optional compression = **lz-string UTF-16** (NOT gzip), `com.fumbbl.ffb.json.LZString`,
  on by default. **Interop: run both sides `command.compression=false`** (raw UTF-8 JSON) to
  avoid porting LZString initially; port it later if needed.

## Wire protocol
- `NetCommand` (abstract) → `ClientCommand` (+optional `entropy`) / `ServerCommand` (+`commandNr`,
  `isReplayable`). `NetCommandId` enum = **138 ids** (`ffb-common/.../net/NetCommandId.java`),
  each `ENUM("jsonName")`; `createNetCommand()` switch instantiates.
- **32 ServerCommand\*** (server→client) + **91 ClientCommand\*** (client→server).
- JSON keys come from `IJsonOption` — **643 typed key constants** (`ffb-common/.../json/IJsonOption.java`).
  Decode via `NetCommandFactory.forJsonValue()` (reads `netCommandId` → create → `initFrom`).
  **The Rust port must reproduce every IJsonOption key string and every serialized enum
  `getName()` string exactly.**
- Shapes: `serverModelSync {commandNr, modelChangeList:{modelChangeArray:[…]}, reportList, animation?, sound, gameTime, turnTime}`;
  `serverGameState {commandNr, game:{…full Game…}}`; `clientActingPlayer {entropy?, playerId, playerAction, jumping}`;
  `clientJoin {clientMode, coach, password, gameId, gameName, teamId, teamName}`;
  `serverJoin {commandNr, coach, clientMode, spectators, playerNames[], spectatorNames[], name}`.

## Join / auth flow
1. `clientRequestVersion` → `serverVersion` (props). 2. `clientPasswordChallenge(coach)` →
`serverPasswordChallenge(challenge)`. 3. `clientJoin(...)` → `serverJoin` → **`serverGameState`**
(full Game once; thereafter `serverModelSync` deltas).
- Auth is per-join. **STANDALONE mode** (target for interop): challenge = null → response =
  `hex(md5(password))`, compared to a local store; account props hardcoded `["DEV","STATE_EDIT"]`.
  Avoids the FUMBBL HTTP backend entirely. (FUMBBL mode = HTTP challenge/response — skip.)
- Challenge algo (if needed): `R = MD5(OPAD + MD5(IPAD + CHL))` (`PasswordChallenge.java:86`).

## Server comm loop
`ServerCommunication` (single thread): WS msg → (decompress?) → `NetCommandFactory` →
`ReceivedCommand(cmd,session)` → mix client `entropy` into Fortuna → `ServerCommandHandlerFactory`
→ authorize sender (home/away) → **`GameState.handleCommand`** (the step engine). After each
step, `UtilServerGame.syncGameModel` → `gameState.fetchChanges()` (a `ModelChangeList`) →
`sendModelSync` builds `serverModelSync` with a **monotonic per-game `commandNr`** and sends to
home+spectators, an **`.transform()`-flipped copy to away**.
- Model→client: `Game extends ModelChangeObservable`; every setter notifies; `GameState`
  collects into `fChangeList`. `ModelChange {modelChangeId, modelChangeKey, modelChangeValue}` —
  value type implied by id. **169 `ModelChangeId` × 35 `ModelChangeDataType`** (id→datatype table
  must be hardcoded; several JSON names diverge from the constant, e.g.
  `ACTING_PLAYER_SET_JUMPING`→`"actingPlayerSetLeaping"`). Apply = `ModelChangeProcessor` switch.
- Full `Game` JSON ≈ 32 top-level keys; **parse gameOptions first** (sets rules factory), then
  teamState gates skeleton vs full teams. `SessionManager` maps gameId↔sessions↔JoinedClient.

## Client (non-GUI) — and the seam (premise correction)
`ffb-client-logic` is **NOT GUI-free** (181 files import Swing/AWT; `UserInterface extends
JFrame` lives here). `ffb-client` is the thin AWT top. So there's no class-for-class portable
client core. **Portable seam to port:** `NetCommandFactory` decode + `ClientCommunication`
single-thread queue + **`ModelChangeList.applyTo(Game)`** (the kernel) + `ClientStateFactory`
(TurnMode+action → state id). Everything touching `UserInterface`/animation/sound/dialogs is GUI
= excluded.

## Current Rust state (gap)
- `ffb-protocol` (~676 LOC): `ClientCommand` enum ~40/91 variants; `ServerCommand` ~17/32;
  serde `#[serde(tag="netCommandId", camelCase)]` (no explicit IJsonOption registry; no LZString).
- `ffb-client` (~903 LOC): real `tokio-tungstenite` transport (connect/send/ping/read→mpsc),
  `state_dispatch` (good port of ClientStateFactory), `network_encoder` (Action→ClientCommand);
  **`ServerModelSync` handler is a STUB (applies nothing)** — the most important command is a
  no-op. Not wire-compatible.

## Feasibility verdict & hardest parts
Feasible. The Rust **server** must: (1) speak every ServerCommand's exact JSON + accept every
ClientCommand (138 ids, 643 keys, enum names byte-exact); (2) implement version→challenge→join
in STANDALONE mode; (3) reproduce model-sync — full Game once + 169-id ModelChange deltas,
monotonic commandNr, away `.transform()`; (4) WS `/command`, binary|text, compression off.
**Hardest, in order:** (1) **model serialization fidelity** (full Game + all 169 ModelChange
payloads with exact IJsonOption keys — the dominant effort); (2) commandNr + away-transform
semantics; (3) framing (binary frames, lz-string if compression can't be off); (4) auth (easy
in STANDALONE). The existing Rust protocol/client prove transport+state-mapping but are far from
this bar.
