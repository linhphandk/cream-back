-- Create organizations table
CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    subdomain TEXT UNIQUE NOT NULL,
    fee_bps INTEGER NOT NULL DEFAULT 500,  -- platform fee in basis points (5% = 500)
    stripe_account_id TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Comment: Organizations (nonprofits, merchants, etc.) that can create campaigns and accept donations
