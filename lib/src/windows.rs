use axum::serve::Listener;
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

impl Listener for PipeListener {
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
