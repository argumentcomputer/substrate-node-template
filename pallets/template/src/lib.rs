#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
extern crate alloc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use alloc::string::{String, ToString};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, runtime_print};
	use frame_system::pallet_prelude::*;
	use sp_std::{rc::Rc, vec::Vec};
	use yatima_core::{
		check::check_def,
		defs::Defs,
		parse::{package::parse_defs, span::Span, term::input_cid},
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		TheoremProved(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ParseError,
		TypeError,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// A dispatchable that takes a serialized Yatima program as a vector of bytes,
		/// then parses and typechecks the theorem to prove its validity
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn theorem_prover(origin: OriginFor<T>, input: Vec<u8>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let who = ensure_signed(origin)?;

			let contents = match String::from_utf8(input) {
				Ok(s) => s,
				Err(_) => String::from("Error: Decode failed"),
			};

			// Parse Yatima code into definitions
			let defs = match parse_defs(input_cid(&contents), Defs::new())(Span::new(&contents)) {
				Ok(d) => Rc::new(d.1 .0),
				Err(e) => {
					runtime_print!("\n\n{}\n", e);
					return Err(Error::<T>::ParseError)?
				},
			};

			// Iterate over Defs and typecheck each
			for (name, _) in defs.names.iter() {
				match check_def(defs.clone(), name, false) {
					Ok(ty) => {
						runtime_print!(
							"✓ {}: {}",
							name.to_string(),
							ty.pretty(Some(&name.to_string()), false)
						)
					},
					Err(err) => {
						runtime_print!("✕ {}: {}", name.to_string(), err);
						return Err(Error::<T>::TypeError)?
					},
				}
			}
			runtime_print!("All proofs complete");

			// Emit an event.
			Self::deposit_event(Event::TheoremProved(who));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}
