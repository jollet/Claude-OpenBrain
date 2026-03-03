# Open Brain 🧠

Vollständig lokaler, containerisierter Knowledge-Store mit semantischer Suche und MCP-Integration.

**Kein Cloud-Zwang. Keine API-Kosten. Deine Daten bleiben lokal.**

## Architektur

```
brain-core (Rust/axum)   →  :3000/api/*  REST API
                         →  :3000/mcp    MCP Server (Streamable HTTP)
                         →  :3000/health Status

embedding-fallback       →  sentence-transformers (wenn LM Studio offline)
brain-ui                 →  :8080         Web-UI (HTMX)
brain (CLI)              →  brain add / brain search / brain list
```

## Schnellstart

### Voraussetzungen

- Podman 5.x (oder Docker Desktop)
- LM Studio auf dem Host mit einem Embedding-Modell (z.B. `nomic-embed-text-v1.5`)
- Für Entwicklung: Rust 1.82+

### 1. Konfiguration

```bash
cp .env.example .env
# .env öffnen und ausfüllen:
# - MCP_ACCESS_KEY generieren: openssl rand -hex 32
# - EMBEDDING_MODEL auf das geladene LM Studio Modell setzen
```

### 2. Starten

```bash
podman compose up -d
```

### 3. Status prüfen

```bash
curl http://localhost:3000/health | jq
```

### 4. CLI installieren

```bash
cargo build --release --bin brain
cp target/release/brain ~/.local/bin/
```

## CLI-Nutzung

```bash
# Gedanken speichern
brain add "Idee: Automatische Zusammenfassung via LLM"
brain add "Sarah erwähnte, dass sie den Job wechseln will" --tags gespräch,sarah

# Auflisten
brain list
brain list --limit 5

# Einzelabruf
brain get 42

# Suchen (Phase 2)
brain search "Karrierewechsel"

# Statistiken
brain stats

# Status
brain health
```

## MCP-Integration (Claude Desktop / Claude Code)

```json
// ~/Library/Application Support/Claude/claude_desktop_config.json
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

## Entwicklung (TDD)

```bash
# Unit-Tests (Rust)
cargo test

# Integrationstests (Python dev-container)
podman compose up -d brain-core embedding-fallback
podman compose --profile test run dev-container

# Linting
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

## Datentrennung

| Was | Wo | GitHub? |
|---|---|---|
| `brain.db` (echte Gedanken) | Host-Volume | ❌ Niemals |
| `.env` mit Secrets | Lokal | ❌ |
| Synthetische Testdaten | `dev-container/tests/fixtures/` | ✅ |
| Schema-Migrations | `migrations/` | ✅ |
| Compose ohne Secrets | `compose.yml` | ✅ |

## MCP Tools

| Tool | Beschreibung |
|---|---|
| `brain_capture_thought` | Gedanken speichern + embedden |
| `brain_search` | Semantische Suche |
| `brain_list_recent` | Neueste Gedanken (paginiert) |
| `brain_get_stats` | Statistiken |

## Meilensteine

- [x] **Phase 0** — Fundament, Health-Endpoint, TDD-Setup
- [ ] **Phase 1** — Capture (REST API, CLI, Embeddings)
- [ ] **Phase 2** — Retrieval (Semantische Suche, MCP Tools)
- [ ] **Phase 3** — Web-UI
- [ ] **Phase 4** — Härtung, CI, v1.0.0 Release

## Lizenz

MIT
