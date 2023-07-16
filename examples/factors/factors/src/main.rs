use methods::{FACTORS_ELF, FACTORS_ID};

use codec::Encode;
use risc0_zkvm::{serde::to_vec, Executor, ExecutorEnv, SegmentReceipt, SessionReceipt};
use std::time::Instant;
use subxt::{
	config::WithExtrinsicParams,
	ext::{
		sp_core::{
			sr25519::{Pair as SubxtPair, Public, Signature},
			Pair as SubxtPairT,
		},
		sp_runtime::{traits::Verify, AccountId32},
	},
	tx::{BaseExtrinsicParams, PairSigner, PlainTip},
	OnlineClient, PolkadotConfig, SubstrateConfig,
};

// // Runtime types, etc
#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod substrate_node {}

use substrate_node::runtime_types::bounded_collections::bounded_vec::BoundedVec;

type ApiType = OnlineClient<
	WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
>;

#[tokio::main]
async fn main() {
	let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
	// This is the well-known //Alice key. Don't use in a real application
	let restored_key = SubxtPair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();
	let signer = PairSigner::new(restored_key);

	let serialized_program = bincode::serialize(FACTORS_ELF).unwrap();
	let deserialized_debug: &[u8] = bincode::deserialize(&serialized_program).unwrap();
	println!("Uploaded for image id {:?}", FACTORS_ID);

	api.tx()
		.sign_and_submit_then_watch_default(
			&substrate_node::tx()
				.prover_mgmt()
				// Send the serialized elf file
				.upload_program(FACTORS_ID, serialized_program),
			&signer,
		)
		.await
		.unwrap()
		.wait_for_finalized()
		.await
		.unwrap();
}
