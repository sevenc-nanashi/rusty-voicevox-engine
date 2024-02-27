mod models;
mod resource_manager;
mod result;
mod routes;
mod utils;
mod vvm_manager;

use crate::{
    resource_manager::{ResourceManager, RESOURCE_MANAGER},
    utils::process_dir,
    vvm_manager::{VvmManager, VVM_MANAGER},
};
use axum::routing::{get, post, put};
use clap::Parser;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, warn};

#[derive(Parser)]
#[clap(name = "rusty-vv", rename_all = "snake_case")]
struct Args {
    /// ポート番号。
    #[clap(long, default_value = "50021")]
    port: u16,

    /// ホスト名。
    #[clap(long, default_value = "127.0.0.1")]
    host: String,

    /// GPU を使用するかどうか。
    #[clap(long, default_value = "false")]
    use_gpu: bool,

    /// CPU のスレッド数。
    #[clap(long, default_value = "0")]
    cpu_num_threads: usize,

    /// CORS のモード。
    /// all：全てのリクエストを許可する。
    /// localapps：app://、localhostのみ許可する。
    #[clap(long, default_value = "localapps")]
    cors_mode: String,

    /// CORS で許可する Origin 一覧。
    #[clap(long)]
    cors_origins: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let opts: Args = Args::parse();

    std::env::set_current_dir(process_dir()).unwrap();

    tracing_subscriber::fmt::init();

    let mut allow_origins: Vec<String> = vec![];
    let cors = CorsLayer::new()
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any());
    let cors = match opts.cors_mode.as_str() {
        "all" => {
            if opts.cors_origins.is_some() {
                warn!("--cors_origins は --cors_mode all の場合には無視されます。");
            }
            allow_origins.push("(Any)".to_string());
            cors.allow_origin(AllowOrigin::any())
        }
        "localapps" => {
            let mut origins = vec!["app://*".to_string()];
            allow_origins.extend(origins.clone());
            if let Some(cors_origins) = opts.cors_origins {
                origins.extend(cors_origins.clone());
                allow_origins.extend(cors_origins.clone());
            }
            cors.allow_origin(AllowOrigin::predicate(move |origin, _req| {
                if let Ok(origin_str) = origin.to_str() {
                    if let Ok(url) = url::Url::parse(origin_str) {
                        if url.host_str() == Some("localhost") {
                            return true;
                        }
                    }
                }
                origins.iter().any(|o| origin == o)
            }))
        }
        _ => unreachable!(),
    };

    info!("CORS mode: {}", opts.cors_mode);

    info!("CORS allow origins:");
    for origin in allow_origins {
        info!("  - {}", origin);
    }

    let app = axum::Router::new()
        .route("/", get(routes::index_get))
        .route("/version", get(routes::version_get))
        .route("/engine_manifest", get(routes::engine_manifest_get))
        .route("/supported_devices", get(routes::supported_devices_get))
        .route("/speakers", get(routes::speakers_get))
        .route("/speaker_info", get(routes::speaker_info_get))
        .route("/user_dict", get(routes::user_dict_get))
        .route("/import_user_dict", post(routes::import_user_dict_post))
        .route("/user_dict_word", post(routes::user_dict_word_post))
        .route(
            "/user_dict_word/:word_uuid",
            put(routes::user_dict_word_put).delete(routes::user_dict_word_delete),
        )
        .route("/is_initialized_speaker", get(routes::is_initialized_speaker_get))
        .route("/initialize_speaker", post(routes::initialize_speaker_post))
        .route("/audio_query", post(routes::audio_query_post))
        .route("/accent_phrases", post(routes::accent_phrases_post))
        .route("/mora_data", post(routes::mora_data_post))
        .route("/mora_pitch", post(routes::mora_pitch_post))
        .route("/mora_length", post(routes::mora_length_post))
        .route("/synthesis", post(routes::synthesis_post))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        );

    info!("Initializing managers...");
    let vvm_manager = VvmManager::new().await;
    VVM_MANAGER.get_or_init(|| Arc::new(Mutex::new(vvm_manager)));
    // let resource_manager = ResourceManager::new().await;
    // RESOURCE_MANAGER.get_or_init(|| Arc::new(Mutex::new(resource_manager)));

    routes::init_synthesizer(opts.use_gpu, opts.cpu_num_threads).await;

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", opts.host, opts.port))
        .await
        .unwrap();

    info!("Listening on {}:{}", opts.host, opts.port);

    axum::serve(listener, app).await.unwrap();
}
