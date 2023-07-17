use clap::Parser;
use futures_util::StreamExt;
use risc0_zkvm::{serde::to_vec, Executor, ExecutorEnv, SegmentReceipt, SessionReceipt};
use subxt::{
	config::WithExtrinsicParams,
	ext::sp_core::{sr25519::Pair as SubxtPair, Pair as SubxtPairT},
	tx::{BaseExtrinsicParams, PairSigner, PlainTip},
	OnlineClient, PolkadotConfig, SubstrateConfig,
};

// // Runtime types, etc
#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod substrate_node {}

type ApiType = OnlineClient<
	WithExtrinsicParams<SubstrateConfig, BaseExtrinsicParams<SubstrateConfig, PlainTip>>,
>;

type ImageId = [u32; 8];

async fn get_program(api: &ApiType, image_id: ImageId) -> Result<Option<Vec<u8>>, subxt::Error> {
	let query = substrate_node::storage().prover_mgmt().programs(&image_id);

	let query_result = api.storage().fetch(&query, None).await;
	query_result
}

async fn get_program_args(
	api: &ApiType,
	image_id: ImageId,
) -> Result<Option<Vec<Vec<u32>>>, subxt::Error> {
	let query = substrate_node::storage().prover_mgmt().proof_requests(&image_id);

	let query_result = api.storage().fetch(&query, None).await;
	query_result
}

// Prove the program which was given as serialized bytes
fn prove_program_execution(onchain_program: Vec<u8>, args: Vec<Vec<u32>>) -> SessionReceipt {
	let mut envbuilder = ExecutorEnv::builder();
	args.iter().for_each(|a| {
		envbuilder.add_input(a);
	});

	let env = envbuilder.build();

	let mut executor =
		Executor::from_elf(env.clone(), bincode::deserialize(&onchain_program).unwrap()).unwrap();

	println!("Starting session");
	let session = executor.run().unwrap();
	println!("Now proving execution");
	let receipt = session.prove().unwrap();
	println!("Done proving");
	receipt
}

async fn upload_proof(api: ApiType, image_id: ImageId, session_receipt: SessionReceipt) {
	let substrate_session_receipt = session_receipt
		.segments
		.into_iter()
		.map(|SegmentReceipt { seal, index }| (seal, index))
		.collect();

	// This is the well-known //Alice key. TODO: Use the key passed through cli to represent the
	// prover
	let restored_key = SubxtPair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();

	let signer = PairSigner::new(restored_key);

	api.tx()
		.sign_and_submit_then_watch_default(
			&substrate_node::tx()
				.prover_mgmt()
				// Upload the proof
				.store_and_verify_proof(
					image_id,
					substrate_session_receipt,
					session_receipt.journal,
				),
			&signer,
		)
		.await
		.unwrap()
		.wait_for_finalized()
		.await
		.unwrap();
	println!("Proof uploaded");
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
	let args = get_program_args(&api, image_id).await;

	println!("Passing args to program :{:?}", args);

	let session_receipt = prove_program_execution(
		program.unwrap().expect("Onchain program should exist"),
		args.unwrap()
			.expect("Args were not provided, or request was not made for program proof"),
	);

	upload_proof(api, image_id, session_receipt).await;
}
