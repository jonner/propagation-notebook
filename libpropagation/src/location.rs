use toasty::Deferred;

use crate::collecting::HarvestEvent;

#[derive(Debug, Clone, toasty::Model)]
pub struct Location {
    #[auto]
    #[key]
    pub id: u64,

    pub name: String,
    pub latitude: f32,
    pub longitude: f32,

    #[has_many(pair=location)]
    pub harvest_events: Deferred<Vec<HarvestEvent>>,
}

impl Location {
    pub fn reference(&self) -> String {
        format!("{}: {}", self.id, self.name)
    }
}
