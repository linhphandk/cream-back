diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Text,
        subdomain -> Text,
        fee_bps -> Int4,
        stripe_account_id -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        stripe_charges_enabled -> Nullable<Bool>,
        stripe_payouts_enabled -> Nullable<Bool>,
        stripe_details_submitted -> Nullable<Bool>,
    }
}

diesel::table! {
    campaigns (id) {
        id -> Uuid,
        organization_id -> Uuid,
        slug -> Text,
        title -> Text,
        description -> Nullable<Text>,
        currency -> Bpchar,
        active -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    donations (id) {
        id -> Uuid,
        campaign_id -> Uuid,
        amount_cents -> Int8,
        currency -> Bpchar,
        status -> Text,
        stripe_payment_intent_id -> Nullable<Text>,
        donor_email -> Nullable<Text>,
        donor_name -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        created_at -> Nullable<Timestamptz>,
        stripe_transfer_id -> Nullable<Text>,
        platform_fee_cents -> Nullable<Int8>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        name -> Text,
        password_hash -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        refresh_token -> Varchar,
        expires_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    organization_members (user_id) {
        org_id -> Uuid,
        user_id -> Uuid,
        role -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    stripe_webhook_events (id) {
        id -> Uuid,
        stripe_event_id -> Text,
        event_type -> Text,
        processed_at -> Timestamptz,
        payload -> Jsonb,
    }
}

diesel::joinable!(campaigns -> organizations (organization_id));
diesel::joinable!(donations -> campaigns (campaign_id));
diesel::joinable!(organization_members -> organizations (org_id));

diesel::allow_tables_to_appear_in_same_query!(organizations, campaigns, donations, users, sessions, stripe_webhook_events, organization_members);
