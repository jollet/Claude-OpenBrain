# Open Brain — Vollständiger Projektplan

**Version:** 1.0  
**Datum:** 2026-03-03  
**Status:** Bereit für Phase 0

---

## Entscheidungsmatrix (final)

| Frage | Entscheidung |
|---|---|
| Capture-Schnittstellen | CLI, REST API, Web-UI |
| Embedding primär | LM Studio (MacBook M2 Pro) |
| Embedding Fallback | Python-Container (sentence-transformers) |
| Core-Sprache | Rust |
| CLI-Implementierung | Rust Binary (separates Crate, spricht REST API) |
| Port-Strategie | Ein Port (3000): `/api/*` REST, `/mcp` MCP |
| Container-Runtime | Podman (rootless, bevorzugt) / Docker-kompatibel |
| Netzwerk | Konfigurierbar per ENV (localhost / Heimnetz) |
| Betriebssysteme | Windows 11, macOS 26.3, Linux (Debian Trixie / Ubuntu 24 LTS) |
| Entwicklungsmethodik | TDD (Red / Green / Refactor) |
| Repository | GitHub, strenge Datentrennung |

---

## Systemarchitektur

```
┌─────────────────────────────────────────────────────────────────┐
│                    Host (Win11 / macOS / Linux)                 │
│                                                                 │
│  ┌─────────────────┐    ┌──────────────────────────────────┐   │
│  │   LM Studio     │    │     Podman Compose               │   │
│  │   MacBook M2    │    │                                  │   │
│  │   Port: 1234    │    │  ┌──────────────────────────┐   │   │
│  │   (Embeddings)  │◄───┼──│   brain-core (Rust)      │   │   │
│  └─────────────────┘    │  │                          │   │   │
│                          │  │   :3000/api/*  → REST    │   │   │
│  ┌─────────────────┐    │  │   :3000/mcp    → MCP     │   │   │
│  │   Claude        │    │  │   :3000/health → Status  │   │   │
│  │   Desktop /     │────┼──│                          │   │   │
│  │   Claude Code   │    │  └──────────┬───────────────┘   │   │
│  └─────────────────┘    │             │ Volume             │   │
│                          │  ┌──────────▼───────────────┐   │   │
│  ┌─────────────────┐    │  │   SQLite + sqlite-vec    │   │   │
│  │   Browser       │────┼──│   /data/brain.db         │   │   │
│  │   (Web-UI)      │    │  │   (lokal, nie im Repo)   │   │   │
│  └─────────────────┘    │  └──────────────────────────┘   │   │
│                          │                                  │   │
│                          │  ┌──────────────────────────┐   │   │
│                          │  │   brain-ui               │   │   │
│                          │  │   :8080 (HTMX + Tailwind)│   │   │
│                          │  └──────────────────────────┘   │   │
│                          │                                  │   │
│                          │  ┌──────────────────────────┐   │   │
│                          │  │   embedding-fallback     │   │   │
│                          │  │   :8001 (Python/FastAPI) │   │   │
│                          │  │   sentence-transformers  │   │   │
│                          │  │   all-MiniLM-L6-v2       │   │   │
│                          │  └──────────────────────────┘   │   │
│                          │                                  │   │
│                          │  ┌──────────────────────────┐   │   │
│                          │  │   dev-container (TDD)    │   │   │
│                          │  │   pytest + pydantic +    │   │   │
│                          │  │   httpx + hypothesis     │   │   │
│                          │  └──────────────────────────┘   │   │
│                          └──────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Embedding-Fallback-Logik (in brain-core)

```
brain-core startet
    │
    ├─► LM Studio Health-Check (EMBEDDING_URL/v1/models)
    │       │
    │       ├─ OK  → LM Studio verwenden
    │       │
    │       └─ FAIL → embedding-fallback Container verwenden
    │                  (EMBEDDING_FALLBACK_URL/embed)
    │
    └─► Status unter GET /health sichtbar
