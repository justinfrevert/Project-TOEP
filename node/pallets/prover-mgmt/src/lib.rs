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
	use frame_support::{
		inherent::Vec,
		pallet_prelude::*,
		traits::{BalanceStatus, Currency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use risc0_zkvm::{SegmentReceipt, SessionReceipt};

	type ImageId = [u32; 8];

	#[pallet::pallet]
	// TODO: Needs proper BoundedVec encoding from offchain in order to get bounded types working
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: Currency<<Self as frame_system::Config>::AccountId>
			+ ReservableCurrency<Self::AccountId>;
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

	#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	// Information related to a requst for proving of a program
	pub struct ProofRequest<T: Config> {
		requester: T::AccountId,
		reward: BalanceOf<T>,
		args: Vec<Vec<u32>>,
	}

	#[pallet::storage]
	/// Store for all programs
	pub(super) type Programs<T: Config> =
		StorageMap<_, Blake2_128Concat, ImageId, Vec<u8>, OptionQuery>;

	#[pallet::storage]
	/// Requests which have been submitted for `ImageId`, unique per set of args + image id
	pub(super) type ProofRequests<T: Config> =
		StorageMap<_, Blake2_128Concat, ImageId, ProofRequest<T>, OptionQuery>;

	#[pallet::storage]
	/// Store Some(proof), if the program's proof was verified
	pub(super) type ProofsByImage<T: Config> =
		StorageMap<_, Blake2_128Concat, ImageId, Vec<(Vec<u32>, u32)>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProofRequested {
			image_id: ImageId,
			args: Vec<Vec<u32>>,
		},
		/// Proof was successfully verified and will be stored
		ProofVerified,
		/// A program was uploaded
		ProgramUploaded {
			image_id: ImageId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Tried to upload a program which already exists
		ProgramAlreadyExists,
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
			// The bincode-serialized program
			program: Vec<u8>,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			ensure!(!Programs::<T>::contains_key(image_id), Error::<T>::ProgramAlreadyExists);

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
			args: Vec<Vec<u32>>,
			reward: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			T::Currency::reserve(&who, reward)?;

			ProofRequests::<T>::insert(
				image_id,
				ProofRequest { requester: who, reward, args: args.clone() },
			);

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
			receipt_data: Vec<(Vec<u32>, u32)>,
			journal: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Programs::<T>::get(image_id).is_some(), Error::<T>::ProgramDoesNotExist);

			// If a request for proof of the program exists, the submitter needs to receive the
			// designated reward
			if let Some(proof_request) = ProofRequests::<T>::get(image_id) {
				T::Currency::repatriate_reserved(
					&proof_request.requester,
					&who,
					proof_request.reward,
					BalanceStatus::Free,
				)?;
			}

			let segments: Vec<SegmentReceipt> = receipt_data
				.clone()
				.into_iter()
				.map(|(seal, index)| SegmentReceipt { seal, index })
				.collect();

			let receipt = SessionReceipt { segments, journal };
			receipt.verify(image_id).map_err(|_| Error::<T>::ProofNotVerified)?;

			Self::deposit_event(Event::<T>::ProofVerified);

			// TODO: Also see if there is some image id verification
			ProofsByImage::<T>::insert(image_id, receipt_data);
			Ok(())
		}
	}
}
