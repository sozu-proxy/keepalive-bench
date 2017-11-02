#![feature(conservative_impl_trait)]

extern crate futures;
extern crate hyper;
extern crate tokio_core;

extern crate pretty_env_logger;

use std::env;
use std::io::{self, Write};

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
    /*
    let client = Client::configure()
        .no_proto()
        // default true: .keep_alive(true)
        .build(&handle);
    */


    let work = future::join_all((1..10).map(|id| {
      let client = HttpClient::new(&handle, id);

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
}

impl HttpClient {
  pub fn new(handle: &Handle, id: u32) -> HttpClient {
    let client = Client::configure()
        .no_proto()
        // default true: .keep_alive(true)
        .build(&handle);

    HttpClient { client, id }
  }

  pub fn call(&self, url: &Uri) -> impl Future<Item = (), Error = hyper::Error> {
    let id: u32 = self.id.clone();
    self.client.get(url.clone()).and_then(move |res| {
        println!("[{}] Response ({}): {}", id, res.version(), res.status());
        println!("Headers: \n{}", res.headers());
        let conn: Option<&header::Connection> = res.headers().get();
        println!("Connection: {:?}", conn);

        /*
        res.body().for_each(|chunk| {
            io::stdout().write_all(&chunk).map_err(From::from)
        })
        */
        Ok(())
    })
    /*.map(|_| {
        println!("\n\nDone.");
    });*/
  }
}
