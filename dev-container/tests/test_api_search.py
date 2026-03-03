"""
Phase 2 Tests — Semantic Search Endpoint
TDD: RED -> GREEN
"""
import httpx
import pytest
from typing import List
from tests.models.thought import Thought

def test_search_endpoint_exists(client: httpx.Client):
    """🔴→🟢 /api/search exists and accepts POST."""
    resp = client.post("/api/search", json={"query": "test"})
    # Should not be 404
    assert resp.status_code != 404

def test_search_returns_relevant_results(client: httpx.Client):
    """🔴→🟢 Semantic search returns conceptually related thoughts, even without exact word matches."""
    # 1. Create a thought about food
    food_resp = client.post("/api/thoughts", json={
        "content": "Ich sollte morgen eine Pizza bestellen.",
        "tags": ["food", "todo"]
    })
    assert food_resp.status_code == 201

    # 2. Create an unrelated thought
    code_resp = client.post("/api/thoughts", json={
        "content": "Rust iterators are zero-cost abstractions.",
        "tags": ["programming"]
    })
    assert code_resp.status_code == 201

    # 3. Search for a related concept (no exact match)
    search_resp = client.post("/api/search", json={
        "query": "Was essen wir zum Abendessen?",
        "limit": 5
    })
    assert search_resp.status_code == 200
    
    results = search_resp.json()
    assert isinstance(results, list)
    assert len(results) > 0

    # The food thought should be ranked higher (or be the only one)
    top_result = Thought.model_validate(results[0])
    assert "Pizza" in top_result.content

def test_search_limit_parameter(client: httpx.Client):
    """🔴→🟢 The limit parameter restricts the number of results."""
    # Create multiple thoughts
    for i in range(3):
        client.post("/api/thoughts", json={"content": f"Test thought number {i}"})
    
    search_resp = client.post("/api/search", json={
        "query": "Test thought",
        "limit": 2
    })
    assert search_resp.status_code == 200
    results = search_resp.json()
    assert len(results) <= 2
