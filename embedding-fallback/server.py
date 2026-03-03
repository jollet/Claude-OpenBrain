from fastapi import FastAPI
from pydantic import BaseModel
from sentence_transformers import SentenceTransformer
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="Open Brain - Embedding Fallback")
model_name = "all-MiniLM-L6-v2"
model = None

@app.on_event("startup")
async def startup_event():
    global model
    logger.info(f"Loading {model_name}...")
    model = SentenceTransformer(model_name)
    logger.info("Model loaded successfully.")

class EmbedRequest(BaseModel):
    text: str

class EmbedResponse(BaseModel):
    embedding: list[float]
    model: str

@app.get("/health")
def health():
    return {"status": "ok", "model_loaded": model is not None}

@app.post("/embed", response_model=EmbedResponse)
def embed(req: EmbedRequest):
    embedding = model.encode(req.text).tolist()
    return EmbedResponse(
        embedding=embedding,
        model=model_name
    )
