#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::{OptionQuery, *},
		traits::{Currency, OnUnbalanced, ReservableCurrency},
		Blake2_128Concat,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::{prelude::vec::Vec, TypeInfo};
	use sp_runtime::traits::{StaticLookup, Zero};

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	type AddressLookup<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
	type NegetiveImbalanceOf<T> =
		<<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		type Currency: ReservableCurrency<Self::AccountId>;
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type Slashed: OnUnbalanced<NegetiveImbalanceOf<Self>>;
		#[pallet::constant]
		type MaxLength: Get<u32>;
		#[pallet::constant]
		type DepositValue: Get<BalanceOf<Self>>;
	}

	#[derive(Debug, Encode, Decode, Default, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct User<T: Config> {
		pub name: BoundedVec<u8, T::MaxLength>,
		pub age: u8,
		pub title: BoundedVec<u8, T::MaxLength>,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_user_info)]
	pub type AccountToUserInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (User<T>, BalanceOf<T>), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		UserInfoUpdated(T::AccountId),
		UserInfoAdded(T::AccountId),
		UserInfoDeleted(T::AccountId),
		ValueReserved(T::AccountId, BalanceOf<T>),
		ValueUnreserved(T::AccountId, BalanceOf<T>),
		SlashedBalance(T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		TooLong,
		UserNotAdded,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn insert_user_info(
			origin: OriginFor<T>,
			name: Vec<u8>,
			age: u8,
			title: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let bounded_name =
				BoundedVec::<u8, T::MaxLength>::try_from(name).map_err(|_| Error::<T>::TooLong)?;
			let bounded_title =
				BoundedVec::<u8, T::MaxLength>::try_from(title).map_err(|_| Error::<T>::TooLong)?;
			let user = User { name: bounded_name, age, title: bounded_title };
			let deposit = if let Some((_, deposit)) = <AccountToUserInfo<T>>::get(&sender) {
				Self::deposit_event(Event::<T>::UserInfoUpdated(sender.clone()));
				deposit
			} else {
				let deposit = T::DepositValue::get();
				T::Currency::reserve(&sender, deposit)?;
				Self::deposit_event(Event::<T>::ValueReserved(sender.clone(), deposit));
				deposit
			};
			<AccountToUserInfo<T>>::insert(&sender, (user, deposit));
			Self::deposit_event(Event::<T>::UserInfoAdded(sender));
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn clear_user_info(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let (_, deposit) =
				<AccountToUserInfo<T>>::get(&sender).ok_or(Error::<T>::UserNotAdded)?;
			T::Currency::unreserve(&sender, deposit);
			Self::deposit_event(Event::<T>::ValueUnreserved(sender.clone(), deposit));
			<AccountToUserInfo<T>>::remove(&sender);
			Self::deposit_event(Event::<T>::UserInfoDeleted(sender));
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn kill_user_info(origin: OriginFor<T>, recipient: AddressLookup<T>) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			let target = T::Lookup::lookup(recipient)?;
			let (_, deposit) =
				<AccountToUserInfo<T>>::get(&target).ok_or(Error::<T>::UserNotAdded)?;
			let (negetive_imbalance, total_balance) = T::Currency::slash_reserved(&target, deposit);
			Self::deposit_event(Event::<T>::SlashedBalance(target.clone(), total_balance));
			T::Slashed::on_unbalanced(negetive_imbalance);
			<AccountToUserInfo<T>>::remove(&target);
			Self::deposit_event(Event::<T>::UserInfoDeleted(target));
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn force_user_info(
			origin: OriginFor<T>,
			recipient: AddressLookup<T>,
			name: Vec<u8>,
			age: u8,
			title: Vec<u8>,
		) -> DispatchResult {
			T::ForceOrigin::ensure_origin(origin)?;
			let bounded_name =
				BoundedVec::<u8, T::MaxLength>::try_from(name).map_err(|_| Error::<T>::TooLong)?;
			let bounded_title =
				BoundedVec::<u8, T::MaxLength>::try_from(title).map_err(|_| Error::<T>::TooLong)?;
			let target = T::Lookup::lookup(recipient)?;
			let user = User { name: bounded_name, age, title: bounded_title };
			let deposit = match <AccountToUserInfo<T>>::get(&target) {
				Some((_, deposit)) => deposit,
				None => Zero::zero(),
			};
			<AccountToUserInfo<T>>::insert(&target, (user, deposit));
			Ok(())
		}
	}
}
