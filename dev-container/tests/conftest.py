import pytest
import httpx
import os

@pytest.fixture
def client():
    # Use the environment variable if present, defaulting to localhost:3000
    base_url = os.environ.get("BRAIN_API_URL", "http://localhost:3000")
    with httpx.Client(base_url=base_url) as client:
        yield client
