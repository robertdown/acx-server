-- Enable UUID extension if not already enabled (for gen_random_uuid())
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


-- #############################################################################
-- PHASE 1: CORE ACCOUNTING & FINANCIALS
-- #############################################################################

-- 1. Users Table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    auth_provider_id VARCHAR(255) UNIQUE NOT NULL, -- e.g., email, Google ID, Auth0 ID
    auth_provider_type VARCHAR(50) NOT NULL,        -- e.g., 'EMAIL_PASSWORD', 'GOOGLE', 'AUTH0'
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),                     -- Nullable if using OAuth/SSO
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Tenants Table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    industry VARCHAR(100),
    base_currency_code CHAR(3) NOT NULL, -- ISO 4217 code, e.g., 'USD', 'EUR'
    fiscal_year_end_month INTEGER NOT NULL CHECK (fiscal_year_end_month >= 1 AND fiscal_year_end_month <= 12),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 3. Currencies Table
-- This is a system-level table, potentially pre-populated
CREATE TABLE currencies (
    code CHAR(3) PRIMARY KEY, -- ISO 4217 code, e.g., 'USD', 'EUR'
    name VARCHAR(100) NOT NULL UNIQUE,
    symbol VARCHAR(10),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id), -- System user
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)  -- System user
);

-- 4. Exchange Rates Table
CREATE TABLE exchange_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id), -- Null for system-wide rates if applicable
    base_currency_code CHAR(3) NOT NULL REFERENCES currencies(code),
    target_currency_code CHAR(3) NOT NULL REFERENCES currencies(code),
    rate NUMERIC(18, 6) NOT NULL CHECK (rate > 0),
    rate_date DATE NOT NULL,
    source VARCHAR(100), -- e.g., 'API', 'Manual'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (tenant_id, base_currency_code, target_currency_code, rate_date) -- Ensure unique rate per day per tenant for a pair
);

-- 5. Account Types Table
-- This is a system-level table, pre-populated with standard accounting types
CREATE TABLE account_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL, -- e.g., 'Asset', 'Liability', 'Equity', 'Revenue', 'Expense'
    normal_balance VARCHAR(10) NOT NULL CHECK (normal_balance IN ('DEBIT', 'CREDIT')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id), -- System user
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)  -- System user
);

-- 6. Accounts Table (Chart of Accounts)
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_type_id UUID NOT NULL REFERENCES account_types(id),
    name VARCHAR(255) NOT NULL,
    account_code VARCHAR(50) UNIQUE, -- Optional account code, unique within a tenant
    description TEXT,
    currency_code CHAR(3) NOT NULL REFERENCES currencies(code), -- Currency for this specific account
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (tenant_id, name), -- Account names must be unique within a tenant
    UNIQUE (tenant_id, account_code) -- Account codes must be unique within a tenant
);

-- 7. Categories Table
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL CHECK (type IN ('INCOME', 'EXPENSE', 'TRANSFER', 'INVESTMENT', 'OTHER')),
    parent_category_id UUID REFERENCES categories(id), -- For hierarchical categories
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (tenant_id, name) -- Category names must be unique within a tenant
);

-- 8. Tags Table
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (tenant_id, name) -- Tag names must be unique within a tenant
);

-- 9. Transactions Table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    transaction_date DATE NOT NULL,
    description TEXT NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('INCOME', 'EXPENSE', 'TRANSFER', 'JOURNAL_ENTRY', 'OPENING_BALANCE', 'ADJUSTMENT')),
    category_id UUID REFERENCES categories(id),
    tags_json JSONB, -- Stores an array of tag UUIDs: ["uuid1", "uuid2"]
    amount NUMERIC(18, 2) NOT NULL, -- Total amount of the transaction
    currency_code CHAR(3) NOT NULL REFERENCES currencies(code),
    is_reconciled BOOLEAN NOT NULL DEFAULT FALSE,
    reconciliation_date DATE,
    notes TEXT,
    source_document_url TEXT, -- URL to uploaded receipt/invoice
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 10. Journal Entries Table
CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL REFERENCES transactions(id),
    account_id UUID NOT NULL REFERENCES accounts(id),
    entry_type VARCHAR(10) NOT NULL CHECK (entry_type IN ('DEBIT', 'CREDIT')),
    amount NUMERIC(18, 2) NOT NULL CHECK (amount >= 0), -- Always positive, type defines debit/credit
    currency_code CHAR(3) NOT NULL REFERENCES currencies(code),
    exchange_rate NUMERIC(18, 6), -- Rate used if transaction currency differs from account currency
    converted_amount NUMERIC(18, 2), -- Amount in account's currency
    memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (transaction_id, account_id, entry_type) -- Ensures unique debit/credit entry per account per transaction
);

