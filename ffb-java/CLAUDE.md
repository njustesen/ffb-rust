# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the Fantasy Football (Blood Bowl) game engine used by [FUMBBL](https://fumbbl.com). It is a distributed desktop application with a Java WebSocket server and an AWT/Swing desktop client, supporting multiple Blood Bowl rule editions (BB2016, BB2020, BB2025).

## Build Commands

```bash
mvn clean install                                    # full build
mvn test                                             # run all tests
mvn test -Dtest=TestClassName#testMethodName         # run a single test
mvn install -DskipTests                              # build without tests
mvn clean install -Pmockito5                         # build with Java 21+ (Mockito 5)
```

## Module Structure (Build Order)

1. **ffb-common** — shared domain models, rules, mechanics, JSON protocol, enums
2. **ffb-tools** — build-time resource utilities
3. **ffb-server** — WebSocket endpoint, game orchestration, database persistence
4. **ffb-client-logic** — device-agnostic client logic (nearly all client code lives here)
5. **ffb-resources** — assets (sounds, icons)
6. **ffb-client** — AWT/Swing UI layer and desktop packaging; bundled libraries in `repo/`

## Architecture

### Server (`ffb-server`)

Entry point: `com.fumbbl.ffb.server.FantasyFootballServer`

Game flow is driven by a **Step/Stack pattern**:
- The game maintains a stack of `Step` objects (`ffb-server/.../server/step/`)
- The top step dequeues client commands and either processes them, advances to a new step, or waits
- `SequenceGenerator` classes (`step/generator/`) build the step sequences for each game phase
- Steps for different rule editions live in `step/bb2016/`, `step/bb2020/`, `step/bb2025/`

Other key server packages:
- `mechanic/`, `injury/`, `inducements/`, `skillbehaviour/` — game rule implementations
- `db/` — database access (MySQL 5.6 / MariaDB 10.4)
- `net/` — Jetty WebSocket communication
- `handler/` — request routing

### Client (`ffb-client-logic` + `ffb-client`)

Entry point: `com.fumbbl.ffb.client.FantasyFootballClientAwt`

Client input handling mirrors the server's Step pattern with **ClientState** classes:
- `ffb-client-logic/.../client/state/` — one `ClientState` per game phase (setup, moving, blocking, etc.)
- `state/logic/` — pluggable logic modules (MoveLogicModule, BlockLogicModule, etc.)
- `net/` — Tyrus WebSocket client
- `ui/`, `layer/`, `overlay/`, `animation/`, `dialog/` — rendering and interaction

### Shared (`ffb-common`)

All rule-set-specific domain logic is isolated under `bb2016/`, `bb2020/`, `bb2025/` sub-packages throughout the codebase. Skills, injuries, inducements, kickoff events, and mechanics are defined here and shared between server and client.

### Network Protocol

Communication uses JSON (minimal-json library). Command and response types are defined in `ffb-common`.

## Code Style

- Java 8 target; package prefix `com.fumbbl.*`; encoding UTF-8
- **Imports:** project imports first (`com.fumbbl.*`), then `javax.*`/`java.*`; no wildcards; one class per line; blank lines between groups
- Test classes end with `Test`; use descriptive method names
- JUnit 5 + Mockito for tests

## Startup

**Server:**
```
com.fumbbl.ffb.server.FantasyFootballServer [standalone|fumbbl|standaloneInitDb|fumbblInitDb] -inifile <path> [-override <path>]
```

**Client:**
```
com.fumbbl.ffb.client.FantasyFootballClientAwt [-player|-spectator|-replay] -server <host> -port <port> -coach <name> -teamid <id> -auth <hex> [-gameId <id>]
```
