use tauri::async_runtime::{self, channel, Receiver, Sender};
use tauri::{AppHandle, Manager};

pub enum Command {
    AddPr(u64),
    RemovePr(u64),
    ClearPrs,
}

pub struct Poller {
    receiver: Receiver<Command>,
    prs: Vec<u64>,
    handle: AppHandle,
}

impl Poller {
    pub fn new(receiver: Receiver<Command>, handle: AppHandle) -> Self {
        Self {
            receiver,
            prs: Vec::new(),
            handle,
        }
    }

    fn handle_command(&mut self, cmd: Command) {
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
        }
        self.handle.emit_all("state", self.prs.clone()).unwrap();
    }
}

#[derive(Clone)]
pub struct Handle {
    sender: Sender<Command>,
}

impl Handle {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let (sender, receiver) = channel(100);
        let poller = Poller::new(receiver, app_handle);
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
