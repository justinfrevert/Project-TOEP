# Substrate Prover Service

The Substrate prover is an offchain, long-running service whose primary responsibility is to prove arbitrary programs which are stored onchain. This one expects to be paired with a Substrate chain which has a specific pallet, `prover_mgmt`. The substrate node, including runtime and pallets lives in `./node`.

The prover listens for the ProofRequested event emitted from the `prover_mgmt` pallet. If received, it takes any arguments passed to it and starts to prove the requested image id. Once done, it should call back to the pallet with the image id and the proof. The prover lives in `./prover`.


## Usage instructions:
### Start the chain
Start the chain by building the code `cargo build --release`, and starting the node: `./target/release/node-template --dev`.

### Upload/Write your own program
See `./examples` for an example of a provable program. To test uploading the program to the chain, run:
```cargo run```
It will return the `image id`, which is handy for proving later

### Prover
Prover nodes can fulfill onchain requests for proofs. The included proving cli application in `./prover` allows someone to pass an `image_id` of an onchain program, retrieve it, prove it, and upload the resulting proof to fulfill the request. To test, pass a hex-encoded, bincode-serialized image id(just copy the output from the `./examples` local execution)
```
cargo run -- --image-id {your image id}