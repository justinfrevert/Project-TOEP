use crate::substrate_node::{
	prover_mgmt::Event, runtime_types::bounded_collections::bounded_vec::BoundedVec,
};
use clap::Parser;
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
async fn prove_program_execution(onchain_program: Vec<u8>) -> SessionReceipt {
	// let program = Program::load_elf(PROVE_ELF, MEM_SIZE as u32).unwrap();
	// let image = MemoryImage::new(&program, PAGE_SIZE as u32).unwrap();
	let env = ExecutorEnv::builder()
		// TODO: conditionally add inputs if there are any args
		.build();

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

async fn upload_proof(api: ApiType, image_id: ImageId, session_receipt: SessionReceipt) {
	api.tx()
		.sign_and_submit_then_watch_default(
			&substrate_node::tx()
				.prover_mgmt()
				// Send the serialized elf file
				.store_and_verify_proof(image_id, session_receipt),
			&signer,
		)
		.await
		.unwrap()
		.wait_for_finalized()
		.await
		.unwrap();
	println!("Done");
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// The hex-encoded, bincode-serialized image id of the onchain program to prove
	#[arg(short, long)]
	image_id: String,
}

#[tokio::main]
async fn main() {
	let args = Args::parse();

	let hex_decoded = hex::decode(&args.image_id).unwrap();

	let image_id = bincode::deserialize(&hex_decoded).unwrap();

	// image id
	let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
	// This is the well-known //Alice key. Don't use in a real application
	let restored_key = SubxtPair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();
	// listen_for_event_then_prove().await;
	let program = get_program(&api, image_id).await;
	let session_receipt =
		prove_program_execution(&api, program.unwrap().expect("Onchain program should exist"))
			.await;

	upload_proof(&api).await;
}
