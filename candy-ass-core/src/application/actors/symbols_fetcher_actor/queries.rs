use crate::application::actors::symbols_fetcher_actor::SymbolsFetcherActor;
use crate::domain::symbol::Symbols;
use actix::{Handler, Message, MessageResult};
use std::sync::Arc;
use tokio::sync::watch::Receiver;

#[derive(Message)]
#[rtype(result = "Receiver<Option<Arc<Symbols>>>")]
pub struct GetReceiver;

impl Handler<GetReceiver> for SymbolsFetcherActor {
    type Result = MessageResult<GetReceiver>;
    fn handle(&mut self, _msg: GetReceiver, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.sender.subscribe())
    }
}
