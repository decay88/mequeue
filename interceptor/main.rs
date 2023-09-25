use ethers_core::types::{Block, Transaction, H256};
use ethers_providers::{Provider, Ws};
use mequeue::executor::Executor;
use tokio::sync::broadcast;

mod collector;

type Ref<T1> = std::sync::Arc<T1>;

#[tokio::main]
async fn main() {
	// Get address of remote node from environment
	let remote = std::env::var("REMOTE").unwrap();

	// Connect to remote node through websocket
	let middleware = Provider::<Ws>::connect(remote).await.unwrap();
	let middleware = Ref::new(middleware);

	// Make channels for both state and event exchange
	let (ws, state) = broadcast::channel(512);
	let (we, event) = async_channel::bounded(512);

	// Construct executor with channels we made above and set workers count to eight
	let executor = Executor::new(state, event, 8);

	let worker = |state: Ref<Block<H256>>, event: Transaction| async move {
		println!("{} | {}", state.hash.unwrap(), event.hash());
	};

	tokio::spawn(executor.receive(worker));

	tokio::join!(
		collector::block::collect(ws, middleware.clone()),
		collector::mempool::collect(we, middleware.clone()),
	);
	()
}
