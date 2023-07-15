use methods::{PROVE_ELF, PROVE_ID};
use risc0_zkvm::{
    SessionReceipt, serde::to_vec, MemoryImage, Program, MEM_SIZE, PAGE_SIZE,
};
use risc0_zkvm::{
    prove::Prover,
};
use std::{fs, time::Duration};

fn create_proof() {
    // create the memoryImg, upload it and return the imageId
    let img_id = {
        let program = Program::load_elf(PROVE_ELF, MEM_SIZE as u32).unwrap();
        let image = MemoryImage::new(&program, PAGE_SIZE as u32).unwrap();
        // let image_id = hex::encode(image.compute_id());
        let image_id = hex::encode(image.get_root());
        let image = bincode::serialize(&image).expect("Failed to serialize memory img");

		println!("{:?}", image);
        // client.upload_img(&image_id, image)?;
        image_id
    };
}

// fn run_prover() -> SessionReceipt {
    fn run_prover() {
    let image_bytes_serialized: String = fs::read_to_string("output.elf").unwrap().parse().unwrap();
    let image: MemoryImage = bincode::deserialize(image_bytes_serialized.as_bytes()).unwrap();
    // Prover::from_image(image);

    // Prover::from_image(
    //     Rc::new(RefCell::new(image.clone())),
        
    // );

    Prover {
        // inner: ProverImpl::new(opts),
        inner: Default::default(),
        image,
        pc,
        cycles: 0,
        preflight_segments: None,
        exit_code: 0,
    }
}

fn main() {
	run_prover();
}