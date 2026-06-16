use tokio::sync::mpsc;

pub fn run() {

}

pub async fn task(mut receiver: mpsc::Receiver<property::PropertySet>) {
    loop {
        let Some(snapshot) = receiver.recv().await else {
            // channel closed
            return;
        };


    }
}
