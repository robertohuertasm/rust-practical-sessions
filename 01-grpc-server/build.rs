fn main() -> Result<(), Box<dyn std::error::Error>> {
    // this will build the proto and put the code inside target/debug/rpts01/out/..
    tonic_build::compile_protos("proto/rpts01.proto")?;

    // this would allow us to configure how to build the proto files.
    // for instance, by only generating the server and compiling the proto to a specific folder
    // tonic_build::configure()
    //     .build_client(false)
    //     .out_dir("./generated")
    //     .compile(&["proto/rpts01.proto"], &["proto"])?;

    println!("## Proto files have been compiled");
    Ok(())
}
