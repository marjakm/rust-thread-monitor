#[macro_use]
extern crate log;

use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};


pub fn spawn_monitored_thread<F>(name: &str, sender: Sender<String>, f: F)
    where F: Send + 'static + FnOnce()
{
    let nam = name.to_string();
    thread::Builder::new().name(format!("{} thread", name)).spawn(move || {
        let _s = Sentinel::new(nam, sender);
        f();
    }).expect(&format!("cannot create {} thread", name));
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

    pub fn sender(&self) -> Sender<String> {
        self.sender.clone()
    }

    pub fn run(&self) -> Result<String, RecvError> {
        self.receiver.recv()
    }

    pub fn spawn_monitored_thread<F>(&self, name: &str, f: F)
        where F: Send + 'static + FnOnce()
    {
        spawn_monitored_thread(name, self.sender.clone(), f);
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
