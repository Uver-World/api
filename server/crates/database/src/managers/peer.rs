use mongodb::{
    bson::doc,
    error::Error,
    results::{DeleteResult, InsertOneResult},
    Collection,
};

use crate::peer::Peer;

pub struct PeersManager {
    pub peers: Collection<Peer>,
}

impl PeersManager {
    pub fn init(peers: Collection<Peer>) -> Self {
        Self { peers }
    }

    pub async fn peers_exist(&self, server_unique_id: impl Into<String>) -> Result<bool, Error> {
        Ok(self
            .peers
            .count_documents(doc! { "server_unique_id": server_unique_id.into() }, None)
            .await?
            != 0)
    }

    pub async fn create_peer(&self, peer: &Peer) -> Result<InsertOneResult, Error> {
        let target = self.peers.insert_one(peer, None).await?;
        Ok(target)
    }

    pub async fn from_server_id(&self, server_unique_id: &str) -> Result<Option<Peer>, Error> {
        match self
            .peers
            .find_one(doc! { "server_unique_id": server_unique_id }, None)
            .await?
        {
            Some(peer) => Ok(Some(peer)),
            None => Ok(None),
        }
    }

    pub async fn delete_peer(
        &self,
        server_unique_id: &str,
    ) -> Result<Option<DeleteResult>, String> {
        Ok(Some(
            self.peers
                .delete_one(doc! {"server_unique_id": server_unique_id}, None)
                .await
                .map_err(|err| err.to_string())?,
        ))
    }
}

impl Clone for PeersManager {
    fn clone(&self) -> Self {
        Self {
            peers: self.peers.clone(),
        }
    }
}