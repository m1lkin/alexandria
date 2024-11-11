use bson::{doc, Bson};
use mongodb::Collection;
use mongodb::results::{InsertOneResult, UpdateResult};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::error::AppError;

pub async fn get_record<T, S>(id: &S, coll: &Collection<T>) -> Result<T, AppError>
where
    T: Send + Sync + DeserializeOwned,
    S: Clone, Bson: From<S>
{
    if let Ok(Some(rec)) = coll.find_one(doc! {"_id": id}).await {
        Ok(rec)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn update_record<T, S>(id: &S, rec: &T, coll: &Collection<T>)
    -> Result<UpdateResult, AppError>
where
    T: Send + Sync + DeserializeOwned + Into<Bson> + Clone,
    S: Clone, Bson: From<S>
{
    if let Ok(result) = coll.update_one(
        doc! {"_id": id},
        doc! {"$set": rec}
    ).await {
        Ok(result)
    } else {
        Err(AppError::InternalServerError)
    }
}

pub async fn create_record<T>(rec: &T, coll: &Collection<T>) -> Result<InsertOneResult, AppError>
where
    T: Send + Sync + DeserializeOwned + Serialize + Into<Bson>
{
    if let Ok(result) = coll.insert_one(rec).await {
        Ok(result)
    } else {
        Err(AppError::InternalServerError)
    }
}
