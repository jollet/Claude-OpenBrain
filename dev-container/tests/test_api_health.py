"""
Phase 0 Tests — Health Endpoint
TDD: Diese Tests wurden ZUERST geschrieben (Red), dann brain-core implementiert (Green).
"""
import httpx
import pytest
from tests.models.thought import HealthResponse


def test_health_returns_200(client: httpx.Client):
    """🔴→🟢 Health-Endpoint muss erreichbar sein."""
    resp = client.get("/health")
    assert resp.status_code == 200


def test_health_response_matches_schema(client: httpx.Client):
    """🔴→🟢 Response muss dem Pydantic-Schema entsprechen."""
    resp = client.get("/health")
    health = HealthResponse.model_validate(resp.json())
    assert health.status in ("ok", "degraded")
    assert health.version != ""


def test_health_shows_db_healthy(client: httpx.Client):
    """🔴→🟢 Datenbank muss nach Start initialisiert sein."""
    resp = client.get("/health")
    health = HealthResponse.model_validate(resp.json())
    assert health.db_healthy is True


def test_health_shows_embedding_backend(client: httpx.Client):
    """🔴→🟢 Embedding-Backend-Name muss bekannt sein."""
    resp = client.get("/health")
    health = HealthResponse.model_validate(resp.json())
    assert isinstance(health.embedding_backend, str)
    assert len(health.embedding_backend) > 0
