pub use web_core::route_prelude::*;

fn build_view_routes(cfg: &mut ServiceConfig) {
    cfg.service(resource("/").to(crate::controllers::views::index));

    // 404.
    cfg.service(resource("/404").route(get().to(crate::controllers::views::not_found)));
}

fn build_greeting_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/greeting/hello")
            .wrap(crate::middlewares::demo::SayHi) // Second one.
            .wrap(crate::middlewares::prerequisites::RequireJson) // First middleware.
            .to(crate::controllers::greeting::hello),
    );

    cfg.service(
        resource("/greeting/hello4")
            .wrap(crate::middlewares::prerequisites::RequireJson) // Second one.
            .wrap(crate::middlewares::prerequisites::ForAjaxReqOnly) // First middleware.
            .route(get().to(crate::controllers::greeting::hello4)),
    );

    cfg.service(
        scope("/greeting").service((crate::controllers::greeting::hello2, crate::controllers::greeting::hello3)),
    );
}

pub fn build_routes(cfg: &mut ServiceConfig) {
    build_greeting_routes(cfg);
    build_view_routes(cfg);
}

pub fn fallback_service() -> Route {
    route().to(|req: HttpRequest| async move {
        if req.derived_from_ajax() {
            return Ok(server_response_failed!(message: Some("Requested resource not found."), status_code: 404.try_into().ok())
            .into());
        }

        if req.method() == Method::GET {
            return server_redirect!("/404");
        }

        // Not found.
        Ok(HttpResponse::NotFound().finish())
    })
}
