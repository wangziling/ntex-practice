use crate::error::Result;

type QueryContentMap = std::collections::HashMap<String, String>;

type QueryMap = ntex::web::types::Query<QueryContentMap>;

type QueryResult = Result<QueryMap, ntex::web::error::QueryPayloadError>;

pub trait HttpRequestExt {
    fn query(&self) -> QueryResult;
    fn extend_query_string(&self, query_params: QueryContentMap) -> Result<String>;
    fn query_to_string(input: QueryMap) -> Result<String>;
}

impl HttpRequestExt for ntex::web::HttpRequest {
    fn query(&self) -> QueryResult {
        QueryMap::from_query(self.query_string())
    }

    fn extend_query_string(&self, query_params: QueryContentMap) -> Result<String> {
        let mut query_map = self.query()?;
        query_map.extend(query_params);

        Self::query_to_string(query_map)
    }

    fn query_to_string(input: QueryMap) -> Result<String> {
        let query_string = serde_urlencoded::to_string(input.into_inner())?;

        Ok(query_string)
    }
}

pub trait RequestUtils {
    fn wants_json(&self) -> bool;
    fn derived_from_json(&self) -> bool;
    fn derived_from_form(&self) -> bool;
    fn derived_from_form_data(&self) -> bool;
    fn derived_from_ajax(&self) -> bool;
}

impl<T: ntex::http::HttpMessage> RequestUtils for T {
    fn wants_json(&self) -> bool {
        header_contains!(self.message_headers(), ntex::http::header::ACCEPT, "json")
    }

    fn derived_from_json(&self) -> bool {
        header_contains!(self.message_headers(), ntex::http::header::CONTENT_TYPE, "application/json")
    }

    fn derived_from_form(&self) -> bool {
        header_contains!(self.message_headers(), ntex::http::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
    }

    fn derived_from_form_data(&self) -> bool {
        header_contains!(self.message_headers(), ntex::http::header::CONTENT_TYPE, "multipart/form-data")
    }

    fn derived_from_ajax(&self) -> bool {
        header_contains!(self.message_headers(), "x-requested-with", "XMLHttpRequest", ignore_case: true)
    }
}
