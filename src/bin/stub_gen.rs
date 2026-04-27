use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    // `stub_info` is a function defined by `define_stub_info_gatherer!` macro.
    let stub = qcel_howmany::stub_info()?;
    println!("{}", stub.python_root.display());
    stub.generate()?;
    std::fs::rename("python/qcel_howmany.pyi", "python/qcel_howmany/qcel_howmany.pyi")?;
    Ok(())
}