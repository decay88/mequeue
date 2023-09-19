use std::{future::Future, ops::Deref};

use async_trait::async_trait;

#[async_trait]
pub trait Worker: Send + Sync {
	async fn execute(&self);
}

#[async_trait]
impl<F1: Send + Sync, R1: Send> Worker for F1
where
	F1: Fn() -> R1,
	R1: Future<Output = ()>,
{
	async fn execute(&self) {
		self.deref()().await
	}
}
