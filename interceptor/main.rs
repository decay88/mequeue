use {futures::StreamExt, tokio::sync::broadcast};

use ethers_core::types::{Block, Transaction, H256};
use ethers_providers::{Middleware, Provider, Ws};

use mequeue::Executor;

type Websocket = std::sync::Arc<Provider<Ws>>;
type Ref<T1> = std::sync::Arc<T1>;

async fn handle(state: Ref<Block<H256>>, event: Transaction) {
	// Just provide formatted output of received data as example.

	println!("{} {}", state.hash.unwrap(), event.hash());
}

async fn receive_block_update(ws: broadcast::Sender<Block<H256>>, middleware: Websocket) {
	let mut stream = middleware.subscribe_blocks().await.unwrap();

	while let Some(block) = stream.next().await {
		ws.send(block).unwrap();
	}
}

async fn receive_from_mempool(we: async_channel::Sender<Transaction>, middleware: Websocket) {
	let mut stream = middleware.subscribe(["newPendingTransactionsWithBody"]).await.unwrap();

	while let Some(transaction) = stream.next().await {
		we.send(transaction).await.unwrap();
	}
}

#[tokio::main]
async fn main() {
	// Get address of remote node from environment.
	let remote = std::env::var("REMOTE").unwrap();

	// Connect to remote node through websocket.
	let middleware = Provider::<Ws>::connect(remote).await.unwrap();
	let middleware = Ref::new(middleware);

	// Make channels for both state and event exchange.
	let (ws, state) = broadcast::channel(512);
	let (we, event) = async_channel::bounded(512);

	// Construct executor with channels we made above and set workers count to eight.
	let executor = Executor::new(state, event, 8);

	// Spawn executor so it can handle incoming events and state changes.
	tokio::spawn(executor.receive(handle));

	tokio::join!(
		receive_block_update(ws, middleware.clone()),
		receive_from_mempool(we, middleware.clone()),
	);
	()
}
