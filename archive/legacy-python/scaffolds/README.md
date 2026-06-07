# Archived Python Scaffolds (grok branch)

These directories were removed from the active tree because the grok branch runtime is Rust-native:

- `services/` — legacy Python collector scaffolds (superseded by `crates/igy6-normalization`, `crates/igy6-write-api`, and gateway routes)
- `policy/` — legacy Python policy constants (superseded by `crates/igy6-policy`)
- `snippet-vault/` — migration archaeology from the Rust cutover
- `Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md` — pre-cutover Python/FastAPI build spec

Nothing here is wired into `infra/docker-compose.yml` on `grok`. Kept for history only.