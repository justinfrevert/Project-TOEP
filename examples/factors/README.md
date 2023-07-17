# Factors offchain example

Example of a provable program, with a simple subxt utility to upload the program

## Guest
The code inside of `./methods/guest` is the code which runs inside of the Risc0 ZKVM, and whose execution will be proven. Your programs's logic should be written inside of here.

## Uploading
To upload the program to the chain, run `cargo run`.