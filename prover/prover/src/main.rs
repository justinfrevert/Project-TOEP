use crate::substrate_node::runtime_types::bounded_collections::bounded_vec::BoundedVec;
// use futures_util::stream::stream::StreamExt;
use futures_util::StreamExt;
use methods::{PROVE_ELF, PROVE_ID};
use risc0_zkvm::{
	prove::Prover, serde::to_vec, Executor, ExecutorEnv, MemoryImage, Program, SessionReceipt,
	MEM_SIZE, PAGE_SIZE,
};
use std::{fs, time::Duration};
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

use substrate_node::runtime_types::frame_system::AccountInfo;

type ApiType = OnlineClient<
	WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
>;

type ImageId = [u8; 32];

async fn get_program(
	api: &ApiType,
	image_id: ImageId,
) -> Result<Option<BoundedVec<u8>>, subxt::Error> {
	let query = substrate_node::storage().prover_mgmt().programs(&image_id);

	let query_result = api.storage().fetch(&query, None).await;
	query_result
}

fn create_proof() -> MemoryImage {
	let program = Program::load_elf(PROVE_ELF, MEM_SIZE as u32).unwrap();
	let image = MemoryImage::new(&program, PAGE_SIZE as u32).unwrap();
	let env = ExecutorEnv::builder()
		// TODO: conditionally add inputs if there are any
		// // Send a & b to the guest
		// .add_input(&to_vec(&a).unwrap())
		// .add_input(&to_vec(&b).unwrap())
		.build();

	let image_serialized = bincode::serialize(&PROVE_ELF).expect("Failed to serialize memory img");

	let mut executor = Executor::from_elf(
		env,
		//  &image
		bincode::deserialize(&image_serialized).unwrap(),
	)
	.unwrap();

	let session = executor.run().unwrap();
	println!("Done");
	image
}

async fn listen_for_event_then_prove() {
	// TODO: get node url here
	let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();

	let mut blocks_sub = api.blocks().subscribe_finalized().await.unwrap();

	// For each block, print a bunch of information about it:
	while let Some(block) = blocks_sub.next().await {
		let block = block.unwrap();

		let block_number = block.header().number;
		let block_hash = block.hash();

		println!("Block #{block_number}:");
		println!("  Hash: {block_hash}");
		println!("  Extrinsics:");

		// Log each of the extrinsic with it's associated events:
		let body = block.body().await.unwrap();
		for ext in body.extrinsics() {
			let idx = ext.index();
			let events = ext.events().await.unwrap();
			let bytes_hex = format!("0x{}", hex::encode(ext.bytes()));

			for evt in events.iter() {
				let evt = evt.unwrap();

				let pallet_name = evt.pallet_name();
				let event_name = evt.variant_name();

				if pallet_name == "ProverMgmt" && event_name == "ProofRequested" {
					// Prove here
					println!("HERE");
				}

				let event_values = evt.field_values().unwrap();

				println!("        {pallet_name}_{event_name}");
				println!("          {}", event_values);
			}
		}
	}
}

#[tokio::main]
async fn main() {
	listen_for_event_then_prove().await;
}
