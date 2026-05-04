from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from neo4j import GraphDatabase
from pydantic import BaseModel

from app.config import Settings, get_settings

router = APIRouter(prefix="/memory/graph", tags=["memory-graph"])


class GraphSchemaStatus(BaseModel):
    constraints: list[dict[str, Any]]


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


def ensure_graph_constraints(settings: Settings) -> GraphSchemaStatus:
    try:
        with _driver(settings) as driver:
            for statement in graph_constraint_statements():
                driver.execute_query(statement)
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_503_SERVICE_UNAVAILABLE, detail=str(exc)) from exc
    return list_graph_constraints(settings)


@router.get("/schema", response_model=GraphSchemaStatus)
def get_graph_schema(settings: Settings = Depends(get_settings)) -> GraphSchemaStatus:
    return list_graph_constraints(settings)


@router.post("/schema/ensure", response_model=GraphSchemaStatus, status_code=status.HTTP_201_CREATED)
def ensure_graph_schema(settings: Settings = Depends(get_settings)) -> GraphSchemaStatus:
    return ensure_graph_constraints(settings)
