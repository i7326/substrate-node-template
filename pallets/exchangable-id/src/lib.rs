#![cfg_attr(not(feature = "std"), no_std)]
use codec::alloc::string::ToString;
use sp_std::vec::Vec;

use codec::{Encode, Decode};
use frame_support::{
	debug, decl_module, decl_storage, decl_error, decl_event, ensure, StorageValue, StorageMap, Parameter,
	traits::{Randomness, Currency, ExistenceRequirement},
	Hashable,
};
use sp_io::hashing::blake2_128;
use frame_system::{self as system, ensure_signed};
use sp_runtime::{DispatchError, traits::{AtLeast32Bit, Bounded, Member, Hash}};
use crate::linked_item::{LinkedList, LinkedItem};
use sp_runtime::print;
use uuid::Uuid;

mod linked_item;

pub type ExchangableId<T> = <T as frame_system::Trait>::Hash;
pub type UUID = Vec<u8>;
pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: Currency<Self::AccountId>;
	type Randomness: Randomness<Self::Hash>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type ExchangableIdLinkedItem<T> = LinkedItem<ExchangableId<T>>;
type OwnedExchangableIdsList<T> = LinkedList<OwnedExchangableIds<T>, <T as system::Trait>::AccountId, ExchangableId<T>>;

decl_storage! {
	trait Store for Module<T: Trait> as ExchangableIds {
		pub ExchangableIdsCount get(fn exchangable_ids_count): u128 = 0;
		pub ExchangableIds get(fn exchangable_ids): map hasher(blake2_128_concat) ExchangableId<T> => Option<UUID>;
		pub OwnedExchangableIds get(fn owned_exchangable_ids): map hasher(blake2_128_concat) (T::AccountId, Option<ExchangableId<T>>) => Option<ExchangableIdLinkedItem<T>>;
		pub ExchangableIdOwners get(fn exchangable_id_owner): map hasher(blake2_128_concat) ExchangableId<T> => Option<T::AccountId>;
		pub ExchangableIdPrices get(fn exchangable_id_price): map hasher(blake2_128_concat) ExchangableId<T> => Option<BalanceOf<T>>;
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		ExchangableIdExists,
		ExchangableIdsCountOverflow,
		InvalidExchangableId,
		RequireDifferentParent,
		RequireOwner,
		NotForSale,
		PriceTooLow,
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		UUID = Vec<u8>,
		ExchangableId = <T as system::Trait>::Hash,
		Balance = BalanceOf<T>,
	{
		Created(AccountId, ExchangableId, UUID),
		Transferred(AccountId, AccountId, ExchangableId),
		Ask(AccountId, ExchangableId, Option<Balance>),
		Sold(AccountId, AccountId, ExchangableId, Balance),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let uuid = Uuid::from_bytes(Self::random_value(&sender)).to_string().into_bytes();
			let exchangable_id = T::Hashing::hash_of(&Self::random_value(&sender));
			ensure!(
				!ExchangableIds::<T>::contains_key(&exchangable_id),
				Error::<T>::ExchangableIdExists
			);
			Self::insert_exchangable_id(&sender, exchangable_id, &uuid);
			Self::deposit_event(RawEvent::Created(sender, exchangable_id, uuid));
		}

		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, exchangable_id: ExchangableId<T>) {
			let sender = ensure_signed(origin)?;

			ensure!(<OwnedExchangableIds<T>>::contains_key((&sender, Some(exchangable_id))), Error::<T>::RequireOwner);

			Self::do_transfer(&sender, &to, exchangable_id);

			Self::deposit_event(RawEvent::Transferred(sender, to, exchangable_id));
		}

		#[weight = 0]
		pub fn buy(origin, exchangable_id: ExchangableId<T>, price: BalanceOf<T>) {
			let sender = ensure_signed(origin)?;
			let owner = Self::exchangable_id_owner(exchangable_id).ok_or(Error::<T>::InvalidExchangableId)?;
			let exchangable_id_price = Self::exchangable_id_price(exchangable_id).ok_or(Error::<T>::NotForSale)?;
			ensure!(price >= exchangable_id_price, Error::<T>::PriceTooLow);
			T::Currency::transfer(&sender, &owner, exchangable_id_price, ExistenceRequirement::KeepAlive)?;
			<ExchangableIdPrices<T>>::remove(exchangable_id);
			Self::do_transfer(&owner, &sender, exchangable_id);
			Self::deposit_event(RawEvent::Sold(owner, sender, exchangable_id, exchangable_id_price));
		}
	}
}

impl<T: Trait> Module<T> {

	fn total() -> u128 {
        Self::total()
    }

	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Module<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}

	fn insert_owned_exchangable_id(owner: &T::AccountId, exchangable_id: ExchangableId<T>) {
		<OwnedExchangableIdsList<T>>::append(owner, exchangable_id);
		<ExchangableIdOwners<T>>::insert(exchangable_id, owner);
	}

	fn insert_exchangable_id(owner: &T::AccountId, exchangable_id: ExchangableId<T>, uuid: &UUID) {
		ExchangableIds::<T>::insert(exchangable_id, uuid);
		ExchangableIdsCount::mutate(|total| *total += 1);
		Self::insert_owned_exchangable_id(owner, exchangable_id);
	}

	fn do_transfer(from: &T::AccountId, to: &T::AccountId, exchangable_id: ExchangableId<T>)  {
		<OwnedExchangableIdsList<T>>::remove(&from, exchangable_id);
		Self::insert_owned_exchangable_id(&to, exchangable_id);
	}
}
