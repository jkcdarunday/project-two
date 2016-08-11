extern crate iron;
extern crate persistent;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate rustc_serialize;

use std::io::Read as StdRead;

// Iron-related dependencies
use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
use persistent::Read;

// Database-related dependencies
use r2d2_redis::RedisConnectionManager;
use redis::Commands;

// Encoding-related dependencies
use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable)]
struct Setter {
    value: i32,
    username: String,
    password: String,
    action: String,
}

#[derive(Copy, Clone)]
pub struct RedisDatabase;
impl Key for RedisDatabase {
    type Value = r2d2::Pool<RedisConnectionManager>;
}

fn hit(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<Read<RedisDatabase>>().unwrap();
    let conn = pool.get().unwrap();

    let mut payload = String::new();
    req.body.read_to_string(&mut payload).unwrap();

    let request: Setter = json::decode(&payload).unwrap();

    let _:bool = conn.set("counter", request.value).unwrap();

    Ok(Response::with((status::Ok, format!("Cut off one head, and {} shall grow.", payload))))
}

pub fn redis_connect(database: &str, pool_size: u32) -> r2d2::Pool<RedisConnectionManager> {
    let config = r2d2::Config::builder().pool_size(pool_size).build();
    let manager = RedisConnectionManager::new(database).unwrap();
    r2d2::Pool::new(config, manager).unwrap()
}

fn main() {
    let mut chain = Chain::new(hit);
    chain.link(Read::<RedisDatabase>::both(redis_connect("redis://localhost", 512)));
    Iron::new(chain).listen_with("0.0.0.0:3000", 512, iron::Protocol::Http, None).unwrap();
}
