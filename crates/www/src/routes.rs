use web_core::route_prelude::*;

fn build_view_routes(cfg: &mut ServiceConfig) {
    cfg.service(resource("/").to(crate::controllers::views::index));

    // 404 - Only Get.
    cfg.service(resource("/404").route(get().to(crate::controllers::views::not_found)));
}

fn build_greeting_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/greeting")
            .wrap(crate::middlewares::demo::SayHi) // Third one.
            .wrap(crate::middlewares::prerequisites::RequireJson) // Second one.
            .wrap(crate::middlewares::extensions::ExtensionDistributeCache) // First middleware.
            .service((
                resource("/hello").to(crate::controllers::greeting::hello),
                resource("/hello4").to(crate::controllers::greeting::hello4),
            )),
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
            let mut uri = "/404".parse::<ntex::http::Uri>()?;

            match req.uri().path_and_query() {
                Some(path_query) => {
                    let query_map = uri.update_query("prev".to_string(), Some(path_query.to_string()))?;

                    return server_redirect!(uri.path().to_string() + "?" + &query_to_string(query_map)?);
                },
                _ => {}
            }

            return server_redirect!(uri.to_string());
        }

        // Not found.
        Ok(HttpResponse::NotFound().finish())
    })
}
