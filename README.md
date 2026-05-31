# Gear Depot

**Layer:** Gear — Infrastructure  
**Role:** sovereign artifact depot and policy gate  
**Mission:** verify, cache, proxy, and distribute artifacts with integrity, provenance, and supply-chain control.

---

## Purpose

`gear-depot` is the logistics and trust layer of the ecosystem. It handles how artifacts — code, packages, models, datasets, build outputs — are cached, verified, governed, and distributed.

It exists to make supply chains sovereign and auditable.

## Owns

- Registry proxy/cache behavior across ecosystems such as Cargo, npm, PyPI, and others.
- Artifact integrity, checksums, signatures, provenance, and policy evidence.
- Supply-chain rules and install/source policy.
- Sovereign distribution boundaries.

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
