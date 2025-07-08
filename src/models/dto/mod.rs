// DTOs for Phase 1 Core Accounting & Financials
pub mod user_dto;
pub mod tenant_dto;
pub mod currency_dto;
pub mod exchange_rate_dto;    // New
pub mod account_type_dto;     // New
pub mod account_dto;          // New
pub mod category_dto;         // New
pub mod tag_dto;              // New
pub mod transaction_dto;
pub mod journal_entry_dto;

// DTOs for Phase 2 Advanced Features & Ecosystem Integration (will add later)
pub mod budget_dto;
pub mod budget_line_item_dto;
pub mod recurring_transaction_dto;
pub mod custom_report_dto;
pub mod dashboard_dto;
pub mod dashboard_widget_dto;
pub mod role_dto;
pub mod permission_dto;
pub mod role_permission_dto;
pub mod user_tenant_role_dto;
pub mod ext_provider_dto;
pub mod ext_conn_dto;
pub mod external_account_dto;
pub mod external_transactions_staging_dto;
pub mod coa_template_dto;
pub mod coa_template_account_dto;

// Placeholder for Authentication DTOs
pub mod auth_dto;