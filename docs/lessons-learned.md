# Lessons Learned — Phase 0

## Container & Images

| Problem | Root Cause | Lösung |
|---------|-----------|--------|
| Podman VM crasht beim Image-Pull | `rust:1.82-bookworm` ~1.5 GB, VM hat wenig RAM | Slim-Images verwenden (`rust:1.84-slim-bookworm`) |
| `cargo build` fehlt `openssl-sys` | Slim-Image ohne Dev-Headers | `libssl-dev` + `pkg-config` explizit installieren |
| Healthcheck schlägt fehl | `curl` nicht in `python:3.12-slim` | `python -c "import urllib.request; ..."` |
| Embedding-Fallback Timeout | Modell-Download + Laden > 60s | `start_period: 120s`, `retries: 5` |

## Python / Tests

| Problem | Root Cause | Lösung |
|---------|-----------|--------|
| `ModuleNotFoundError: tests` | Fehlende `__init__.py` | `__init__.py` in `tests/` und `tests/models/` |
| Gepinnte pip-Versionen nicht verfügbar | Platform-spezifische Wheels fehlen | `>=` statt `==` bei Python-Deps |

## Rust / Cargo

| Problem | Root Cause | Lösung |
|---------|-----------|--------|
| Workspace-Build bricht ab | `brain-cli/Cargo.toml` fehlte | Alle Workspace-Members anlegen bevor `cargo build` |
| `main.rs` kompiliert nicht | Referenziert Module die nicht existieren | Phase 0: minimaler Server nur mit Health-Endpoint |
| Dockerfile Dependency-Caching scheitert | Dummy-Build hinterlässt inkonsistente Artifacts | Einfacher COPY-all-then-build für PoC |

## Allgemein

- **Repo-Struktur zuerst** — Komplett aufsetzen bevor Code geschrieben wird.
- **Compose validieren** — Jeder Service braucht ein gültiges Build-Context + Dockerfile, auch wenn nur ein Stub.
- **Schrittweise verifizieren** — Jeden Container einzeln bauen, nicht alles auf einmal.

# Lessons Learned — Phase 1

## Rust / axum

| Problem | Root Cause | Lösung |
|---------|-----------|--------|
| Alle API-Requests hängen (ReadTimeout) | `Mutex`-Deadlock: `insert_thought` hielt Lock, rief `get_thought` auf das erneut lockt | Statische interne Methoden die `&Connection` direkt nehmen statt `self.conn.lock()` |
| PyTorch CUDA Downloads dauern ewig | `sentence-transformers` zieht ~4GB nvidia-* Pakete | `--extra-index-url https://download.pytorch.org/whl/cpu` in requirements.txt |
| `podman compose up --build` nutzt altes Image | Compose cached Images aggressiv | `podman build` separat, dann `podman stop/rm/compose up` |

# Lessons Learned — Phase 2

## Allgemein & Tooling

| Problem | Root Cause | Lösung |
|---------|-----------|--------|
| Test-Timeouts bei Fallback-Logik in TDD | Wenn die primäre API (LM Studio) down ist, summieren sich Verbindungs-Timeouts auf, was zu failing Tests (`httpx.ReadTimeout`) führt. | Für `dev-container` Tests den primären Endpunkt (`EMBEDDING_URL=""`) explizit deaktivieren, um sofort den lokalen Fallback zu nutzen. |
| Rad neu erfinden vermeiden | MCP in Rust via SSE und JSON-RPC manuell zu bauen ist aufwendig | Vorab-Analyse des Ökosystems zeigte fertige Crates (`model-context-protocol`, `rust-mcp-sdk`). Immer *zuerst* OSS-Vorarbeit prüfen. |
