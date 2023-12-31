use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Form, Router,
};
const DEFAULT_ADDR: &str = "127.0.0.1:33333";
use faststr::FastStr;
use serde::Deserialize;
use volo_gen::volo::example::{ItemServiceClientBuilder, ItemServiceClient, ItemServiceServer};
use volo_gen::volo::example::GetItemRequest;
type RpcClient = ItemServiceClient;
type RpcClientBuilder = ItemServiceClientBuilder;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = DEFAULT_ADDR.parse().unwrap();
    let rpc_cli = RpcClientBuilder::new("rpcdemo").address(addr).build();

    // build the application with router
    let app = Router::new()
        .route("/ping/:keys", get(ping_key).with_state(rpc_cli.clone()))
        .route("/ping", get(ping))
        .route("/get/:keys", get(get_key).with_state(rpc_cli.clone()))
        .route(
            "/set",
            get(show_set_form).post(set_key).with_state(rpc_cli.clone()),
        )
        .route("/del", get(show_del_form).post(del_key).with_state(rpc_cli));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ping() -> (StatusCode, &'static str) {
    (StatusCode::OK, "pong")
}

async fn ping_key(
    Path(key): Path<String>, 
    State(rpc_cli): State<RpcClient>
) -> Response {
    let res = rpc_cli.get_item(
        GetItemRequest {
            op: "ping".into(),
            key: " ".into(),
            val: " ".into(),
        }
    ).await;
    match res {
        Ok(v) => {
            (StatusCode::OK, v.val.to_string()).into_response()
        },
        Err(e) => {
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

/// Get a key
async fn get_key(Path(key): Path<String>, State(rpc_cli): State<RpcClient>) -> Response {
    let res = rpc_cli.get_item(
        GetItemRequest {
            op: "get".into(),
            key: FastStr::from(Arc::new(key)),
            val: " ".into(),
        }
    ).await;
    match res {
        Ok(v) => {
            (StatusCode::OK, v.val.to_string()).into_response()
        },
        Err(e) => {
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

#[derive(Deserialize, Debug)]
struct FormKey {
    key: String,
    value: String
}

/// Show the form for set a key
async fn show_set_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/set" method="post">
                    <label for="key">
                        Enter key:
                        <input type="text" name="key">
                    </label>
                    <label for="key">
                        Enter value:
                        <input type="text" name="value">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

/// Set a key
async fn set_key(State(rpc_cli): State<RpcClient>, Form(setkey): Form<FormKey>) -> Response {
    let res = rpc_cli.get_item(
        GetItemRequest {
            op: "set".into(),
            key: FastStr::from(Arc::new(setkey.key)),
            val: FastStr::from(Arc::new(setkey.value)),
        }
    ).await;
    match res {
        Ok(v) => {
            (StatusCode::OK, v.val.to_string()).into_response()
        },
        Err(e) => {
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

async fn show_del_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/del" method="post">
                    <label for="key">
                        Enter key:
                        <input type="text" name="key">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

#[derive(Deserialize, Debug)]
struct FormKey2 {
    key: String
}

async fn del_key(
    State(rpc_cli): State<RpcClient>,
    Form(delkey): Form<FormKey2>,
) -> (StatusCode, &'static str) {
    let res = rpc_cli.get_item(
        GetItemRequest {
            op: "del".into(),
            key: FastStr::from(Arc::new(delkey.key)),
            val: " ".into(),
        }
    ).await;
    match res {
        Ok(_v) => {
            (StatusCode::OK, "Successful delete")
        },
        Err(_e) => {
            (StatusCode::NOT_FOUND, "Delete error")
        }
    }
}