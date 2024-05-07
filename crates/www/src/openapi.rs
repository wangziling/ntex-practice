use ntex::util::Bytes;
use std::sync::Arc;
use utoipa::OpenApi;
use web_core::error::internal_app_error;
use web_core::error::internal_error::InternalAppError;
use web_core::handler_prelude::*;

use crate::controllers;

use crate::models::controllers::HelloWorld;
use utoipa::ToSchema;
use web_core::response::ResponseStatus;

#[allow(unused)]
#[derive(ToSchema)]
#[aliases(ServerResponseNullData=ServerResponseSchema<String>, ServerResponseHelloWorld=ServerResponseSchema<HelloWorld>)]
struct ServerResponseSchema<D> {
    data: Option<D>,
    message: Option<String>,
    status: ResponseStatus,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::greeting::hello,
        controllers::greeting::hello2,
        controllers::greeting::hello3,
        controllers::greeting::hello4
    ),
    components(schemas(
        HelloWorld,
        ResponseStatus,
        ServerResponseNullData,
        ServerResponseHelloWorld,
        InternalAppError
    ))
)]
struct ApiDoc;

pub async fn get_swagger(
    tail: Path<String>,
    openapi_conf: State<Arc<utoipa_swagger_ui::Config<'static>>>,
) -> AppResult<HttpResponse> {
    if tail.as_ref() == "swagger.json" {
        let spec = ApiDoc::openapi().to_json()?;
        return Ok(HttpResponse::Ok().content_type("application/json").body(spec));
    }
    let conf = openapi_conf.as_ref().clone();
    let file = utoipa_swagger_ui::serve(&tail, conf.into())
        .map_err(|err| internal_app_error(format!("Error serving Swagger UI: {}", err).into()))?
        .ok_or_else(|| internal_app_error(format!("path not found: {}", tail).into()))?;

    let bytes = Bytes::from(file.bytes.to_vec());

    Ok(HttpResponse::Ok().content_type(file.content_type).body(bytes))
}
