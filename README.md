# Gear Depot

[![CI](https://github.com/constantin-jais/gear-depot/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/constantin-jais/gear-depot/actions/workflows/ci.yml)
[![Security](https://github.com/constantin-jais/gear-depot/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/constantin-jais/gear-depot/actions/workflows/security.yml)
[![Contracts](https://github.com/constantin-jais/gear-depot/actions/workflows/contracts.yml/badge.svg?branch=main)](https://github.com/constantin-jais/gear-depot/actions/workflows/contracts.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**Layer:** Gear — Infrastructure  
**Role:** sovereign artifact depot and policy gate  
**Mission:** verify, cache, proxy, and distribute artifacts with integrity, provenance, and supply-chain control.

---

## Stack role

- **Layer:** Gear — Infrastructure.
- **Role:** sovereign artifact depot and policy gate.
- **Mission:** verify, cache, proxy, and distribute artifacts with integrity, provenance, and supply-chain control.
- **Maturity:** `contract-first`.
- **Scale-ready:** no — contracts/tests exist, but storage/cache policy and real artifact integrations are still next steps.
- **Current increment:** P0 artifact contract.
- **Learning value:** artifact trust, safe metadata, retention, provenance, and supply-chain policy.
- **Next quality step:** add storage/cache policy and integrate real handoff/report artifacts.

See the ecosystem cockpit in [`constantin-jais/ecosystem/status.md`](https://github.com/constantin-jais/constantin-jais/blob/main/ecosystem/status.md).

## Dogfooding

This repository is part of the forge dogfooding loop: the ecosystem should use its own tools to make specs, maturity, contracts, releases, and product documentation observable.

Current visible evidence:

- contract and security workflows exercise artifact and policy assumptions;
- README maturity notes keep storage/cache limits explicit;
- fixtures and contracts frame provenance and retention behavior.

Expected next evidence:

- publish example artifact manifests and policy reports;
- show Gear Cable handoff evidence through fixture-backed workflows.

Dogfooding claims should stay backed by visible commands, fixtures, CI workflows, generated reports, or linked docs.

## Contributing

See:

- [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines;
- [ROADMAP.md](ROADMAP.md) for current contribution priorities;
- [issue templates](.github/ISSUE_TEMPLATE/) for bugs, docs issues, fixture/example requests, and design discussions.

## Forge role

`gear-depot` is Gear supply-chain infrastructure. It gives Rumble products, Bolt workflows, and Wrench tools a governed place for verified artifacts, manifests, provenance, retention, and distribution policy.

## Boundary

It must not become a generic file store, release planner, memory substrate, product workflow, or orchestrator. Release wiring belongs to `gear-cable`; memory belongs to `gear-memory`; product and execution decisions stay above it.

## Purpose

`gear-depot` is the logistics and trust layer of the ecosystem. It handles how artifacts — code, packages, models, datasets, build outputs — are cached, verified, governed, and distributed.

It exists to make supply chains sovereign and auditable.

## Owns

- Registry proxy/cache behavior across ecosystems such as Cargo, npm, PyPI, and others.
- Artifact integrity, checksums, signatures, provenance, and policy evidence.
- Supply-chain rules and install/source policy.
- Sovereign distribution boundaries.
- `ArtifactRef` and `ArtifactManifest` substrate contracts.

## Does Not Own

- Product UX: belongs to Rumble.
- Release planning and cross-target packaging: belongs to `gear-cable`.
- Memory/search semantics: belongs to `gear-memory`.
- Orchestration decisions: belongs to Bolt.

## Allowed Dependencies

- Consumes artifact plans and release metadata from `gear-cable`.
- Exposes verified artifacts to Rumble/Bolt/Wrench consumers.
- May integrate with self-hostable registries and storage backends.

## Product Vision Challenge

`gear-depot` must not become a generic file store. Its product is trust in distribution: policy, provenance, cache, verification, and sovereignty.

## P0 Contracts

`gear-depot` currently exposes minimal serializable Rust contracts:

- `ArtifactRef`: stable reference to produced output that can be packaged,
  verified, retained, revoked, or distributed.
- `ArtifactManifest`: manifest for package type, checksums, provenance,
  retention, distribution metadata, and safe metadata.
- `RetentionMetadata`: policy and lifecycle timestamps.
- `DistributionMetadata`: channel, install floor, and publish timestamp
  metadata.

Rumble products own artifact-producing workflows and UX. `gear-cable` owns
release planning and build provenance. `gear-depot` owns verification, trust
state, retention, revocation, and distribution availability.

## Validation Rules

Validation is explicit through `validate()` on `ArtifactRef` and
`ArtifactManifest`.

- Required IDs, producer, version, and provenance references must be non-empty.
- `artifact.manifest_ref` must equal the owning manifest's `manifest_id`.
- Artifact hashes use `sha256:<64 hex chars>`.
- P0 manifests must contain at least one SHA-256 checksum; checksum values are
  64 hexadecimal characters.
- Timestamps use RFC3339 / ISO 8601 with an explicit offset, for example
  `2026-06-30T00:00:00Z`.
- Metadata rejects secret-like keys: `secret`, `token`, `password`,
  `credential`, and `api_key`.
- `Debug` output for metadata redacts values; callers must still validate before
  persistence.

## Example

```rust
use gear_depot::{
    ArtifactManifest, ArtifactRef, ArtifactState, ArtifactType, Checksum,
    ChecksumAlgorithm, DistributionMetadata, PackageType, RetentionMetadata,
    SafeMetadata,
};

let manifest = ArtifactManifest {
    manifest_id: "manifest_01".to_string(),
    artifact: ArtifactRef {
        artifact_id: "art_01".to_string(),
        artifact_type: ArtifactType::ReleaseAsset,
        producer: "gear-cable".to_string(),
        version: "1.0.0".to_string(),
        hash: format!("sha256:{}", "a".repeat(64)),
        manifest_ref: "manifest_01".to_string(),
        state: ArtifactState::Active,
        created_at: "2026-06-30T00:00:00Z".to_string(),
    },
    package_type: PackageType::Binary,
    checksums: vec![Checksum {
        algorithm: ChecksumAlgorithm::Sha256,
        value: "b".repeat(64),
    }],
    provenance_id: "prov_01".to_string(),
    retention: RetentionMetadata::default(),
    distribution: DistributionMetadata::default(),
    metadata: SafeMetadata::default(),
};

manifest.validate()?;
let manifest_hash = manifest.stable_hash();
```
