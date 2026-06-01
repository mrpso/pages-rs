use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[unsafe(no_mangle)]
pub fn start() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("rust-main")
        .build()
        .unwrap();
    if let Err(e) = runtime.block_on(app()) {
        eprintln!("Error: {:?}", e);
    }
}

async fn app() -> std::io::Result<()> {
    println!("构建 App 路由！");
    let router = Router::new()
        .route("/api/abc", get(|| async { "Hello, abc!" }))
        .route("/api/efg", get(|| async { "Hello, efg!" }))
        .route("/api/hello", get(|| async { "Hello, world!" }))
        .with_state(0);

    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    axum::serve(listener, router).await
}
