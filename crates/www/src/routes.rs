use web_core::route_prelude::*;

fn build_view_routes(cfg: &mut ServiceConfig) {
    cfg.service(resource("/").to(crate::controllers::views::index));

    // 404 - Only Get.
    cfg.service(resource("/404").route(get().to(crate::controllers::views::not_found)));

    // 500 - Only Get.
    cfg.service(resource("/500").route(get().to(crate::controllers::views::internal_server_error)));
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
