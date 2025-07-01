pub mod commands;
pub mod errors;
pub mod queries;

pub use self::queries::GetReceiver;
use crate::application::actors::symbols_fetcher_actor::commands::Command::Refresh;
use crate::domain::symbol::Symbols;
use crate::integrations::http::binance::spot_http_client::ExchangeInfoApi;
use actix::{Actor, AsyncContext, Context};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::sync::watch::Sender;
use tracing::{debug, info};
use watch::Receiver;

#[derive(Debug, Clone)]
pub enum RefreshPolicy {
    Lazy,
    OneShot,
    Periodic(Duration),
}

pub struct SymbolsFetcherActor {
    refresh_policy: RefreshPolicy,
    binance_client: Arc<dyn ExchangeInfoApi + Send + Sync>,

    sender: Sender<Option<Arc<Symbols>>>,
    _receiver: Receiver<Option<Arc<Symbols>>>,
}

impl SymbolsFetcherActor {
    pub fn new(refresh_policy: RefreshPolicy, binance_client: Arc<dyn ExchangeInfoApi + Send + Sync>) -> Self {
        let (sender, _receiver) = watch::channel::<Option<Arc<Symbols>>>(None);
        Self {
            refresh_policy,
            sender,
            binance_client,
            _receiver,
        }
    }
}

impl Actor for SymbolsFetcherActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("[SymbolsFetcherActor] started with refresh_policy: {:?}", &self.refresh_policy);

        match self.refresh_policy {
            RefreshPolicy::Lazy => {
                debug!("[SymbolsFetcherActor] is in Lazy mode, waiting for Refresh message");
            }
            RefreshPolicy::OneShot => {
                ctx.address().do_send(Refresh);
            }
            RefreshPolicy::Periodic(duration) => {
                ctx.address().do_send(Refresh);
                ctx.run_interval(duration, |_, ctx| {
                    ctx.address().do_send(Refresh);
                });
            }
        }
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("[SymbolsFetcherActor] is stopped");
    }
}
