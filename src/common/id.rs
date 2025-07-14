use std::marker::PhantomData;

pub struct Id<T>(String, PhantomData<T>);

impl<T> Id<T> {
    pub fn new(id: String) -> Self {
        Id(id, PhantomData)
    }

    pub fn as_string(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id(self.0.clone(), PhantomData)
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({})", self.0)
    }
}

impl<T> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct TransactionMarker;
pub struct RefundMarker;
pub struct InvoiceMarker;
pub struct PaymentMarker;
pub struct WithdrawMarker;
pub struct DepositMarker;
pub struct TransferMarker;

pub type TransactionId = Id<TransactionMarker>;
pub type RefundId = Id<RefundMarker>;
pub type InvoiceId = Id<InvoiceMarker>;
pub type PaymentId = Id<PaymentMarker>;
pub type WithdrawId = Id<WithdrawMarker>;
pub type DepositId = Id<DepositMarker>;
pub type TranserId = Id<TransferMarker>;

impl TransactionId {
    pub fn new_collection(id: String) -> Self {
        Id::new(format!("collection_{}", id))
    }
}