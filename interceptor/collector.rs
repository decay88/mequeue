use std::sync::Arc;

use ethers_providers::{Middleware, Ws};
use futures::StreamExt;

use tokio::sync::mpsc;

use crate::{Event, Maybe};

type Provider = Arc<ethers_providers::Provider<Ws>>;

pub mod mempool {
	use super::*;

	pub async fn collect(event_sender: mpsc::Sender<Maybe<Event>>, middleware: Provider) {
		let mut stream = middleware.subscribe(["newPendingTransactionsWithBody"]).await.unwrap();

		while let Some(transaction) = stream.next().await {
			event_sender.send(Some(Event::Transaction(transaction))).await.unwrap();
		}
	}
}

pub mod block {
	use super::*;

	pub async fn collect(event_sender: mpsc::Sender<Maybe<Event>>, middleware: Provider) {
		let mut stream = middleware.subscribe_blocks().await.unwrap();

		while let Some(block) = stream.next().await {
			event_sender.send(Some(Event::Block(block))).await.unwrap();
		}
	}
}
