#![deny(missing_docs)]
//! # Transmute
//!
//! Transmute gives you the ability to define your own async "magic handler" functions. A
//! magic handler is a function that can take any number of arguments and return a future,
//! so long as those arguments can be extracted from a single input value.
//!
//! Magic handlers are popular in several parts of the rust community, such as the axum
//! and actix-web crates, and the bevy crate. They let you split your application into
//! two layers: what it is, and what it does. However, the implementation of these magic
//! handlers can be tough to understand, and so they tend to be tucked away inside their
//! respective crates.
//!
//! Transmute aims to make it easy to define your own magic handlers, so you can use them
//! yourself for your own programs. It's a more general implementation of the pattern, so
//! you can pop it into your project and make a magic handler for whatever you want without
//! leveraging those other crates.
//!
//! ## Dependency injection & clarity
//!
//! Magic handlers are a kind of dependency injection pattern. Your handlers know what they
//! need to work, and your extractors know how to get it from a core type. This makes it easy
//! to model out logic but it makes it hard to trace cause and effect. It's a tradeoff, but
//! the ergonomics of magic handlers tend to broaden the amount of people who can work on code.
//!
//! It's up to you whether or not you feel as though magic handlers serve you and your team.
//! If you have a ton of people doing work on different layers, it might be ideal. Otherwise,
//! it might add too much extra time to debugging work!
//!
mod function_impl;

use std::future::Future;

/// # Extractor
///
/// Define the extractor trait for any type you want to use as an argument to your magic handler.
/// Extractors have several IO types: the input type, the state type, and the error type. These
/// need to line up with the types of the handler you're using.
///
/// ## Input Type
///
/// The input type is the type that your extractor will take as input. This is the type that
/// your handler will take as input, and the type that your extractor will extract from. Usually
/// this is a core concern of your application, and ultimately will pass through every handler
/// and extractor at some point.
///
/// ## State Type
///
/// The state type is the type that your extractor will take as context. This is a "hole" in the
/// magic handler pattern that allows you to keep it extensible to unknown consumer systems. In
/// other words, you can usually model out your whole system without knowing how it interacts with
/// an outside system, and then later use the state type to fill in that context gap.
///
/// State has to be uniform in the same way the input type does, because of how the extract and
/// handler traits are defined.
///
/// ## Error Type
///
/// The error type is the type that your extractor will return if it fails to extract. This is
/// another "hole" similar to the state type, because extractors might never fail. However,
/// because some might, the possibility of failure has to be considered for every extractor.
///
/// Like the state and input types, the error type has to somehow connect to the error type of
/// the handler. This isn't a one to one link in the case of the error type: anything that can
/// understand extractor errors and how to cast from them using `From`/`Into` can be used.
///
pub trait Extractor<T, State> {
    /// The error type that the extractor can return.
    type Error;

    /// Extract the input type from the input value and the given state context.
    fn extract(topic: T, context: &impl Into<State>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// # Handler
///
/// The Handler trait often doesn't need to be defined at all in order to work. Transmute
/// provides an automatic implementation for any function that has uniformity in its extractors.
///
/// That means that generally, all you need to implement to get the handler to work is:
///
/// - Extractors for the input types.
/// - Casting traits for error types.
/// - A function that takes the input types and returns a future.
///
pub trait Handler<T, Args, State>: Clone {
    /// The response type that the handler will return.
    type Response: Into<T>;
    /// The error type that the handler will return.
    type Error;
    /// The future type that the handler will return.
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    /// Invoke the handler with the given input value and state context.
    fn invoke(&self, frame: impl Into<T>, state: State) -> Self::Future;
}