-- #############################################################################
-- PHASE 2: ADVANCED FEATURES & ECOSYSTEM INTEGRATION
-- #############################################################################

-- 11. Budgets Table
CREATE TABLE budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    budget_type VARCHAR(50) NOT NULL CHECK (budget_type IN ('MONTHLY', 'ANNUAL', 'CUSTOM')),
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (tenant_id, name) -- Budget names unique per tenant
);

-- 12. Budget Line Items Table
CREATE TABLE budget_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id UUID NOT NULL REFERENCES budgets(id),
    category_id UUID REFERENCES categories(id), -- Target category for this budget line
    amount NUMERIC(18, 2) NOT NULL CHECK (amount >= 0),
    frequency_type VARCHAR(50) NOT NULL CHECK (frequency_type IN ('MONTHLY', 'ANNUALLY', 'ONCE', 'QUARTERLY')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (budget_id, category_id) -- A category can only have one budget line item per budget
);

-- 13. Recurring Transactions Table
CREATE TABLE recurring_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    description TEXT NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('INCOME', 'EXPENSE', 'TRANSFER')),
    category_id UUID REFERENCES categories(id),
    account_id UUID NOT NULL REFERENCES accounts(id), -- The primary account involved
    amount NUMERIC(18, 2) NOT NULL,
    currency_code CHAR(3) NOT NULL REFERENCES currencies(code),
    frequency_value INTEGER NOT NULL CHECK (frequency_value > 0), -- e.g., 1 for 'monthly'
    frequency_unit VARCHAR(50) NOT NULL CHECK (frequency_unit IN ('DAY', 'WEEK', 'MONTH', 'YEAR')),
    start_date DATE NOT NULL,
    end_date DATE, -- Nullable for indefinite recurring transactions
    last_generated_date DATE,
    next_due_date DATE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 14. Custom Reports Table
CREATE TABLE custom_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id), -- User who created the report
    name VARCHAR(255) NOT NULL,
    description TEXT,
    report_type VARCHAR(50) NOT NULL CHECK (report_type IN ('TRANSACTION_LIST', 'SUMMARY_BY_CATEGORY', 'ACCOUNT_BALANCE_SUMMARY', 'INCOME_EXPENSE_STATEMENT')),
    configuration JSONB NOT NULL, -- Flexible JSON for report parameters (date ranges, filters, grouping)
    is_public BOOLEAN NOT NULL DEFAULT FALSE, -- Visible to other users in tenant
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (tenant_id, name) -- Custom report names unique per tenant
);

-- 15. Dashboards Table
CREATE TABLE dashboards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id), -- User who owns this dashboard
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT FALSE, -- The default dashboard for the user/tenant
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (user_id, name) -- Dashboard names unique per user
);

-- 16. Dashboard Widgets Table (Revised)
CREATE TABLE dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES dashboards(id),
    widget_type VARCHAR(50) NOT NULL CHECK (widget_type IN ('cash_balance_summary', 'spending_by_category', 'income_vs_expense_summary', 'account_balance_list', 'custom_report_link')),
    title VARCHAR(255) NOT NULL,
    order_index INTEGER NOT NULL,
    parameters JSONB, -- Data parameters for the widget
    properties JSONB, -- Flexible JSON for layout and presentation properties
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 17. Roles Table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    is_system_role BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 18. Permissions Table
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL, -- e.g., 'tx.create', 'account.view_any'
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 19. Role Permissions Table (Junction Table)
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id),
    permission_id UUID NOT NULL REFERENCES permissions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    PRIMARY KEY (role_id, permission_id)
);

