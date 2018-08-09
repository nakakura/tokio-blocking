extern crate futures;
extern crate tokio;
extern crate tokio_threadpool;
extern crate chrono;

use futures::*;
use futures::future::{err, ok};
use tokio_threadpool::{blocking};
use chrono::*;

use std::thread;
use std::error::Error;

pub struct WaitInAnotherThread {
    token: usize,
    end_time: DateTime<Utc>,
    running: bool,
}

impl WaitInAnotherThread {
    pub fn new(token: usize, how_long: Duration) -> WaitInAnotherThread {
        WaitInAnotherThread {
            token: token,
            end_time: Utc::now() + how_long,
            running: false,
        }
    }
}

impl Future for WaitInAnotherThread {
    type Item = ();
    type Error = Box<Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        blocking(|| {
            println!("start {}, now: {:?}, end_time: {:?}", self.token, Utc::now(), self.end_time);
            while Utc::now() < self.end_time {
                let delta_sec = self.end_time.timestamp() - Utc::now().timestamp();
                println!("{} will sleep {:?}", self.token, delta_sec);
                if delta_sec > 0 {
                    thread::sleep(::std::time::Duration::from_secs(delta_sec as u64));
                }
            }
        });
        println!("id: {} has ended. the time has come == {:?}!", self.token, self.end_time);
        Ok(Async::Ready(()))
    }
}

fn main() {
    let mut builder = tokio::executor::thread_pool::Builder::new();
    builder.pool_size(1).max_blocking(3);
    let mut core = tokio::runtime::Builder::new().threadpool_builder(builder).build().unwrap();

    let wiat = WaitInAnotherThread::new(0, Duration::seconds(3));
    let wiat2 = WaitInAnotherThread::new(1, Duration::seconds(2));
    let wiat3 = WaitInAnotherThread::new(2, Duration::seconds(10));
    let wiat4 = WaitInAnotherThread::new(3, Duration::seconds(10));
    let w = wiat3.and_then(|_| wiat4);
    println!("wait future started");
    //let ret = core.run(wiat).unwrap();
    core.spawn(wiat.map_err(|_| ()));
    core.spawn(wiat2.map_err(|_| ()));
    core.shutdown_on_idle().wait().unwrap();

    println!("next");
    let mut builder = tokio::executor::thread_pool::Builder::new();
    builder.pool_size(1).max_blocking(3);
    let mut core = tokio::runtime::Builder::new().threadpool_builder(builder).build().unwrap();
    core.spawn(w.map_err(|_| ()));
    core.shutdown_on_idle().wait().unwrap();
}
