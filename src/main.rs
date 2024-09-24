use dotenv::dotenv;
#[macro_use]
extern crate rocket;
use rocket::http::Method;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

mod routes;

pub struct CORS;

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let config = rocket::Config {
        address: std::net::Ipv4Addr::new(0,0,0,0).into(),
        port: 8000,
        ..Default::default()
    };

    rocket::custom(config)
        .attach(CORS)
        .mount(
            "/",
            routes![
                routes::accounts_handler,
                routes::transactions_handler,
                routes::chart_handler
            ],
        )
}

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to requests",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        let allowed_origins = vec!["http://localhost:3000", "https://solanamirror.xyz"];
        if let Some(origin) = _request.headers().get_one("Origin") {
            if allowed_origins.contains(&origin) {
                response.set_header(Header::new("Access-Control-Allow-Origin", origin));
            }
        }
        
        response.set_header(Header::new("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type, Authorization"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));

        if _request.method() == Method::Options {
            response.set_header(Header::new("Access-Control-Max-Age", "86400"));
            response.set_status(rocket::http::Status::Ok);
        }
    }
}
