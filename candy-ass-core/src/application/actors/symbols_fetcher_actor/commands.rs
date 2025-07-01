use crate::application::actors::symbols_fetcher_actor::SymbolsFetcherActor;
use crate::application::actors::symbols_fetcher_actor::commands::Command::{Refresh, Shutdown};
use crate::application::actors::symbols_fetcher_actor::errors::FailedToFetchSymbolsError;
use crate::domain::symbol::Symbols;
use actix::{ActorContext, AsyncContext, Handler, Message, ResponseFuture, WrapFuture};
use futures_util::{FutureExt, TryFutureExt};
use std::sync::Arc;
use tracing::info;

#[derive(Message)]
#[rtype(result = "()")]
pub enum Command {
    Refresh,
    Shutdown,
}

impl Handler<Command> for SymbolsFetcherActor {
    type Result = ();

    fn handle(&mut self, msg: Command, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Refresh => {
                let address = ctx.address().clone();
                ctx.spawn(async move { address.do_send(RefreshAndGet) }.into_actor(self));
            }
            Shutdown => {
                info!("[SymbolsFetcherActor] is completing it's work");
                ctx.stop();
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "Result<Arc<Symbols>, FailedToFetchSymbolsError>")]
pub struct RefreshAndGet;

impl Handler<RefreshAndGet> for SymbolsFetcherActor {
    type Result = ResponseFuture<Result<Arc<Symbols>, FailedToFetchSymbolsError>>;

    fn handle(&mut self, _msg: RefreshAndGet, _ctx: &mut Self::Context) -> Self::Result {
        let binance_client = self.binance_client.clone();
        let sender = self.sender.clone();

        async move {
            binance_client
                .fetch_binance_exchange_info()
                .map_ok(|(exchange_info, _)| {
                    let symbols = exchange_info.to_symbols();
                    let arc_symbols = Arc::new(symbols);
                    let _ = sender.send(Some(arc_symbols.clone()));
                    Ok(arc_symbols)
                })
                .await?
        }
        .boxed()
    }
}
