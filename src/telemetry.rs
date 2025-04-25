use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

/// 여러 레이어들을 하나의 `tracing`의 subscriber로 구성한다.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: EnvFilter,
    sink: Sink,
) -> impl Subscriber + Send + Sync
// 모든 라이프타입 파라미터 `'a`에 대해 `MakeWriter` 트레이트를 구현한다.
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(env_filter);
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// subscriber를 글로벌 기본값으로 등록해서 span 데이터를 처리한다.
///
/// __!!!주의!!!__  
/// 한 차례만 호출되어야 한다.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger.");
    set_global_default(subscriber).expect("Failed to set subscriber.");
}