-- 20. User Tenant Roles Table (Junction Table)
CREATE TABLE user_tenant_roles (
    user_id UUID NOT NULL REFERENCES users(id),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    role_id UUID NOT NULL REFERENCES roles(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- Added for consistency
    updated_by UUID NOT NULL REFERENCES users(id), -- Added for consistency
    PRIMARY KEY (user_id, tenant_id, role_id)
);

-- 21. External Providers Table (Lookup Table)
CREATE TABLE ext_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE, -- e.g., 'Plaid', 'Stripe'
    code VARCHAR(50) NOT NULL UNIQUE, -- e.g., 'PLAID', 'STRIPE'
    type VARCHAR(50) NOT NULL CHECK (type IN ('BANKING_AGGREGATOR', 'PAYMENT_GATEWAY', 'E_COMMERCE', 'PAYROLL', 'OTHER')),
    description TEXT,
    logo_url TEXT,
    api_base_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 22. External Connections Table
CREATE TABLE ext_conns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    provider_id UUID NOT NULL REFERENCES ext_providers(id), -- NEW FK to ext_providers
    provider_access_token TEXT NOT NULL, -- Encrypted token
    provider_item_id VARCHAR(255),
    status VARCHAR(50) NOT NULL CHECK (status IN ('CONNECTED', 'DISCONNECTED', 'ERROR', 'PENDING_REAUTH', 'DISABLED')),
    last_sync_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 23. External Accounts Table
CREATE TABLE external_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ext_conn_id UUID NOT NULL REFERENCES ext_conns(id), -- Changed from bank_connection_id
    account_id UUID REFERENCES accounts(id),
    provider_account_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    mask VARCHAR(10),
    type VARCHAR(50),
    subtype VARCHAR(50),
    currency_code CHAR(3) NOT NULL REFERENCES currencies(code),
    current_balance NUMERIC(18, 2),
    available_balance NUMERIC(18, 2),
    last_sync_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (ext_conn_id, provider_account_id)
);

-- 24. External Transactions Staging Table
CREATE TABLE external_transactions_staging (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_account_id UUID NOT NULL REFERENCES external_accounts(id),
    provider_transaction_id VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    amount NUMERIC(18, 2) NOT NULL,
    transaction_date DATE NOT NULL,
    posted_date DATE,
    status VARCHAR(50) NOT NULL CHECK (status IN ('PENDING_REVIEW', 'CONVERTED', 'MATCHED_MANUALLY', 'IGNORED', 'DUPLICATE', 'ERROR')),
    tx_id UUID REFERENCES transactions(id),
    raw_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (external_account_id, provider_transaction_id)
);

-- 25. CoA Templates Table
CREATE TABLE coa_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    industry VARCHAR(100),
    description TEXT,
    is_system_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);

-- 26. CoA Template Accounts Table
CREATE TABLE coa_template_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    coa_template_id UUID NOT NULL REFERENCES coa_templates(id),
    account_type_id UUID NOT NULL REFERENCES account_types(id),
    name VARCHAR(255) NOT NULL,
    account_code VARCHAR(50),
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_default_currency_account BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id)
);


-- #############################################################################
-- INDEXES FOR PERFORMANCE
-- #############################################################################

-- Users
CREATE INDEX idx_users_email ON users (email);

-- Tenants
CREATE INDEX idx_tenants_created_by ON tenants (created_by);

-- Exchange Rates
CREATE INDEX idx_exchange_rates_base_target_date ON exchange_rates (base_currency_code, target_currency_code, rate_date);
CREATE INDEX idx_exchange_rates_tenant_id ON exchange_rates (tenant_id);

