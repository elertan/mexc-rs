#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Topic {
    AccountDeals,
    AccountOrders,
    AccountUpdate,
}

impl Topic {
    pub fn requires_auth(&self) -> bool {
        match self {
            Topic::AccountDeals => true,
            Topic::AccountOrders => true,
            Topic::AccountUpdate => true,
        }
    }

    pub fn to_topic_subscription_string(&self) -> String {
        match self {
            Topic::AccountDeals => "spot@private.deals.v3.api".to_string(),
            Topic::AccountOrders => "spot@private.orders.v3.api".to_string(),
            Topic::AccountUpdate => "spot@private.account.v3.api".to_string(),
        }
    }
}
