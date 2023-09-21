use std::sync::Arc;

use async_channel::Sender;
use ethers_core::types::Transaction;
use ethers_core::types::{Block, H256};
use ethers_providers::{Middleware, Ws};
use futures::StreamExt;
use tokio::sync::broadcast;

type Provider = Arc<ethers_providers::Provider<Ws>>;

pub mod mempool {
	use super::*;

	pub async fn collect(we: Sender<Transaction>, middleware: Provider) {
		let mut stream = middleware.subscribe(["newPendingTransactionsWithBody"]).await.unwrap();

		while let Some(transaction) = stream.next().await {
			we.send(transaction).await.unwrap();
		}
	}
}

pub mod block {
	use super::*;

	pub async fn collect(we: broadcast::Sender<Block<H256>>, middleware: Provider) {
		let mut stream = middleware.subscribe_blocks().await.unwrap();

		while let Some(block) = stream.next().await {
			we.send(block).unwrap();
		}
	}
}
