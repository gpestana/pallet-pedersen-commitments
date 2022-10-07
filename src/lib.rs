#![cfg_attr(not(feature = "std"), no_std)]
pub use frame_system::pallet::*;

use codec::{Decode, Encode};
use frame_support::pallet_prelude::{MaxEncodedLen, RuntimeDebug};
use frame_support::storage::types::StorageMap;
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use curve25519_dalek::scalar::Scalar;
use sha2::Sha512;

#[cfg(test)]
mod mock;

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Copy)]
pub struct Commitment<BlockNumber> {
    point_g: [u8; 64],
    point_h: [u8; 64],
    payload: [u8; 64],
    committed_at: BlockNumber,
    revealed_at: Option<BlockNumber>,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum CommitmentState {
    // Commitment exists and has not been revealed yet
    CommitmentSecret,
    // Commitment has been revealed
    CommitmentRevealed,
    // Commitment has never been made
    CommitmentNotFound,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // Declare the pallet type, this is a placeholder
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Add runtime configurations, pallet types and constants
    #[pallet::config]
    pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>
			+ TryInto<Event<Self>>;

        #[pallet::constant]
        type MaxLenCommitMessage: Get<u32>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // Successful commitment revealed
        CommitmentRevealed {
            revealer: <T as frame_system::Config>::AccountId,
            revealed_at: <T as frame_system::Config>::BlockNumber,
            message: Vec<u8>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        // No unrevealed or commited commitments for the caller
        NoActiveCommitmentForOrigin,
        // Reveal didn't match the commitment
        UnableToReveal,
        // Message revelead is too large
        CommitmentMessageIsTooLarge,
    }

    // Declares storage types
    #[pallet::storage]
    #[pallet::getter(fn commitment)]
    pub type CommitmentsStorage<T: Config> =
        StorageMap<_, Twox256, T::AccountId, Commitment<T::BlockNumber>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)] // TODO: set proper weight
        pub fn commit(
            origin: OriginFor<T>,
            payload: [u8; 64],
            point_g: [u8; 64], // TODO: refactor -- is a constant for ristretto
            point_h: [u8; 64],
        ) -> DispatchResult {
            let commiter = ensure_signed(origin.clone())?;

            let committed_at = frame_system::Pallet::<T>::block_number();

            let commitment = Commitment {
                payload,
                point_g,
                point_h,
                committed_at,
                revealed_at: None,
            };

            <CommitmentsStorage<T>>::insert(commiter, commitment);

            Ok(())
        }

        #[pallet::weight(10_000)] // TODO: set proper weight
        pub fn reveal_and_verify(
            origin: OriginFor<T>,
            message: Vec<u8>,
            secret: Vec<u8>,
        ) -> DispatchResult {
            let revealer = ensure_signed(origin)?;

            ensure!(
                message.len() <= T::MaxLenCommitMessage::get() as usize,
                Error::<T>::CommitmentMessageIsTooLarge
            );

            // refactor to crypto module
            let r_secret = Scalar::hash_from_bytes::<Sha512>(&secret);
            let r_message = Scalar::hash_from_bytes::<Sha512>(&message);

            let mut commitment =
                Self::commitment(&revealer).ok_or(Error::<T>::NoActiveCommitmentForOrigin)?;

            let r_commitment_payload = Scalar::hash_from_bytes::<Sha512>(&commitment.payload);
            let r_commitment_point_g = Scalar::hash_from_bytes::<Sha512>(&commitment.point_g);
            let r_commitment_point_h = Scalar::hash_from_bytes::<Sha512>(&commitment.point_h);

            let reveal_ok = {
                let gm = r_message * r_commitment_point_g;
                let hr = r_secret * r_commitment_point_h;

                r_commitment_payload == gm + hr
            };

            if !reveal_ok {
                return Err(Error::<T>::UnableToReveal.into());
            }

            commitment.revealed_at = Some(frame_system::Pallet::<T>::block_number());
            <CommitmentsStorage<T>>::insert(&revealer, commitment);

            Self::deposit_event(Event::CommitmentRevealed {
                revealer,
                revealed_at: commitment.revealed_at.expect("block_number exists"),
                message,
            });

            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test() {
		assert!(true, "");
	}
	
}
