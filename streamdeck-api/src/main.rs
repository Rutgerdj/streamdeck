use actix::{Actor, Addr};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use hidapi::HidApi;
use streamdeck_interface::connectionmanager::ConnectionManager;
use streamdeck_interface::deckstate::DeckHandler;
use streamdeck_interface::hub::DeckHub;

mod ws_session;

#[actix_rt::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let api = HidApi::new().unwrap();

    let handler = DeckHandler::load();

    let hub = DeckHub::new(handler).start();

    let cm = ConnectionManager::new(hub.clone(), api);

    cm.start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(hub.clone()))
            .route("/ws", web::get().to(ws_upgrade))
    })
    .disable_signals()
    .workers(1)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/// Entry point for websocket route
async fn ws_upgrade(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<DeckHub>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        ws_session::WsSession::new(srv.get_ref().clone()),
        &req,
        stream,
    )
}
