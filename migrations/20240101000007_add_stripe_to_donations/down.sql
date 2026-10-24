ALTER TABLE donations
    DROP COLUMN IF EXISTS stripe_transfer_id,
    DROP COLUMN IF EXISTS platform_fee_cents;

DROP INDEX IF EXISTS idx_donations_stripe_payment_intent_id;
DROP INDEX IF EXISTS idx_donations_status;
