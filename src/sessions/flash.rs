use actix_web::middleware::session::{SessionStorage, RequestSession, Session};
use actix_web::Result;

use crate::templates::Page;

/// Add a flash message to the current session
pub trait SessionFlash {
    fn flash<T: Into<String>>(&self, message: T) -> Result<()>;
    fn apply_flash(&self, page: &mut Page) -> Result<()>;
}

const FLASH_MESSAGES_KEY: &'static str = "flash-msgs";

impl SessionFlash for Session {
    fn flash<T: Into<String>>(&self, message: T) -> Result<()> {
        let value = self.get::<Vec<String>>(FLASH_MESSAGES_KEY)?;
        let mut messages = match value {
            Some(messages) => messages,
            None => vec![],
        };
        messages.push(message.into());
        self.set(FLASH_MESSAGES_KEY, messages)?;
        Ok(())
    }

    fn apply_flash(&self, page: &mut Page) -> Result<()>  {
        let value = self.get::<Vec<String>>(FLASH_MESSAGES_KEY)?;
        if let Some(messages) = value {
            for message in messages {
                page.info(message);
            }
        }
        self.remove(FLASH_MESSAGES_KEY);
        Ok(())
    }
}
