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
}
