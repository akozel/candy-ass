use crate::application::history_downloader::candlesticks_downloader_actor::CandlesticksDownloaderActor;
use actix::{ActorContext, Handler, Message};
use tracing::info;

#[derive(Message)]
#[rtype(result = "()")]
pub enum Command {
    Shutdown,
}

impl Handler<Command> for CandlesticksDownloaderActor {
    type Result = ();

    fn handle(&mut self, msg: Command, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Command::Shutdown => {
                info!("[CandlesticksDownloaderActor] is completing it's work");
                ctx.stop();
            }
        }
    }
}
