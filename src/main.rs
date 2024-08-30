use dotenv::dotenv;

#[macro_use]
extern crate rocket;

mod routes;

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build().mount(
        "/",
        routes![
            routes::accounts_handler,
            routes::transactions_handler,
            routes::chart_handler
        ],
    )
}
