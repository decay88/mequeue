use {crate::worker::MappedWorker, async_channel::Receiver};

pub type Wal<T1> = std::sync::Arc<std::sync::Mutex<std::option::Option<T1>>>;

pub async fn dispatch<E1: Clone, W1>(wal: Wal<E1>, receiver: Receiver<E1>, worker: W1)
where
	W1: MappedWorker<E1>,
{
	let maybe = { wal.lock().unwrap().clone() };

	if let Some(event) = maybe {
		// Try to handle last not properly handled value

		worker.execute(event.clone()).await;

		wal.lock().unwrap().take();
	}

	let update = |event: E1| {
		let mut wal = wal.lock().unwrap();

		*wal = Some(event);
	};

	while let Ok(event) = receiver.recv().await {
		// Keep event in state to ensure cancel safety

		(update(event.clone()), worker.execute(event).await);

		wal.lock().unwrap().take();
	}
}
