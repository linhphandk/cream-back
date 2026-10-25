-- Cache only Stripe's raw values; derive "complete"/"status" in code.
ALTER TABLE organizations ADD COLUMN stripe_details_submitted BOOLEAN;

UPDATE organizations
SET stripe_details_submitted = (stripe_account_status IN ('active', 'pending'))
WHERE stripe_account_status IS NOT NULL;

ALTER TABLE organizations
    DROP COLUMN stripe_onboarding_complete,
    DROP COLUMN stripe_account_status;
