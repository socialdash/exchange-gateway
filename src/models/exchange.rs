use std::borrow::Cow;
use std::collections::HashMap;
use std::time::SystemTime;

use validator::{Validate, ValidationError, ValidationErrors};

use config::CurrenciesLimits;
use models::*;
use schema::exchanges;

#[derive(Debug, Queryable, Clone)]
pub struct Exchange {
    pub id: ExchangeId,
    pub from_: Currency,
    pub to_: Currency,
    pub amount: Amount,
    pub expiration: SystemTime,
    pub rate: f64,
    pub user_id: UserId,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Default for Exchange {
    fn default() -> Self {
        Self {
            id: ExchangeId::generate(),
            from_: Currency::Eth,
            to_: Currency::Btc,
            amount: Amount::default(),
            expiration: SystemTime::now(),
            rate: 0.34343,
            user_id: UserId::generate(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

impl From<NewExchange> for Exchange {
    fn from(new_exchange: NewExchange) -> Self {
        Self {
            id: new_exchange.id,
            from_: new_exchange.from_,
            to_: new_exchange.to_,
            amount: new_exchange.amount,
            rate: new_exchange.rate,
            expiration: new_exchange.expiration,
            user_id: new_exchange.user_id,
            ..Default::default()
        }
    }
}

#[derive(Debug, Insertable, Validate, Clone)]
#[table_name = "exchanges"]
pub struct NewExchange {
    pub id: ExchangeId,
    pub from_: Currency,
    pub to_: Currency,
    pub amount: Amount,
    pub expiration: SystemTime,
    pub rate: f64,
    pub user_id: UserId,
}

impl Default for NewExchange {
    fn default() -> Self {
        Self {
            id: ExchangeId::generate(),
            from_: Currency::Eth,
            to_: Currency::Btc,
            amount: Amount::default(),
            expiration: SystemTime::now(),
            rate: 0.34343,
            user_id: UserId::generate(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateSellOrder {
    pub id: ExchangeId,
    pub from: Currency,
    pub to: Currency,
    pub actual_amount: Amount,
}

impl From<CreateSellOrder> for GetExchange {
    fn from(sell: CreateSellOrder) -> Self {
        Self {
            id: sell.id,
            from: sell.from,
            to: sell.to,
            actual_amount: sell.actual_amount,
        }
    }
}

impl Default for CreateSellOrder {
    fn default() -> Self {
        Self {
            id: ExchangeId::generate(),
            from: Currency::Eth,
            to: Currency::Btc,
            actual_amount: Amount::default(),
        }
    }
}

pub fn validate(from: Currency, amount: Amount, limits: CurrenciesLimits) -> Result<(), ValidationErrors> {
    let limit = match from {
        Currency::Btc => limits.btc,
        Currency::Eth => limits.eth,
        Currency::Stq => limits.stq,
    };
    let quantity = from.to_f64(amount);

    let mut errors = ValidationErrors::new();
    if quantity < limit.min || quantity > limit.max {
        let error = ValidationError {
            code: Cow::from("limit"),
            message: Some(Cow::from(format!(
                "Amount should be between {} and {}",
                from.from_f64(limit.min),
                from.from_f64(limit.max)
            ))),
            params: HashMap::new(),
        };
        errors.add("actual_amount", error);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug, Clone)]
pub struct GetExchange {
    pub id: ExchangeId,
    pub from: Currency,
    pub to: Currency,
    pub actual_amount: Amount,
}

impl Default for GetExchange {
    fn default() -> Self {
        Self {
            id: ExchangeId::generate(),
            from: Currency::Eth,
            to: Currency::Btc,
            actual_amount: Amount::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetRate {
    pub id: ExchangeId,
    pub from: Currency,
    pub to: Currency,
    pub amount: Amount,
}

impl NewExchange {
    pub fn new(req: GetRate, expiration: SystemTime, rate: f64, user_id: UserId) -> Self {
        Self {
            id: ExchangeId::generate(),
            from_: req.from,
            to_: req.to,
            amount: req.amount,
            expiration,
            rate,
            user_id,
        }
    }
}

impl Default for GetRate {
    fn default() -> Self {
        Self {
            id: ExchangeId::generate(),
            from: Currency::Eth,
            to: Currency::Btc,
            amount: Amount::default(),
        }
    }
}
