use gear_depot::{
    ArtifactManifest, ArtifactRef, ArtifactState, ArtifactType, Checksum, ChecksumAlgorithm,
    DistributionMetadata, PackageType, RetentionMetadata, SafeMetadata,
};

fn hash() -> String {
    format!("sha256:{}", "0".repeat(64))
}

#[test]
fn canvas_handoff_spec_package_artifact_contract_is_valid() {
    let artifact = ArtifactRef {
        artifact_id: "artifact:package-demo".to_string(),
        artifact_type: ArtifactType::SpecPackage,
        producer: "rumble-canvas".to_string(),
        version: "0.1.0".to_string(),
        hash: hash(),
        manifest_ref: "manifest:package-demo".to_string(),
        state: ArtifactState::Active,
        created_at: "2026-06-30T00:00:00Z".to_string(),
    };

    artifact.validate().expect("artifact ref is valid");

    let manifest = ArtifactManifest {
        manifest_id: "manifest:package-demo".to_string(),
        artifact,
        package_type: PackageType::JsonBundle,
        checksums: vec![Checksum {
            algorithm: ChecksumAlgorithm::Sha256,
            value: hash(),
        }],
        provenance_id: "provenance:handoff-demo-valid".to_string(),
        retention: RetentionMetadata::default(),
        distribution: DistributionMetadata::default(),
        metadata: SafeMetadata::from_pairs([
            (
                "contract".to_string(),
                "implementation-handoff.v0.1".to_string(),
            ),
            (
                "fixture".to_string(),
                "canvas-minimal.valid.json".to_string(),
            ),
        ]),
    };

    manifest.validate().expect("artifact manifest is valid");
    assert!(manifest.stable_hash().starts_with("sha256:"));
}

#[test]
fn canvas_handoff_artifact_contract_rejects_non_sha256_hash() {
    let artifact = ArtifactRef {
        artifact_id: "artifact:package-demo".to_string(),
        artifact_type: ArtifactType::SpecPackage,
        producer: "rumble-canvas".to_string(),
        version: "0.1.0".to_string(),
        hash: "md5:not-allowed".to_string(),
        manifest_ref: "manifest:package-demo".to_string(),
        state: ArtifactState::Active,
        created_at: "2026-06-30T00:00:00Z".to_string(),
    };

    assert!(artifact.validate().is_err());
}
