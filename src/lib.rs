use std::{fmt, io};

use async_trait::async_trait;
use deadpool::managed::{Manager, Pool, PoolError, RecycleResult};
use napi::{CallContext, Env, JsObject, JsString, JsUndefined, Property, Status};
use napi_derive::{js_function, module_exports};
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

type Result<T> = std::result::Result<T, Error>;

struct RedmondManager {
    config: Config,
}

impl RedmondManager {
    fn new(url: &str) -> crate::Result<Self> {
        let config = Config::from_ado_string(url)?;

        Ok(Self { config })
    }
}

#[async_trait]
impl Manager<Client<Compat<TcpStream>>, Error> for RedmondManager {
    async fn create(&self) -> crate::Result<Client<Compat<TcpStream>>> {
        let tcp = TcpStream::connect(self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        let client = Client::connect(self.config.clone(), tcp.compat_write()).await?;

        Ok(client)
    }

    async fn recycle(&self, _: &mut Client<Compat<TcpStream>>) -> RecycleResult<Error> {
        Ok(())
    }
}

#[derive(Clone)]
struct SqlEngine {
    pool: Pool<Client<Compat<TcpStream>>, Error>,
}

#[derive(Debug)]
enum Error {
    Tiberius(tiberius::error::Error),
    Timeout,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tiberius(e) => write!(f, "{}", e),
            Self::Io(e) => write!(f, "{}", e),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}

impl From<tiberius::error::Error> for Error {
    fn from(err: tiberius::error::Error) -> Self {
        Self::Tiberius(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<PoolError<Error>> for Error {
    fn from(e: PoolError<Error>) -> Self {
        match e {
            PoolError::Timeout(_) => Error::Timeout,
            PoolError::Backend(e) => e,
        }
    }
}

impl From<Error> for napi::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::Tiberius(e) => napi::Error::new(Status::GenericFailure, e.to_string()),
            Error::Io(e) => napi::Error::new(Status::GenericFailure, e.to_string()),
            Error::Timeout => napi::Error::new(Status::GenericFailure, String::from("Timeout")),
        }
    }
}

impl std::error::Error for Error {}

impl SqlEngine {
    fn new(uri: &str) -> crate::Result<Self> {
        let manager = RedmondManager::new(uri)?;
        let pool = Pool::new(manager, 5);

        Ok(Self { pool })
    }

    async fn select_1(&self) -> crate::Result<i32> {
        let mut conn = self.pool.get().await?;

        let row = conn.query("SELECT 1", &[]).await?.into_row().await?.unwrap();

        Ok(row.get(0).unwrap())
    }
}

#[js_function(1)]
fn sql_constructor(ctx: CallContext) -> napi::Result<JsUndefined> {
    let url = ctx.get::<JsString>(0)?.into_utf8()?;

    let mut this: JsObject = ctx.this_unchecked();
    let engine = SqlEngine::new(url.as_str()?)?;

    ctx.env.wrap(&mut this, engine)?;
    ctx.env.get_undefined()
}

#[js_function(0)]
fn add_select_1(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let sql_engine: &SqlEngine = ctx.env.unwrap(&this)?;
    let sql_engine = sql_engine.clone();

    ctx.env
        .execute_tokio_future(async move { Ok(sql_engine.select_1().await?) }, |&mut env, res| {
            env.create_int32(res)
        })
}

#[module_exports]
pub fn init(mut exports: JsObject, env: Env) -> napi::Result<()> {
    let sql_engine = env.define_class(
        "SqlEngine",
        sql_constructor,
        &[Property::new(&env, "select_1")?.with_method(add_select_1)],
    )?;

    exports.set_named_property("SqlEngine", sql_engine)?;

    Ok(())
}
