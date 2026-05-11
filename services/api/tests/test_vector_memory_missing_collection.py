import sys
import unittest
from pathlib import Path
from unittest.mock import patch

import httpx

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from app.config import Settings
from app.vector_memory import ChunkVectorSearchRequest, search_chunk_vectors


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


if __name__ == "__main__":
    unittest.main()
