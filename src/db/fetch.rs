use diesel::PgConnection;
use actix_web::Result;

/// A conversion which references `self` to fetch `R` (Row) from the database
pub trait Fetch<R> {
    fn fetch(&self, conn: &PgConnection) -> Result<R>;
}
