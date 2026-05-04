from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from neo4j import GraphDatabase
from pydantic import BaseModel
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.config import Settings, get_settings
from app.db import get_db
from app.models import Chunk, EvidenceItem, NormalizedDocument, RawArtifact, Source

router = APIRouter(prefix="/memory/graph", tags=["memory-graph"])


class GraphSchemaStatus(BaseModel):
    constraints: list[dict[str, Any]]


class GraphLineageSyncResult(BaseModel):
    nodes: int
    relationships: int


class GraphRelationshipRead(BaseModel):
    direction: str
    relationship_type: str
    neighbor_label: str | None
    neighbor_id: str | None


class GraphRelationshipList(BaseModel):
    node_label: str
    node_id: str
    relationships: list[GraphRelationshipRead]


def allowed_graph_node_labels() -> set[str]:
    return {"Source", "RawArtifact", "Document", "Chunk", "EvidenceItem"}


def lineage_relationship_types() -> list[str]:
    return [
        "SOURCE_HAS_ARTIFACT",
        "ARTIFACT_HAS_DOCUMENT",
        "DOCUMENT_HAS_CHUNK",
        "DOCUMENT_HAS_EVIDENCE",
        "CHUNK_HAS_EVIDENCE",
    ]


def graph_constraint_statements() -> list[str]:
    return [
        "CREATE CONSTRAINT source_id_unique IF NOT EXISTS FOR (node:Source) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT raw_artifact_id_unique IF NOT EXISTS FOR (node:RawArtifact) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT document_id_unique IF NOT EXISTS FOR (node:Document) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT chunk_id_unique IF NOT EXISTS FOR (node:Chunk) REQUIRE node.id IS UNIQUE",
        "CREATE CONSTRAINT evidence_item_id_unique IF NOT EXISTS FOR (node:EvidenceItem) REQUIRE node.id IS UNIQUE",
    ]


def _driver(settings: Settings):
    return GraphDatabase.driver(
        settings.neo4j_uri,
        auth=(settings.neo4j_user, settings.neo4j_password),
    )


def list_graph_constraints(settings: Settings) -> GraphSchemaStatus:
    try:
        with _driver(settings) as driver:
            records = driver.execute_query("SHOW CONSTRAINTS YIELD name, type, labelsOrTypes, properties RETURN *")
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=str(exc)) from exc
    return GraphSchemaStatus(constraints=[dict(record) for record in records.records])


def _run_write(driver, statement: str, parameters: dict[str, Any]) -> None:
    driver.execute_query(statement, parameters)


def sync_graph_lineage(db: Session, settings: Settings) -> GraphLineageSyncResult:
    sources = list(db.scalars(select(Source)).all())
    artifacts = list(db.scalars(select(RawArtifact)).all())
    documents = list(db.scalars(select(NormalizedDocument)).all())
    chunks = list(db.scalars(select(Chunk)).all())
    evidence_items = list(db.scalars(select(EvidenceItem)).all())

    node_count = 0
    relationship_count = 0
    try:
        with _driver(settings) as driver:
            for statement in graph_constraint_statements():
                driver.execute_query(statement)

            for source in sources:
                _run_write(
                    driver,
                    "MERGE (:Source {id: $id})",
                    {"id": source.id},
                )
                node_count += 1

            for artifact in artifacts:
                _run_write(
                    driver,
                    "MERGE (:RawArtifact {id: $id})",
                    {"id": artifact.id},
                )
                node_count += 1
                if artifact.source_id is not None:
                    _run_write(
                        driver,
                        """
                        MATCH (source:Source {id: $source_id})
                        MATCH (artifact:RawArtifact {id: $artifact_id})
                        MERGE (source)-[:SOURCE_HAS_ARTIFACT]->(artifact)
                        """,
                        {"source_id": artifact.source_id, "artifact_id": artifact.id},
                    )
                    relationship_count += 1

            for document in documents:
                _run_write(
                    driver,
                    "MERGE (:Document {id: $id})",
                    {"id": document.id},
                )
                node_count += 1
                if document.raw_artifact_id is not None:
                    _run_write(
                        driver,
                        """
                        MATCH (artifact:RawArtifact {id: $artifact_id})
                        MATCH (document:Document {id: $document_id})
                        MERGE (artifact)-[:ARTIFACT_HAS_DOCUMENT]->(document)
                        """,
                        {"artifact_id": document.raw_artifact_id, "document_id": document.id},
                    )
                    relationship_count += 1

            for chunk in chunks:
                _run_write(
                    driver,
                    "MERGE (:Chunk {id: $id})",
                    {"id": chunk.id},
                )
                node_count += 1
                _run_write(
                    driver,
                    """
                    MATCH (document:Document {id: $document_id})
                    MATCH (chunk:Chunk {id: $chunk_id})
                    MERGE (document)-[:DOCUMENT_HAS_CHUNK]->(chunk)
                    """,
                    {"document_id": chunk.document_id, "chunk_id": chunk.id},
                )
                relationship_count += 1

            for evidence_item in evidence_items:
                _run_write(
                    driver,
                    "MERGE (:EvidenceItem {id: $id})",
                    {"id": evidence_item.id},
                )
                node_count += 1
                if evidence_item.document_id is not None:
                    _run_write(
                        driver,
                        """
                        MATCH (document:Document {id: $document_id})
                        MATCH (evidence:EvidenceItem {id: $evidence_id})
                        MERGE (document)-[:DOCUMENT_HAS_EVIDENCE]->(evidence)
                        """,
                        {"document_id": evidence_item.document_id, "evidence_id": evidence_item.id},
                    )
                    relationship_count += 1
                if evidence_item.chunk_id is not None:
                    _run_write(
                        driver,
                        """
                        MATCH (chunk:Chunk {id: $chunk_id})
                        MATCH (evidence:EvidenceItem {id: $evidence_id})
                        MERGE (chunk)-[:CHUNK_HAS_EVIDENCE]->(evidence)
                        """,
                        {"chunk_id": evidence_item.chunk_id, "evidence_id": evidence_item.id},
                    )
                    relationship_count += 1
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=str(exc)) from exc

    return GraphLineageSyncResult(nodes=node_count, relationships=relationship_count)


