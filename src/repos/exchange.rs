use chrono::NaiveDateTime;
use diesel;

use super::error::*;
use super::executor::with_tls_connection;
use super::*;
use models::*;
use prelude::*;
use schema::exchanges::dsl::*;

pub trait ExchangesRepo: Send + Sync + 'static {
    fn create(&self, payload: NewExchange) -> RepoResult<Exchange>;
    fn get(&self, req: GetExchange) -> RepoResult<Option<Exchange>>;
    fn get_by_id(&self, exchange_id: ExchangeId) -> RepoResult<Option<Exchange>>;
    fn update_expiration(&self, exchange_id: ExchangeId, expiration: NaiveDateTime) -> RepoResult<Exchange>;
}

#[derive(Clone, Default)]
pub struct ExchangesRepoImpl;

impl<'a> ExchangesRepo for ExchangesRepoImpl {
    fn create(&self, payload: NewExchange) -> RepoResult<Exchange> {
        with_tls_connection(|conn| {
            diesel::insert_into(exchanges)
                .values(payload.clone())
                .get_result::<Exchange>(conn)
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => payload)
                })
        })
    }

    fn get(&self, req: GetExchange) -> RepoResult<Option<Exchange>> {
        with_tls_connection(|conn| {
            exchanges
                .filter(id.eq(req.id))
                .filter(from_.eq(req.from))
                .filter(to_.eq(req.to))
                .filter(amount.ge(req.actual_amount))
                .filter(expiration.ge(::chrono::Utc::now().naive_utc()))
                .limit(1)
                .get_result(conn)
                .optional()
                .map_err(move |e| {
                    let error_kind = ErrorKind::from(&e);
                    ectx!(err e, error_kind => req)
                })
        })
    }

    fn get_by_id(&self, exchange_id: ExchangeId) -> RepoResult<Option<Exchange>> {
        with_tls_connection(|conn| {
            exchanges.filter(id.eq(exchange_id)).get_result(conn).optional().map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => exchange_id)
            })
        })
    }

    fn update_expiration(&self, exchange_id: ExchangeId, expiration_: NaiveDateTime) -> RepoResult<Exchange> {
        with_tls_connection(|conn| {
            let command = diesel::update(exchanges.filter(id.eq(exchange_id))).set(expiration.eq(expiration_));

            command.get_result(conn).map_err(move |e| {
                let error_kind = ErrorKind::from(&e);
                ectx!(err e, error_kind => exchange_id)
            })
        })
    }
}

#[cfg(test)]
pub mod tests {
    use diesel::r2d2::ConnectionManager;
    use diesel::PgConnection;
    use futures_cpupool::CpuPool;
    use r2d2;
    use tokio_core::reactor::Core;

    use super::*;
    use config::Config;
    use repos::DbExecutorImpl;

    fn create_executor() -> DbExecutorImpl {
        let config = Config::new().unwrap();
        let manager = ConnectionManager::<PgConnection>::new(config.database.url);
        let db_pool = r2d2::Pool::builder().build(manager).unwrap();
        let cpu_pool = CpuPool::new(1);
        DbExecutorImpl::new(db_pool.clone(), cpu_pool.clone())
    }

    #[test]
    fn exchanges_create() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let exchanges_repo = ExchangesRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_exchange = NewExchange::default();
            let res = exchanges_repo.create(new_exchange);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn exchanges_read() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let exchanges_repo = ExchangesRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction(move || {
            let new_exchange = NewExchange::default();
            let _ = exchanges_repo.create(new_exchange).unwrap();
            let get = GetExchange::default();
            let res = exchanges_repo.get(get);
            assert!(res.is_ok());
            res
        }));
    }

    #[test]
    fn exchanges_get_by_id() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let exchanges_repo = ExchangesRepoImpl::default();
        let _ = core.run(db_executor.execute_test_transaction::<_, _, error::Error>(move || {
            let new_exchange = NewExchange::default();
            let exchange = exchanges_repo.create(new_exchange).expect("failed to create rate");
            exchanges_repo
                .get_by_id(exchange.id)
                .expect("failed to get rate by id")
                .expect("get_by_id returned None value");
            Ok(())
        }));
    }

    #[test]
    fn exchanges_update_expiration() {
        let mut core = Core::new().unwrap();
        let db_executor = create_executor();
        let exchanges_repo = ExchangesRepoImpl::default();
        let first_datetime = ::chrono::NaiveDateTime::from_timestamp(0, 0);
        let second_datetime = ::chrono::NaiveDateTime::from_timestamp(100, 0);

        let new_exchange = NewExchange {
            expiration: first_datetime.clone(),
            ..NewExchange::default()
        };

        let _ = core.run(db_executor.execute_test_transaction::<_, _, error::Error>(move || {
            let exchange = exchanges_repo.create(new_exchange).expect("failed to create rate");
            let exchange = exchanges_repo
                .update_expiration(exchange.id, second_datetime)
                .expect("failed to refresh rate");
            assert_eq!(second_datetime, exchange.expiration);
            Ok(())
        }));
    }
}
