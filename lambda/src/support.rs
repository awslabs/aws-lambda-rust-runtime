// Borrowed from https://github.com/seanmonstar/reqwest/blob/master/tests/client.rs

use crate::client::MakeSvc;
pub use http::{Request, Response};
use std::{net, sync::mpsc as std_mpsc, thread, time::Duration};
use tokio::{runtime, sync::oneshot};

pub struct Server {
    addr: net::SocketAddr,
    panic_rx: std_mpsc::Receiver<()>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Server {
    pub fn addr(&self) -> net::SocketAddr {
        self.addr
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if !::std::thread::panicking() {
            self.panic_rx
                .recv_timeout(Duration::from_secs(3))
                .expect("test server should not panic");
        }
    }
}

pub fn http(svc: MakeSvc) -> Server {
    // Spawn new runtime in thread to prevent reactor execution context conflict
    thread::spawn(move || {
        let mut rt = runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .expect("new rt");
        let srv =
            rt.block_on(async move { hyper::Server::bind(&([127, 0, 0, 1], 0).into()).serve(svc) });

        let addr = srv.local_addr();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let srv = srv.with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });

        let (panic_tx, panic_rx) = std_mpsc::channel();
        let tname = format!(
            "test({})-support-server",
            thread::current().name().unwrap_or("<unknown>")
        );
        thread::Builder::new()
            .name(tname)
            .spawn(move || {
                rt.block_on(srv).unwrap();
                let _ = panic_tx.send(());
            })
            .expect("thread spawn");

        Server {
            addr,
            panic_rx,
            shutdown_tx: Some(shutdown_tx),
        }
    })
    .join()
    .unwrap()
}
