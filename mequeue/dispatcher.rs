use {crate::worker::Worker, async_channel::Receiver};

type Maybe<T1> = std::option::Option<T1>;
type Mut<T1> = std::sync::Arc<std::sync::Mutex<T1>>;

pub async fn dispatch<E1: Clone, W1>(state: Mut<Maybe<E1>>, receiver: Receiver<E1>, worker: W1)
where
	W1: Worker<E1>,
{
	let maybe = { state.lock().unwrap().clone() };

	if let Some(event) = maybe {
		// Try to handle last not properly handled value

		worker.execute(event.clone()).await;

		state.lock().unwrap().take();
	}

	let update = |event: E1| {
		let mut state = state.lock().unwrap();

		*state = Some(event);
	};

	while let Ok(event) = receiver.recv().await {
		// Keep event in state to ensure cancel safety

		(update(event.clone()), worker.execute(event).await);

		state.lock().unwrap().take();
	}
}
