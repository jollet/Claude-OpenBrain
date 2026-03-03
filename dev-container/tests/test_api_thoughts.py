"""
Phase 1 Tests — Thoughts CRUD
TDD: Diese Tests wurden ZUERST geschrieben (Red), dann brain-core implementiert (Green).
"""
import httpx
import pytest


def test_post_thought_returns_201_with_id(client: httpx.Client):
    """🔴 POST /api/thoughts → 201 + { id, content, created_at }"""
    resp = client.post("/api/thoughts", json={"content": "Mein erster Gedanke"})
    assert resp.status_code == 201
    data = resp.json()
    assert "id" in data
    assert data["content"] == "Mein erster Gedanke"
    assert "created_at" in data


def test_post_thought_with_tags(client: httpx.Client):
    """🔴 POST /api/thoughts mit Tags → Tags werden gespeichert."""
    resp = client.post("/api/thoughts", json={
        "content": "Gedanke mit Tags",
        "tags": ["test", "phase1"]
    })
    assert resp.status_code == 201
    data = resp.json()
    assert "tags" in data
    assert set(data["tags"]) == {"test", "phase1"}


def test_post_thought_empty_content_returns_400(client: httpx.Client):
    """🔴 Leerer Content → 400 Bad Request."""
    resp = client.post("/api/thoughts", json={"content": ""})
    assert resp.status_code == 400


def test_get_thoughts_returns_list(client: httpx.Client):
    """🔴 GET /api/thoughts → 200 + Liste."""
    # Erst einen Gedanken erstellen
    client.post("/api/thoughts", json={"content": "Für Liste"})
    resp = client.get("/api/thoughts")
    assert resp.status_code == 200
    data = resp.json()
    assert isinstance(data, list)
    assert len(data) >= 1


def test_get_thoughts_pagination(client: httpx.Client):
    """🔴 GET /api/thoughts?limit=2&offset=0 → paginierte Ergebnisse."""
    # Mehrere Gedanken erstellen
    for i in range(3):
        client.post("/api/thoughts", json={"content": f"Paginated {i}"})
    resp = client.get("/api/thoughts", params={"limit": 2, "offset": 0})
    assert resp.status_code == 200
    data = resp.json()
    assert len(data) <= 2


def test_get_thought_by_id(client: httpx.Client):
    """🔴 GET /api/thoughts/{id} → 200 + Einzelabruf."""
    create_resp = client.post("/api/thoughts", json={"content": "Einzelabruf-Test"})
    thought_id = create_resp.json()["id"]
    resp = client.get(f"/api/thoughts/{thought_id}")
    assert resp.status_code == 200
    data = resp.json()
    assert data["id"] == thought_id
    assert data["content"] == "Einzelabruf-Test"


def test_get_thought_not_found(client: httpx.Client):
    """🔴 GET /api/thoughts/99999 → 404."""
    resp = client.get("/api/thoughts/99999")
    assert resp.status_code == 404


def test_delete_thought(client: httpx.Client):
    """🔴 DELETE /api/thoughts/{id} → 204."""
    create_resp = client.post("/api/thoughts", json={"content": "Zum Löschen"})
    thought_id = create_resp.json()["id"]
    resp = client.delete(f"/api/thoughts/{thought_id}")
    assert resp.status_code == 204
    # Verify it's gone
    get_resp = client.get(f"/api/thoughts/{thought_id}")
    assert get_resp.status_code == 404