def ensure_graph_constraints(settings: Settings) -> GraphSchemaStatus:
    try:
        with _driver(settings) as driver:
            for statement in graph_constraint_statements():
                driver.execute_query(statement)
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=str(exc)) from exc
    return list_graph_constraints(settings)


def list_node_relationships(
    *,
    settings: Settings,
    node_label: str,
    node_id: str,
    limit: int = 100,
) -> GraphRelationshipList:
    if node_label not in allowed_graph_node_labels():
        raise HTTPException(status_code=status.HTTP_422_UNPROCESSABLE_ENTITY, detail="Unsupported graph node label")

    statement = f"""
    MATCH (node:{node_label} {{id: $node_id}})
    OPTIONAL MATCH (node)-[outgoing]->(out_neighbor)
    WITH node, collect({{
        direction: 'outgoing',
        relationship_type: type(outgoing),
        neighbor_label: labels(out_neighbor)[0],
        neighbor_id: out_neighbor.id
    }}) AS outgoing_relationships
    OPTIONAL MATCH (in_neighbor)-[incoming]->(node)
    WITH outgoing_relationships + collect({{
        direction: 'incoming',
        relationship_type: type(incoming),
        neighbor_label: labels(in_neighbor)[0],
        neighbor_id: in_neighbor.id
    }}) AS relationships
    UNWIND relationships AS relationship
    WITH relationship
    WHERE relationship.relationship_type IS NOT NULL
    RETURN relationship
    LIMIT $limit
    """
    try:
        with _driver(settings) as driver:
            result = driver.execute_query(statement, {"node_id": node_id, "limit": limit})
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=str(exc)) from exc

    return GraphRelationshipList(
        node_label=node_label,
        node_id=node_id,
        relationships=[GraphRelationshipRead(**record["relationship"]) for record in result.records],
    )


@router.get("/schema", response_model=GraphSchemaStatus)
def get_graph_schema(settings: Settings = Depends(get_settings)) -> GraphSchemaStatus:
    return list_graph_constraints(settings)


@router.post("/schema/ensure", response_model=GraphSchemaStatus, status_code=status.HTTP_201_CREATED)
def ensure_graph_schema(settings: Settings = Depends(get_settings)) -> GraphSchemaStatus:
    return ensure_graph_constraints(settings)


@router.post("/lineage/sync", response_model=GraphLineageSyncResult)
def sync_lineage_graph(
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> GraphLineageSyncResult:
    return sync_graph_lineage(db, settings)


@router.get("/nodes/{node_label}/{node_id}/relationships", response_model=GraphRelationshipList)
def get_node_relationships(
    node_label: str,
    node_id: str,
    settings: Settings = Depends(get_settings),
) -> GraphRelationshipList:
    return list_node_relationships(settings=settings, node_label=node_label, node_id=node_id)
