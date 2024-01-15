use crate::{
    data::{Id, PSP34Event},
    PSP34Error,
};
use ink::{prelude::{vec::Vec, vec}, storage::Mapping};

#[ink::storage_item]
#[derive(Default, Debug)]
pub struct Data {
    attributes: Mapping<(Id, Vec<u8>), Vec<u8>>,
}

impl Data {
    pub fn get_attribute(&self, id: Id, key: Vec<u8>) -> Option<Vec<u8>> {
        self.attributes.get((&id, &key))
    }

    pub fn set_attribute(
        &mut self,
        id: Id,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<Vec<PSP34Event>, PSP34Error> {
        self.attributes.insert((&id, &key), &value);
        Ok(vec![PSP34Event::AttributeSet {
            id,
            key,
            data: value,
        }])
    }
}