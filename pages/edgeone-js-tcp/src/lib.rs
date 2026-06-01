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
        let path = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            match true {
                cfg!(windows) => format!(r"\\.\pipe\pages-{}", time.as_millis()),
                cfg!(unix) => format!("/tmp/pages-{}", time.as_millis()),
            }
        };

        #[cfg(unix)]
        if let Ok(ipc) = UnixListener::bind(&path) {
            tokio::spawn(axum::serve(ipc, router).into_future());
            return Channel::IPC(path);
        }

        #[cfg(windows)]
        {
            use tokio::net::windows::named_pipe::*;

            pub struct PipeListener {
                path: String,
                first: bool,
            }

            impl PipeListener {
                pub fn bind(path: impl ToString) -> std::io::Result<Self> {
                    let (path, first) = (path.to_string(), true);
                    ServerOptions::new().create(path.as_str())?;

                    Ok(Self { path, first })
                }

                async fn accept(&mut self) -> std::io::Result<NamedPipeServer> {
                    let server = ServerOptions::new()
                        .first_pipe_instance(self.first)
                        .create(self.path.as_str())?;
                    self.first = self.first && false;
                    server.connect().await?;

                    Ok(server)
                }
            }

            impl axum::serve::Listener for PipeListener {
                type Io = NamedPipeServer;

                type Addr = String;

                async fn accept(&mut self) -> (Self::Io, Self::Addr) {
                    loop {
                        match self.accept().await {
                            Ok(server) => return (server, self.path.clone()),
                            Err(e) => {
                                eprintln!("Windows 命名管道监听失败: {}\n详细信息: {:#?}", e, e);
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            }
                        };
                    }
                }

                fn local_addr(&self) -> tokio::io::Result<Self::Addr> {
                    Ok(self.path.clone())
                }
            }

            if let Ok(pipe) = PipeListener::bind(&path) {
                tokio::spawn(axum::serve(pipe, router).into_future());
                return Channel::IPC(path);
            }
        }

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
