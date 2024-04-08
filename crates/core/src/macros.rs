#[macro_export]
macro_rules! __server_response_impl {
    ($type: ident) => {
        $crate::response::ServerResponse::<String, String>::$type(None, None, Option::<&'static str>::None)
    };
    ($type: ident, $data: expr) => {
        $crate::response::ServerResponse::$type($data, Option::<String>::None, Option::<&'static str>::None)
    };
    ($type: ident, $data: expr, $message: expr) => {
        $crate::response::ServerResponse::$type($data, $message, Option::<&'static str>::None)
    };
    ($type: ident, $data: expr, $message: expr, $status_code: expr) => {
        $crate::response::ServerResponse::$type($data, $message, $status_code)
    };
}

// No need to export.
macro_rules! header_contains {
    ($headers_map: expr, $key:expr, $target_val:expr) => {
        $headers_map.get_all($key).into_iter().any(|val| val.to_str().unwrap_or_default().contains($target_val))
    };

    ($headers_map: expr, $key:expr, $target_val:expr, ignore_case: $ignore_case:expr) => {
        $headers_map.get_all($key).into_iter().any(|val| {
            if $ignore_case {
                val.to_str().unwrap_or_default().to_lowercase().contains($target_val.to_lowercase().as_str())
            } else {
                val.to_str().unwrap_or_default().contains($target_val)
            }
        })
    };
}
