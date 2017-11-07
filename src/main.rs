#![feature(conservative_impl_trait)]

extern crate csv;
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate pretty_env_logger;

use std::{env,str,time};
use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use tokio_core::reactor::Handle;
use futures::*;
use futures::stream::Stream;

use hyper::{Client, Uri, Error, header};
use hyper::client::HttpConnector;

fn main() {
    pretty_env_logger::init().unwrap();

    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return;
        }
    };

    let url = url.parse::<hyper::Uri>().unwrap();
    if url.scheme() != Some("http") {
        println!("This example only works with 'http' URLs.");
        return;
    }

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let mut wtr = csv::Writer::from_path("data.csv").expect("could not open path to data file");
    wtr.write_record(&["time", "client", "status", "backend", "source"]);

    let shared_wtr = Arc::new(Mutex::new(wtr));
    let start = time::Instant::now();

    let work = future::join_all((1..10).map(|id| {
      let client = HttpClient::new(&handle, id, shared_wtr.clone(), start.clone());

      let stream = stream::repeat::<_, Error>(id);
      let u = url.clone();

      stream.for_each(move |_| {
        client.call(&u)
      })
    }));

    core.run(work).unwrap();
}

struct HttpClient {
  client: Client<HttpConnector>,
  id:     u32,
  writer: Arc<Mutex<csv::Writer<File>>>,
  start:  time::Instant,
}

impl HttpClient {
  pub fn new(handle: &Handle, id: u32, writer: Arc<Mutex<csv::Writer<File>>>, start: time::Instant) -> HttpClient {
    let client = Client::configure()
        //.no_proto() if you do that there will be no keep alive
        .keep_alive(true)
        // default true: .keep_alive(true)
        .build(&handle);

    HttpClient { client, id, writer, start }
  }

  pub fn call(&self, url: &Uri) -> impl Future<Item = (), Error = hyper::Error> {
    let id: u32  = self.id.clone();
    let id2: u32 = self.id.clone();
    let shared_writer  = self.writer.clone();
    let shared_writer2 = self.writer.clone();
    let start  = self.start.clone();
    let start2 = self.start.clone();

    self.client.get(url.clone()).and_then(move |res| {
        let duration = start.elapsed();
        let secs     = duration.as_secs();
        let nano     = duration.subsec_nanos();
        let elapsed  = (nano / 1000000) as u64 + (secs * 1000);

        let status_code = res.status().as_u16();

        if let Some(backend_id_header) = res.headers().get_raw("Backend-Id") {
          let backend_id: u32 = backend_id_header.one().and_then(|val| str::from_utf8(val).ok())
                          .expect("there should be only one value")
                          .parse().expect("could not parse id");
          let backend_connection_port: u32 = res.headers().get_raw("Source-Port").expect("Source-Port header not found")
                          .one().and_then(|val| str::from_utf8(val).ok()).expect("there should be only one value")
                          .parse().expect("could not parse id");

          println!("[{}] client: {} status: {} backend: {} port: {}", elapsed, id, status_code, backend_id, backend_connection_port);

          if let Ok(mut writer) = shared_writer.try_lock() {
            writer.write_record(&[format!("{}", elapsed), format!("{}", id), format!("{}", status_code),
              format!("{}", backend_id), format!("{}", backend_connection_port)]);
          }
        } else {
          println!("[{}] client: {} status: {} backend not available", elapsed, id, status_code);

          if let Ok(mut writer) = shared_writer.try_lock() {
            writer.write_record(&[format!("{}", elapsed), format!("{}", id), format!("{}", status_code),
              "".to_string(), "".to_string()]);
          }
        }

        Ok(())
    }).or_else(move |e| {
      let duration = start2.elapsed();
      let secs     = duration.as_secs();
      let nano     = duration.subsec_nanos();
      let elapsed  = (nano / 1000000) as u64 + (secs * 1000);

      println!("[{}] client: {} got error: {:?}", elapsed, id, e);
      if let Ok(mut writer) = shared_writer2.try_lock() {
        writer.write_record(&[format!("{}", elapsed), format!("{}", id2), format!("{}", e),
          "".to_string(), "".to_string()]);
      }
      Ok(())
    })
    /*.map(|_| {
        println!("\n\nDone.");
    });*/
  }
}
