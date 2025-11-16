use std::collections::BTreeSet;

use iroh::EndpointId;
use iroh_gossip::TopicId;
use serde::{Deserialize, Serialize};
use anyhow::Result;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ticket {
    pub topic_id: TopicId,
    pub bootstrap: BTreeSet<EndpointId>,
}

impl Ticket {
    pub fn new_random() -> Self {
        let topic_id = TopicId::from_bytes(rand::random());
        Self::new(topic_id)
    }

    pub fn new(topic_id: TopicId) -> Self {
        Self {
            topic_id,
            bootstrap: Default::default(),
        }
    }
    pub fn deserialize(input: &str) -> Result<Self> {
        <Self as iroh_tickets::Ticket>::deserialize(input).map_err(Into::into)
    }
    pub fn serialize(&self) -> String {
        <Self as iroh_tickets::Ticket>::serialize(self)
    }
}

impl iroh_tickets::Ticket for Ticket {
    const KIND: &'static str = "sm64crypto-";

    fn to_bytes(&self) -> Vec<u8> {
        postcard::to_stdvec(&self).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, iroh_tickets::ParseError> {
        let ticket = postcard::from_bytes(bytes)?;
        Ok(ticket)
    }
}