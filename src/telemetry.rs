use tracing::Subscriber;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{MakeWriter, format::FmtSpan},
    layer::SubscriberExt,
};

/// Subscriber를 조립한다
pub fn get_subscriber<Sink>(env_filter: &str, sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_line_number(true)
        .with_file(true)
        .with_writer(sink)
        .json()
        .with_current_span(true)
        .flatten_event(true);

    Registry::default().with(env_filter).with(formatting_layer)
}

/// subscriber를 굴로벌 기본값으로 등록해서 Span 데이터를 처리한다
///
/// 한번만 호출되어야 한다
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
