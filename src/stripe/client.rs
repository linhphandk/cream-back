use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, HeaderName}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use uuid::Uuid;
use crate::shared::errors::{AppError, AppResult};

const STRIPE_API_BASE_V1: &str = "https://api.stripe.com/v1";
const STRIPE_API_BASE_V2: &str = "https://api.stripe.com/v2/core";

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct StripeClient {
    http: Client,
    secret_key: String,
    webhook_secret: String,
    api_version: String,
}

impl StripeClient {
    pub fn new(secret_key: String, webhook_secret: String, api_version: String) -> AppResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        let http = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| AppError::Internal)?;
        
        Ok(Self {
            http,
            secret_key,
            webhook_secret,
            api_version,
        })
    }

    fn auth_header_v1(&self) -> HeaderValue {
        let auth = format!("Bearer {}", self.secret_key);
        HeaderValue::from_str(&auth).unwrap()
    }

    fn auth_header_v2(&self) -> HeaderValue {
        let auth = format!("Bearer {}", self.secret_key);
        HeaderValue::from_str(&auth).unwrap()
    }

    async fn post_v1_form<T: for<'de> Deserialize<'de>>(&self, path: &str, form: &[(&str, &str)]) -> AppResult<T> {
        let url = format!("{}{}", STRIPE_API_BASE_V1, path);
        let response = self.http
            .post(&url)
            .header(AUTHORIZATION, self.auth_header_v1())
            .form(form)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Stripe v1 request failed: {:?}", e);
                AppError::Internal
            })?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            tracing::error!("Stripe v1 error: {}", err_text);
            return Err(AppError::Internal);
        }

        response.json().await.map_err(|e| {
            tracing::error!("Stripe v1 JSON parse failed: {:?}", e);
            AppError::Internal
        })
    }

    async fn post_v2<T: for<'de> Deserialize<'de>>(&self, path: &str, body: &Value) -> AppResult<T> {
        let url = format!("{}{}", STRIPE_API_BASE_V2, path);
        let response = self.http
            .post(&url)
            .header(AUTHORIZATION, self.auth_header_v2())
            .header(HeaderName::from_static("stripe-version"), HeaderValue::from_str(&self.api_version).unwrap())
            .json(body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Stripe v2 request failed: {:?}", e);
                AppError::Internal
            })?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            tracing::error!("Stripe v2 error: {}", err_text);
            return Err(AppError::Internal);
        }

        response.json().await.map_err(|e| {
            tracing::error!("Stripe v2 JSON parse failed: {:?}", e);
            AppError::Internal
        })
    }

    async fn get_v2<T: for<'de> Deserialize<'de>>(&self, path: &str, query: &[(&str, &str)]) -> AppResult<T> {
        let url = format!("{}{}", STRIPE_API_BASE_V2, path);
        let response = self.http
            .get(&url)
            .header(AUTHORIZATION, self.auth_header_v2())
            .header(HeaderName::from_static("stripe-version"), HeaderValue::from_str(&self.api_version).unwrap())
            .query(query)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Stripe v2 GET request failed: {:?}", e);
                AppError::Internal
            })?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            tracing::error!("Stripe v2 GET error: {}", err_text);
            return Err(AppError::Internal);
        }

        response.json().await.map_err(|e| {
            tracing::error!("Stripe v2 GET JSON parse failed: {:?}", e);
            AppError::Internal
        })
    }

    // ============ v2 Account APIs ============

    pub async fn create_account_v2(&self, req: &crate::stripe::models::CreateStripeAccountRequest, _frontend_base: &str) -> AppResult<CreateAccountResponse> {
        let body = serde_json::json!({
            "contact_email": req.org_admin_email,
            "display_name": req.org_name,
            "dashboard": "express",
            "identity": {
                "country": req.country,
                "entity_type": req.entity_type.as_str(),
                "business_details": {
                    "registered_name": req.org_name
                }
            },
            "configuration": {
                "merchant": {
                    "capabilities": {
                        "card_payments": { "requested": true }
                    }
                }
            },
            "defaults": {
                "currency": "usd",
                "responsibilities": {
                    "fees_collector": "stripe",
                    "losses_collector": "stripe"
                },
                "locales": ["en-US"]
            },
            "metadata": { "cream_org_id": req.org_id.to_string() }
        });

        let resp: CreateAccountResponse = self.post_v2("/accounts", &body).await?;
        Ok(resp)
    }

    pub async fn create_account_session(&self, account_id: &str, frontend_base: &str, org_id: Uuid) -> AppResult<CreateAccountSessionResponse> {
        let return_url = format!("{}/onboarding/stripe/return?org_id={}", frontend_base, org_id);
        let refresh_url = format!("{}/onboarding/stripe/refresh?org_id={}", frontend_base, org_id);

        // v2 hosted onboarding is an Account Link; return/refresh urls live here, not in /account_sessions
        let body = serde_json::json!({
            "account": account_id,
            "use_case": {
                "type": "account_onboarding",
                "account_onboarding": {
                    "return_url": return_url,
                    "refresh_url": refresh_url
                }
            }
        });

        let resp: CreateAccountSessionResponse = self.post_v2("/account_links", &body).await?;
        Ok(resp)
    }

    pub async fn retrieve_account(&self, account_id: &str) -> AppResult<RetrieveAccountResponse> {
        let query = &[
            ("include[]", "configuration.merchant"),
            ("include[]", "requirements"),
        ];
        let resp: RetrieveAccountResponse = self.get_v2(&format!("/accounts/{}", account_id), query).await?;
        Ok(resp)
    }

    // ============ v1 PaymentIntent APIs ============

    pub async fn create_payment_intent(
        &self,
        amount_cents: i64,
        currency: &str,
        platform_fee_cents: i64,
        connected_account_id: &str,
        donation_id: Uuid,
        org_id: Uuid,
        donor_email: Option<&str>,
        donor_name: Option<&str>,
        metadata: Option<&Value>,
    ) -> AppResult<CreatePaymentIntentResponse> {
        // Build owned strings first
        let mut form_owned: Vec<(String, String)> = vec![
            ("amount".to_string(), amount_cents.to_string()),
            ("currency".to_string(), currency.to_string()),
            ("application_fee_amount".to_string(), platform_fee_cents.to_string()),
            ("on_behalf_of".to_string(), connected_account_id.to_string()),
            ("automatic_payment_methods[enabled]".to_string(), "true".to_string()),
            ("metadata[cream_org_id]".to_string(), org_id.to_string()),
            ("metadata[cream_donation_id]".to_string(), donation_id.to_string()),
        ];

        if let Some(email) = donor_email {
            form_owned.push(("receipt_email".to_string(), email.to_string()));
            form_owned.push(("metadata[cream_donor_email]".to_string(), email.to_string()));
        }
        if let Some(name) = donor_name {
            form_owned.push(("metadata[cream_donor_name]".to_string(), name.to_string()));
        }
        if let Some(m) = metadata {
            if let Some(obj) = m.as_object() {
                for (k, v) in obj {
                    form_owned.push((format!("metadata[{}]", k), v.to_string()));
                }
            }
        }

        // Convert to references for the form call
        let form_refs: Vec<(&str, &str)> = form_owned.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

        let resp: CreatePaymentIntentResponse = self.post_v1_form("/payment_intents", &form_refs).await?;
        Ok(resp)
    }

    // ============ Webhook verification ============

    pub fn verify_webhook(&self, payload: &[u8], signature: &str) -> AppResult<serde_json::Value> {
        // Parse Stripe signature header: "t=timestamp,v1=signature"
        let mut timestamp = None;
        let mut signatures = Vec::new();
        
        for part in signature.split(',') {
            let kv: Vec<&str> = part.split('=').collect();
            if kv.len() == 2 {
                match kv[0] {
                    "t" => timestamp = Some(kv[1]),
                    "v1" => signatures.push(kv[1]),
                    _ => {}
                }
            }
        }

        let timestamp = timestamp.ok_or_else(|| AppError::BadRequest("Missing timestamp in signature".to_string()))?;
        
        // Verify timestamp is within tolerance (5 minutes)
        let now = chrono::Utc::now().timestamp();
        let ts: i64 = timestamp.parse().map_err(|_| AppError::BadRequest("Invalid timestamp".to_string()))?;
        if (now - ts).abs() > 300 {
            return Err(AppError::BadRequest("Webhook timestamp too old".to_string()));
        }

        // Compute expected signature
        let signed_payload = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|_| AppError::Internal)?;
        mac.update(signed_payload.as_bytes());
        let expected_sig = hex::encode(mac.finalize().into_bytes());

        // Constant-time comparison
        let valid = signatures.iter().any(|sig| {
            sig.len() == expected_sig.len() && 
            sig.as_bytes().iter().zip(expected_sig.as_bytes()).all(|(a, b)| a == b)
        });

        if !valid {
            return Err(AppError::BadRequest("Invalid webhook signature".to_string()));
        }

        // Parse and return the event JSON
        serde_json::from_slice(payload).map_err(|e| {
            tracing::error!("Webhook JSON parse failed: {:?}", e);
            AppError::BadRequest("Invalid webhook payload".to_string())
        })
    }

    pub fn webhook_secret(&self) -> &str {
        &self.webhook_secret
    }
}

