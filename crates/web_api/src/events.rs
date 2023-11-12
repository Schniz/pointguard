#[derive(Debug, Clone)]
pub struct EnqueuedTasks {
    pub tx: flume::Sender<usize>,
    pub rx: flume::Receiver<usize>,
}

impl From<(flume::Sender<usize>, flume::Receiver<usize>)> for EnqueuedTasks {
    fn from((tx, rx): (flume::Sender<usize>, flume::Receiver<usize>)) -> Self {
        Self { tx, rx }
    }
}
