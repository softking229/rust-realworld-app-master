#![feature(custom_attribute)]
extern crate bson;

extern crate iis;
extern crate hyper;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate chrono;

extern crate crypto;

extern crate futures;
extern crate tokio_core;

#[cfg(feature = "tiberius")]
extern crate tiberius;

extern crate toml;

#[macro_use]
extern crate lazy_static;

extern crate reroute;

extern crate jwt;

extern crate futures_state_stream;

extern crate slug;

extern crate rand;

extern crate unicase;

#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel_codegen;
#[cfg(feature = "diesel")]
extern crate dotenv;

#[cfg(feature = "diesel")]
pub mod schema;
#[cfg(feature = "diesel")]
pub mod models;
#[cfg(feature = "diesel")]
use models::*;

#[cfg(feature = "diesel")]
use dotenv::dotenv;

#[cfg(feature = "tiberius")]
use futures::Future;
#[cfg(feature = "tiberius")]
use tokio_core::reactor::Core;

#[cfg(feature = "tiberius")]
use tiberius::SqlConnection;
#[cfg(feature = "tiberius")]
use tiberius::stmt::ResultStreamExt;

use chrono::prelude::*;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::path::PathBuf;

use hyper::server::{Server, Request, Response};
use reroute::{RouterBuilder, Captures};
use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders};

use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "diesel")]
use diesel::prelude::*;
#[cfg(feature = "diesel")]
use diesel::pg::PgConnection;

pub fn since_the_epoch() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect(
        "Time went backwards",
    );
    since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000
}

#[cfg(test)]
use hyper::Client;

trait Container<T> {
    fn create_new_with_items(Vec<T>) -> Self;
}

#[cfg(feature = "tiberius")]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct User {
    email: String,
    token: String,
    username: String,
    bio: Option<String>,
    image: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[allow(non_snake_case)]
