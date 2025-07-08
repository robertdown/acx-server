use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use crate::models::category::CategoryType; // Import the enum

// DTO for creating a new Category
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateCategoryDto {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub r#type: CategoryType, // Use the enum
    pub parent_category_id: Option<Uuid>, // Nullable for hierarchical categories
    // tenant_id and created_by will be derived from context
}

// DTO for updating an existing Category
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateCategoryDto {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<CategoryType>, // Use the enum
    pub parent_category_id: Option<Uuid>, // Nullable, can be updated
    pub is_active: Option<bool>,
    // updated_by will be derived from context
}