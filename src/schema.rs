diesel::table! {
    organizations (id) {
        id -> Uuid,
        name -> Text,
        subdomain -> Text,
        fee_bps -> Int4,
        stripe_account_id -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
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
    }
}

diesel::joinable!(campaigns -> organizations (organization_id));
diesel::joinable!(donations -> campaigns (campaign_id));

diesel::allow_tables_to_appear_in_same_query!(organizations, campaigns, donations);
