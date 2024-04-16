use crate::error::Result;

pub(crate) type QueryItemKey = String;
pub(crate) type QueryItemContent = String;

pub(crate) type QueryContentMap = std::collections::HashMap<QueryItemKey, QueryItemContent>;

pub(crate) type QueryMap = ntex::web::types::Query<QueryContentMap>;

type QueryResult = Result<QueryMap>;

pub trait HttpRequestExt {
    fn query(&self) -> QueryResult;
    fn extend_query_string(&self, query_params: QueryContentMap) -> Result<String>;
    fn query_to_string(input: QueryMap) -> Result<String>;
}

impl HttpRequestExt for ntex::web::HttpRequest {
    fn query(&self) -> QueryResult {
        QueryMap::from_query(self.query_string()).map_err(Into::into)
    }

    fn extend_query_string(&self, query_params: QueryContentMap) -> Result<String> {
        let mut query_map = self.query()?;
        query_map.extend(query_params);

        Self::query_to_string(query_map)
    }

    fn query_to_string(input: QueryMap) -> Result<String> {
        crate::utils::query_to_string(input)
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
    #[inline]
    fn wants_json(&self) -> bool {
        header_contains!(self.message_headers(), ntex::http::header::ACCEPT, b"json")
    }

    #[inline]
    fn derived_from_json(&self) -> bool {
        header_contains!(self.message_headers(), ntex::http::header::CONTENT_TYPE, b"application/json")
    }

    #[inline]
    fn derived_from_form(&self) -> bool {
        header_contains!(self.message_headers(), ntex::http::header::CONTENT_TYPE, b"application/x-www-form-urlencoded", ignore_case: true)
    }

    #[inline]
    fn derived_from_form_data(&self) -> bool {
        header_contains!(self.message_headers(), ntex::http::header::CONTENT_TYPE, b"multipart/form-data")
    }

    #[inline]
    fn derived_from_ajax(&self) -> bool {
        header_contains!(self.message_headers(), "x-requested-with", b"XMLHttpRequest", ignore_case: true)
    }
}

pub trait UriUtils {
    fn query_map(&self) -> QueryResult;
    fn update_query(&mut self, key: QueryItemKey, value: Option<QueryItemContent>) -> QueryResult;
    fn update_query_map(&mut self, target_query_map: QueryContentMap) -> QueryResult;
    fn remove_query<T>(&mut self, keys: T) -> QueryResult
    where
        T: Iterator<Item = QueryItemKey>;
}

impl UriUtils for ntex::http::Uri {
    fn query_map(&self) -> QueryResult {
        match self.query() {
            Some(query) => QueryMap::from_query(query).map_err(Into::into),
            None => ntex::web::types::Query::from_query(Default::default()).map_err(Into::into),
        }
    }

    fn update_query(&mut self, key: QueryItemKey, value: Option<QueryItemContent>) -> QueryResult {
        let mut query_map = self.query_map()?;

        crate::utils::update_query(&mut query_map, key, value)?;

        Ok(query_map)
    }

    fn update_query_map(&mut self, target_query_map: QueryContentMap) -> QueryResult {
        let mut query_map = self.query_map()?;

        crate::utils::update_query_map(&mut query_map, target_query_map)?;

        Ok(query_map)
    }

    fn remove_query<T>(&mut self, keys: T) -> QueryResult
    where
        T: Iterator<Item = QueryItemKey>,
    {
        let mut query_map = self.query_map()?;

        crate::utils::remove_query(&mut query_map, keys)?;

        Ok(query_map)
    }
}
