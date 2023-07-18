use clap::Parser;
use methods::{FACTORS_ELF, FACTORS_ID};
use risc0_zkvm::serde::to_vec;
use subxt::{
	ext::sp_core::{sr25519::Pair as SubxtPair, Pair as SubxtPairT},
	tx::PairSigner,
	OnlineClient, PolkadotConfig,
};

// Runtime types, etc
#[subxt::subxt(runtime_metadata_path = "./metadata.scale")]
pub mod substrate_node {}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Whether to include an additional onchain request for a proof of the uploaded program
	#[arg(short, long, default_value_t = true)]
	with_request_proof: bool,
}

#[tokio::main]
async fn main() {
	let args = Args::parse();

	let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
	// This is the well-known //Alice key. Don't use in a real application
	let restored_key = SubxtPair::from_string(
		"0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
		None,
	)
	.unwrap();
	let signer = PairSigner::new(restored_key);

	let serialized_program = bincode::serialize(FACTORS_ELF).unwrap();

	println!(
		"Uploading program. Image id: {:?}",
		hex::encode(bincode::serialize(&FACTORS_ID).unwrap())
	);

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
	println!("Upload complete");

	// Example of requesting a proof of the program's execution, passing some arguments. This is
	// optional.
	if args.with_request_proof {
		// Assuming our program requires two arguments...
		let arg_1: u64 = 17_u64;
		let arg_2: u64 = 23_u64;

		// Any args we want to pass to the program must be serialized using Risc0's serde serialized
		// methods
		let args = vec![to_vec(&17_u64).unwrap(), to_vec(&23_u64).unwrap()];

		// The reward for the correct proof submission
		let reward: u128 = 42_000_000_000_000;

		println!(
			"Requesting proof of with program args: {:?}, {:?} and reward amount: {:?}",
			arg_1, arg_2, reward
		);

		api.tx()
			.sign_and_submit_then_watch_default(
				&substrate_node::tx().prover_mgmt().request_proof(FACTORS_ID, args, reward),
				&signer,
			)
			.await
			.unwrap()
			.wait_for_finalized()
			.await
			.unwrap();
		println!("Proof request submitted successfully");
	}
}
