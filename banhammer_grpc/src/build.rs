fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .out_dir("./src")
        .compile(
            &["./proto/nauthz.proto", "./proto/validationcontrol.proto"],
            &["./proto"],
        )?;

    Ok(())
}
