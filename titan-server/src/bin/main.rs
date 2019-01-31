#![feature(plugin, decl_macro, custom_attribute, proc_macro_hygiene)]

use rocket::{routes, fairing::AdHoc};

use libtitan::routes;
use libtitan::db;
use libtitan::config;
use libtitan::accounts;
use libtitan::organizations;
use libtitan::config::AppConfig;

fn main() {
    rocket::ignite()
        .attach(db::TitanPrimary::fairing())
        .attach(db::UnksoMainForums::fairing())
        .attach(AppConfig::fairing())
        .mount("/api/auth/pulse", routes![routes::health_check])
        .mount("/api/auth", accounts::get_auth_routes())
        .mount("/api/users", accounts::get_user_routes())
        .mount("/api/organizations", organizations::get_routes())
        .launch();
}