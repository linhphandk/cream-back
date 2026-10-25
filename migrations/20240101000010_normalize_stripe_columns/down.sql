ALTER TABLE organizations
    ADD COLUMN stripe_onboarding_complete BOOLEAN DEFAULT FALSE,
    ADD COLUMN stripe_account_status TEXT;

-- Best-effort restore: details_submitted is not recoverable from the remaining columns.
UPDATE organizations
SET stripe_onboarding_complete = (stripe_charges_enabled AND stripe_payouts_enabled),
    stripe_account_status = CASE
        WHEN stripe_charges_enabled AND stripe_payouts_enabled THEN 'active'
        ELSE 'pending'
    END;

ALTER TABLE organizations DROP COLUMN stripe_details_submitted;
