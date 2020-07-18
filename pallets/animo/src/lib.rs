#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error,
	dispatch::{DispatchResult, Vec},
	ensure
};
use frame_system::{self as system, ensure_signed};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use codec::{Decode, Encode};

use sp_core::H256;
use sp_runtime::{
	transaction_validity::{TransactionLongevity, ValidTransaction},
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// Modification to be dispatched
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct Modification {
	pub changes: Vec<Change>,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct Change {
	/// primary object of relation
	pub primary: ID,

	/// description of relation between primary object and value
	pub relation: IDS,

	/// value before modification
	pub before: Value,

	/// value after modification
	pub after: Value,
}

pub type ID = H256;
pub type IDS = Vec<ID>;
pub type Value = Vec<u8>;


// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as AnimoModule {
		AnimoStore get(fn animo_store): double_map hasher(blake2_128_concat) ID, hasher(blake2_128_concat) IDS => Option<Value>
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Modification was applied
		ModificationAccepted(Modification, AccountId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		/// Dispatch a single modification
		#[weight = 1_000_000]
		pub fn modify(origin, modification: Modification) -> DispatchResult {
			// Check it was signed and get the signer.
			let who = ensure_signed(origin)?;

			let validity = Self::validate_modification(&modification)?;

			// TODO check permissions

			Self::update_storage(&modification, validity.priority)?;

			Self::deposit_event(RawEvent::ModificationAccepted(modification, who));

			Ok(())
		}
	}
}

// "Internal" function, callable by code
impl<T: Trait> Module<T> {

	pub fn validate_modification(modification: &Modification) -> Result<ValidTransaction, &'static str> {
		for change in modification.changes.iter() {
			ensure!(!change.relation.is_empty(), "no relation description");
			ensure!(!change.before.is_empty(), "no before state");
			ensure!(!change.after.is_empty(), "no after state");
		}

		Ok(ValidTransaction {
			requires: Vec::new(),
			provides: Vec::new(),
			priority: 1 as u64,
			longevity: TransactionLongevity::max_value(),
			propagate: true
		})
	}

	fn update_storage(modification: &Modification, _reward: u64) -> DispatchResult {
		for change in modification.changes.iter() {
			let mut relation = change.relation.clone();
			relation.sort();

			<AnimoStore>::insert(change.primary, relation, change.after.clone());
		}

		Ok(())
	}
}
