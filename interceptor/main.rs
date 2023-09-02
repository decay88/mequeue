use ethers_core::types::{Block, Transaction, H256};
use ethers_providers::{Provider, Ws};

mod collector;

pub type Topic<'a> = &'a str;

#[derive(Debug)]
pub enum Event {
	Block(Block<H256>),
	Transaction(Transaction),
}

pub type Ret<'a> = Option<(Topic<'a>, Event)>;

#[tokio::main]
async fn main() {
	let remote = std::env::var("REMOTE").unwrap();
	let middleware = Provider::<Ws>::connect(remote).await.unwrap();
	let middleware = std::sync::Arc::new(middleware);

	let block = |e| async move {
		match e {
			Event::Block(b) => println!("|b| {}", b.hash.unwrap()),
			_ => panic!(),
		}
		None
	};
	let transaction = |e| async move {
		match e {
			Event::Transaction(t) => println!("|t| {}", t.hash),
			_ => panic!(),
		}
		None
	};

	let executor = mequeue::Root::new(|_| async { None })
		.map("block", block)
		.map("transaction", transaction)
		.execute();
	let executor = std::sync::Arc::new(executor);

	tokio::join!(
		collector::mempool::collect(executor.clone(), middleware.clone()),
		collector::block::collect(executor.clone(), middleware.clone())
	);
	()
}
