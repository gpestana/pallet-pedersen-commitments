#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

use codec::{Decode, Encode};
use frame_support::pallet_prelude::{MaxEncodedLen, RuntimeDebug};
use frame_support::storage::types::StorageMap;
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use sha2::Sha512;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

// TODO(gpestana): abstract undelying crypto scheme
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Copy)]
pub struct Commitment<BlockNumber> {
    point_g: [u8; 32],
    point_h: [u8; 32],
    payload: [u8; 32],
    committed_at: BlockNumber,
    revealed_at: Option<BlockNumber>,
}

impl<T> Commitment<T> {
    pub fn verify_commitment(self, secret: &Vec<u8>, message: &Vec<u8>) -> bool {
        let r = Scalar::hash_from_bytes::<Sha512>(&secret);
        let m = Scalar::hash_from_bytes::<Sha512>(&message);

        // TODO(gpestana): handle unwraps and map to results
        let commitment_payload = CompressedRistretto::from_slice(&self.payload)
            .decompress()
            .unwrap();
        let commitment_g = CompressedRistretto::from_slice(&self.point_g)
            .decompress()
            .unwrap();
        let commitment_h = CompressedRistretto::from_slice(&self.point_h)
            .decompress()
            .unwrap();

        let payload = {
            let gm = m * commitment_g;
            let hr = r * commitment_h;

            //r_commitment_payload == gm + hr
            gm + hr
        };

        commitment_payload == payload
    }
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

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
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
    #[pallet::getter(fn commitments)]
    pub type CommitmentsStorage<T: Config> =
        StorageMap<_, Twox256, T::AccountId, Commitment<T::BlockNumber>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)] // TODO: set proper weight
        pub fn commit(
            origin: OriginFor<T>,
            payload: [u8; 32],
            point_g: [u8; 32],
            point_h: [u8; 32],
        ) -> DispatchResult {
            let commiter = ensure_signed(origin)?;

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

            let mut commitment =
                Self::commitments(&revealer).ok_or(Error::<T>::NoActiveCommitmentForOrigin)?;

            if !commitment.verify_commitment(&secret, &message) {
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