```

---

## Repository-Struktur

```
open-brain/                          # GitHub Repository
├── .github/
│   └── workflows/
│       ├── ci.yml                   # cargo test + clippy + Python-Tests
│       └── release.yml              # Binary-Build für alle Plattformen
│
├── brain-core/                      # Rust: Hauptservice
│   ├── src/
│   │   ├── main.rs                  # Server-Start, Config-Loading
│   │   ├── config.rs                # ENV-basierte Konfiguration
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── thoughts.rs          # POST/GET /api/thoughts
│   │   │   ├── search.rs            # POST /api/search
│   │   │   ├── stats.rs             # GET /api/stats
│   │   │   └── health.rs            # GET /health
│   │   ├── mcp/
│   │   │   ├── mod.rs
│   │   │   ├── server.rs            # MCP JSON-RPC Handler
│   │   │   └── tools/
│   │   │       ├── brain_capture.rs
│   │   │       ├── brain_search.rs
│   │   │       ├── brain_list.rs
│   │   │       └── brain_stats.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── migrations.rs        # Schema-Init
│   │   │   └── repository.rs        # CRUD + Vektorsuche
│   │   ├── embeddings/
│   │   │   ├── mod.rs
│   │   │   ├── client.rs            # Trait: EmbeddingProvider
│   │   │   ├── lm_studio.rs         # LM Studio Implementierung
│   │   │   └── fallback.rs          # Fallback-Container Implementierung
│   │   └── models/
│   │       ├── thought.rs
│   │       └── search.rs
│   ├── tests/
│   │   ├── api_integration.rs
│   │   └── mcp_integration.rs
│   └── Cargo.toml
│
├── brain-cli/                       # Rust: CLI-Binary
│   ├── src/
│   │   └── main.rs                  # clap-basierte Subcommands
│   └── Cargo.toml
│
├── brain-ui/                        # Web-UI
│   ├── Dockerfile
│   ├── nginx.conf
│   └── static/
│       ├── index.html               # Capture + Suche
│       └── timeline.html            # Zeitstrahl
│
├── embedding-fallback/              # Python Fallback-Service
│   ├── Dockerfile
│   ├── requirements.txt
│   └── server.py                    # FastAPI + sentence-transformers
│
├── dev-container/                   # TDD-Umgebung
│   ├── Dockerfile
│   ├── requirements.txt             # pytest, pydantic, httpx, hypothesis
│   └── tests/
│       ├── conftest.py              # Fixtures, API-Client-Setup
│       ├── models/
│       │   ├── thought.py           # Pydantic Response-Modelle
│       │   └── search.py
│       ├── test_api_thoughts.py     # CRUD-Tests (Red/Green)
│       ├── test_api_search.py       # Semantische Suche
│       ├── test_api_health.py       # Health + Embedding-Status
│       ├── test_mcp_tools.py        # MCP JSON-RPC Tests
│       └── fixtures/
│           ├── thoughts.json        # Synthetische Testdaten (→ GitHub OK)
│           └── search_cases.json    # Erwartete Suchergebnisse
│
├── migrations/
│   └── 001_initial.sql              # Schema (→ GitHub OK)
│
├── compose.yml                      # Podman/Docker Compose (ohne Secrets)
├── compose.override.example.yml     # Lokale Overrides (Ports, Volumes)
├── .env.example                     # Alle Variablen als Platzhalter
├── .gitignore
└── README.md
```

### Datentrennung (strikt)

| Artefakt | Speicherort | In GitHub? |
|---|---|---|
| `brain.db` (echte Gedanken) | Host-Volume | ❌ Niemals |
| `.env` mit echten Werten | Lokal | ❌ In `.gitignore` |
| LM Studio API Key | `.env` lokal | ❌ |
| `compose.override.yml` | Lokal | ❌ In `.gitignore` |
| Synthetische Fixtures | `dev-container/tests/fixtures/` | ✅ |
| `migrations/001_initial.sql` | Repo | ✅ |
| `.env.example` | Repo | ✅ |
| `compose.yml` (keine Secrets) | Repo | ✅ |

---

## Technologie-Stack

| Schicht | Technologie | Version / Crate |
|---|---|---|
| Sprache Core | Rust | 1.82+ (stable) |
| Web-Framework | axum | 0.8 |
| Async Runtime | tokio | 1.x |
| HTTP-Client | reqwest | 0.12 |
| Datenbank | rusqlite | 0.32 |
| Vektor-Extension | sqlite-vec | 0.1.x |
| CLI-Args | clap | 4.x |
| Serialisierung | serde / serde_json | 1.x |
| Fehlerbehandlung | thiserror / anyhow | 2.x |
| Logging | tracing / tracing-subscriber | 0.1.x |
| Web-UI | HTMX 2.x + Tailwind CDN | Kein Build-Step |
| Embedding Fallback | FastAPI + sentence-transformers | Python 3.12 |
| Test-Framework | pytest 8.x | Dev-Container |
| Pydantic | pydantic v2 | Dev-Container |
| HTTP-Testclient | httpx | Dev-Container |
| Container | Podman 5.x | Rootless |
| CI | GitHub Actions | ubuntu-latest |

---

## API-Spezifikation

### REST API (`/api/*`)

```
POST   /api/thoughts          Gedanken speichern
GET    /api/thoughts          Liste (paginiert, ?limit=20&offset=0)
GET    /api/thoughts/{id}     Einzelabruf
DELETE /api/thoughts/{id}     Löschen

POST   /api/search            Semantische Suche
       Body: { "query": "...", "limit": 10, "threshold": 0.7 }

GET    /api/stats             Statistiken
GET    /health                Service-Status + Embedding-Backend
```

### MCP Tools (`/mcp` — JSON-RPC 2.0)

| Tool | Annotation | Beschreibung |
|---|---|---|
| `brain_capture_thought` | destructive=false | Gedanken speichern + embedden |
| `brain_search` | readOnly=true | Semantische Suche |
| `brain_list_recent` | readOnly=true | Neueste Gedanken (paginiert) |
| `brain_get_stats` | readOnly=true | Anzahl, Topics, Zeitraum |

MCP-Transport: **Streamable HTTP** (kein stdio, da Multi-Client-Szenario)

### Embedding-Backend API (intern)

```
# LM Studio (OpenAI-kompatibel):
POST http://host.docker.internal:1234/v1/embeddings
     Body: { "model": "...", "input": "text" }

# Fallback-Container:
POST http://embedding-fallback:8001/embed
     Body: { "text": "..." }
     Response: { "embedding": [0.1, 0.2, ...], "model": "all-MiniLM-L6-v2" }
```

---

## TDD-Strategie

### Testpyramide

```
        ┌──────────────┐
        │  E2E / MCP   │  ← Python dev-container
        │  Tests       │     (wenige, langsam)
        ├──────────────┤
        │ Integrations │  ← Python dev-container
        │ Tests (API)  │     (mittel, gegen laufenden Core)
        ├──────────────┤
        │  Unit Tests  │  ← cargo test
        │  (Rust)      │     (viele, schnell)
        └──────────────┘
```

### Red/Green Zyklen pro Phase

**Phase 1 — Capture:**
```
🔴 RED:   test_post_thought_returns_201_with_id
🟢 GREEN: POST /api/thoughts → 201 + { id, content, timestamp }

🔴 RED:   test_thought_persisted_after_restart
🟢 GREEN: SQLite-Insert implementieren

🔴 RED:   test_embedding_field_in_response
🟢 GREEN: LM Studio Client + Fallback-Logik

🔴 RED:   test_health_shows_embedding_backend
🟢 GREEN: GET /health → { status, embedding_backend, db }
```

**Phase 2 — Retrieval:**
```
🔴 RED:   test_search_returns_semantically_similar
🟢 GREEN: sqlite-vec KNN-Suche implementieren

🔴 RED:   test_mcp_capture_tool_stores_thought
🟢 GREEN: MCP tool=brain_capture_thought implementieren

🔴 RED:   test_mcp_search_returns_results
🟢 GREEN: MCP tool=brain_search implementieren
```

### Pydantic-Modelle (dev-container)

```python
# dev-container/tests/models/thought.py
from pydantic import BaseModel, Field
from datetime import datetime

class ThoughtCreate(BaseModel):
    content: str = Field(min_length=1, max_length=10_000)
    tags: list[str] = []

class ThoughtResponse(BaseModel):
    id: int
    content: str
    tags: list[str]
    created_at: datetime
    has_embedding: bool

class SearchRequest(BaseModel):
    query: str = Field(min_length=1)
    limit: int = Field(default=10, ge=1, le=100)
    threshold: float = Field(default=0.7, ge=0.0, le=1.0)

class SearchResult(BaseModel):
    thought: ThoughtResponse
    similarity: float = Field(ge=0.0, le=1.0)

class SearchResponse(BaseModel):
    results: list[SearchResult]
    query: str
    total: int
```

---

## Meilensteine

### Phase 0 — Fundament (Woche 1)
**Ziel:** `podman compose up` startet alle Container, DB wird initialisiert.

- [ ] GitHub Repo anlegen (`open-brain`), Branch-Strategie: `main` (stabil) + `dev`
- [ ] `compose.yml` Grundgerüst (alle 4 Services)
- [ ] Rust Workspace init (`Cargo.toml` mit `brain-core` + `brain-cli`)
- [ ] SQLite + sqlite-vec einbinden, `001_initial.sql` schreiben
- [ ] `GET /health` Endpoint (minimaler Server, lauffähig)
- [ ] Dev-Container mit pytest + pydantic + httpx lauffähig
- [ ] Erster failing Test: `test_health_returns_200` → dann grün
- [ ] `.env.example`, `.gitignore`, `compose.override.example.yml`

**Deliverable:** Grüner Health-Test im Dev-Container.

---

### Phase 1 — Capture (Woche 2)
**Ziel:** `brain add "Mein Gedanke"` funktioniert end-to-end.

- [ ] `POST /api/thoughts` + `GET /api/thoughts` + `GET /api/thoughts/{id}`
- [ ] SQLite Repository (CRUD)
- [ ] Embedding-Client Trait + LM Studio Implementierung
- [ ] Fallback-Container (Python FastAPI + sentence-transformers)
- [ ] Fallback-Logik in brain-core (Health-Check → Routing)
- [ ] `GET /health` zeigt Embedding-Backend-Status
- [ ] `brain-cli`: `brain add`, `brain list`, `brain get <id>`
- [ ] Alle Routes: failing Python-Tests zuerst, dann implementieren

**Deliverable:** CLI-Demo: `brain add "Test" && brain list`

---

### Phase 2 — Retrieval (Woche 3)
**Ziel:** Claude Desktop / Claude Code findet Gedanken via MCP.

- [x] `POST /api/search` (sqlite-vec KNN-Suche)
- [x] `GET /api/stats`
- [ ] MCP Server (`/mcp`, JSON-RPC 2.0, Streamable HTTP)
- [ ] 4 MCP Tools: `brain_capture_thought`, `brain_search`, `brain_list_recent`, `brain_get_stats`
- [ ] MCP Tool Annotations (readOnly, destructive Hints)
- [x] `brain-cli`: `brain search "query"`, `brain stats`
- [ ] Python-Tests: MCP JSON-RPC direkt testen

**Deliverable:** Claude Desktop MCP-Config funktioniert, semantische Suche liefert Ergebnisse.

---

### Phase 3 — Web-UI (Woche 4)
**Ziel:** `http://localhost:8080` zeigt vollständige UI.

- [ ] `brain-ui` Container (nginx, statisches HTML)
- [ ] Capture-Formular (HTMX POST an brain-core API)
- [ ] Suche mit Live-Ergebnissen (HTMX)
- [ ] Zeitstrahl der letzten Gedanken
- [ ] Mobile-freundlich (Tailwind)

**Deliverable:** Browser-Demo: Gedanken erfassen + suchen.

---

### Phase 4 — Härtung (Woche 5)
**Ziel:** Produktionsreif, CI grün, dokumentiert.

- [ ] GitHub Actions CI: `cargo test`, `cargo clippy --deny warnings`, Python-Tests
- [ ] `DELETE /api/thoughts/{id}` + Paginierungs-Tests
- [ ] Performance-Test: 1.000 Thoughts einfügen, Suchlatenz < 50ms messen
- [ ] Cross-Platform Test: Windows 11 (WSL2), macOS, Linux
- [ ] `README.md` vollständig (Setup, Konfiguration, Beispieldaten)
- [ ] Release-Workflow: Binary-Build für alle Plattformen

**Deliverable:** v1.0.0 Release auf GitHub.

---

## Konfiguration (ENV-Variablen)

```bash
# .env.example

# Server
BRAIN_HOST=127.0.0.1          # 0.0.0.0 für Heimnetz-Zugriff
BRAIN_PORT=3000

# Datenbank
BRAIN_DB_PATH=/data/brain.db  # Volume-Pfad

# Embedding — Primär (LM Studio)
EMBEDDING_URL=http://host.docker.internal:1234
EMBEDDING_MODEL=nomic-embed-text-v1.5

# Embedding — Fallback
EMBEDDING_FALLBACK_URL=http://embedding-fallback:8001
EMBEDDING_FALLBACK_ENABLED=true

# MCP Sicherheit
MCP_ACCESS_KEY=                # Per openssl rand -hex 32 generieren

# Web-UI
BRAIN_UI_PORT=8080
BRAIN_API_URL=http://brain-core:3000
```

---

## Sicherheit

- MCP-Endpoint prüft `X-Brain-Key` Header auf jeden Request
- REST API: Kein Auth (localhost-only Default), optional via `BRAIN_REQUIRE_AUTH=true`
- DNS-Rebinding-Schutz: `BRAIN_HOST=127.0.0.1` als Default
- Keine Secrets im Repository (`.gitignore` + `.env.example`)
- Container läuft rootless (Podman)
- sqlite-vec: Prepared Statements, kein direktes SQL-Injection-Risiko

---

## MCP-Konfiguration für Claude

```json
// Claude Desktop: ~/Library/Application Support/Claude/claude_desktop_config.json
{
  "mcpServers": {
    "open-brain": {
      "type": "http",
      "url": "http://localhost:3000/mcp",
      "headers": {
        "x-brain-key": "dein-key-aus-.env"
      }
    }
  }
}
```

```bash
# Claude Code
claude mcp add open-brain \
  --transport http \
  --url http://localhost:3000/mcp \
  --header "x-brain-key: dein-key-aus-.env"
```

---

## Risiken & Mitigationen

| Risiko | Wahrscheinlichkeit | Mitigation |
|---|---|---|
| sqlite-vec auf Windows/ARM instabil | Mittel | Fallback auf Pure-Rust-Vektorsuche (linfa) |
| LM Studio API-Änderungen | Niedrig | Embedding-Trait abstrahiert den Client |
| Podman auf Windows 11 komplex | Mittel | Docker Desktop als Fallback (compose-kompatibel) |
| MCP-Protokolländerungen | Niedrig | JSON-RPC direkt implementiert, kein Framework-Lock-in |
| sentence-transformers Modell-Download schlägt fehl | Niedrig | Modell in Image vorbacken (Dockerfile COPY) |

---

## Definition of Done

Ein Feature gilt als fertig, wenn:
1. Alle zugehörigen Rust Unit-Tests grün (`cargo test`)
2. Alle Python Integrationstests grün (dev-container)
3. `cargo clippy --deny warnings` ohne Fehler
4. Kein echter Datenbankinhalt oder Secrets im Commit
5. `README.md` um das Feature ergänzt

Das Projekt gilt als v1.0.0-fertig, wenn Phase 0–4 abgeschlossen sind und der GitHub Actions CI auf `main` durchgehend grün ist.
