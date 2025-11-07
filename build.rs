fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf files
    tonic_build::compile_protos("proto/market_data.proto")?;
    Ok(())
}
