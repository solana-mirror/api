#[macro_use]
extern crate rocket;

mod routes;

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            routes::accounts::accounts_handler,
            routes::transactions::transactions_handler
        ],
    )
}
