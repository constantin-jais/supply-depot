//! gear-depot — Sovereign registry proxy/cache and supply-chain policy POC backed by Starmetal.
//!
//! This crate is intentionally a minimal skeleton. The first implementation
//! increments must keep the upstream boundary explicit and preserve the
//! sovereign constraints documented in `docs/adr/0001-scope-and-upstream-policy.md`.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Static project metadata used by the CLI and smoke tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectCard {
    pub name: &'static str,
    pub role: &'static str,
    pub upstream: &'static str,
    pub relationship: &'static str,
}

/// The repository's initial scope card.
pub const PROJECT: ProjectCard = ProjectCard {
    name: "gear-depot",
    role: "sovereign supply-chain depot",
    upstream: "Starmetal",
    relationship: "Infrastructure POC for registry caching and policy enforcement; not critical production path until promoted.",
};

/// Human-readable summary for CLI smoke runs.
pub fn summary() -> String {
    format!(
        "{} — {} (upstream: {})",
        PROJECT.name, PROJECT.role, PROJECT.upstream
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    SpecPackage,
    HandoffPayload,
    CuratedExport,
    LearningExport,
    ReleaseAsset,
    InspectionReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactState {
    Active,
    Revoked,
    Superseded,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub artifact_id: String,
    pub artifact_type: ArtifactType,
    pub producer: String,
    pub version: String,
    pub hash: String,
    pub manifest_ref: String,
    pub state: ArtifactState,
    pub created_at: String,
}

impl ArtifactRef {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        validate_non_empty_field("artifact_id", &self.artifact_id)?;
        validate_non_empty_field("producer", &self.producer)?;
        validate_non_empty_field("version", &self.version)?;
        validate_non_empty_field("manifest_ref", &self.manifest_ref)?;
        validate_sha256_field("artifact.hash", &self.hash)?;
        validate_timestamp_field("created_at", &self.created_at)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageType {
    JsonBundle,
    Tar,
    Zip,
    Binary,
    Container,
    RegistryPackage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecksumAlgorithm {
    Sha256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksum {
    pub algorithm: ChecksumAlgorithm,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionMetadata {
    pub policy_ref: Option<String>,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub delete_after: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistributionMetadata {
    pub channels: Vec<String>,
    pub install_floor: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestValidationError {
    EmptyField(&'static str),
    MissingSha256Checksum,
    SecretLikeMetadataKey(String),
    ManifestRefMismatch {
        manifest_id: String,
        manifest_ref: String,
    },
    MalformedSha256 {
        field: &'static str,
        value: String,
    },
    MalformedTimestamp {
        field: &'static str,
        value: String,
    },
}

impl fmt::Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "field `{field}` must not be empty"),
            Self::MissingSha256Checksum => write!(formatter, "manifest requires a sha256 checksum"),
            Self::SecretLikeMetadataKey(key) => {
                write!(formatter, "metadata key `{key}` may contain a secret")
            }
            Self::ManifestRefMismatch {
                manifest_id,
                manifest_ref,
            } => write!(
                formatter,
                "artifact manifest_ref `{manifest_ref}` does not match manifest_id `{manifest_id}`"
            ),
            Self::MalformedSha256 { field, value } => {
                write!(formatter, "field `{field}` is not a sha256 hash: `{value}`")
            }
            Self::MalformedTimestamp { field, value } => {
                write!(formatter, "field `{field}` is not RFC3339: `{value}`")
            }
        }
    }
}

impl std::error::Error for ManifestValidationError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub manifest_id: String,
    pub artifact: ArtifactRef,
    pub package_type: PackageType,
    pub checksums: Vec<Checksum>,
    pub provenance_id: String,
    pub retention: RetentionMetadata,
    pub distribution: DistributionMetadata,
    pub metadata: SafeMetadata,
}

impl ArtifactManifest {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        validate_non_empty_field("manifest_id", &self.manifest_id)?;
        validate_non_empty_field("provenance_id", &self.provenance_id)?;

        if let Err(MetadataValidationError::SecretLikeKey(key)) = self.metadata.validate() {
            return Err(ManifestValidationError::SecretLikeMetadataKey(key));
        }

        self.artifact.validate()?;

        if self.artifact.manifest_ref != self.manifest_id {
            return Err(ManifestValidationError::ManifestRefMismatch {
                manifest_id: self.manifest_id.clone(),
                manifest_ref: self.artifact.manifest_ref.clone(),
            });
        }

        validate_optional_timestamp_field(
            "retention.expires_at",
            self.retention.expires_at.as_deref(),
        )?;
        validate_optional_timestamp_field(
            "retention.revoked_at",
            self.retention.revoked_at.as_deref(),
        )?;
        validate_optional_timestamp_field(
            "retention.delete_after",
            self.retention.delete_after.as_deref(),
        )?;
        validate_optional_timestamp_field(
            "distribution.published_at",
            self.distribution.published_at.as_deref(),
        )?;

        if self
            .checksums
            .iter()
            .any(|checksum| checksum.algorithm == ChecksumAlgorithm::Sha256)
        {
            for checksum in &self.checksums {
                if checksum.algorithm == ChecksumAlgorithm::Sha256 {
                    validate_sha256_field("checksums.sha256", &checksum.value)?;
                }
            }

            return Ok(());
        }

        Err(ManifestValidationError::MissingSha256Checksum)
    }

    pub fn stable_hash(&self) -> String {
        stable_json_hash(self)
    }
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeMetadata {
    values: BTreeMap<String, String>,
}

impl SafeMetadata {
    pub fn from_pairs<const N: usize>(pairs: [(String, String); N]) -> Self {
        Self {
            values: BTreeMap::from(pairs),
        }
    }

    pub fn stable_hash(&self) -> String {
        stable_json_hash(&self.values)
    }

    pub fn validate(&self) -> Result<(), MetadataValidationError> {
        for key in self.values.keys() {
            if is_secret_like_key(key) {
                return Err(MetadataValidationError::SecretLikeKey(key.clone()));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataValidationError {
    SecretLikeKey(String),
}

impl fmt::Display for MetadataValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SecretLikeKey(key) => {
                write!(formatter, "metadata key `{key}` may contain a secret")
            }
        }
    }
}

impl std::error::Error for MetadataValidationError {}

impl fmt::Debug for SafeMetadata {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted = self
            .values
            .keys()
            .map(|key| (key, "<redacted>"))
            .collect::<BTreeMap<_, _>>();

        formatter
            .debug_tuple("SafeMetadata")
            .field(&redacted)
            .finish()
    }
}

fn to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn is_secret_like_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();

    normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("password")
        || normalized.contains("credential")
        || normalized.contains("api_key")
}

fn validate_sha256_field(field: &'static str, value: &str) -> Result<(), ManifestValidationError> {
    if is_valid_sha256(value) {
        return Ok(());
    }

    Err(ManifestValidationError::MalformedSha256 {
        field,
        value: value.to_string(),
    })
}

fn validate_optional_timestamp_field(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), ManifestValidationError> {
    if let Some(value) = value {
        validate_timestamp_field(field, value)?;
    }

    Ok(())
}

fn validate_timestamp_field(
    field: &'static str,
    value: &str,
) -> Result<(), ManifestValidationError> {
    if OffsetDateTime::parse(value, &Rfc3339).is_ok() {
        return Ok(());
    }

    Err(ManifestValidationError::MalformedTimestamp {
        field,
        value: value.to_string(),
    })
}

fn validate_non_empty_field(
    field: &'static str,
    value: &str,
) -> Result<(), ManifestValidationError> {
    if value.trim().is_empty() {
        return Err(ManifestValidationError::EmptyField(field));
    }

    Ok(())
}

fn is_valid_sha256(value: &str) -> bool {
    const PREFIX: &str = "sha256:";
    let Some(hex) = value.strip_prefix(PREFIX) else {
        return false;
    };

    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn stable_json_hash<T>(value: &T) -> String
where
    T: Serialize,
{
    let canonical_json = serde_json::to_string(value).expect("serializable contract value");
    let digest = Sha256::digest(canonical_json.as_bytes());

    format!("sha256:{}", to_lower_hex(&digest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_card_names_the_repo_and_upstream() {
        assert_eq!(PROJECT.name, "gear-depot");
        assert_eq!(PROJECT.upstream, "Starmetal");
        assert!(summary().contains(PROJECT.role));
    }

    #[test]
    fn artifact_manifest_roundtrips_with_revocation_metadata() {
        let manifest = ArtifactManifest {
            manifest_id: "manifest_01".to_string(),
            artifact: ArtifactRef {
                artifact_id: "art_01".to_string(),
                artifact_type: ArtifactType::SpecPackage,
                producer: "rumble-canvas".to_string(),
                version: "1.0.0".to_string(),
                hash: "sha256:abc123".to_string(),
                manifest_ref: "manifest_01".to_string(),
                state: ArtifactState::Revoked,
                created_at: "2026-06-30T00:00:00Z".to_string(),
            },
            package_type: PackageType::JsonBundle,
            checksums: vec![Checksum {
                algorithm: ChecksumAlgorithm::Sha256,
                value: "abc123".to_string(),
            }],
            provenance_id: "prov_01".to_string(),
            retention: RetentionMetadata {
                policy_ref: Some("retention-policy-01".to_string()),
                expires_at: None,
                revoked_at: Some("2026-07-01T00:00:00Z".to_string()),
                delete_after: None,
            },
            distribution: DistributionMetadata {
                channels: vec!["stable".to_string()],
                install_floor: None,
                published_at: Some("2026-06-30T00:01:00Z".to_string()),
            },
            metadata: SafeMetadata::from_pairs([(
                "build_host".to_string(),
                "runner-01".to_string(),
            )]),
        };

        let encoded = serde_json::to_string(&manifest).expect("manifest serializes");
        let decoded: ArtifactManifest =
            serde_json::from_str(&encoded).expect("manifest deserializes");

        assert_eq!(decoded, manifest);
    }

    #[test]
    fn artifact_ref_rejects_missing_required_hash() {
        let payload = r#"{
            "artifact_id": "art_01",
            "artifact_type": "spec_package",
            "producer": "rumble-canvas",
            "version": "1.0.0",
            "manifest_ref": "manifest_01",
            "state": "active",
            "created_at": "2026-06-30T00:00:00Z"
        }"#;

        let error = serde_json::from_str::<ArtifactRef>(payload).expect_err("hash is required");

        assert!(error.to_string().contains("missing field `hash`"));
    }

    #[test]
    fn artifact_manifest_requires_sha256_checksum() {
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
            checksums: Vec::new(),
            provenance_id: "prov_01".to_string(),
            retention: RetentionMetadata::default(),
            distribution: DistributionMetadata::default(),
            metadata: SafeMetadata::default(),
        };

        let error = manifest
            .validate()
            .expect_err("sha256 checksum is required");

        assert_eq!(error, ManifestValidationError::MissingSha256Checksum);
    }

    #[test]
    fn stable_manifest_hash_is_independent_from_metadata_insertion_order() {
        let mut left = valid_release_manifest();
        left.metadata = SafeMetadata::from_pairs([
            ("runner".to_string(), "a".to_string()),
            ("profile".to_string(), "release".to_string()),
        ]);

        let mut right = valid_release_manifest();
        right.metadata = SafeMetadata::from_pairs([
            ("profile".to_string(), "release".to_string()),
            ("runner".to_string(), "a".to_string()),
        ]);

        assert_eq!(left.stable_hash(), right.stable_hash());
    }

    #[test]
    fn metadata_debug_redacts_values() {
        let metadata =
            SafeMetadata::from_pairs([("token_source".to_string(), "secret-token".to_string())]);

        let debug = format!("{metadata:?}");

        assert!(debug.contains("token_source"));
        assert!(!debug.contains("secret-token"));
    }

    #[test]
    fn artifact_manifest_validation_rejects_secret_like_metadata_keys() {
        let mut manifest = valid_release_manifest();
        manifest.metadata = SafeMetadata::from_pairs([(
            "registry_password".to_string(),
            "password-should-not-be-stored".to_string(),
        )]);

        let error = manifest
            .validate()
            .expect_err("secret-like metadata keys are rejected");

        assert_eq!(
            error,
            ManifestValidationError::SecretLikeMetadataKey("registry_password".to_string())
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_malformed_artifact_hash() {
        let mut manifest = valid_release_manifest();
        manifest.artifact.hash = "sha256:not-hex".to_string();

        let error = manifest
            .validate()
            .expect_err("malformed artifact hash is rejected");

        assert_eq!(
            error,
            ManifestValidationError::MalformedSha256 {
                field: "artifact.hash",
                value: "sha256:not-hex".to_string()
            }
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_malformed_checksum() {
        let mut manifest = valid_release_manifest();
        manifest.checksums = vec![Checksum {
            algorithm: ChecksumAlgorithm::Sha256,
            value: "abc123".to_string(),
        }];

        let error = manifest
            .validate()
            .expect_err("malformed checksum is rejected");

        assert_eq!(
            error,
            ManifestValidationError::MalformedSha256 {
                field: "checksums.sha256",
                value: "abc123".to_string()
            }
        );
    }

    #[test]
    fn artifact_ref_validation_rejects_empty_producer() {
        let mut artifact = valid_artifact_ref();
        artifact.producer = " ".to_string();

        let error = artifact.validate().expect_err("empty producer is rejected");

        assert_eq!(error, ManifestValidationError::EmptyField("producer"));
    }

    #[test]
    fn artifact_manifest_validation_rejects_manifest_ref_mismatch() {
        let mut manifest = valid_release_manifest();
        manifest.artifact.manifest_ref = "other_manifest".to_string();

        let error = manifest
            .validate()
            .expect_err("artifact manifest_ref must point at owning manifest");

        assert_eq!(
            error,
            ManifestValidationError::ManifestRefMismatch {
                manifest_id: "manifest_01".to_string(),
                manifest_ref: "other_manifest".to_string()
            }
        );
    }

    #[test]
    fn artifact_ref_validation_rejects_malformed_created_at() {
        let mut artifact = valid_artifact_ref();
        artifact.created_at = "2026/06/30".to_string();

        let error = artifact
            .validate()
            .expect_err("non-RFC3339 created_at is rejected");

        assert_eq!(
            error,
            ManifestValidationError::MalformedTimestamp {
                field: "created_at",
                value: "2026/06/30".to_string()
            }
        );
    }

    #[test]
    fn artifact_manifest_validation_rejects_malformed_published_at() {
        let mut manifest = valid_release_manifest();
        manifest.distribution.published_at = Some("30-06-2026".to_string());

        let error = manifest
            .validate()
            .expect_err("non-RFC3339 published_at is rejected");

        assert_eq!(
            error,
            ManifestValidationError::MalformedTimestamp {
                field: "distribution.published_at",
                value: "30-06-2026".to_string()
            }
        );
    }

    fn valid_release_manifest() -> ArtifactManifest {
        ArtifactManifest {
            manifest_id: "manifest_01".to_string(),
            artifact: valid_artifact_ref(),
            package_type: PackageType::Binary,
            checksums: vec![Checksum {
                algorithm: ChecksumAlgorithm::Sha256,
                value: "b".repeat(64),
            }],
            provenance_id: "prov_01".to_string(),
            retention: RetentionMetadata::default(),
            distribution: DistributionMetadata::default(),
            metadata: SafeMetadata::default(),
        }
    }

    fn valid_artifact_ref() -> ArtifactRef {
        ArtifactRef {
            artifact_id: "art_01".to_string(),
            artifact_type: ArtifactType::ReleaseAsset,
            producer: "gear-cable".to_string(),
            version: "1.0.0".to_string(),
            hash: format!("sha256:{}", "a".repeat(64)),
            manifest_ref: "manifest_01".to_string(),
            state: ArtifactState::Active,
            created_at: "2026-06-30T00:00:00Z".to_string(),
        }
    }
}
