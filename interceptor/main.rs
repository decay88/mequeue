use ethers_core::types::{Block, Transaction, H256};
use ethers_providers::{Provider, Ws};

type Maybe<T1> = std::option::Option<T1>;
type Ref<T1> = std::sync::Arc<T1>;

mod collector;

#[derive(Debug)]
pub enum Event {
	Block(Block<H256>),
	Transaction(Transaction),
}

#[tokio::main]
async fn main() {
	let remote = std::env::var("REMOTE").unwrap();
	let middleware = Provider::<Ws>::connect(remote).await.unwrap();
	let middleware = std::sync::Arc::new(middleware);

	let event_dispatch = |event| async move {
		match event {
			Event::Block(b) => println!("block | {}", b.hash.unwrap()),
			Event::Transaction(t) => println!("transaction | {}", t.hash),
		};
		None
	};

	let event_dispatch = Ref::new(event_dispatch);

	let await_dispatch = Ref::new(|_| async move {});

	let (event_sender, jh) = mequeue::execute(512, event_dispatch, await_dispatch);

	tokio::join!(
		collector::mempool::collect(event_sender.clone(), middleware.clone()),
		collector::block::collect(event_sender.clone(), middleware.clone()),
		jh,
	)
	.2
	.unwrap();

	()
}
