use dotenv::dotenv;

#[macro_use]
extern crate rocket;

mod routes;

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let config = rocket::Config {
        address: std::net::Ipv4Addr::new(0,0,0,0).into(),
        port: 8000,
        ..Default::default()
    };

    rocket::custom(config).mount(
        "/",
        routes![
            routes::accounts_handler,
            routes::transactions_handler,
            routes::chart_handler
        ],
    )
}
