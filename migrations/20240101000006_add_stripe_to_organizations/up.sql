ALTER TABLE organizations 
    ADD COLUMN IF NOT EXISTS stripe_onboarding_complete BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS stripe_charges_enabled BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS stripe_payouts_enabled BOOLEAN DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS stripe_account_status TEXT;

CREATE INDEX IF NOT EXISTS idx_organizations_stripe_account_id ON organizations(stripe_account_id);
