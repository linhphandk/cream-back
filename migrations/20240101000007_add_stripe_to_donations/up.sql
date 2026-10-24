ALTER TABLE donations
    ADD COLUMN IF NOT EXISTS stripe_transfer_id TEXT,
    ADD COLUMN IF NOT EXISTS platform_fee_cents BIGINT;

CREATE INDEX IF NOT EXISTS idx_donations_stripe_payment_intent_id ON donations(stripe_payment_intent_id);
CREATE INDEX IF NOT EXISTS idx_donations_status ON donations(status);
