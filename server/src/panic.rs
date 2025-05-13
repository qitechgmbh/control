use smol::channel::Sender;

pub fn send_panic<T>(signal: T, thread_panic_tx: Sender<T>)
where
    T: Send + Sync + Clone + 'static,
{
    let old_hook = std::panic::take_hook();
    let signal_clone = signal.clone();
    std::panic::set_hook(Box::new(move |panic_info| {
        smol::block_on(async {
            let _ = thread_panic_tx.send(signal_clone.clone()).await;
        });
        old_hook(panic_info);
    }));
}

pub fn send_panic_error<T>(signal: T, thread_panic_tx: Sender<(T, anyhow::Error)>)
where
    T: Send + Sync + Clone + 'static,
{
    let old_hook = std::panic::take_hook();
    let signal_clone = signal.clone();
    std::panic::set_hook(Box::new(move |panic_info| {
        smol::block_on(async {
            let _ = thread_panic_tx
                .send((
                    signal_clone.clone(),
                    anyhow::anyhow!("Thread panicked: {}", panic_info.to_string()),
                ))
                .await;
        });
        old_hook(panic_info);
    }));
}
