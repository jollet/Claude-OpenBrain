# Entwicklungsvorgaben — Open Brain

## Sicherheit & Datenschutz

- **Keine Secrets im Code** — Passwörter, API Keys, Tokens ausschließlich in `.env` / YAML. Immer in `.gitignore`.
- **Keine IPs** — Private IPs (`192.168.x.x`) durch Platzhalter ersetzen (`<VM_IP>`, `<HA_IP>`).
- **Config und Secrets strikt getrennt vom Code** — `.env.example` ins Repo, `.env` mit echten Werten nie.
- **`git diff` vor jedem Commit** auf Secrets und IPs prüfen.

## Entwicklungsmethodik

- **TDD: Red → Green → Refactor** — Failing Test zuerst, dann minimale Implementierung, dann aufräumen.
- **Commits erst nach dem Task** — Einen Commit gibt es nur, wenn ein Task (oder eine logische Einheit) vollständig umgesetzt ist und die Tests "Grün" sind.
- **Bibliotheken prüfen** — *Zuerst* nachschauen, ob es fertige Crates oder Pakete gibt, bevor man komplexe Spezifikationen (wie MCP) selbst baut.
- **Einfache, robuste Lösungen für den PoC** — Verfeinerung kommt später. YAGNI.
- **Strikte Datentrennung** — `brain.db` nie im Repo. Synthetische Testdaten in `fixtures/` sind OK.

## Tooling

- **`uv` statt `pip`** in allen Python-Containern (schneller, reproduzierbarer).
- **Podman-Images prüfen** (`podman images`) bevor neue Base-Images gepullt werden.
- **Slim-Images bevorzugen** — `python:3.12-slim`, `rust:X-slim-bookworm`, `debian:bookworm-slim`.

## Container & Compose

- **Healthchecks ohne `curl`** — In Python-Containern `python -c "import urllib.request; ..."` verwenden.
- **`start_period` großzügig setzen** — Embedding-Modelle brauchen Zeit zum Laden (≥120s).
- **Alle Workspace-Members müssen existieren** bevor `cargo build` im Container läuft.
