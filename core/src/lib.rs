pub mod identity;
pub mod provider;
pub mod session;

use uuid::Uuid;

pub trait ObjectId {
    fn id(&self) -> Option<Uuid>;
    fn set_id(&mut self, id: Uuid);
}