-- Accounts
CREATE INDEX idx_accounts_tenant_id ON accounts (tenant_id);
CREATE INDEX idx_accounts_account_type_id ON accounts (account_type_id);
CREATE INDEX idx_accounts_currency_code ON accounts (currency_code);

-- Categories
CREATE INDEX idx_categories_tenant_id ON categories (tenant_id);
CREATE INDEX idx_categories_parent_id ON categories (parent_category_id);

-- Tags
CREATE INDEX idx_tags_tenant_id ON tags (tenant_id);

-- Transactions
CREATE INDEX idx_transactions_tenant_id ON transactions (tenant_id);
CREATE INDEX idx_transactions_date ON transactions (transaction_date);
CREATE INDEX idx_transactions_category_id ON transactions (category_id);
CREATE INDEX idx_transactions_type ON transactions (type);

-- Journal Entries
CREATE INDEX idx_journal_entries_transaction_id ON journal_entries (transaction_id);
CREATE INDEX idx_journal_entries_account_id ON journal_entries (account_id);

-- Budgets
CREATE INDEX idx_budgets_tenant_id ON budgets (tenant_id);

-- Budget Line Items
CREATE INDEX idx_budget_line_items_budget_id ON budget_line_items (budget_id);
CREATE INDEX idx_budget_line_items_category_id ON budget_line_items (category_id);

-- Recurring Transactions
CREATE INDEX idx_recurring_transactions_tenant_id ON recurring_transactions (tenant_id);
CREATE INDEX idx_recurring_transactions_next_due_date ON recurring_transactions (next_due_date);

-- Custom Reports
CREATE INDEX idx_custom_reports_tenant_id ON custom_reports (tenant_id);
CREATE INDEX idx_custom_reports_user_id ON custom_reports (user_id);

-- Dashboards
CREATE INDEX idx_dashboards_tenant_id ON dashboards (tenant_id);
CREATE INDEX idx_dashboards_user_id ON dashboards (user_id);

-- Dashboard Widgets
CREATE INDEX idx_dashboard_widgets_dashboard_id ON dashboard_widgets (dashboard_id);

-- Roles
-- No specific indexes beyond PK needed for now for this small lookup table

-- Permissions
-- No specific indexes beyond PK needed for now for this small lookup table

-- Role Permissions
CREATE INDEX idx_role_permissions_permission_id ON role_permissions (permission_id);

-- User Tenant Roles
CREATE INDEX idx_user_tenant_roles_tenant_user ON user_tenant_roles (tenant_id, user_id);
CREATE INDEX idx_user_tenant_roles_role_id ON user_tenant_roles (role_id);

-- Ext Providers
CREATE INDEX idx_ext_providers_code ON ext_providers (code);

-- Ext Connections
CREATE INDEX idx_ext_conns_tenant_id ON ext_conns (tenant_id);
CREATE INDEX idx_ext_conns_user_id ON ext_conns (user_id);
CREATE INDEX idx_ext_conns_provider_id ON ext_conns (provider_id);
CREATE INDEX idx_ext_conns_provider_item_id ON ext_conns (provider_item_id);

-- External Accounts
CREATE INDEX idx_external_accounts_ext_conn_id ON external_accounts (ext_conn_id);
CREATE INDEX idx_external_accounts_account_id ON external_accounts (account_id);

-- External Transactions Staging
CREATE INDEX idx_external_transactions_staging_ext_account_id ON external_transactions_staging (external_account_id);
CREATE INDEX idx_external_transactions_staging_status ON external_transactions_staging (status);
CREATE INDEX idx_external_transactions_staging_tx_id ON external_transactions_staging (tx_id);
CREATE INDEX idx_external_transactions_staging_date ON external_transactions_staging (transaction_date);

-- CoA Templates
-- No specific indexes beyond PK needed for now

-- CoA Template Accounts
CREATE INDEX idx_coa_template_accounts_template_id ON coa_template_accounts (coa_template_id);
CREATE INDEX idx_coa_template_accounts_account_type_id ON coa_template_accounts (account_type_id);