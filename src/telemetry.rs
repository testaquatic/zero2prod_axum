use tracing::{Subscriber, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

/// 여러 레이어들을 하나의 Subscriber로 구성한다.
pub fn get_subscriber<Sink>(env_filter: String, sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter));
    // https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html 이 문서를 참고했다.
    let formatting_layer = tracing_subscriber::fmt::layer().with_writer(sink);

    Registry::default().with(env_filter).with(formatting_layer)
}

/// Subscriber를 글로벌 기본 값으로 등록해서 span 데이터를 처리한다.
/// 한차례만 호출해야 한다.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger.");
    set_global_default(subscriber).expect("Failed to set subscriber.");
}