// ============ Response types ============

#[derive(Debug, Deserialize)]
pub struct CreateAccountResponse {
    pub id: String,
    #[serde(default)]
    pub requirements: Option<AccountRequirements>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AccountRequirements {
    #[serde(default)]
    pub currently_due: Vec<String>,
    #[serde(default)]
    pub eventually_due: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountSessionResponse {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct RetrieveAccountResponse {
    pub id: String,
    pub configuration: Option<AccountConfiguration>,
    pub requirements: Option<AccountRequirements>,
    pub charges_enabled: bool,
    pub payouts_enabled: bool,
    pub details_submitted: bool,
}

#[derive(Debug, Deserialize)]
pub struct AccountConfiguration {
    pub merchant: Option<MerchantConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct MerchantConfiguration {
    pub capabilities: MerchantCapabilities,
}

#[derive(Debug, Deserialize)]
pub struct MerchantCapabilities {
    #[serde(rename = "card_payments")]
    pub card_payments: CapabilityStatus,
    #[serde(rename = "stripe_balance.payouts")]
    pub payouts: Option<CapabilityStatus>,
}

#[derive(Debug, Deserialize)]
pub struct CapabilityStatus {
    pub active: bool,
    pub requested: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentIntentResponse {
    pub id: String,
    pub client_secret: String,
    pub status: String,
    pub amount: i64,
    pub currency: String,
}
