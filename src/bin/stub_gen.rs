use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    // `stub_info` is a function defined by `define_stub_info_gatherer!` macro.
    let stub = quclif::stub_info()?;
    println!("{}", stub.python_root.display());
    stub.generate()?;
    std::fs::rename("python/quclif.pyi", "python/quclif/quclif.pyi")?;
    Ok(())
}