use parking_lot::Mutex;

use async_channel as mpmc;
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
			// Keep in mind that this closure will be called on every event

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
			// Spawn a few workers to use the full power of async

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
			// Stop and start workers when we receive new state

			let (worker, state) = (worker.clone(), Ref::new(state));

			self.respawn(worker, state).await;
		}
	}
}
