// use pallet_evm::BalanceConverter;
use frame_system::RawOrigin;
use pallet_evm::{
	AddressMapping, ExitError, ExitSucceed, HashedAddressMapping, PrecompileFailure,
	PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use sp_core::U256;
use sp_core::{hashing::keccak_256, H160};
use sp_runtime::traits::{BlakeTwo256, UniqueSaturatedInto};
// use crate::precompiles::{Self::dispatch, get_method_id, get_slice};
use sp_runtime::traits::Dispatchable;
use sp_runtime::AccountId32;
use sp_std::vec;

use crate::{Runtime, RuntimeCall, RuntimeOrigin};
pub const TEMPLATE_PRECOMPILE_INDEX: u64 = 2049;
// this is Template smart contract's(0x0000000000000000000000000000000000000801) sr25519 address
pub const Template_CONTRACT_ADDRESS: &str = "0x5CwnBK9Ack1mhznmCnwiibCNQc174pYQVktYW3ayRpLm4K2X";
pub struct TemplatePrecompile;

impl TemplatePrecompile {
	pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		let txdata = handle.input();
		let method_id = Self::get_slice(txdata, 0, 4)?;
		let method_input = txdata
			.get(4..)
			.map_or_else(vec::Vec::new, |slice| slice.to_vec()); // Avoiding borrowing conflicts

		match method_id {
			id if id == Self::get_method_id("doSomething(uint32)") => {
				Self::do_something(handle, &method_input)
			}
			_ => Err(PrecompileFailure::Error {
				exit_status: ExitError::InvalidRange,
			}),
		}
	}

	fn do_something(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
		let something = Self::parse_something(data)?.into();
		// let amount: U256 = handle.context().apparent_value;
		// let amount_sub =
		// 	<Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
		// 		.ok_or(ExitError::OutOfFund)?;

		// Create the add_stake call
		let call = RuntimeCall::TemplateModule(pallet_template::Call::<Runtime>::do_something {
			something,
		});
		// Self::dispatch the add_stake call
		Self::dispatch(handle, call)
	}

	fn parse_something(data: &[u8]) -> Result<u32, PrecompileFailure> {
		if data.len() < 32 {
			return Err(PrecompileFailure::Error {
				exit_status: ExitError::InvalidRange,
			});
		}
		let mut bytes = [0u8; 4];
		bytes.copy_from_slice(Self::get_slice(data, 28, 32)?);
		Ok(u32::from_be_bytes(bytes))
	}

	fn dispatch(handle: &mut impl PrecompileHandle, call: RuntimeCall) -> PrecompileResult {
		let account_id =
			<HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
				handle.context().caller,
			);

		let result = call.dispatch(RuntimeOrigin::signed(handle.context().caller.into()));
		match &result {
			Ok(post_info) => log::info!("Self::dispatch succeeded. Post info: {:?}", post_info),
			Err(dispatch_error) => {
				log::error!("Self::dispatch failed. Error: {:?}", dispatch_error)
			}
		}
		match result {
			Ok(_) => Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: vec![],
			}),
			Err(_) => Err(PrecompileFailure::Error {
				exit_status: ExitError::Other("Subtensor call failed".into()),
			}),
		}
	}

	pub fn get_slice(data: &[u8], from: usize, to: usize) -> Result<&[u8], PrecompileFailure> {
		let maybe_slice = data.get(from..to);
		if let Some(slice) = maybe_slice {
			Ok(slice)
		} else {
			log::error!(
				"fail to get slice from data, {:?}, from {}, to {}",
				&data,
				from,
				to
			);
			Err(PrecompileFailure::Error {
				exit_status: ExitError::InvalidRange,
			})
		}
	}

	pub fn bytes_to_account_id(account_id_bytes: &[u8]) -> Result<AccountId32, PrecompileFailure> {
		AccountId32::try_from(account_id_bytes).map_err(|_| {
			log::info!("Error parsing account id bytes {:?}", account_id_bytes);
			PrecompileFailure::Error {
				exit_status: ExitError::InvalidRange,
			}
		})
	}

	pub fn get_method_id(method_signature: &str) -> [u8; 4] {
		// Calculate the full Keccak-256 hash of the method signature
		let hash = keccak_256(method_signature.as_bytes());

		// Extract the first 4 bytes to get the method ID
		[hash[0], hash[1], hash[2], hash[3]]
	}
}
