use async_trait::async_trait;
use crate::ws::{ClientMessagePayload, MexcWsClient};

#[derive(Debug)]
pub struct SubscribeParams {
    pub subscription_requests: Vec<SubscriptionRequest>,
}

#[derive(Debug)]
pub enum SubscriptionRequest {
    SpotDeals(SpotDealsSubscriptionRequest),
}

#[derive(Debug)]
pub struct SpotDealsSubscriptionRequest {
    pub symbol: String,
}

impl SubscriptionRequest {
    pub fn to_subscription_param(&self) -> String {
        match self {
            SubscriptionRequest::SpotDeals(spot_deals_sr) => format!("spot@public.deals.v3.api@{}", spot_deals_sr.symbol)
        }
    }
}

#[derive(Debug)]
pub struct SubscribeOutput {}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {}

#[async_trait]
pub trait Subscribe {
    async fn subscribe(&self, params: SubscribeParams) -> Result<SubscribeOutput, SubscribeError>;
}

#[async_trait]
impl Subscribe for MexcWsClient {
    async fn subscribe(&self, params: SubscribeParams) -> Result<SubscribeOutput, SubscribeError> {
        let payload_params = params.subscription_requests
            .iter()
            .map(|sr| sr.to_subscription_param())
            .collect::<Vec<String>>();
        let payload = ClientMessagePayload {
            method: "SUBSCRIBE",
            params: payload_params,
        };
        todo!()
    }
}
