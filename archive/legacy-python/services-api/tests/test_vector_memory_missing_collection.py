import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import httpx

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from app.config import Settings
from app.vector_memory import ChunkVectorSearchRequest, search_chunk_vectors, upsert_chunk_vectors_by_ids


class _ScalarResult:
    def __init__(self, rows: list[object]) -> None:
        self._rows = rows

    def all(self) -> list[object]:
        return self._rows


class _FakeDb:
    def __init__(self, rows: list[object]) -> None:
        self.rows = rows
        self.commit_count = 0

    def scalars(self, statement: object) -> _ScalarResult:
        return _ScalarResult(self.rows)

    def commit(self) -> None:
        self.commit_count += 1


class MissingQdrantCollectionSearchTest(unittest.TestCase):
    def test_missing_configured_collection_returns_empty_search_result(self) -> None:
        settings = Settings(qdrant_chunk_collection="igy6_chunks")
        response = httpx.Response(
            status_code=404,
            text='{"status":{"error":"Not found: Collection `igy6_chunks` doesn\'t exist!"}}',
            request=httpx.Request("POST", "http://qdrant:6333/collections/igy6_chunks/points/search"),
        )

        with patch("app.vector_memory.httpx.post", return_value=response) as post:
            result = search_chunk_vectors(settings, ChunkVectorSearchRequest(query="What does IGY6 know?", limit=5))

        self.assertEqual(result.query, "What does IGY6 know?")
        self.assertEqual(result.collection_name, "igy6_chunks")
        self.assertFalse(result.collection_exists)
        self.assertEqual(result.hits, [])
        post.assert_called_once()

    def test_ingestion_vector_upsert_creates_missing_collection_and_populates_points(self) -> None:
        settings = Settings(qdrant_chunk_collection="igy6_chunks", qdrant_chunk_vector_size=8)
        chunk = SimpleNamespace(
            id="chunk-1",
            document_id="document-1",
            chunk_index=0,
            text_content="IGY6 stores local evidence for retrieval.",
            embedding_status="not_started",
            metadata_json={},
        )
        db = _FakeDb([chunk])
        get_responses = [
            httpx.Response(status_code=404, text="not found"),
            httpx.Response(
                status_code=200,
                json={"result": {"status": "green"}},
                request=httpx.Request("GET", "http://qdrant:6333/collections/igy6_chunks"),
            ),
        ]
        put_responses = [
            httpx.Response(status_code=200, text="created"),
            httpx.Response(status_code=200, text="upserted"),
        ]

        with patch("app.vector_memory.httpx.get", side_effect=get_responses) as get:
            with patch("app.vector_memory.httpx.put", side_effect=put_responses) as put:
                result = upsert_chunk_vectors_by_ids(db, settings, ["chunk-1"])

        self.assertEqual(result.collection_name, "igy6_chunks")
        self.assertTrue(result.collection_exists)
        self.assertEqual(result.chunks_selected, 1)
        self.assertEqual(result.chunks_upserted, 1)
        self.assertEqual(chunk.embedding_status, "completed")
        self.assertEqual(chunk.metadata_json["embedding_method"], "local_hash_v1")
        self.assertEqual(chunk.metadata_json["vector_collection"], "igy6_chunks")
        self.assertEqual(db.commit_count, 1)
        self.assertEqual(get.call_count, 2)
        self.assertEqual(put.call_count, 2)


if __name__ == "__main__":
    unittest.main()
