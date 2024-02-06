#[macro_export]
macro_rules! route_with_method {
    ($app:expr, $method:expr, $endpoint:expr, $handler:expr) => {
        match $method {
            $crate::structs::HttpMethod::Get => $app.route($endpoint, axum::routing::get($handler)),
            $crate::structs::HttpMethod::Post => {
                $app.route($endpoint, axum::routing::post($handler))
            }
            $crate::structs::HttpMethod::Put => $app.route($endpoint, axum::routing::put($handler)),
            $crate::structs::HttpMethod::Patch => {
                $app.route($endpoint, axum::routing::patch($handler))
            }
            $crate::structs::HttpMethod::Delete => {
                $app.route($endpoint, axum::routing::delete($handler))
            }
        }
    };
}
