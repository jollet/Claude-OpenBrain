from pydantic import BaseModel
from typing import Optional

class HealthResponse(BaseModel):
    status: str
    version: str
    db_healthy: bool
    embedding_backend: str
