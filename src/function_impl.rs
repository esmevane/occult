use std::{future::Future, pin::Pin};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

macro_rules! define_handler_for_tuple ({ $($param:ident)* } => {
#[allow(non_snake_case, unused_mut, unused_variables)]
impl<T, Func, Future, State, Output, $($param,)*>
    crate::Handler<T, ($($param,)*), State> for Func
where
    T: Clone + Send + 'static,
    Func: FnOnce($($param,)*) -> Future + Clone + Send + 'static,
    State: Clone + Send + Sync + 'static,
    Output: Into<T>,
    Future: std::future::Future<Output = Output> + Send + 'static,
    $(
        $param: crate::Extractor<T, State> + Send,
        $param::Error: Into<Box<dyn std::error::Error>>,
    )*
{
    type Response = T;
    type Error = Box<dyn std::error::Error>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn invoke(&self, input: impl Into<T>, state: State) -> Self::Future {
        let handler = self.clone();
        let input = input.into();

        Box::pin(async move {
            let input = &input;
            let handler = handler;
            let context = &state;

            $(
                let $param = match $param::extract(input.clone(), context) {
                    Ok(value) => value,
                    Err(rejection) => return Err(rejection.into()),
                };
            )*

            let response = handler($($param,)*).await;

            Ok(response.into())
        })
    }
}
});

define_handler_for_tuple! {}
define_handler_for_tuple! { A }
define_handler_for_tuple! { A B }
define_handler_for_tuple! { A B C }
define_handler_for_tuple! { A B C D }
define_handler_for_tuple! { A B C D E }
define_handler_for_tuple! { A B C D E F }
define_handler_for_tuple! { A B C D E F G }
define_handler_for_tuple! { A B C D E F G H }
define_handler_for_tuple! { A B C D E F G H I }
define_handler_for_tuple! { A B C D E F G H I J }
define_handler_for_tuple! { A B C D E F G H I J K }
define_handler_for_tuple! { A B C D E F G H I J K L }
