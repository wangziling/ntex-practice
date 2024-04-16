macro_rules! header_values {
    ($prefix: tt, $value: tt) => {
        paste::item! {
          pub const $prefix: &str = $value;
          pub const [<$prefix _BYTES>]: &[u8] = $value.as_bytes();
        }
    };
}

pub const REQUESTED_WITH_HEADER_NAME: &str = "x-requested-with";

header_values!(JSON_HEADER_VALUE, "application/json");
header_values!(FORM_HEADER_VALUE, "application/x-www-form-urlencoded");
header_values!(FORM_DATA_HEADER_VALUE, "multipart/form-data");
header_values!(REQUESTED_WITH_AJAX_HEADER_VALUE, "XMLHttpRequest");
