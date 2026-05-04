from dataclasses import dataclass
from hashlib import sha256
from pathlib import Path


class ArtifactStoreError(RuntimeError):
    pass


@dataclass(frozen=True)
class StoredArtifact:
    content_hash: str
    storage_path: str
    size_bytes: int
    existed: bool


def _relative_hash_path(content_hash: str) -> Path:
    return Path("sha256") / content_hash[:2] / content_hash[2:4] / content_hash


def store_artifact_bytes(content: bytes, artifact_store_path: str) -> StoredArtifact:
    content_hash = sha256(content).hexdigest()
    relative_path = _relative_hash_path(content_hash)
    root = Path(artifact_store_path).expanduser().resolve()
    target = root / relative_path
    target.parent.mkdir(parents=True, exist_ok=True)

    if target.exists():
        existing_hash = sha256(target.read_bytes()).hexdigest()
        if existing_hash != content_hash:
            raise ArtifactStoreError("Existing artifact path content hash does not match expected hash")
        return StoredArtifact(
            content_hash=content_hash,
            storage_path=relative_path.as_posix(),
            size_bytes=len(content),
            existed=True,
        )

    try:
        with target.open("xb") as artifact_file:
            artifact_file.write(content)
    except FileExistsError:
        existing_hash = sha256(target.read_bytes()).hexdigest()
        if existing_hash != content_hash:
            raise ArtifactStoreError("Concurrent artifact write produced mismatched content")
        return StoredArtifact(
            content_hash=content_hash,
            storage_path=relative_path.as_posix(),
            size_bytes=len(content),
            existed=True,
        )

    return StoredArtifact(
        content_hash=content_hash,
        storage_path=relative_path.as_posix(),
        size_bytes=len(content),
        existed=False,
    )


def read_artifact_bytes(storage_path: str, artifact_store_path: str) -> bytes:
    relative_path = Path(storage_path)
    if relative_path.is_absolute():
        raise ArtifactStoreError("Artifact storage path must be relative")

    root = Path(artifact_store_path).expanduser().resolve()
    target = (root / relative_path).resolve()
    try:
        target.relative_to(root)
    except ValueError as exc:
        raise ArtifactStoreError("Artifact storage path escapes artifact store") from exc

    if not target.is_file():
        raise ArtifactStoreError("Artifact file not found")
    return target.read_bytes()
