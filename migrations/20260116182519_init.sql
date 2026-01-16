-- Vaults table
CREATE TABLE vaults (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner VARCHAR(44) NOT NULL UNIQUE,
    vault_address VARCHAR(44) NOT NULL UNIQUE,
    token_mint VARCHAR(44) NOT NULL,
    total_balance BIGINT NOT NULL DEFAULT 0,
    locked_balance BIGINT NOT NULL DEFAULT 0,
    available_balance BIGINT NOT NULL DEFAULT 0,
    total_deposited BIGINT NOT NULL DEFAULT 0,
    total_withdrawn BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Vault events table
CREATE TABLE vault_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vault_owner VARCHAR(44) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (vault_owner) REFERENCES vaults(owner) ON DELETE CASCADE
);

-- Transaction logs table
CREATE TABLE transaction_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signature VARCHAR(88) NOT NULL UNIQUE,
    vault_owner VARCHAR(44),
    transaction_type VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    slot BIGINT,
    block_time BIGINT,
    fee BIGINT,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Authorized programs table
CREATE TABLE authorized_programs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    program_pubkey VARCHAR(44) NOT NULL UNIQUE,
    is_active BOOLEAN DEFAULT TRUE,
    added_by VARCHAR(44),
    added_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    removed_at TIMESTAMP WITH TIME ZONE,
    removed_by VARCHAR(44)
);

-- Audit trail table
CREATE TABLE audit_trail (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(100) NOT NULL,
    actor VARCHAR(44) NOT NULL,
    target VARCHAR(44),
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_vaults_owner ON vaults(owner);
CREATE INDEX idx_vault_events_vault_owner ON vault_events(vault_owner);
CREATE INDEX idx_vault_events_created_at ON vault_events(created_at);
CREATE INDEX idx_transaction_logs_signature ON transaction_logs(signature);
CREATE INDEX idx_transaction_logs_vault_owner ON transaction_logs(vault_owner);
CREATE INDEX idx_authorized_programs_active ON authorized_programs(is_active);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply triggers
CREATE TRIGGER update_vaults_updated_at
    BEFORE UPDATE ON vaults
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_transaction_logs_updated_at
    BEFORE UPDATE ON transaction_logs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();