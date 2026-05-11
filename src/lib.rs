use std::{collections::HashMap, sync::LazyLock};

use axum::{Router, body::Body, routing::get};
use http_body_util::BodyExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use tower::ServiceExt;

#[napi(object)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Buffer>,
}

#[napi(object)] // 导出为 JS 纯对象
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<Buffer>,
}

static APP: LazyLock<Router> = LazyLock::new(|| {
    println!("构建 App 路由！");
    Router::new()
        .route("/api/abc", get(|| async { "Hello, abc!" }))
        .route("/api/efg", get(|| async { "Hello, efg!" }))
        .route("/api/hello", get(|| async { "Hello, world!" }))
        .with_state(0)
});

#[napi]
pub async fn http(req: Request) -> Response {
    let mut builder = http::Request::builder()
        .method(req.method.as_bytes())
        .uri(req.url.as_bytes());

    for (key, value) in req.headers {
        builder = builder.header(key, value);
    }

    let body = Body::from(req.body.map(|x| x.to_vec()).unwrap_or_default());
    let request = builder.body(body).unwrap();

    println!("Request: {:#?}", request);

    let response = APP.clone().oneshot(request).await.unwrap();
    Response {
        status: response.status().as_u16(),
        headers: response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect(),
        body: match response.into_body().collect().await {
            Ok(collected) => Some(Buffer::from(collected.to_bytes().to_vec())),
            Err(_) => None,
        },
    }
}
