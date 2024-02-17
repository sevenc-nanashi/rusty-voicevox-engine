mod routes;
use axum::routing::get;
use clap::Parser;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
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

    let cors = match opts.cors_mode.as_str() {
        "all" => {
            if opts.cors_origins.is_some() {
                warn!("--cors-origins は --cors-mode all の場合には無視されます。");
            }
            CorsLayer::new().allow_origin(AllowOrigin::any())
        }
        "localapps" => {
            let mut origins = vec!["app://".to_string(), "http://localhost".to_string()];
            if let Some(cors_origins) = opts.cors_origins {
                origins.extend(cors_origins);
            }
            CorsLayer::new().allow_origin(AllowOrigin::list(
                origins
                    .iter()
                    .map(|s| s.parse::<http::HeaderValue>().unwrap()),
            ))
        }
        _ => unreachable!(),
    };

    let app = axum::Router::new()
        .route("/", get(routes::index::get))
        .route("/version", get(routes::version::get))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    tracing_subscriber::fmt::init();

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", opts.host, opts.port))
        .await
        .unwrap();

    info!("Listening on {}:{}", opts.host, opts.port);

    axum::serve(listener, app).await.unwrap();
}
