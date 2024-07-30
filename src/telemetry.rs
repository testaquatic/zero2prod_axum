use std::{str::FromStr, sync::Once};

use tokio::task::JoinHandle;
use tracing::{dispatcher::set_global_default, level_filters::LevelFilter, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

/// 여러 레이어들을 하나의 `tracing`와 subscriber로 구성한다.
///
/// # 구현 노트
///
/// 'impl Subscriber`를 반환 타입으로 사용해서 반환된 subscriber의 실제 타입에 관할 설명을 피한다.
/// 반환된 subscriber를 `init_subscriber`로 나중에 전달하기 위해 명시적으로 `Send`이고 `Sync`임을 알려야 한다.
pub fn get_tracing_subscriber<Sink>(
    env_filter: LevelFilter,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    // 이 이상한 구문은 higher-ranked trait bound이다.
    // 기본적으로 Sink가 모든 라이프타임 파라미터 `'a`에 대해 'MakeWrtier' 트레이트를 구현한다는 것을 의미한다.
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // RUST_LOG 환경 변수가 설정되어 있지 않으면 info 레벨 및 그 이상의 모든 span을 출력한다.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(
        EnvFilter::from_str("tower_http=debug,axum::rejection=trace")
            .unwrap_or_default()
            .add_directive(env_filter.into()),
    );

    let formatting_layer = BunyanFormattingLayer::new("Z2PA".into(), sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// subscriber를 글로벌 기본값으로 등록해서 span 데이터를 처리한다.
/// 한차례만 실행된다.
pub fn init_tracing_subscriber(tracing_subscriber: impl Subscriber + Send + Sync) {
    // `Once`를 사용해서 한차례만 실행된다.
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        set_global_default(tracing_subscriber.into()).expect("Failed to set subscriber.");
        LogTracer::builder().init().expect("Failed to set logger.");
    })
}

// `spawn_blocking`으로부터 트레이트 바운드와 시그니처를 복사했다.
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    // 이것이 실행된 후 새로운 스레드를 실행한다.
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
