use async_channel as mpmc;
use std::sync::Mutex;

use tokio::{sync::broadcast, task::JoinSet};

use crate::{dispatcher, worker::Worker};

type Ref<T1> = std::sync::Arc<T1>;

pub struct Executor<S1, E1> {
	state: broadcast::Receiver<S1>,
	event: mpmc::Receiver<E1>,

	inner: JoinSet<()>,
	count: usize,

	wal: Vec<dispatcher::Wal<E1>>,
}

impl<S1: Clone, E1: Clone> Executor<S1, E1>
where
	S1: Send + Sync + 'static,
	E1: Send + Sync + 'static,
{
	pub fn new(state: broadcast::Receiver<S1>, event: mpmc::Receiver<E1>, count: usize) -> Self {
		let (inner, mut wal) = (JoinSet::new(), Vec::new());

		for _ in 0..count {
			let value = Ref::new(Mutex::new(None));

			wal.push(value);
		}
		Self {
			state,
			event,
			inner,
			count,
			wal,
		}
	}

	async fn execute<W1: 'static>(&mut self, wi: usize, worker: Ref<W1>, state: Ref<S1>)
	where
		W1: Worker<S1, E1>,
	{
		let (wal, receiver) = (self.wal[wi].clone(), self.event.clone());

		let worker = move |event| {
			let (worker, state) = (worker.clone(), state.clone());

			async move { worker.execute(state, event).await }
		};

		self.inner.spawn(dispatcher::dispatch(wal, receiver, worker));
	}

	async fn respawn<W1: 'static>(&mut self, worker: Ref<W1>, state: Ref<S1>)
	where
		W1: Worker<S1, E1>,
	{
		self.inner.shutdown().await;

		for wi in 0..self.count {
			let (worker, state) = (worker.clone(), state.clone());

			self.execute(wi, worker, state).await;
		}
	}

	pub async fn receive<W1: 'static>(mut self, worker: W1)
	where
		W1: Worker<S1, E1>,
	{
		let worker = Ref::new(worker);

		while let Ok(state) = self.state.recv().await {
			let (worker, state) = (worker.clone(), Ref::new(state));

			self.respawn(worker, state).await;
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	use {std::time::Duration, tokio::sync::mpsc};

	#[tokio::test]
	async fn wal() {
		let (we, event) = async_channel::bounded(512);
		let (ws, state) = broadcast::channel(512);

		let executor = Executor::new(state, event, 12);

		let (ck, mut check) = mpsc::channel(512);

		let worker = move |state: Ref<u8>, event| {
			let (state, ck) = (state.clone(), ck.clone());

			async move {
				ck.send((*state, event)).await.unwrap();

				tokio::time::sleep(Duration::from_secs(100)).await;
			}
		};
		tokio::spawn(executor.receive(worker));

		// Send event to channel but there are no workers yet.
		we.send(0).await.unwrap();

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
}
