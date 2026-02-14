pub mod identity;
pub mod provider;
pub mod session;

use serde::Serialize;
use uuid::Uuid;

pub trait ObjectId {
    fn id(&self) -> Option<Uuid>;
    fn set_id(&mut self, id: Uuid);
    fn username(&self) -> String;
}

pub trait IntoPublic {
    type Public: Serialize;
    fn into_public(self) -> Self::Public;
}
