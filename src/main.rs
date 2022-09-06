use clap::Parser;
use rand::RngCore;
use warp::Filter;
use std::sync::Arc;

mod database;
use database::*;

mod hasher;
use hasher::*;

mod lookup;
use lookup::*;

#[derive(Debug,Parser)]
struct Args
{
    /// The database connection string (in tokio_postgres format)
    #[clap(short,long)]
    connect: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    pretty_env_logger::init();

    let args = Args::parse();

    let database = Database::new(&args.connect).await.unwrap();

    let key: [u8; 16] = if let Some(key) = database.metadata("hash_key").await.unwrap()
    {
        base64::decode(key).unwrap().try_into().unwrap()
    }
    else
    {
        let mut new_key = [0; 16];
        rand::thread_rng().fill_bytes(&mut new_key);
        database.set_metadata("hash_key", &base64::encode(new_key.as_slice())).await.unwrap()    ;
        new_key
    };

    let hasher = NickHasher::new(&key);

    let state = Arc::new(LookupState::new(database, hasher));

    let routes = filters::lookup(state.clone())
                .or(filters::record(state.clone()))
                .with(warp::log("seen_nick"));

    warp::serve(routes).run(([0,0,0,0],3000)).await;

    Ok(())
}

mod filters
{
    use super::*;

    fn with_state(state: Arc<LookupState>) -> impl Filter<Extract = (Arc<LookupState>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || state.clone())
    }

    pub fn lookup(state: Arc<LookupState>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::path!("lookup")
            .and(warp::post())
            .and(with_state(state.clone()))
            .and(warp::body::bytes())
            .then(handlers::lookup)
    }

    pub fn record(state: Arc<LookupState>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::path!("record")
            .and(warp::post())
            .and(with_state(state.clone()))
            .and(warp::body::bytes())
            .then(handlers::record)
    }
}

mod handlers
{
    use bytes::Bytes;

    use super::*;

    pub async fn lookup(state: Arc<LookupState>, nick: Bytes) -> impl warp::Reply
    {
        if let Ok(nick) = String::from_utf8(nick.to_vec())
        {
            match state.lookup(&nick).await
            {
                Ok(result) =>
                {
                    let reply = if result { "true" } else { "false" };
                    warp::reply::with_status(reply, warp::http::StatusCode::OK)
                }
                Err(e) =>
                {
                    log::error!("got lookup error {}", e);
                    warp::reply::with_status("", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        else
        {
            warp::reply::with_status("", warp::http::StatusCode::BAD_REQUEST)
        }
    }

    pub async fn record(state: Arc<LookupState>, nick: Bytes) -> impl warp::Reply
    {
        if let Ok(nick) = String::from_utf8(nick.to_vec())
        {
            if let Ok(()) = state.record(&nick).await
            {
                warp::reply::with_status("", warp::http::StatusCode::OK)
            }
            else
            {
                warp::reply::with_status("", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        else
        {
            warp::reply::with_status("", warp::http::StatusCode::BAD_REQUEST)
        }
    }
}