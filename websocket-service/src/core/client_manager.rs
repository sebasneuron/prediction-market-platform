use std::{collections::HashMap, sync::Arc};

use utility_helpers::ws::types::ChannelType;
use uuid::Uuid;

use crate::core::SafeSender;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SpecialKindOfClients {
    OrderService,
}

#[derive(Debug)]
pub struct SubscriptionAndClientManager {
    subscription: HashMap<ChannelType, HashMap<Uuid, SafeSender>>,
    special_clients: HashMap<SpecialKindOfClients, Uuid>,
}

impl SubscriptionAndClientManager {
    pub fn new() -> Self {
        Self {
            subscription: HashMap::new(),
            special_clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, channel: ChannelType, client_id: Uuid, tx: SafeSender) {
        self.subscription
            .entry(channel)
            .or_insert_with(HashMap::new)
            .insert(client_id, tx);
    }

    pub fn remove_client(&mut self, channel: &ChannelType, client_id: &Uuid) {
        if let Some(clients) = self.subscription.get_mut(channel) {
            clients.remove(client_id);
            if clients.is_empty() {
                self.subscription.remove(&channel);
            }
        }
    }
    pub fn get_clients(&self, channel: &ChannelType) -> Option<&HashMap<Uuid, SafeSender>> {
        self.subscription.get(channel)
    }

    pub fn cleanup(&mut self) {
        self.subscription.clear();
    }

    pub fn remove_client_without_channel(&mut self, client_id: &Uuid) {
        for (_, clients) in self.subscription.iter_mut() {
            if let Some(_) = clients.get(client_id) {
                clients.remove(client_id);
            }
        }
    }

    pub fn set_special_client(
        &mut self,
        client_id: Uuid,
        sender: SafeSender,
        channel: ChannelType,
        kind: SpecialKindOfClients,
    ) {
        if !self.is_client_id_exist(&client_id) {
            // adding client as normal client
            self.add_client(channel, client_id, sender);
        }
        self.special_clients.insert(kind, client_id);
    }

    pub fn get_special_client(&self, kind: SpecialKindOfClients) -> Option<SafeSender> {
        let client_id = self.special_clients.get(&kind);
        if let Some(client_id) = client_id {
            let client_tx = self.get_client_transmitter(client_id);
            if let Some(tx) = client_tx {
                return Some(tx);
            }
        }

        None
    }

    pub fn unset_special_client(&mut self, client_type: &SpecialKindOfClients) {
        self.special_clients.remove(client_type);
    }

    pub fn get_client_transmitter(&self, client_id: &Uuid) -> Option<SafeSender> {
        for (_, clients) in self.subscription.iter() {
            for (client, tx) in clients {
                if *client == *client_id {
                    return Some(Arc::clone(tx));
                }
            }
        }
        None
    }

    fn is_client_id_exist(&self, client_id: &Uuid) -> bool {
        for (_, clients) in self.subscription.iter() {
            for (uuid, _) in clients {
                if *client_id == *uuid {
                    return true;
                }
            }
        }
        false
    }
}
