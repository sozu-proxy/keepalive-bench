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
use tokio_core::net::TcpStream;

use futures::*;
use futures::stream::Stream;

use hyper::{Client, Uri, Error, header};
use hyper::client::{HttpConnector,Service};

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
    wtr.write_record(&["time", "client", "status", "instance", "front port", "back port"]);

    let shared_wtr = Arc::new(Mutex::new(wtr));
    let shared_status = Arc::new(Mutex::new(Status::new()));
    let start = time::Instant::now();

    let work = future::join_all((0..10).map(|id| {
      let client = HttpClient::new(&handle, id, shared_wtr.clone(), start.clone(), shared_status.clone());

      let stream = stream::repeat::<_, Error>(id);
      let u = url.clone();

      stream.for_each(move |_| {
        client.call(&u)
      })
    }));

    core.run(work).unwrap();
}

struct Status {
  success: u64,
  failure: u64,
  backend_connection_closed: u64,
  frontend_connection_closed: u64,
}

impl Status {
  pub fn new() -> Status {
    Status {
      success: 0,
      failure: 0,
      backend_connection_closed: 0,
      frontend_connection_closed: 0,
    }
  }
}

struct HttpClient {
  client: Client<DebugConnector>,
  id:     u32,
  writer: Arc<Mutex<csv::Writer<File>>>,
  start:  time::Instant,
  status: Arc<Mutex<Status>>,
  backend_port: Arc<Mutex<Option<u32>>>,
  frontend_port: Arc<Mutex<Option<u16>>>,
}

impl HttpClient {
  pub fn new(handle: &Handle, id: u32, writer: Arc<Mutex<csv::Writer<File>>>, start: time::Instant, status: Arc<Mutex<Status>>) -> HttpClient {
    let frontend_port = Arc::new(Mutex::new(None));

    let client = Client::configure()
        .connector(DebugConnector(HttpConnector::new(1, &handle), frontend_port.clone(), status.clone()))
        //.no_proto() if you do that there will be no keep alive
        .keep_alive(true)
        // default true: .keep_alive(true)
        .build(&handle);

    let backend_port = Arc::new(Mutex::new(None));
    HttpClient { client, id, writer, start, status, backend_port, frontend_port }
  }

  pub fn call(&self, url: &Uri) -> impl Future<Item = (), Error = hyper::Error> {
    let id:  u32 = self.id.clone();
    let id2: u32 = self.id.clone();
    let shared_writer  = self.writer.clone();
    let shared_writer2 = self.writer.clone();
    let start  = self.start.clone();
    let start2 = self.start.clone();
    let status  = self.status.clone();
    let status2 = self.status.clone();

    let frontend_port = self.frontend_port.clone();
    let backend_port = self.backend_port.clone();

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

          let port_number = if let Ok(mut port) = frontend_port.try_lock() {
              if let Some(p) = *port {
                format!("{}", p)
              } else {
              "".to_string()
              }
            } else {
              "".to_string()
            };

          print!("\r[{}] client: {} status: {} front: {} instance: {} back: {}                           ",
            elapsed, id, status_code, port_number, backend_id, backend_connection_port);

          if let Ok(mut st) = status.try_lock() {
            st.success += 1;

            if let Ok(mut port) = backend_port.try_lock() {
              if let Some(p) = *port {
                if p != backend_connection_port {
                  st.backend_connection_closed += 1;
                }
              }

              *port = Some(backend_connection_port);

            }

            print!("\n[{}] success: {} failure: {}, front changed: {} back changed: {}",
              elapsed, st.success, st.failure, st.frontend_connection_closed, st.backend_connection_closed);
            io::stdout().flush().unwrap();
          }

          if let Ok(mut writer) = shared_writer.try_lock() {
            writer.write_record(&[format!("{}", elapsed), format!("{}", id), format!("{}", status_code),
              format!("{}", backend_id), port_number, format!("{}", backend_connection_port)]);
          }
        } else {
          let port_number = if let Ok(mut port) = frontend_port.try_lock() {
              if let Some(p) = *port {
                format!("{}", p)
              } else {
              "".to_string()
              }
            } else {
              "".to_string()
            };

          print!("\r[{}] client: {} status: {} front: {} backend not available                      ",
            elapsed, id, status_code, port_number);

          if let Ok(mut st) = status.try_lock() {
            st.failure += 1;
            print!("\n[{}] success: {} failure: {}, front changed: {} back changed: {}",
              elapsed, st.success, st.failure, st.frontend_connection_closed, st.backend_connection_closed);
            io::stdout().flush().unwrap();
          }

          if let Ok(mut writer) = shared_writer.try_lock() {
            if let Ok(mut port) = frontend_port.try_lock() {

              writer.write_record(&[format!("{}", elapsed), format!("{}", id), format!("{}", status_code),
                "".to_string(), port_number, "".to_string()]);
            }
          }
        }

        Ok(())
    }).or_else(move |e| {
      let duration = start2.elapsed();
      let secs     = duration.as_secs();
      let nano     = duration.subsec_nanos();
      let elapsed  = (nano / 1000000) as u64 + (secs * 1000);

      print!("\r[{}] client: {} got error: {:?}                                           ", elapsed, id, e);
      if let Ok(mut st) = status2.try_lock() {
        st.failure += 1;
        print!("\n[{}] success: {} failure: {}, front changed: {} back changed: {}",
          elapsed, st.success, st.failure, st.frontend_connection_closed, st.backend_connection_closed);
        io::stdout().flush().unwrap();
      }

      if let Ok(mut writer) = shared_writer2.try_lock() {
        writer.write_record(&[format!("{}", elapsed), format!("{}", id2), format!("{}", e),
          "".to_string(), "".to_string(), "".to_string()]);
      }
      Ok(())
    })
    /*.map(|_| {
        println!("\n\nDone.");
    });*/
  }
}

struct DebugConnector(HttpConnector, Arc<Mutex<Option<u16>>>, Arc<Mutex<Status>>);

impl Service for DebugConnector {
  type Request = Uri;
  type Response = TcpStream;
  type Error = io::Error;
  type Future = Box<Future<Item = TcpStream, Error = io::Error>>;

  fn call(&self, uri: Uri) -> Self::Future {
    let saved_port = self.1.clone();
    let status     = self.2.clone();

    Box::new(self.0.call(uri).map(move |s| {
      let port = s.local_addr().expect("there shuld be an address").port();
println!("new port: {}", port);
      if let Ok(mut p) = saved_port.try_lock() {
        *p = Some(port);
      }

      if let Ok(mut st) = status.try_lock() {
        st.frontend_connection_closed += 1;
      }

      s
    }))
  }
}
