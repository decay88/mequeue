use {async_trait::async_trait, std::future::Future};

type Output<T1, E1> = Option<(T1, E1)>;

#[async_trait]
pub trait Path<T1, E1>: Send + Sync {
	async fn execute(&self, event: E1) -> Output<T1, E1>;
}

#[async_trait]
impl<F1: Send + Sync, R1: Send, T1, E1: Send + 'static> Path<T1, E1> for F1
where
	F1: Fn(E1) -> R1,
	R1: Future<Output = Output<T1, E1>>,
{
	async fn execute(&self, event: E1) -> Output<T1, E1> {
		(*self)(event).await
	}
}

#[async_trait]
pub trait Step<T1, E1>: Send + Sync {
	async fn enqueue(&self, topic: T1, event: E1) -> Output<T1, E1>;
}

pub struct Executor<B1> {
	inner: B1,
}

impl<B1> Executor<B1> {
	pub async fn enqueue<T1, E1>(&mut self, mut topic: T1, mut event: E1)
	where
		B1: Step<T1, E1>,
	{
		while let Some(value) = self.inner.enqueue(topic, event).await {
			topic = value.0;
			event = value.1;
		}
	}
}

pub struct Router<B1, T1, R1> {
	inner: B1,
	topic: T1,
	route: R1,
}

impl<B1, T1, R1> Router<B1, T1, R1> {
	pub fn map<E1, R2>(self, topic: T1, route: R2) -> Router<Self, T1, R2>
	where
		B1: Step<T1, E1>,
	{
		Router {
			inner: self,
			topic,
			route,
		}
	}

	pub fn execute(self) -> Executor<Self> {
		Executor { inner: self }
	}
}

#[async_trait]
impl<B1, T1: Send + Sync, R1, E1: Send + 'static> Step<T1, E1> for Router<B1, T1, R1>
where
	B1: Step<T1, E1>,
	R1: Path<T1, E1>,
	T1: Eq,
{
	async fn enqueue(&self, topic: T1, event: E1) -> Output<T1, E1> {
		if topic == self.topic {
			self.route.execute(event).await
		} else {
			self.inner.enqueue(topic, event).await
		}
	}
}

pub struct Root<R1> {
	route: R1,
}

impl<R1> Root<R1> {
	pub fn new<T1, E1>(route: R1) -> Self
	where
		R1: Path<T1, E1>,
	{
		Self { route }
	}

	pub fn map<T1, E1, R2>(self, topic: T1, route: R2) -> Router<Self, T1, R2>
	where
		R2: Path<T1, E1>,
	{
		Router {
			inner: self,
			topic,
			route,
		}
	}
}

#[async_trait]
impl<T1: Send + 'static, E1: Send + 'static, R1> Step<T1, E1> for Root<R1>
where
	R1: Path<T1, E1>,
{
	async fn enqueue(&self, _: T1, event: E1) -> Output<T1, E1> {
		self.route.execute(event).await
	}
}
