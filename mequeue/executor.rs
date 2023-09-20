use async_channel as mpmc;
use std::sync::Mutex;

use tokio::{sync::broadcast, task::JoinSet};

use crate::{dispatcher, worker::Worker};

type Ref<T1> = std::sync::Arc<T1>;

pub struct Executor<S1, E1> {
	state: broadcast::Receiver<S1>,
	event: mpmc::Receiver<E1>,
	inner: JoinSet<()>,
}

impl<S1: Clone, E1: Clone> Executor<S1, E1>
where
	S1: Send + Sync + 'static,
	E1: Send + Sync + 'static,
{
	pub fn new(state: broadcast::Receiver<S1>, event: mpmc::Receiver<E1>) -> Self {
		let inner = JoinSet::new();

		Self {
			state,
			event,
			inner,
		}
	}

	async fn execute<W1: 'static>(&mut self, worker: Ref<W1>, state: Ref<S1>)
	where
		W1: Worker<S1, E1>,
	{
		let wcurrent = Ref::new(Mutex::new(None));
		let receiver = self.event.clone();

		let worker = move |event| {
			let (worker, state) = (worker.clone(), state.clone());

			async move { worker.execute(&state, event).await }
		};

		self.inner.spawn(dispatcher::dispatch(wcurrent, receiver, worker));
	}

	async fn respawn<W1: 'static>(&mut self, worker: Ref<W1>, state: Ref<S1>, wcount: usize)
	where
		W1: Worker<S1, E1>,
	{
		self.inner.shutdown().await;

		for _ in 0..wcount {
			let (worker, state) = (worker.clone(), state.clone());

			self.execute(worker, state).await;
		}
	}

	pub async fn receive<W1: 'static>(mut self, worker: W1, wcount: usize)
	where
		W1: Worker<S1, E1>,
	{
		let worker = Ref::new(worker);

		while let Ok(state) = self.state.recv().await {
			let (worker, state) = (worker.clone(), Ref::new(state));

			self.respawn(worker, state, wcount).await;
		}
	}
}
