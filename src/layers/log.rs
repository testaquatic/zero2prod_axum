use std::fmt;

use tower::{Layer, Service};

pub struct LogLayer {
    target: &'static str,
}

// 로그를 구현하는 서비스 구현
pub struct LogService<S> {
    target: &'static str,
    service: S,
}

impl<S> Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, service: S) -> Self::Service {
        LogService {
            target: self.target,
            service,
        }
    }
}

impl<S, Request> Service<Request> for LogService<S>
where
    S: Service<Request>,
    Request: fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        println!("request = {:?}, target = {:?}", request, self.target);
        self.service.call(request)
    }
}
