use ethers_core::types::{Block, Transaction, H256};
use ethers_providers::{Provider, Ws};
use mequeue::executor::Executor;
use tokio::sync::broadcast;

mod collector;

type Ref<T1> = std::sync::Arc<T1>;

#[tokio::main]
async fn main() {
	let remote = std::env::var("REMOTE").unwrap();
	let middleware = Provider::<Ws>::connect(remote).await.unwrap();
	let middleware = std::sync::Arc::new(middleware);

	let (ws, state) = broadcast::channel(512);
	let (we, event) = async_channel::bounded(512);

	let executor = Executor::new(state, event, 12);

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
