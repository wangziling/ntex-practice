use web_core::route_prelude::*;

use crate::constants::{INTERNAL_SERVER_ERROR_REQ_PATH, NOT_FOUND_REQ_PATH};

fn build_view_routes(cfg: &mut ServiceConfig) {
    cfg.service(resource("/").wrap(crate::middlewares::view::prerequisites()).to(crate::controllers::views::index));

    cfg.service(scope("/").wrap(crate::middlewares::view::prerequisites()).service((
        resource(NOT_FOUND_REQ_PATH).route(get().to(crate::controllers::views::not_found)),
        resource(INTERNAL_SERVER_ERROR_REQ_PATH).route(get().to(crate::controllers::views::internal_server_error)),
    )));
}

fn build_greeting_routes(cfg: &mut ServiceConfig) {
    cfg.service(resource("/greeting/hello2").to(crate::controllers::greeting::hello2));

    cfg.service(resource("/greeting/hello3").to(crate::controllers::greeting::hello3));

    cfg.service(
        scope("/greeting") // Third one.
            .wrap(crate::middlewares::prerequisites::RequireJson) // Second one. // First middleware.
            .service((
                resource("/hello").to(crate::controllers::greeting::hello),
                resource("/hello4").to(crate::controllers::greeting::hello4),
            )),
    );
}

pub fn build_routes(cfg: &mut ServiceConfig) {
    build_greeting_routes(cfg);
    build_view_routes(cfg);
}
