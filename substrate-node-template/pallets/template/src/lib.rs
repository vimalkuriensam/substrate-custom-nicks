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
		Blake2_128Concat,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::{prelude::vec::Vec, TypeInfo};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		#[pallet::constant]
		type MaxLength: Get<u32>;
	}

	#[derive(Debug, Encode, Decode, Default, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct User<T: Config> {
		pub name: BoundedVec<u8, T::MaxLength>,
		pub age: u8,
		pub title: BoundedVec<u8, T::MaxLength>,
	}

	#[pallet::storage]
	pub type AccountToUserInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, User<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		UserInfoAdded(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		TooLong,
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
			<AccountToUserInfo<T>>::insert(&sender, user);
			Self::deposit_event(Event::<T>::UserInfoAdded(sender));
			Ok(())
		}
	}
}
