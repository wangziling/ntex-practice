use crate::error::Result;
use crate::features::{QueryContentMap, QueryItemContent, QueryItemKey, QueryMap};

pub fn update_query(query_map: &mut QueryMap, key: QueryItemKey, value: Option<QueryItemContent>) -> Result<()> {
    match value {
        Some(val) => {
            let _ = query_map.insert(key, val);
        }
        None => {
            query_map.remove(&key);
        }
    }

    Ok(())
}

pub fn update_query_map(query_map: &mut QueryMap, target_query_map: QueryContentMap) -> Result<()> {
    for (key, value) in target_query_map.into_iter() {
        update_query(query_map, key, Some(value))?;
    }

    Ok(())
}

pub fn remove_query<T>(query_map: &mut QueryMap, keys: T) -> Result<()>
where
    T: Iterator<Item = QueryItemKey>,
{
    for key in keys {
        query_map.remove(&key);
    }

    Ok(())
}

pub fn query_to_string(input: QueryMap) -> Result<String> {
    let query_string = serde_urlencoded::to_string(input.into_inner())?;

    Ok(query_string)
}

pub fn parse_into_status_code<S>(status_code: S) -> Option<ntex::http::StatusCode>
where
    S: TryInto<ntex::http::StatusCode>,
{
    TryInto::<ntex::http::StatusCode>::try_into(status_code).ok()
}