struct ArticleDTO {
    slug: String,
    title: String,
    description: String,
    body: String,
    tagList: Vec<String>,
    createdAt: NaiveDateTime,
    updatedAt: Option<NaiveDateTime>,
    favorited: bool,
    favoritesCount: i32,
    author: Profile,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct UpdateArticle {
    article: UpdateArticleDetail,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct UpdateArticleDetail {
    title: Option<String>,
    description: Option<String>,
    body: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Profile {
    username: String,
    bio: Option<String>,
    image: Option<String>,
    following: bool,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct ProfileResult {
    profile: Profile,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct CommentResult {
    pub comment: Comment,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[allow(non_snake_case)]
struct CommentsResult {
    pub comments: Vec<Comment>,
}

impl Container<Comment> for CommentsResult {
    fn create_new_with_items(comments: Vec<Comment>) -> CommentsResult {
        CommentsResult { comments: comments }
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct InternalError {
    errors: ErrorDetail,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct ErrorDetail {
    body: Vec<String>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct RegistrationDetails {
    email: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Registration {
    user: RegistrationDetails,
}

#[derive(Serialize, Deserialize)]
pub struct UserResult {
    user: User,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct LoginDetails {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Login {
    user: LoginDetails,
}

#[derive(Debug, Deserialize)]
struct Config {
    database: Option<DatabaseConfig>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DatabaseConfig {
    connection_string: Option<String>,
    database_name: Option<String>,
    create_database_secret: Option<String>,
    #[cfg(not(feature = "tiberius"))]
    DATABASE_URL: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct UpdateUser {
    user: UpdateUserDetail,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct AddComment {
    comment: AddCommentDetail,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[allow(non_snake_case)]
struct AddCommentDetail {
    body: String,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct CreateArticle {
    article: CreateArticleDetail,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct CreateArticleResult {
    article: ArticleDTO,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[allow(non_snake_case)]
struct CreateArticleDetail {
    title: String,
    description: String,
    body: String,
    tagList: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct UpdateUserDetail {
    email: Option<String>,
    username: Option<String>,
    password: Option<String>,
    bio: Option<String>,
    image: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct GetTagsResult {
    tags: Vec<String>,
}

static CONFIG_FILE_NAME: &'static str = r#"conduit.toml"#;

#[cfg(feature = "tiberius")]
lazy_static! {
    pub static ref CONNECTION_STRING : String = match get_database_config().connection_string {
            Some(cnn) => cnn,
            None => panic!("connection string not present in [database] section in {}", CONFIG_FILE_NAME),
        };
    pub static ref DATABASE_NAME : String = match get_database_config().database_name {
            Some(db_name) => db_name,
            None => panic!("database name not present in [database] section in {}", CONFIG_FILE_NAME),
        };  
    pub static ref CREATE_DATABASE_SECRET : String = match get_database_config().create_database_secret {
            Some(db_name) => db_name,
            None => panic!("create database secret not present in [database] section in {}", CONFIG_FILE_NAME),
        };  
}
#[cfg(not(feature = "tiberius"))]
lazy_static! {
    pub static ref DATABASE_URL : String = match get_database_config().DATABASE_URL {
            Some(db_name) => db_name,
            None => panic!("DATABASE_URL not present in [database] section in {}", CONFIG_FILE_NAME),
        };  
}

fn get_database_config() -> DatabaseConfig {

    let env_config = match env::var("DATABASECONFIG") {
        Ok(lang) => lang,
        Err(_) => "".to_string(),
    };
    let mut content = env_config.replace("&&&", "\n");

    let mut path = PathBuf::from(env::current_dir().unwrap());
    path.push(CONFIG_FILE_NAME);
    let display = path.display();

    if path.exists() {
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };

        match file.read_to_string(&mut content) {
            Err(why) => panic!("couldn't read {}: {}", display, why.description()),
            Ok(_) => print!("{} contains:\n{}", display, content),
        }
    }

    let toml_str: &str = &content;
    let config: Config = toml::from_str(toml_str).unwrap();

    let database_config: DatabaseConfig = match config.database {
        Some(database_config) => database_config,
        None => panic!("database not present in {}", CONFIG_FILE_NAME),
    };

    database_config
}

use hyper::header::{Authorization, Bearer};

fn prepare_parameters(mut req: Request) -> (String, i32) {
    let mut body = String::new();
    let _ = req.read_to_string(&mut body);
    let token = req.headers.get::<Authorization<Bearer>>();
    let logged_id: i32 = match token {
        Some(token) => {
            let jwt = &token.0.token;
            login(&jwt).unwrap()

        }
        _ => 0,
    };

    println!("body: {}, logged_id: {}", body, logged_id);
    (body, logged_id)
}

use unicase::UniCase;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

#[cfg(feature = "diesel")]
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

#[cfg(feature = "tiberius")]
fn process<'a, T>(
    mut res: Response,
    sql_command: &'static str,
    sql_select_command: &'static str,
    get_t_from_row: fn(tiberius::query::QueryRow) -> Option<T>,
    sql_params: &'a [&'a tiberius::ty::ToSql],
) where
    T: serde::Serialize,
{
    println!("process entered.");

    let mut result: Option<T> = None;
    {
        let mut sql = Core::new().unwrap();
        let get_cmd = SqlConnection::connect(sql.handle(), CONNECTION_STRING.as_str())
            .and_then(|conn| {
                conn.query(
                    format!("{};{}", sql_command, sql_select_command),
                    sql_params,
                ).for_each_row(|row| {
                        result = get_t_from_row(row);
                        Ok(())
                    })
            });
        sql.run(get_cmd).unwrap();
    }

    res.headers_mut().set(AccessControlAllowOrigin::Any);
    res.headers_mut().set(AccessControlAllowHeaders(vec![
        UniCase("content-type".to_owned()),
        UniCase("authorization".to_owned()),
    ]));
    res.headers_mut().set(ContentType(Mime(
        TopLevel::Application,
        SubLevel::Json,
        vec![(Attr::Charset, Value::Utf8)],
    )));

    if result.is_some() {
        let result = result.unwrap();
        let result = serde_json::to_string(&result).unwrap();
        println!("Sending '{:?}'", result.to_owned());
        let result: &[u8] = result.as_bytes();
        res.send(&result).unwrap();
    }
}

#[cfg(feature = "diesel")]
fn process<'a, T, U>(mut res: Response, process_params: fn(U) -> Option<T>, params: U)
where
    T: serde::Serialize,
    U: std::fmt::Debug,
{
    println!("process entered with params {:?}.", params);

    let result: Option<T> = process_params(params);

    res.headers_mut().set(AccessControlAllowOrigin::Any);
    res.headers_mut().set(AccessControlAllowHeaders(vec![
        UniCase("content-type".to_owned()),
        UniCase("authorization".to_owned()),
    ]));
    res.headers_mut().set(ContentType(Mime(
        TopLevel::Application,
        SubLevel::Json,
        vec![(Attr::Charset, Value::Utf8)],
    )));

    if result.is_some() {
        let result = result.unwrap();
        let result = serde_json::to_string(&result).unwrap();
        println!("Sending '{:?}'", result.to_owned());
        let result: &[u8] = result.as_bytes();
        res.send(&result).unwrap();
    }
}

#[cfg(feature = "tiberius")]
fn process_container<'a, T, U>(
    mut res: Response,
    sql_command: &'static str,
    sql_select_command: &'static str,
    get_t_from_row: fn(tiberius::query::QueryRow) -> Option<T>,
    _fix_u: fn(result: U),
    sql_params: &'a [&'a tiberius::ty::ToSql],
) where
    T: serde::Serialize,
    U: Container<T>,
    U: serde::Serialize,
{
    let mut items: Vec<T> = Vec::new();
    {
        let mut sql = Core::new().unwrap();
        let get_cmd = SqlConnection::connect(sql.handle(), CONNECTION_STRING.as_str())
            .and_then(|conn| {
                conn.query(
                    format!("{};{}", sql_command, sql_select_command),
                    sql_params,
                ).for_each_row(|row| {
                        let item = get_t_from_row(row);
                        if item.is_some() {
                            items.push(item.unwrap());
                        }
                        Ok(())
                    })
            });
        sql.run(get_cmd).unwrap();
    }

    res.headers_mut().set(AccessControlAllowOrigin::Any);
    res.headers_mut().set(AccessControlAllowHeaders(vec![
        UniCase("content-type".to_owned()),
        UniCase("authorization".to_owned()),
    ]));
    res.headers_mut().set(ContentType(Mime(
        TopLevel::Application,
        SubLevel::Json,
        vec![(Attr::Charset, Value::Utf8)],
    )));

    let result = U::create_new_with_items(items);
    let result = serde_json::to_string(&result).unwrap();
    let result: &[u8] = result.as_bytes();
    res.send(&result).unwrap();
}

#[cfg(feature = "diesel")]
fn process_container<'a, T, U, V>(
    mut res: Response,
    _fix_u: fn(result: U),
    process_params: fn(V) -> Vec<T>,
    params: V,
) where
    T: serde::Serialize,
    U: Container<T>,
    U: serde::Serialize,
    V: std::fmt::Debug,
{
    println!("process_container entered with params {:?}.", params);

    let mut items: Vec<T> = process_params(params);

    res.headers_mut().set(AccessControlAllowOrigin::Any);
    res.headers_mut().set(AccessControlAllowHeaders(vec![
        UniCase("content-type".to_owned()),
        UniCase("authorization".to_owned()),
    ]));
    res.headers_mut().set(ContentType(Mime(
        TopLevel::Application,
        SubLevel::Json,
        vec![(Attr::Charset, Value::Utf8)],
    )));

    let result = U::create_new_with_items(items);
    let result = serde_json::to_string(&result).unwrap();
    println!("Sending '{:?}'", result.to_owned());
    let result: &[u8] = result.as_bytes();
    res.send(&result).unwrap();
}

mod user;
use user::*;

mod article;
use article::*;

mod comment;
use comment::*;

#[cfg(feature = "tiberius")]
fn handle_row_no_value(_: tiberius::query::QueryRow) -> tiberius::TdsResult<()> {
    Ok(())
}
#[cfg(feature = "tiberius")]
fn handle_row_none(_: tiberius::query::QueryRow) -> Option<i32> {
    None
}

#[cfg(test)]
#[test]
fn get_tags_test() {
    let client = Client::new();

    let mut res = client.get("http://localhost:6767/api/tags").send().unwrap();
    let mut buffer = String::new();
    res.read_to_string(&mut buffer).unwrap();
    assert_eq!(res.status, hyper::Ok);
}


fn test_handler(_: Request, res: Response, _: Captures) {
    res.send(b"Test works.").unwrap();
}

fn hello_handler(_: Request, res: Response, _: Captures) {
    res.send(
        b"Hello from Rust application in Hyper running in Azure IIS.",
    ).unwrap();
}

#[cfg(feature = "tiberius")]
fn create_db_handler(mut req: Request, mut res: Response, _: Captures) {
    use hyper::status::StatusCode;

    let mut body = String::new();
    let _ = req.read_to_string(&mut body);
    if body == CREATE_DATABASE_SECRET.as_str() {
        let mut script = String::new();
        let mut f = File::open("database.sql").expect("Unable to open file");
        f.read_to_string(&mut script).expect(
            "Unable to read string",
        );

        let mut lp = Core::new().unwrap();
        let future = SqlConnection::connect(lp.handle(), CONNECTION_STRING.as_str())
            .and_then(|conn| {
                conn.query(script, &[]).for_each_row(handle_row_no_value)
            });
        lp.run(future).unwrap();
        res.send(b"Database created.").unwrap();
    } else {
        *res.status_mut() = StatusCode::Unauthorized;
    }
}
fn options_handler(_: Request, mut res: Response, _: Captures) {
    res.headers_mut().set(AccessControlAllowOrigin::Any);
    res.headers_mut().set(AccessControlAllowHeaders(vec![
        UniCase("content-type".to_owned()),
        UniCase("authorization".to_owned()),
    ]));
    res.headers_mut().set(ContentType(Mime(
        TopLevel::Application,
        SubLevel::Json,
        vec![(Attr::Charset, Value::Utf8)],
    )));
}

fn get_tags_handler(_: Request, mut res: Response, _: Captures) {
    #[cfg(feature = "diesel")] {
      process(res, get_tag_names, "");
    } 
    
    #[cfg(feature = "tiberius")]{
        let mut result: Option<GetTagsResult> = None;

        {
            let mut sql = Core::new().unwrap();
            let get_tags_cmd = SqlConnection::connect(sql.handle(), CONNECTION_STRING.as_str())
                .and_then(|conn| {
                    conn.query("SELECT STRING_AGG(Tag, ',') FROM [dbo].[Tags]", &[])
                        .for_each_row(|row| {
                            let all_tags: &str = row.get(0);
                            result = Some(GetTagsResult {
                                tags: all_tags.split(",").map(|q| q.to_string()).collect(),
                            });
                            Ok(())
                        })
                });
            sql.run(get_tags_cmd).unwrap();
        }
        
        res.headers_mut().set(AccessControlAllowOrigin::Any);
        res.headers_mut().set(ContentType(Mime(
            TopLevel::Application,
            SubLevel::Json,
            vec![(Attr::Charset, Value::Utf8)],
        )));

        if result.is_some() {
            let result = result.unwrap();
            let result = serde_json::to_string(&result).unwrap();
            let result: &[u8] = result.as_bytes();
            res.send(&result).unwrap();
        }
    }
}


fn main() {
    #[cfg(feature = "diesel")]
    {
        dotenv().ok();
    }

    let port = iis::get_port();

    let listen_on = format!("127.0.0.1:{}", port);

    println!("Listening on {}", listen_on);

    let mut builder = RouterBuilder::new();

    // Use raw strings so you don't need to escape patterns.
    builder.get(r"/", hello_handler);

    #[cfg(feature = "tiberius")] builder.post(r"/createdb", create_db_handler);

    builder.post(r"/api/users/login", authentication_handler);
    builder.post(r"/api/users", registration_handler);
    builder.get(r"/api/user", get_current_user_handler);
    builder.get(r"/test", test_handler);
    builder.put(r"/api/user", update_user_handler);
    builder.get(r"/api/profiles/.*", get_profile_handler);
    builder.post(r"/api/profiles/.*/follow", follow_handler);
    builder.delete(r"/api/profiles/.*/follow", unfollow_handler);
    builder.post(r"/api/articles", create_article_handler);

    builder.get(r"/api/tags", get_tags_handler);

    builder.post(r"/api/articles/.*/comments", add_comment_handler);
    builder.post(r"/api/articles/.*/favorite", favorite_article_handler);
    builder.delete(r"/api/articles/.*/favorite", unfavorite_article_handler);
    builder.put(r"/api/articles/.*", update_article_handler);
    builder.delete(r"/api/articles/.*/comments/.*", delete_comment_handler);
    builder.delete(r"/api/articles/.*", delete_article_handler);
    builder.get(r"/api/articles/feed", feed_handler);
    builder.get(r"/api/articles/.*/comments", get_comments_handler);
    builder.get(r"/api/articles/.*", get_article_handler);
    builder.get(r"/api/articles?.*", list_article_handler);
    builder.options("/api/.*", options_handler);

    let router = builder.finalize().unwrap();

    Server::http(listen_on).unwrap().handle(router).unwrap();

}
