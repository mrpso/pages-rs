use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::LazyLock,
};

use napi::bindgen_prelude::*;
use tokio::{net::*, runtime::Runtime};

pub enum Channel {
    IPC(String),
    TCP(SocketAddr),
}

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());
static CHANNEL: LazyLock<Channel> = LazyLock::new(|| {
    let instant = std::time::Instant::now();
    RUNTIME.block_on(async {
        let router = pages::router();

        #[cfg(any(unix, windows))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};

            let random = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(time) => time.as_millis(),
                Err(_) => instant.elapsed().as_nanos(),
            };

            let path = match true {
                cfg!(unix) => std::env::temp_dir()
                    .join(format!("pages-{}", random))
                    .display()
                    .to_string(),
                cfg!(windows) => format!(r"\\.\pipe\pages-{}", random),
            };

            #[cfg(unix)]
            let ipc = tokio::net::UnixListener::bind(&path);
            #[cfg(windows)]
            let ipc = pages_rs::windows::PipeListener::bind(&path);
            if let Ok(ipc) = ipc {
                tokio::spawn(axum::serve(ipc, router).into_future());
                return Channel::IPC(path);
            }
        };

        let number = instant.elapsed().as_nanos() % 0xFFFFFC;
        let ip = Ipv4Addr::from(number as u32 + 0x7f000001);
        let tcp = TcpListener::bind((ip, 0)).await.unwrap();
        let addr = tcp.local_addr().unwrap();
        tokio::spawn(axum::serve(tcp, router).into_future());
        Channel::TCP(addr)
    })
});

#[napi_derive::napi(module_exports)]
pub fn init(mut exports: Object) -> Result<()> {
    match std::ops::Deref::deref(&CHANNEL) {
        Channel::IPC(path) => exports.set_named_property("path", path)?,
        Channel::TCP(addr) => {
            exports.set_named_property("host", addr.ip().to_string())?;
            exports.set_named_property("port", addr.port())?;
        }
    };

    Ok(())
}
