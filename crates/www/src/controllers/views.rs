use web_core::handler_prelude::*;
use web_core::view_template::ViewTemplate;

#[web_view_template]
#[template(path = "index.html")]
struct IndexTemplate {
    name: &'static str,
    messages: Vec<&'static str>,
}

#[web_view_template]
#[template(path = "404.html")]
struct NotFoundTemplate {}

#[web_view_template]
#[template(path = "500.html")]
struct InternalServerErrorTemplate {}

#[instrument(skip_all, err)]
pub async fn index(_request: HttpRequest, _state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    // let query = request.query()?;

    // info!("query: {:?}", query);
    // info!("query.a: {:?}", query.get("a").ok_or_else(|| anyhow!("Failed to get query: a."))?);

    let ctx = IndexTemplate { name: "test", messages: vec!["111", "222"], ..Default::default() };

    let result = ctx.render_once().map(map_view_render_result)?;

    Ok(result)
}

#[instrument(skip_all, err)]
pub async fn not_found(_request: HttpRequest, _state: State<crate::app::AppState>) -> AppResult<impl Responder> {
    let mut ctx = NotFoundTemplate { ..Default::default() };

    ctx.set_title("NOT FOUND".to_string());

    let result = ctx.render_once().map(map_view_render_result)?;

    Ok(result)
}

#[instrument(skip_all, err)]
pub async fn internal_server_error(
    _request: HttpRequest,
    _state: State<crate::app::AppState>,
) -> AppResult<impl Responder> {
    let mut ctx = InternalServerErrorTemplate { ..Default::default() };

    ctx.set_title("INTERNAL SERVER ERROR".to_string());

    let result = ctx.render_once().map(map_view_render_result)?;

    Ok(result)
}
