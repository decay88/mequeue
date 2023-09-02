use std::sync::Arc;

use ethers_providers::Middleware;
use ethers_providers::Ws;
use futures::StreamExt;
use mequeue::Step;

use crate::{Event, Ret};

type Executor<B1> = Arc<mequeue::Executor<B1>>;
type Provider = Arc<ethers_providers::Provider<Ws>>;

pub mod mempool {
	use super::*;

	pub async fn collect<'a, B1>(executor: Executor<B1>, middleware: Provider) -> Ret<'a>
	where
		B1: Step<crate::Topic<'a>, Event>,
	{
		let mut stream = middleware.subscribe(["newPendingTransactionsWithBody"]).await.unwrap();

		while let Some(transaction) = stream.next().await {
			executor.enqueue("transaction", Event::Transaction(transaction)).await;
		}
		None
	}
}

pub mod block {
	use super::*;

	pub async fn collect<'a, M1>(executor: Executor<M1>, middleware: Provider) -> Ret<'a>
	where
		M1: Step<crate::Topic<'a>, Event>,
	{
		let mut stream = middleware.subscribe_blocks().await.unwrap();

		while let Some(block) = stream.next().await {
			executor.enqueue("block", Event::Block(block)).await;
		}
		None
	}
}
