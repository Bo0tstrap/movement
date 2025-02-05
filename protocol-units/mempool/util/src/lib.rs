use serde::{Deserialize, Serialize};

use movement_types::{Block, Transaction, Id};
use std::cmp::Ordering;

pub trait MempoolTransactionOperations {
    
    // todo: move mempool_transaction methods into separate trait

    /// Checks whether a mempool transaction exists in the mempool.
    async fn has_mempool_transaction(&self, transaction_id :Id) -> Result<bool, anyhow::Error>;

    /// Adds a mempool transaction to the mempool.
    async fn add_mempool_transaction(&self, tx: MempoolTransaction) -> Result<(), anyhow::Error>;

    /// Removes a mempool transaction from the mempool.
    async fn remove_mempool_transaction(&self, transaction_id:Id) -> Result<(), anyhow::Error>;

    /// Pops mempool transaction from the mempool.
    async fn pop_mempool_transaction(&self) -> Result<Option<MempoolTransaction>, anyhow::Error>;
 
    /// Gets a mempool transaction from the mempool.
    async fn get_mempool_transaction(&self, transaction_id:Id) -> Result<Option<MempoolTransaction>, anyhow::Error>;

    /// Pops the next n mempool transactions from the mempool.
    async fn pop_mempool_transactions(&self, n: usize) -> Result<Vec<MempoolTransaction>, anyhow::Error> {
        let mut mempool_transactions = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(mempool_transaction) = self.pop_mempool_transaction().await? {
                mempool_transactions.push(mempool_transaction);
            } else {
                break;
            }
        }
        Ok(mempool_transactions)
    }

    /// Checks whether the mempool has the transaction.
    async fn has_transaction(&self, transaction_id:Id) -> Result<bool, anyhow::Error> {
        self.has_mempool_transaction(transaction_id).await
    }

    /// Adds a transaction to the mempool.
    async fn add_transaction(&self, tx: Transaction) -> Result<(), anyhow::Error> {

        if self.has_transaction(tx.id()).await? {
            return Ok(());
        }

        let mempool_transaction = MempoolTransaction::slot_now(tx);
        self.add_mempool_transaction(mempool_transaction).await

    }

    /// Removes a transaction from the mempool.
    async fn remove_transaction(&self, transaction_id:Id) -> Result<(), anyhow::Error> {
        self.remove_mempool_transaction(transaction_id).await
    }

    /// Pops transaction from the mempool.
    async fn pop_transaction(&self) -> Result<Option<Transaction>, anyhow::Error> {
        let mempool_transaction = self.pop_mempool_transaction().await?;
        Ok(mempool_transaction.map(|mempool_transaction| mempool_transaction.transaction))
    }

    /// Gets a transaction from the mempool.
    async fn get_transaction(&self, transaction_id:Id) -> Result<Option<Transaction>, anyhow::Error> {
        let mempool_transaction = self.get_mempool_transaction(transaction_id).await?;
        Ok(mempool_transaction.map(|mempool_transaction| mempool_transaction.transaction))
    }

    /// Pops the next n transactions from the mempool.
    async fn pop_transactions(&self, n: usize) -> Result<Vec<Transaction>, anyhow::Error> {
        let mempool_transactions = self.pop_mempool_transactions(n).await?;
        Ok(mempool_transactions.into_iter().map(|mempool_transaction| mempool_transaction.transaction).collect())
    }

}


pub trait MempoolBlockOperations {
    
    /// Checks whether a block exists in the mempool.
    async fn has_block(&self, block_id :Id) -> Result<bool, anyhow::Error>;

    /// Adds a block to the mempool.
    async fn add_block(&self, block: Block) -> Result<(), anyhow::Error>;

    /// Removes a block from the mempool.
    async fn remove_block(&self, block_id:Id) -> Result<(), anyhow::Error>;
 
    /// Gets a block from the mempool.
    async fn get_block(&self, block_id:Id) -> Result<Option<Block>, anyhow::Error>;

}

/// Wraps a transaction with a timestamp for help ordering.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MempoolTransaction {
    pub transaction: Transaction,
    pub timestamp: u64,
    pub slot_seconds : u64
}

impl PartialOrd for MempoolTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Ordered first by slot_seconds, then by transaction.
/// This allows us to use a BTreeSet to order transactions by slot_seconds, and then by transaction and pop them off in order.
impl Ord for MempoolTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        // First, compare by slot_seconds
        match self.slot_seconds.cmp(&other.slot_seconds) {
            Ordering::Equal => {}
            non_equal => return non_equal,
        }
        // If slot_seconds are equal, then compare by transaction
        self.transaction.cmp(&other.transaction)
    }
}

impl MempoolTransaction {

    const SLOT_SECONDS : u64 = 2;

    /// Creates a test MempoolTransaction.
    pub fn test() -> Self {
        Self {
            transaction: Transaction::test(),
            timestamp: 0,
            slot_seconds: Self::SLOT_SECONDS
        }
    }

    pub fn at_time(transaction: Transaction, timestamp: u64) -> Self {
        let floor = (
            timestamp / Self::SLOT_SECONDS
        ) * Self::SLOT_SECONDS;
        Self {
            transaction,
            timestamp : floor,
            slot_seconds: Self::SLOT_SECONDS
        }
    }

    pub fn new(transaction: Transaction, timestamp: u64, slot_seconds: u64) -> Self {
        Self {
            transaction,
            timestamp,
            slot_seconds
        }
    }

    /// Creates a new MempoolTransaction with the current timestamp floored to the nearest slot.
    /// todo: probably want to move this out to a factory.
    pub fn slot_now(transaction : Transaction) -> MempoolTransaction {
        
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    
        Self::at_time(transaction, timestamp)
        
    }

    pub fn id(&self) ->Id {
        self.transaction.id()
    }
    

}