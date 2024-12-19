use occult::{Extractor, Handler, State};

// Create a core type - "Frame", a byte buffer envelope.
#[derive(Debug, Clone, PartialEq)]
struct Frame(Vec<u8>);

impl From<Vec<u8>> for Frame {
    fn from(vec: Vec<u8>) -> Self {
        Frame(vec)
    }
}

// If we want to have our handlers return String, we need to implement From<String> for Frame.
impl From<String> for Frame {
    fn from(string: String) -> Self {
        Frame(string.into_bytes())
    }
}

// We make a FromStr so we can coerce it
impl std::str::FromStr for Frame {
    type Err = std::string::FromUtf8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Frame(s.as_bytes().to_vec()))
    }
}

// Topics will be frames turned into strings.
#[derive(Debug, Clone)]
struct Topic(String);

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// We tell the extractor how to turn a frame into a topic. We don't care about state, so
// it stays generic.
impl<State> Extractor<Frame, State> for Topic {
    type Error = String;

    fn extract<Context>(topic: Frame, _: &Context) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let topic = String::from_utf8(topic.0).map_err(|err| err.to_string())?;
        Ok(Topic(topic))
    }
}

#[tokio::test]
async fn simple_extract() -> Result<(), Box<dyn std::error::Error>> {
    // Create a handler
    async fn handler(topic: Topic) -> String {
        format!("Hello, {topic}!")
    }

    assert_eq!(
        handler.invoke(Frame(b"world".to_vec()), ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    Ok(())
}

#[tokio::test]
async fn async_closures() -> Result<(), Box<dyn std::error::Error>> {
    // Create a handler
    let handler = |topic: Topic| async move { format!("Hello, {topic}!") };

    assert_eq!(
        handler.invoke(Frame(b"world".to_vec()), ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    Ok(())
}

#[tokio::test]
async fn coerced_types() -> Result<(), Box<dyn std::error::Error>> {
    async fn handler(topic: Topic) -> String {
        format!("Hello, {topic}!")
    }

    assert_eq!(
        handler.invoke("world".to_string(), ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    assert_eq!(
        handler.invoke(b"world".to_vec(), ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    Ok(())
}

#[tokio::test]
async fn coerced_types_closures() -> Result<(), Box<dyn std::error::Error>> {
    // Create a handler
    let handler = |topic: Topic| async move { format!("Hello, {topic}!") };

    // It can also be called with coerced types
    assert_eq!(
        handler.invoke("world".to_string(), ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    assert_eq!(
        handler.invoke(b"world".to_vec(), ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    Ok(())
}

#[tokio::test]
async fn multiple_args() -> Result<(), Box<dyn std::error::Error>> {
    // Handlers can have a bunch of arguments
    async fn multi_handler(topic: Topic, topic2: Topic, topic3: Topic) -> String {
        format!("Hello, {topic} {topic2} {topic3}!")
    }

    assert_eq!(
        multi_handler.invoke(Frame(b"world".to_vec()), ()).await?,
        Frame(b"Hello, world world world!".to_vec())
    );

    Ok(())
}

#[tokio::test]
async fn narrowing_types() -> Result<(), Box<dyn std::error::Error>> {
    // Taking handlers directly can be cumbersome, so the more you know about your handler
    // the better. That allows you to make decisions which can reduce the burden of your
    // generic code by making it more specific. For example, you usually know what your
    // core handler type is, so you can make a more specific handler type signature.
    //
    // Usually the only arguments you won't know ahead of time are the Args and State types.
    // So let's define everything else we need: the handler type, the input type, and the
    // output type.
    async fn handle_the_handler<Args, State>(
        handler: impl Handler<Frame, Args, State, Error = Box<dyn std::error::Error>>,
        state: State,
    ) -> Result<Frame, Box<dyn std::error::Error>> {
        match handler.invoke(Frame(b"world".to_vec()), state).await {
            Ok(frame) => Ok(frame.into()),
            Err(err) => Err(err),
        }
    }

    // We can be more terse!
    async fn tersely_handle_the_handler<Args, State>(
        handler: impl Handler<Frame, Args, State, Error = Box<dyn std::error::Error>>,
        state: State,
    ) -> Result<Frame, Box<dyn std::error::Error>> {
        handler
            .invoke(Frame(b"world".to_vec()), state)
            .await
            .map(Into::into)
    }

    async fn handler(topic: Topic) -> String {
        format!("Hello, {topic}!")
    }

    // And we can call it with any of the above handlers.
    assert_eq!(
        handle_the_handler(&handler, ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    assert_eq!(
        tersely_handle_the_handler(&handler, ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    Ok(())
}

#[tokio::test]
async fn narrowing_types_closures() -> Result<(), Box<dyn std::error::Error>> {
    let handler = |topic: Topic| async move { format!("Hello, {topic}!") };

    async fn tersely_handle_the_handler<Args, State>(
        handler: impl Handler<Frame, Args, State, Error = Box<dyn std::error::Error>>,
        state: State,
    ) -> Result<Frame, Box<dyn std::error::Error>> {
        handler
            .invoke(Frame(b"world".to_vec()), state)
            .await
            .map(Into::into)
    }

    assert_eq!(
        tersely_handle_the_handler(handler, ()).await?,
        Frame(b"Hello, world!".to_vec())
    );

    Ok(())
}

#[tokio::test]
//using state
async fn extracting_from_state() -> Result<(), Box<dyn std::error::Error>> {
    // Because the state is generic, we can use it to pass in any context we want.
    // This is left up to the consumer to decide, but more importantly, it's a way
    // to avoid deciding on that until the last minute. This means you can model
    // your application as you learn about the things you need to do, without having
    // to re-implement your extractors and handlers that don't care.
    #[derive(Debug, Clone)]
    struct ArbitraryState;

    impl ArbitraryState {
        fn how_arbitrary(&self) -> &'static str {
            "very arbitrary"
        }
    }

    async fn handler(topic: Topic, State(state): State<ArbitraryState>) -> String {
        format!(
            "Hello, {topic} - {how_arbitrary}!",
            how_arbitrary = state.how_arbitrary()
        )
    }

    assert_eq!(
        handler
            .invoke(Frame(b"world".to_vec()), ArbitraryState)
            .await?,
        Frame(b"Hello, world - very arbitrary!".to_vec())
    );

    Ok(())
}
