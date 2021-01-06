mod engine;
mod error;
mod manager;

use engine::SqlEngine;
use error::Error;
use napi::{CallContext, Env, JsObject, JsString, JsUndefined, Property};
use napi_derive::{js_function, module_exports};

type Result<T> = std::result::Result<T, Error>;

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
