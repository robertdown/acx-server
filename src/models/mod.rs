// Core Models (mapping directly to DB tables)
pub mod account;
pub mod account_type;
pub mod category; // New
pub mod currency;
pub mod exchange_rate; // New
pub mod journal_entry;
pub mod tag; // New
pub mod tenant;
pub mod transaction;
pub mod user;

// Phase 2 Models (will add later in a subsequent response)
// pub mod budget;
// pub mod budget_line_item;
// pub mod recurring_transaction;
// pub mod custom_report;
// pub mod dashboard;
// pub mod dashboard_widget;
// pub mod role;
// pub mod permission;
// pub mod role_permission;
// pub mod user_tenant_role;
// pub mod ext_provider;
// pub mod ext_conn;
// pub mod external_account;
// pub mod external_transactions_staging;
// pub mod coa_template;
// pub mod coa_template_account;

// Data Transfer Objects (DTOs)
pub mod dto;

// --- Re-exports for easier access ---

// Re-export core model structs
pub use account::Account;
pub use account_type::{AccountNormalBalance, AccountType}; // Include enum
pub use category::{Category, CategoryType}; // Include enum
pub use currency::Currency;
pub use exchange_rate::ExchangeRate;
pub use journal_entry::{JournalEntry, JournalEntryType};
pub use tag::Tag;
pub use tenant::Tenant;
pub use transaction::{Transaction, TransactionType}; // Include enum
pub use user::User; // Include enum

// Re-export Phase 2 model structs (will uncomment as they are generated)
// pub use budget::{Budget};
// pub use budget_line_item::{BudgetLineItem};
// pub use recurring_transaction::{RecurringTransaction};
// pub use custom_report::{CustomReport};
// pub use dashboard::{Dashboard};
// pub use dashboard_widget::{DashboardWidget};
// pub use role::{Role};
// pub use permission::{Permission};
// pub use role_permission::{RolePermission};
// pub use user_tenant_role::{UserTenantRole};
// pub use ext_provider::{ExtProvider};
// pub use ext_conn::{ExtConn};
// pub use external_account::{ExternalAccount};
// pub use external_transactions_staging::{ExternalTransactionsStaging};
// pub use coa_template::{CoaTemplate};
// pub use coa_template_account::{CoaTemplateAccount};

// Re-export DTO structs from the dto submodule
pub use dto::account_dto::{CreateAccountDto, UpdateAccountDto};
pub use dto::account_type_dto::{CreateAccountTypeDto, UpdateAccountTypeDto};
pub use dto::category_dto::{CreateCategoryDto, UpdateCategoryDto};
pub use dto::currency_dto::{CreateCurrencyDto, UpdateCurrencyDto};
pub use dto::exchange_rate_dto::{CreateExchangeRateDto, UpdateExchangeRateDto};
pub use dto::journal_entry_dto::{CreateJournalEntryDto, UpdateJournalEntryDto};
pub use dto::tag_dto::{CreateTagDto, UpdateTagDto};
pub use dto::tenant_dto::{CreateTenantDto, UpdateTenantDto};
pub use dto::transaction_dto::{CreateTransactionDto, UpdateTransactionDto};
pub use dto::user_dto::{CreateUserDto, UpdateUserDto};

// Re-export Phase 2 DTOs (will uncomment as they are generated)
// pub use dto::budget_dto::{CreateBudgetDto, UpdateBudgetDto};
// pub use dto::budget_line_item_dto::{CreateBudgetLineItemDto, UpdateBudgetLineItemDto};
// pub use dto::recurring_transaction_dto::{CreateRecurringTransactionDto, UpdateRecurringTransactionDto};
// pub use dto::custom_report_dto::{CreateCustomReportDto, UpdateCustomReportDto};
// pub use dto::dashboard_dto::{CreateDashboardDto, UpdateDashboardDto};
// pub use dto::dashboard_widget_dto::{CreateDashboardWidgetDto, UpdateDashboardWidgetDto};
// pub use dto::role_dto::{CreateRoleDto, UpdateRoleDto};
// pub use dto::permission_dto::{CreatePermissionDto, UpdatePermissionDto};
// pub use dto::role_permission_dto::{CreateRolePermissionDto};
// pub use dto::user_tenant_role_dto::{CreateUserTenantRoleDto};
// pub use dto::ext_provider_dto::{CreateExtProviderDto, UpdateExtProviderDto};
// pub use dto::ext_conn_dto::{CreateExtConnDto, UpdateExtConnDto};
// pub use dto::external_account_dto::{CreateExternalAccountDto, UpdateExternalAccountDto};
// pub use dto::external_transactions_staging_dto::{CreateExternalTransactionsStagingDto, UpdateExternalTransactionsStagingDto};
// pub use dto::coa_template_dto::{CreateCoaTemplateDto, UpdateCoaTemplateDto};
// pub use dto::coa_template_account_dto::{CreateCoaTemplateAccountDto, UpdateCoaTemplateAccountDto};
// Placeholder for authentication DTOs
pub use dto::auth_dto::{LoginRequest, RegisterRequest};
