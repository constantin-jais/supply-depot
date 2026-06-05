use gear_depot::{
    ArtifactManifest, ArtifactRef, ArtifactState, ArtifactType, Checksum, ChecksumAlgorithm,
    DistributionMetadata, PackageType, RetentionMetadata, SafeMetadata,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let _manifest_hash = manifest.stable_hash();

    Ok(())
}
