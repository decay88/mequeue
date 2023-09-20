use {async_trait::async_trait, std::future::Future};

#[async_trait]
pub trait Worker<S1, E1>: Send + Sync {
	async fn execute(&self, state: &S1, event: E1);
}

#[async_trait]
impl<S1: Send + Sync, E1: Send + 'static, R1: Send, F1: Send + Sync> Worker<S1, E1> for F1
where
	F1: Fn(&S1, E1) -> R1,
	R1: Future<Output = ()>,
{
	async fn execute(&self, state: &S1, event: E1) {
		(*self)(state, event).await
	}
}

#[async_trait]
pub trait MappedWorker<E1>: Send + Sync {
	async fn execute(&self, event: E1);
}

#[async_trait]
impl<E1: Send + 'static, R1: Send, F1: Send + Sync> MappedWorker<E1> for F1
where
	F1: Fn(E1) -> R1,
	R1: Future<Output = ()>,
{
	async fn execute(&self, event: E1) {
		(*self)(event).await
	}
}
