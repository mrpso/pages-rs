use std::{collections::HashMap, sync::OnceLock};

use axum::{Router, body::Body};
use http_body_util::BodyExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use tower::ServiceExt;

static ROUTER: OnceLock<Router> = OnceLock::new();

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

#[napi]
pub async fn http(req: Request) -> Response {

    // let router = ROUTER.get_or_init(|| pages::router());
    let router = ROUTER.get_or_init(|| Router::new());
    let mut builder = http::Request::builder()
        .method(req.method.as_bytes())
        .uri(req.url.as_bytes());

    for (key, value) in req.headers {
        builder = builder.header(key, value);
    }

    let body = Body::from(req.body.map(|x| x.to_vec()).unwrap_or_default());
    let request = builder.body(body).unwrap();

    println!("Request: {:#?}", request);

    let response = router.clone().oneshot(request).await.unwrap();
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

// impl From<Request> for http::Request<Body> {
//     fn from(value: Request) -> Self {
//         let mut request = http::Request::builder();

//         let a = request.headers_mut();

//         todo!()
//     }
// }

impl Request {
    pub fn http(self) -> http::Request<Body> {
        let mut builder = http::Request::builder()
            .method(self.method.as_bytes())
            .uri(self.url.as_bytes());

        for (key, value) in self.headers {
            builder = builder.header(key, value);
        }

        let body = Body::from(self.body.map(|x| x.to_vec()).unwrap_or_default());
        builder.body(body).unwrap()
    }
}
