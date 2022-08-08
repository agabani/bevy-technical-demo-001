use tokio::io::AsyncWriteExt;

fn main() -> bevy_technical_demo::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(run())?;

    Ok(())
}

async fn run() -> bevy_technical_demo::Result<()> {
    let certificate = rcgen::generate_simple_self_signed(["localhost".to_string()])?;

    let mut certificate_pem = tokio::fs::File::create("tls.crt").await?;
    let mut private_key_pem = tokio::fs::File::create("tls.key").await?;

    certificate_pem
        .write_all(certificate.serialize_pem()?.as_bytes())
        .await?;

    private_key_pem
        .write_all(certificate.serialize_private_key_pem().as_bytes())
        .await?;

    Ok(())
}
