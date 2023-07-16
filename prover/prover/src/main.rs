use crate::substrate_node::{
	prover_mgmt::Event, runtime_types::bounded_collections::bounded_vec::BoundedVec,
};
use codec::Decode;
use futures_util::StreamExt;
use methods::{PROVE_ELF, PROVE_ID};
use risc0_zkvm::{
	prove::Prover, serde::to_vec, Executor, ExecutorEnv, MemoryImage, Program, SessionReceipt,
	MEM_SIZE, PAGE_SIZE,
};
use std::{fs, time::Duration};
use subxt::{
	config::WithExtrinsicParams,
	events::StaticEvent,
	ext::{
		scale_value::Composite,
		sp_core::{
			sr25519::{Pair as SubxtPair, Public, Signature},
			Pair as SubxtPairT,
		},
		sp_runtime::{traits::Verify, AccountId32},
	},
	tx::{BaseExtrinsicParams, PairSigner, PlainTip},
	OnlineClient, PolkadotConfig, SubstrateConfig,
};
use tokio::task;

// // Runtime types, etc
#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod substrate_node {}

use substrate_node::runtime_types::frame_system::AccountInfo;

type ApiType = OnlineClient<
	WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
>;

type ImageId = [u32; 8];

async fn get_program(
	api: &ApiType,
	image_id: ImageId,
	// ) -> Result<Option<BoundedVec<u8>>, subxt::Error> {
) -> Result<Option<Vec<u8>>, subxt::Error> {
	let query = substrate_node::storage().prover_mgmt().programs(&image_id);

	let query_result = api.storage().fetch(&query, None).await;
	query_result
}

// Take a request for a proof, find the stored program, prove it, and return the proof to the
// extrinsic
// Update this to take the bytes retrieved from onchain stored program
async fn fulfill_proof_request(api: &ApiType, image_id: ImageId) -> SessionReceipt {
	// let program = Program::load_elf(PROVE_ELF, MEM_SIZE as u32).unwrap();
	// let image = MemoryImage::new(&program, PAGE_SIZE as u32).unwrap();
	let env = ExecutorEnv::builder()
		// TODO: conditionally add inputs if there are any args
		.build();
	println!("Checking for program from onchain");

	let onchain_program = get_program(api, image_id).await.unwrap().unwrap();

	println!("Got program from onchain: {:?}", image_id);

	let mut executor = Executor::from_elf(
		env.clone(),
		//  &image
		bincode::deserialize(&onchain_program).unwrap(),
	)
	.unwrap();

	println!("Starting session");
	let session = executor.run().unwrap();
	let receipt = session.prove().unwrap();
	println!("Done");
	receipt
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
			let events_details = ext.events().await.unwrap();
			let bytes_hex = format!("0x{}", hex::encode(ext.bytes()));
                let events = events_details.find::<substrate_node::prover_mgmt::events::ProgramUploaded>();
				for event in events {
					println!("ProgramUploaded Event {:?}", event);
				}
                let events = events_details.find::<substrate_node::prover_mgmt::events::ProofRequested>();
				for event in events {
					println!("ProofRequested Event {:?}", event);
				}
		}
	}
}

#[tokio::main]
async fn main() {
	listen_for_event_then_prove().await;
}
