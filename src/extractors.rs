/// An extractor that allows the given state to be extracted. This lets you inject and
/// extract "last mile" state into your handlers, freeing you up to model your application
/// without fully knowing how it will interact with the outside world.
///
/// State requires that the given state type is cloneable, because it will clone the state
/// when extracting it, for every handler invocation or extractor that uses it.
///
/// # Example
///
/// ```rust
/// use occult::{Extractor, Handler};
///
/// struct ArbitraryState;
///
/// async fn handler(State(state): State<ArbitraryState>) -> String {
///    format!("Hello, world!")
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let state = ArbitraryState;
///
///     let response = handler.invoke((), state).await.unwrap();
///     assert_eq!(response, "Hello, world!");
/// }
/// ```
///
pub struct State<T>(pub T);

impl<T, GivenState> crate::Extractor<T, GivenState> for State<GivenState>
where
    GivenState: Clone,
{
    type Error = String;

    fn extract<Context>(_: T, context: &Context) -> Result<Self, Self::Error>
    where
        Self: Sized,
        Context: Into<GivenState> + Clone,
    {
        let context = context.clone();
        let context = GivenState::from(context.into());
        Ok(State(context.clone()))
    }
}
