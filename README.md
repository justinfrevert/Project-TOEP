# Substrate Prover Service

The Substrate prover is an offchain, long-running service whose primary responsibility is to prove arbitrary programs which are stored onchain. This one expects to be paired with a Substrate chain which has a specific pallet, `prover_mgmt`. The substrate node, including runtime and pallets lives in `./node`.

The prover listens for the ProofRequested event emitted from the `prover_mgmt` pallet. If received, it takes any arguments passed to it and starts to prove the requested image id. Once done, it should call back to the pallet with the image id and the proof. The prover lives in `./prover`.

## Usage instructions:
### Start the chain
Start the chain by building the code `cargo build --release`, and starting the node: `./target/release/node-template --dev`.

### Upload/Write your own program
See `./examples`. In that directory, run `cargo run` to prepare and upload the hello world example program to the chain.

### Start prover node
In `./prover`, run `cargo run`. The prover will continuously listen to the events emitted onchain.

### Request Proof
Call the `prover_mgmt.request_proof` extrinsic with your program's image id(as of now, this is just hardcoded in the prover). The prover will respond to this event by receiving the elf, and proving its execution. 
