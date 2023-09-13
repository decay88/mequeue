use {async_trait::async_trait, std::future::Future};

#[async_trait]
pub trait Executor<E1>: Send + Sync {
	async fn execute(&self, event: E1) -> Option<E1>;
}

#[async_trait]
impl<F1: Send + Sync, E1: Send + 'static, R1: Send> Executor<E1> for F1
where
	F1: Fn(E1) -> R1,
	R1: Future<Output = Option<E1>>,
{
	async fn execute(&self, event: E1) -> Option<E1> {
		(*self)(event).await
	}
}

pub struct Step<B1, R1> {
	inner: B1,
	route: R1,
}

impl<B1, R1> Step<B1, R1> {
	pub fn map<E1, R2>(self, route: R2) -> Step<Self, R2>
	where
		B1: Executor<E1>,
	{
		Step { inner: self, route }
	}
}

#[async_trait]
impl<B1, E1: Send + 'static, R1> Executor<E1> for Step<B1, R1>
where
	B1: Executor<E1>,
	R1: Executor<E1>,
{
	async fn execute(&self, event: E1) -> Option<E1> {
		if let Some(event) = self.inner.execute(event).await {
			self.route.execute(event).await
		} else {
			None
		}
	}
}

pub struct Root<R1> {
	route: R1,
}

pub fn new<E1, R1>(route: R1) -> Root<R1>
where
	R1: Executor<E1>,
{
	Root { route }
}

impl<R1> Root<R1> {
	pub fn map<E1, R2>(self, route: R2) -> Step<Self, R2>
	where
		R2: Executor<E1>,
	{
		Step { inner: self, route }
	}
}

#[async_trait]
impl<E1: Send + 'static, R1> Executor<E1> for Root<R1>
where
	R1: Executor<E1>,
{
	async fn execute(&self, event: E1) -> Option<E1> {
		self.route.execute(event).await
	}
}
