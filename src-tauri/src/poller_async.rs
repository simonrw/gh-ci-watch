use std::time::Duration;

use tauri::async_runtime::{self, channel, JoinHandle, Receiver, Sender};
use tauri::{AppHandle, Manager};
use tokio::time;

pub enum Command {
    AddPr(u64),
    RemovePr(u64),
    ClearPrs,
    Tick,
}

pub struct Poller {
    receiver: Receiver<Command>,
    prs: Vec<u64>,
    handle: AppHandle,
    heartbeat_handle: JoinHandle<()>,
}

async fn heartbeat(tx: Sender<Command>) {
    let mut interval = time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let _ = tx.send(Command::Tick).await;
            },
        }
    }
}

impl Poller {
    pub fn new(
        internal_queue: Sender<Command>,
        receiver: Receiver<Command>,
        handle: AppHandle,
    ) -> Self {
        let heartbeat_handle = async_runtime::spawn(async move {
            heartbeat(internal_queue).await;
        });

        Self {
            receiver,
            prs: Vec::new(),
            handle,
            heartbeat_handle,
        }
    }

    fn handle_command(&mut self, cmd: Command) {
        let mut should_emit = true;
        match cmd {
            Command::AddPr(number) => {
                self.prs.push(number);
            }
            Command::RemovePr(number) => {
                // TODO: remove the clone
                self.prs = self
                    .prs
                    .clone()
                    .into_iter()
                    .filter(|pr| *pr != number)
                    .collect()
            }
            Command::ClearPrs => self.prs.clear(),
            Command::Tick => {
                tracing::debug!("got heartbeat tick");
                should_emit = false;
            }
        }
        if should_emit {
            self.handle.emit_all("state", self.prs.clone()).unwrap();
        }
    }
}

#[derive(Clone)]
pub struct Handle {
    sender: Sender<Command>,
}

impl Handle {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let (sender, receiver) = channel(100);
        let heartbeat_handle = sender.clone();
        let poller = Poller::new(heartbeat_handle, receiver, app_handle);
        async_runtime::spawn(run_poller(poller));
        Self { sender }
    }

    pub fn add_pr(&self, pr_number: u64) {
        let cmd = Command::AddPr(pr_number);
        if let Err(e) = self.sender.blocking_send(cmd) {
            tracing::warn!(error = %e, pr = %pr_number, "poller is down");
        }
    }

    pub fn remove_pr(&self, pr_number: u64) {
        let cmd = Command::RemovePr(pr_number);
        if let Err(e) = self.sender.blocking_send(cmd) {
            tracing::warn!(error = %e, pr = %pr_number, "poller is down");
        }
    }

    pub fn clear_prs(&self) {
        let cmd = Command::ClearPrs;
        if let Err(e) = self.sender.blocking_send(cmd) {
            tracing::warn!(error = %e, "poller is down");
        }
    }
}

pub async fn run_poller(mut poller: Poller) {
    while let Some(cmd) = poller.receiver.recv().await {
        poller.handle_command(cmd);
    }
}
