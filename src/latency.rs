use axum::{
    body::Body,
    http::{self, Response},
    response::IntoResponse,
};
use std::{
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::sleep;
use tower::{layer::layer_fn, Service};

use crate::structs::LatencyMiddleware;

impl<S, ReqBody, ResBody> Service<ReqBody> for LatencyMiddleware<S>
where
    S: Service<ReqBody, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: IntoResponse + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: ReqBody) -> Self::Future {
        let future = self.inner.call(req);
        let (tx, rx) = tokio::sync::oneshot::channel();
        let delay = self.delay;

        tokio::spawn(async move {
            sleep(delay).await;
            let result = future.await;
            let _ = tx.send(result);
        });

        Box::pin(async move { rx.await.unwrap() })
    }
}

pub fn with_latency<S>(
    delay: Duration,
) -> impl tower::Layer<S, Service = LatencyMiddleware<S>> + Clone
where
    S: Service<http::Request<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    layer_fn(move |inner| LatencyMiddleware { inner, delay })
}
