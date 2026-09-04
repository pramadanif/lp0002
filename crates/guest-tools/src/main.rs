//! Wraps a guest ELF into a risc0 `ProgramBinary` and reports its ImageID.
//!
//! On LEZ a program's `ProgramId` **is** the risc0 ImageID of its deployed binary — which is how
//! `docs/VERSIONS.md` pinned the testnet's LEZ version. So this tool produces both the artefact that
//! gets deployed and the identifier that must appear on chain.
//!
//! risc0 3.x does not take a bare ELF: a program is a `ProgramBinary` pairing the user ELF with the
//! v1compat kernel. Handing `compute_image_id` a raw ELF fails with "Malformed ProgramBinary".
//!
//! ```text
//! pmsig-image-id <user-elf> [--out <program-binary.bin>]
//! ```
//!
//! **Reproducibility.** A locally built ELF depends on the host toolchain. The artefact that gets
//! deployed and quoted in the submission must come from `cargo risczero build`, which builds inside
//! a pinned container (LEZ v0.2.4 uses `RISC0_DOCKER_CONTAINER_TAG=r0.1.91.1`). This tool reports
//! the ImageID of whatever it is given; it does not make a non-reproducible build reproducible.

use std::path::PathBuf;

use anyhow::{Context as _, Result};
use risc0_binfmt::ProgramBinary;
use risc0_zkvm::compute_image_id;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let elf_path: PathBuf = args
        .next()
        .context("usage: pmsig-image-id <user-elf> [--out <program-binary.bin>]")?
        .into();

    let mut out_path: Option<PathBuf> = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--out" => out_path = args.next().map(PathBuf::from),
            other => anyhow::bail!("unknown argument: {other}"),
        }
    }

    let user_elf = std::fs::read(&elf_path)
        .with_context(|| format!("reading guest ELF {}", elf_path.display()))?;

    let binary = ProgramBinary::new(&user_elf, risc0_zkos_v1compat::V1COMPAT_ELF);
    let encoded = binary.encode();

    let image_id = compute_image_id(&encoded).context("computing image id")?;
    let words: [u32; 8] = image_id
        .as_words()
        .try_into()
        .context("image id must be 8 words")?;

    println!("elf:          {}", elf_path.display());
    println!("elf bytes:    {}", user_elf.len());
    println!("binary bytes: {}", encoded.len());
    println!("image id:     {image_id}");
    println!("program id:   {words:?}");

    if let Some(out) = out_path {
        std::fs::write(&out, &encoded)
            .with_context(|| format!("writing program binary {}", out.display()))?;
        println!("wrote:        {}", out.display());
    }

    Ok(())
}
