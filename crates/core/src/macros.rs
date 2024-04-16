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
    ($headers_map: expr, $key:expr, $target_val:expr) => {{
        use memmem::{Searcher, TwoWaySearcher};
        use std::rc::Rc;

        let searcher = Rc::new(TwoWaySearcher::new($target_val));

        $headers_map.get_all($key).into_iter().any(|val| searcher.search_in(val.as_bytes()).is_some())
    }};

    ($headers_map: expr, $key:expr, $target_val:expr, ignore_case: $ignore_case:expr) => {{
        use memmem::{Searcher, TwoWaySearcher};
        use std::rc::Rc;

        $headers_map.get_all($key).into_iter().any(|val| {
            if $ignore_case {
                let target = $target_val.to_ascii_lowercase();
                let target = target.as_slice();

                let searcher = Rc::new(TwoWaySearcher::new(target));
                searcher.search_in(val.as_bytes().to_ascii_lowercase().as_slice()).is_some()
            } else {
                let searcher = Rc::new(TwoWaySearcher::new($target_val));
                searcher.search_in(val.as_bytes()).is_some()
            }
        })
    }};
}
