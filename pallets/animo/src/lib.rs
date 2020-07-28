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
pub struct Mutation {
	pub changes: Vec<Change>,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, Hash, Debug)]
pub struct Change {
	/// primary object of relation
	pub primary: ID,

	/// description of relation between primary object and value
	pub relation: Vec<ID>,

	/// value before modification
	pub before: Option<Value>,

	/// value after modification
	pub after: Option<Value>,
}

pub type ID = H256;
// avoid because of bug at typegen
// pub type IDS = Vec<ID>;
pub type Value = Vec<u8>;

// This pallet's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as AnimoModule {
		AnimoStore get(fn animo_store): double_map hasher(blake2_128_concat) ID, hasher(blake2_128_concat) Vec<ID> => Option<Value>
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		/// Changes applied
		MutationAccepted(Mutation, AccountId),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// no changes
		EmptyChanges,
		/// too many changes
		TooManyChanges,
		/// no relations
		EmptyRelations,
		/// relation vector is not ordered
		RelationsIsNotOrdered,
		/// change must have state mutation
		BeforeAndAfterStatesAreEqual,
		/// before state mismatch
		BeforeStateMismatch
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
		pub fn modify(origin, mutation: Mutation) -> DispatchResult {
			// Check it was signed and get the signer.
			let who = ensure_signed(origin)?;

			let validity = Self::validate_mutation(&mutation)?;

			// TODO check permissions

			Self::update_storage(&mutation, validity.priority)?;

			Self::deposit_event(RawEvent::MutationAccepted(mutation, who));

			Ok(())
		}
	}
}

// "Internal" function, callable by code
impl<T: Trait> Module<T> {

	pub fn validate_mutation(mutation: &Mutation) -> Result<ValidTransaction, Error::<T>> {
		ensure!(!mutation.changes.is_empty(), Error::<T>::EmptyChanges);

		ensure!(mutation.changes.len() < 256, Error::<T>::TooManyChanges);

		for change in mutation.changes.iter() {
			ensure!(!change.relation.is_empty(), Error::<T>::EmptyRelations);
			// ensure!(!change.before.is_empty(), "no before state");
			// ensure!(!change.after.is_empty(), "no after state");

			ensure!(change.relation.windows(2).all(|w| w[0] <= w[1]), Error::<T>::RelationsIsNotOrdered);

			ensure!(change.before != change.after, Error::<T>::BeforeAndAfterStatesAreEqual);

			let current = <AnimoStore>::get(change.primary, change.relation.clone());
			let error = match &current {
				None => {
					match &change.before {
						None => false,
						Some(_) => true
					}
				}
				Some(current) => {
					match &change.before {
						None => true,
						Some(before) => current != before
					}
				}
			};
			// println!("before state do not match {:?} vs {:?} [ {:?} ]", current, change.before, error);
			ensure!(!error, Error::<T>::BeforeStateMismatch);
		}

		Ok(ValidTransaction {
			requires: Vec::new(),
			provides: Vec::new(),
			priority: 1 as u64,
			longevity: TransactionLongevity::max_value(),
			propagate: true
		})
	}

	fn update_storage(mutation: &Mutation, _reward: u64) -> DispatchResult {
		for change in mutation.changes.iter() {
			match &change.after {
				Some(after) => {
					<AnimoStore>::insert(change.primary, change.relation.clone(), after.clone());
				}
				None => {
					<AnimoStore>::remove(change.primary, change.relation.clone());
				}
			}
		}

		Ok(())
	}
}
