// Actix server setup, task spawning

mod api;
mod error;
mod exchanges;
mod models;


#[actix_web::main]
async fn main() -> std::io::Result<()>{
   println!("Stating Global BTC/USDT Price Index API ...");
   api::start_server().await
}
