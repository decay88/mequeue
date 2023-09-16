use tokio::{sync::mpsc, task::JoinHandle};

use {async_trait::async_trait, std::future::Future};

type Maybe<T1> = std::option::Option<T1>;
type Ref<T1> = std::sync::Arc<T1>;

#[async_trait]
pub trait EventDispatch<E1>: Send + Sync {
	async fn dispatch(&self, event: E1) -> Maybe<E1>;
}

#[async_trait]
impl<F1: Send + Sync, E1: Send + 'static, R1: Send> EventDispatch<E1> for F1
where
	F1: Fn(E1) -> R1,
	R1: Future<Output = Maybe<E1>>,
{
	async fn dispatch(&self, event: E1) -> Maybe<E1> {
		(*self)(event).await
	}
}

#[async_trait]
pub trait AwaitDispatch<E1>: Send + Sync {
	async fn dispatch(&self, event: E1);
}

#[async_trait]
impl<F1: Send + Sync, E1: Send + 'static, R1: Send> AwaitDispatch<E1> for F1
where
	F1: Fn(E1) -> R1,
	R1: Future<Output = ()>,
{
	async fn dispatch(&self, event: E1) {
		(*self)(event).await
	}
}

type ChannelResult<E1> = Result<(), mpsc::error::SendError<E1>>;
type EventSenderResult<E1> = ChannelResult<Maybe<E1>>;
type EventSender<E1> = mpsc::Sender<Maybe<E1>>;

pub fn new<F1: 'static, F2: 'static, E1: Send + 'static>(
	event_queue_size: usize,
	event_dispatcher: Ref<F1>,
	await_dispatcher: Ref<F2>,
) -> (EventSender<E1>, JoinHandle<()>)
where
	F1: EventDispatch<E1>,
	F2: AwaitDispatch<EventSenderResult<E1>>,
{
	let (event_sender, mut event_receiver) = mpsc::channel(event_queue_size);

	let executor = event_sender.clone();

	let dispatch = move |event| {
		let (event_dispatcher, event_sender) = (event_dispatcher.clone(), event_sender.clone());

		async move {
			let event = event_dispatcher.dispatch(event).await;

			event_sender.send(event).await
		}
	};
	let (dispatch, await_dispatcher) = (Ref::new(dispatch), await_dispatcher.clone());

	let inner_dispatcher = move |event| {
		let (dispatch, await_dispatcher) = (dispatch.clone(), await_dispatcher.clone());

		async move {
			let event = dispatch(event).await;

			await_dispatcher.dispatch(event).await;
		}
	};
	let inner_dispatcher = Ref::new(inner_dispatcher);

	let receive = async move {
		while let Some(maybe_event) = event_receiver.recv().await {
			let inner_dispatcher = inner_dispatcher.clone();

			if let Some(event) = maybe_event {
				inner_dispatcher(event).await
			}
		}
	};
	let inner = tokio::spawn(receive);

	(executor, inner)
}
