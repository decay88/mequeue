use std::time::Duration;

use async_channel as mpmc;
use tokio::sync::{broadcast, broadcast::Sender, mpsc};
use tokio::time::sleep;

use crate::executor::Executor;

type Ref<T1> = std::sync::Arc<T1>;

async fn new() -> (mpmc::Sender<u8>, Sender<u8>, mpsc::Receiver<(u8, u8)>) {
	//

	let (we, event) = async_channel::bounded(512);
	let (ws, state) = broadcast::channel(512);

	let executor = Executor::new(state, event, 12);

	let (wk, check) = mpsc::channel(512);

	let worker = move |state: Ref<u8>, event| {
		let (state, wk) = (state.clone(), wk.clone());

		async move {
			wk.send((*state, event)).await.unwrap();

			sleep(Duration::from_secs(100)).await;
		}
	};
	tokio::spawn(executor.receive(worker));

	(we, ws, check)
}

#[tokio::test]
async fn wal() {
	// Spawn new executor that will satisfy our requirements.
	let (we, ws, mut check) = new().await;

	// Send event to channel but there are no workers yet.
	we.send(0u8).await.unwrap();

	// Update state and workers will be started.
	ws.send(0).unwrap();

	// Check that worker received event.
	let val = check.recv().await.unwrap();

	assert_eq!(val, (0, 0));

	// Event is not processed because worker is not finished it's execution.
	// Worker not finished it's execution because there is sleep in it.
	// By sending new state we will abort execution of worker.
	// Then worker will be restarted. There is an event remaining in wal.
	// Wal was not cleared because event was not processed.
	ws.send(1).unwrap();

	// After restart worker will try to process event from wal first.
	let val = check.recv().await.unwrap();

	assert_eq!(val, (1, 0));

	()
}
