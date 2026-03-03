# Phase 2 Design: Retrieval & MCP Server

## Overview
Phase 2 focuses on semantic retrieval of thoughts using vector search and exposing the functionality to AI assistants via the Model Context Protocol (MCP).

## Architecture Decisions

### 1. Vector Database (`sqlite-vec`)
- **Technology**: We will use the `sqlite-vec` C-extension loaded into `rusqlite`.
- **Storage**: A virtual table `vec_thoughts(id integer primary key, embedding float[384])` will store the embeddings.
- **Workflow**:
  - `POST /api/thoughts` will immediately fetch the embedding from the `EmbeddingClient` before saving to SQLite.
  - The vector (e.g., 384 dimensions for `all-MiniLM-L6-v2`) will be inserted into `vec_thoughts`.
  - If the embedding backend fails, the thought is saved with `has_embedding = false` and skipped in the vector table.
- **Why**: Keeps all data local and single-file (SQLite). Extremely fast for our scale without deploying standalone vector databases like Qdrant or Pinecone.

### 2. Semantic Search (Retrieval)
- **Endpoint**: `POST /api/search`
- **Request**: `{"query": "dinner ideas", "limit": 10}`
- **Workflow**:
  1. Generate embedding vector $V$ for `"dinner ideas"` via `EmbeddingClient`.
  2. Query SQLite using KNN: 
     ```sql
     SELECT thought_id, distance 
     FROM vec_thoughts 
     WHERE embedding MATCH $V AND k = $limit
     ORDER BY distance
     ```
  3. Join the `thought_id`s with the `thoughts` and `tags` tables to construct the full response objects.
- **CLI**: Implement `brain search <query>` in the `brain-cli` to interact with this new endpoint.

### 3. MCP Server Integration (Option A: HTTP/SSE)
- **Technology**: Expose MCP over HTTP Server-Sent Events (SSE) directly from the `brain-core` axum web server.
- **Endpoints**: 
  - `GET /mcp/sse` (Stream connection from server to client)
  - `POST /mcp/message` (JSON-RPC messages from client to server)
- **Registered Tools**:
  1. `add_thought(content: str, tags: list[str]) -> id: int`
  2. `search_thoughts(query: str, limit: int) -> list[Thought]`
  3. `get_stats() -> dict` (Count of thoughts, DB size, limits)
  4. `get_health() -> dict` (Embedding backend availability, core status)
- **Why**: Since `brain-core` is already a running server (inside a container), exposing HTTP routes is significantly easier than managing standard I/O (stdio) pipes between containers and desktop applications.

### 4. Embedding Fallback
- **Decision**: We will keep the Python `embedding-fallback` container for now (Option C).
- **Future Note**: We will investigate `sentence-transformers-rs` during implementation to see if it's stable enough to completely replace the Python container. If successful, the Python container will be dropped.

## Implementation Steps (TDD)
1. Add `sqlite-vec` to `rusqlite` dependencies and update DB schemas.
2. Write red tests for `POST /api/search`.
3. Implement `POST /api/search` in `brain-core`.
4. Update `brain-cli` with `search` and `stats` commands.
5. Write tests for MCP SSE endpoints.
6. Implement the MCP Server and the 4 tools.
