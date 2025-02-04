#[macro_export]
macro_rules! web_server {
    ($route:expr) => {{
        let (tx_server, rx_server) = std::sync::mpsc::channel();
        let (tx_address, rx_address) = std::sync::mpsc::channel();

        let server = std::thread::spawn(move || {
            let sys = actix_web::rt::System::new();
            let srv = actix_web::HttpServer::new(move || actix_web::App::new().service($route))
                .bind(("127.0.0.1", 0))
                .unwrap();

            tx_address.send(format!("http://{:#}/", srv.addrs().first().unwrap())).unwrap();

            let srv = srv.run();

            tx_server.send(srv.handle()).unwrap();
            sys.block_on(srv).unwrap();
        });

        (server, rx_server.recv().unwrap(), rx_address.recv().unwrap())
    }};
}
