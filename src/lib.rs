#[macro_use]
extern crate log;

use std::thread::{Builder, JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};


pub fn spawn_monitored_thread<F, T>(name: &str, sender: Sender<String>, f: F) -> JoinHandle<T>
    where F: FnOnce() -> T,
          F: Send + 'static,
          T: Send + 'static
{
    let nam = name.to_string();
    Builder::new().name(format!("{} thread", name)).spawn(move || {
        let _s = Sentinel::new(nam, sender);
        f()
    }).expect(&format!("cannot create {} thread", name))
}

pub struct ThreadMonitor {
    sender:   Sender<String>,
    receiver: Receiver<String>,
}

impl ThreadMonitor {
    pub fn new() -> Self {
        let (s,r) = channel();
        ThreadMonitor {
            sender:   s,
            receiver: r,
        }
    }

    pub fn spawner(&self) -> MonitoredThreadSpawner {
        MonitoredThreadSpawner {
            sender: self.sender.clone()
        }
    }

    pub fn sender(&self) -> Sender<String> {
        self.sender.clone()
    }

    pub fn run(&self) -> Result<String, RecvError> {
        self.receiver.recv()
    }

    pub fn spawn_monitored_thread<F, T>(&self, name: &str, f: F) -> JoinHandle<T>
        where F: FnOnce() -> T,
              F: Send + 'static,
              T: Send + 'static
    {
        spawn_monitored_thread(name, self.sender.clone(), f)
    }
}

pub struct MonitoredThreadSpawner {
    sender: Sender<String>
}

impl MonitoredThreadSpawner {
    pub fn spawn_monitored_thread<F, T>(&self, name: &str, f: F) -> JoinHandle<T>
        where F: FnOnce() -> T,
              F: Send + 'static,
              T: Send + 'static
    {
        spawn_monitored_thread(name, self.sender.clone(), f)
    }
}

struct Sentinel {
    name:  String,
    sender: Sender<String>,
}

impl Sentinel {
    pub fn new(name: String, sender: Sender<String>) -> Self {
        Sentinel {
            name:   name,
            sender: sender,
        }
    }
}

impl Drop for Sentinel {
    fn drop(&mut self) {
        match self.sender.send(self.name.clone()) {
            Ok(()) => debug!("Sentinel exit: {}", self.name),
            Err(e) => error!("Sentinel exit: {} error: {:?}", self.name, e)
        }
    }
}
