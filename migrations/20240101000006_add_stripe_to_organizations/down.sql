ALTER TABLE organizations 
    DROP COLUMN IF EXISTS stripe_onboarding_complete,
    DROP COLUMN IF EXISTS stripe_charges_enabled,
    DROP COLUMN IF EXISTS stripe_payouts_enabled,
    DROP COLUMN IF EXISTS stripe_account_status;

DROP INDEX IF EXISTS idx_organizations_stripe_account_id;
