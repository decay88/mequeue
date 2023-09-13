use std::sync::Arc;

use ethers_providers::{Middleware, Ws};
use futures::StreamExt;
use stepwise::Step;

use crate::Event;

type Executor<B1> = Arc<B1>;
type Provider = Arc<ethers_providers::Provider<Ws>>;

pub mod mempool {
	use super::*;

	pub async fn collect<'a, B1>(executor: Executor<B1>, middleware: Provider) -> Option<Event>
	where
		B1: Step<Event>,
	{
		let mut stream = middleware.subscribe(["newPendingTransactionsWithBody"]).await.unwrap();

		while let Some(transaction) = stream.next().await {
			executor.enqueue(Event::Transaction(transaction)).await;
		}
		None
	}
}

pub mod block {
	use super::*;

	pub async fn collect<'a, M1>(executor: Executor<M1>, middleware: Provider) -> Option<Event>
	where
		M1: Step<Event>,
	{
		let mut stream = middleware.subscribe_blocks().await.unwrap();

		while let Some(block) = stream.next().await {
			executor.enqueue(Event::Block(block)).await;
		}
		None
	}
}
