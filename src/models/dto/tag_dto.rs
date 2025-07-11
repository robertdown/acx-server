use serde::{Deserialize, Serialize};
use validator::Validate;

// DTO for creating a new Tag
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTagDto {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    // tenant_id and created_by will be derived from context
}

// DTO for updating an existing Tag
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateTagDto {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    // updated_by will be derived from context
}
