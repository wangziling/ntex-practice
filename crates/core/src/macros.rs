#[macro_export]
macro_rules! __server_response_impl {
    ($type: ident) => {
        $crate::response::ServerResponse::<String, String>::$type(None, None, Option::<u16>::None)
    };
    ($type: ident, $data: expr) => {
        $crate::response::ServerResponse::$type($data, Option::<String>::None, Option::<u16>::None)
    };
    ($type: ident, $data: expr, $message: expr) => {
        $crate::response::ServerResponse::$type($data, $message, Option::<u16>::None)
    };
    ($type: ident, $data: expr, $message: expr, $status_code: expr) => {
        $crate::response::ServerResponse::$type($data, $message, $status_code)
    };
}

// No need to export.
macro_rules! header_contains {
    ($headers_map: expr, $key:expr, $target_val:expr) => {
        $headers_map.get_all($key).into_iter().any(|val| val.as_bytes().starts_with($target_val))
    };

    ($headers_map: expr, $key:expr, $target_val:expr, ignore_case: $ignore_case:expr) => {
        $headers_map.get_all($key).into_iter().any(|val| {
            if $ignore_case {
                val.as_bytes().to_ascii_lowercase().starts_with($target_val.to_ascii_lowercase().as_slice())
            } else {
                val.as_bytes().starts_with($target_val)
            }
        })
    };
}
