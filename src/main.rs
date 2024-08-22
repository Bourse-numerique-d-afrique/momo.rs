use poem::{get, handler, listener::TcpListener, web::Path, Route, Server};

#[handler]
fn callback(Path(name): Path<String>) -> String {
    format!("hello")
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("callback", get(callback));
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
