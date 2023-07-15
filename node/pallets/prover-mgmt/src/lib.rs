#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use risc0_zkvm::{serde::from_slice, sha::Digest, SegmentReceipt, SessionReceipt};

	type ImageId = [u8; 32];

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		type MaxArgsLength: Get<u32>;
		// Max length of programs
		type MaxProgramLength: Get<u32>;
		// Max Length of proofs
		type MaxProofLength: Get<u32>;
	}

	#[pallet::storage]
	/// Store for all programs
	pub(super) type Programs<T: Config> =
		StorageMap<_, Blake2_128Concat, ImageId, BoundedVec<u8, T::MaxProgramLength>, OptionQuery>;

	#[pallet::storage]
	/// Store Some(proof), if the program's proof was verified
	pub(super) type ProofsByImage<T: Config> =
		StorageMap<_, Blake2_128Concat, ImageId, BoundedVec<u32, T::MaxProofLength>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProofRequested {
			image_id: ImageId,
			args: BoundedVec<u8, T::MaxArgsLength>,
		},
		/// A program was uploaded
		ProgramUploaded {
			image_id: ImageId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Tried to verify a proof but the program did not exist
		ProgramDoesNotExist,
		/// Could not verify proof
		ProofInvalid,
		/// Proof did not pass verification
		ProofNotVerified,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn upload_program(
			origin: OriginFor<T>,
			// Todo: find how to verify image id
			image_id: ImageId,
			program: BoundedVec<u8, T::MaxProgramLength>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			<Programs<T>>::insert(image_id, program);

			Self::deposit_event(Event::ProgramUploaded { image_id });
			Ok(())
		}

		/// Request a proof of a known program, passing some arguments
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn request_proof(
			origin: OriginFor<T>,
			image_id: ImageId,
			args: BoundedVec<u8, T::MaxArgsLength>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			Self::deposit_event(Event::ProofRequested { image_id, args });

			Ok(())
		}

		/// An extrinsic which verifies proofs for programs, forming a trustless relationship for
		/// others to check the verification result
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn store_and_verify_proof(
			origin: OriginFor<T>,
			image_id: ImageId,
			proof_vec: BoundedVec<u32, T::MaxProofLength>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			ensure!(Programs::<T>::get(image_id).is_some(), Error::<T>::ProgramDoesNotExist);

			let receipt: SessionReceipt =
				from_slice(&proof_vec).map_err(|_| Error::<T>::ProofInvalid)?;

			receipt.verify(image_id).map_err(|_| Error::<T>::ProofNotVerified)?;

			// TODO: Also see if there is some image id verification
			ProofsByImage::<T>::insert(image_id, proof_vec);
			Ok(())
		}
	}
}
