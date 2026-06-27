use crate::api::tenants::{mark_subscription, Plan, SubscriptionStatus};
use axum::{http::HeaderMap, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CheckoutRequest {
    user_id: Option<String>,
    success_url: Option<String>,
    cancel_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct CheckoutResponse {
    checkout_url: String,
    mode: &'static str,
}

#[derive(Debug, Deserialize)]
struct BillingWebhook {
    user_id: String,
    event_type: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/v1/billing/checkout", post(create_checkout))
        .route("/v1/billing/webhook", post(handle_webhook))
}

async fn create_checkout(
    headers: HeaderMap,
    Json(payload): Json<CheckoutRequest>,
) -> Json<CheckoutResponse> {
    let user_id = payload
        .user_id
        .or_else(|| {
            headers
                .get("x-atlas-user-id")
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "demo-user".to_string());
    let success_url = payload
        .success_url
        .unwrap_or_else(|| "/settings".to_string());
    let cancel_url = payload
        .cancel_url
        .unwrap_or_else(|| "/settings".to_string());

    let checkout_url = std::env::var("STRIPE_CHECKOUT_URL").unwrap_or_else(|_| {
        format!(
            "https://billing.stripe.com/p/login/test_atlas?client_reference_id={}&success_url={}&cancel_url={}",
            user_id, success_url, cancel_url
        )
    });

    Json(CheckoutResponse {
        checkout_url,
        mode: "stripe_checkout",
    })
}

async fn handle_webhook(Json(payload): Json<BillingWebhook>) -> Json<serde_json::Value> {
    let (plan, status) = match payload.event_type.as_str() {
        "checkout.session.completed"
        | "customer.subscription.created"
        | "customer.subscription.updated" => (Plan::Pro, SubscriptionStatus::Active),
        "customer.subscription.trial_will_end" => (Plan::Pro, SubscriptionStatus::Trialing),
        "invoice.payment_failed" => (Plan::Pro, SubscriptionStatus::PastDue),
        "customer.subscription.deleted" => (Plan::Free, SubscriptionStatus::Canceled),
        _ => (Plan::Free, SubscriptionStatus::Free),
    };

    let tenant = mark_subscription(&payload.user_id, plan, status);

    Json(serde_json::json!({
        "ok": true,
        "tenant": tenant
    }))
}

#[cfg(test)]
mod tests {
    use crate::api::tenants::{mark_subscription, Plan, SubscriptionStatus};

    #[test]
    fn pro_subscription_raises_monthly_quota() {
        let tenant = mark_subscription("billing-test", Plan::Pro, SubscriptionStatus::Active);

        assert_eq!(tenant.monthly_resolve_quota, 5_000);
    }
}
