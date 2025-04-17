use smol::channel::Sender;

#[derive(Debug)]
pub struct PanicDetails {
    pub thread_name: &'static str,
}

pub fn send_panic(thread_name: &'static str, thread_panic_tx: Sender<PanicDetails>) {
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        smol::block_on(async {
            let _ = thread_panic_tx.send(PanicDetails { thread_name }).await;
        });
        old_hook(panic_info);
    }));
}
