-- Org ↔ user membership. PK on user_id = one org per user (frontend assumes 0 or 1);
-- drop that constraint when multi-org users are needed.
CREATE TABLE organization_members (
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'owner',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id)
);

-- One owner per org; other roles (treasurer, staff, ...) can be many.
CREATE UNIQUE INDEX organization_members_one_owner
    ON organization_members (org_id) WHERE role = 'owner';

-- shortcut: only 'owner' exists until a real role model lands; loosen with one ALTER then
ALTER TABLE organization_members
    ADD CONSTRAINT organization_members_role_check CHECK (role = 'owner');
