from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from app.artifact_store import StoredArtifact, store_artifact_bytes


class LocalProjectCollectionError(RuntimeError):
    pass


@dataclass(frozen=True)
class CollectedLocalProjectFile:
    source_path: str
    relative_path: str
    artifact: StoredArtifact


@dataclass(frozen=True)
class LocalProjectCollectionResult:
    total_files: int
    collected_files: int
    skipped_files: list[dict[str, Any]] = field(default_factory=list)
    files: list[CollectedLocalProjectFile] = field(default_factory=list)


def _is_relative_to(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def _scope_paths(source_root: Path, permission_scope: dict[str, Any]) -> list[Path]:
    raw_paths = permission_scope.get("paths")
    if not isinstance(raw_paths, list) or not raw_paths:
        raise LocalProjectCollectionError("local_project collection requires permission scope paths")

    resolved_paths: list[Path] = []
    for raw_path in raw_paths:
        if not isinstance(raw_path, str) or not raw_path:
            raise LocalProjectCollectionError("permission scope paths must be non-empty strings")
        candidate = Path(raw_path).expanduser()
        if not candidate.is_absolute():
            candidate = source_root / candidate
        resolved = candidate.resolve()
        if not _is_relative_to(resolved, source_root):
            raise LocalProjectCollectionError("permission scope path escapes the source location")
        resolved_paths.append(resolved)
    return resolved_paths


def _iter_candidate_files(path: Path) -> list[Path]:
    if path.is_symlink():
        return []
    if path.is_file():
        return [path]
    if not path.is_dir():
        return []
    return sorted(candidate for candidate in path.rglob("*") if not candidate.is_symlink() and candidate.is_file())


def collect_local_project_files(
    *,
    source_location: str | None,
    permission_scope: dict[str, Any],
    artifact_store_path: str,
) -> LocalProjectCollectionResult:
    if not source_location:
        raise LocalProjectCollectionError("local_project source requires a location")

    source_root = Path(source_location).expanduser().resolve()
    if not source_root.is_dir():
        raise LocalProjectCollectionError("local_project source location must be an existing directory")

    max_files = int(permission_scope.get("max_files", 100))
    max_file_bytes = int(permission_scope.get("max_file_bytes", 1_000_000))
    if max_files < 1:
        raise LocalProjectCollectionError("max_files must be at least 1")
    if max_file_bytes < 1:
        raise LocalProjectCollectionError("max_file_bytes must be at least 1")

    collected: list[CollectedLocalProjectFile] = []
    skipped: list[dict[str, Any]] = []
    candidates: list[Path] = []

    for scoped_path in _scope_paths(source_root, permission_scope):
        candidates.extend(_iter_candidate_files(scoped_path))

    unique_candidates = sorted(dict.fromkeys(candidates))
    for candidate in unique_candidates:
        if len(collected) >= max_files:
            skipped.append({"path": str(candidate), "reason": "max_files_reached"})
            continue
        if not _is_relative_to(candidate.resolve(), source_root):
            skipped.append({"path": str(candidate), "reason": "escaped_source_location"})
            continue
        size_bytes = candidate.stat().st_size
        if size_bytes > max_file_bytes:
            skipped.append({"path": str(candidate), "reason": "max_file_bytes_exceeded", "size_bytes": size_bytes})
            continue
        artifact = store_artifact_bytes(candidate.read_bytes(), artifact_store_path)
        collected.append(
            CollectedLocalProjectFile(
                source_path=str(candidate),
                relative_path=candidate.relative_to(source_root).as_posix(),
                artifact=artifact,
            )
        )

    return LocalProjectCollectionResult(
        total_files=len(unique_candidates),
        collected_files=len(collected),
        skipped_files=skipped,
        files=collected,
    )
