use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectResource {
    // Value: "storage#object"
    pub kind: String,
    pub id: String,
    pub self_link: String,
    pub name: String,
    pub bucket: String,
    pub generation: String,
    pub metageneration: String,
    pub content_type: String,
    pub time_created: String,
    pub updated: String,
    pub time_deleted: Option<String>,
    pub temporary_hold: Option<bool>,
    pub event_based_hold: Option<bool>,
    pub retention_expiration_time: Option<String>,
    pub storage_class: String,
    pub time_storage_class_updated: Option<String>,
    pub size: String,
    pub md5_hash: String,
    pub media_link: String,
    pub content_encoding: Option<String>,
    pub content_disposition: Option<String>,
    pub content_language: Option<String>,
    pub cache_control: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub acl: Option<Vec<ObjectAclResource>>,
    pub owner: Option<ObjectOwner>,
    pub crc32c: String,
    pub component_count: Option<String>,
    pub etag: String,
    pub customer_encryption: Option<ObjectCustomerEncryption>,
    pub kms_key_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectOwner {
    pub entity: String,
    pub entity_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectCustomerEncryption {
    pub encryption_algorithm: String,
    pub key_sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAclResource {
    /// Value: "storage#objectAccessControl"
    pub kind: String,
    pub entity: String,
    pub role: String,
    pub email: String,
    pub entity_id: String,
    pub domain: String,
    pub project_team: ObjectAclProjectTeam,
    pub etag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAclProjectTeam {
    pub project_number: String,
    pub team: String,
}
