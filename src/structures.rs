use bson::{doc, Bson};
use chrono::{DateTime, Utc};
use mongodb::{bson, Collection, Database};
use serde::{Deserialize, Serialize};
use crate::{db::{create_record, get_record, update_record}, error::AppError};

pub struct IdGenerator {
    sequence_collection: Collection<Counter>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Counter {
    #[serde(rename = "_id")]
    pub id: String,
    counter: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Rating {
    Up,
    Down,
    None,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct RatedPost {
    pub post: i64,
    pub rating: Rating,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    pub password_hash: String,
    summary: Vec<i64>,
    pub rated: Vec<RatedPost>,
    last_upload: DateTime<Utc>,
    register_date: DateTime<Utc>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    #[serde(rename = "_id")]
    pub id: i64,
    title: String,
    description: String,
    author: String,
    author_name: String,
    keywords: Vec<String>,
    pub files: Vec<File>,
    pub rating: i32,
    upload_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResource {
    #[serde(rename = "_id")]
    pub id: i64,
    title: String,
    description: String,
    author: String,
    author_name: String,
    keywords: Vec<String>,
    pub files: Vec<File>,
    pub rating: i32,
    upload_time: DateTime<Utc>,
    pub rate: Rating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub(crate) filename: String,
    pub(crate) size: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateResource {
    title: String,
    description: String,
    keywords: Vec<String>,
}

impl IdGenerator {
    pub async fn new(db: Database) -> IdGenerator {
        IdGenerator {
            sequence_collection: db.collection("counters"),
        }
    }
    pub async fn get_id(&self, count_id: String) -> Result<i64, AppError> {
        let mut counter = match get_record(&count_id, &self.sequence_collection).await {
            Ok(v) => v,
            Err(e) => {
                if let AppError::InternalServerError = e {
                    return Err(e);
                }
                let counter = Counter {id: count_id, counter: 0};
                create_record(&counter, &self.sequence_collection).await?;
                counter
            }
        };
        
        counter.counter += 1;
        update_record(&counter.id, &counter, &self.sequence_collection).await?;
        
        Ok(counter.counter)
    }
}

impl CreateResource {
    pub async fn into_resource(
        mut self,
        author: String,
        author_name: String,
        id_gen: &IdGenerator,
    ) -> Result<Resource, AppError> {
        Ok(Resource {
            id: id_gen.get_id("post".into()).await?,
            title: self.title,
            description: self.description,
            author,
            author_name,
            keywords: self.keywords,
            files: vec![],
            rating: 0,
            upload_time: Utc::now(),
        })
    }
}

impl Resource {
    pub fn into_send_resource(self, rating: Rating) -> SendResource {
        SendResource {
            id: self.id,
            title: self.title,
            author: self.author,
            author_name: self.author_name,
            description: self.description,
            keywords: self.keywords,
            files: self.files,
            rating: self.rating,
            upload_time: self.upload_time,
            rate: rating,
        }
    }
} 

impl User {
    pub fn new(id: String, username: String, password_hash: String) -> Self {
        User {
            id,
            username,
            password_hash,
            summary: vec![],
            rated: vec![],
            last_upload: Utc::now(),
            register_date: Utc::now(),
        }
    }

    pub fn add_rated(&mut self, rated_post: RatedPost) -> i32 {
        let mut rating = 0;
        if let Some(post) = self.rated.iter().find(|v| v.post == rated_post.post) {
            match post.rating {
                Rating::Up => rating -= 1,
                Rating::Down => rating += 1,
                Rating::None => {}
            }
            self.rated.retain(|v| rated_post.post != v.post);
        }
        match rated_post.rating {
            Rating::Up => rating += 1,
            Rating::Down => rating -= 1,
            Rating::None => {}
        }
        self.rated.push(rated_post);
        println!("{}", rating);
        rating
    }
}

impl From<User> for Bson {
    fn from(user: User) -> Bson {
        Bson::Document(doc! {
            "_id": user.id,
            "username": user.username,
            "password_hash": user.password_hash,
            "summary": user.summary,
            "rated": user.rated,
            "last_upload": user.last_upload.to_rfc3339(),
            "register_date": user.register_date.to_rfc3339(),
        })
    }
}

impl From<Resource> for Bson {
    fn from(value: Resource) -> Self {
        Bson::Document(doc! {
            "_id": value.id,
            "title": value.title,
            "description": value.description,
            "author": value.author,
            "author_name": value.author_name,
            "keywords": value.keywords,
            "files": value.files,
            "rating": value.rating,
            "upload_time": value.upload_time.to_rfc3339(),
        })
    }
}

impl From<File> for Bson {
    fn from(value: File) -> Self {
        Bson::Document(doc! {
            "filename": value.filename,
            "size": value.size,
        })
    }
}

impl From<Counter> for Bson {
    fn from(value: Counter) -> Self {
        Bson::Document(doc! {
            "_id": value.id,
            "counter": value.counter
        })
    }
}

impl From<RatedPost> for Bson {
    fn from(value: RatedPost) -> Self {
        Bson::Document(doc! {
            "post": value.post,
            "rating": match value.rating {
                
            Rating::Up => {"Up"}Rating::Down => {"Down"}Rating::None => {"None"}}
        })
    }
}