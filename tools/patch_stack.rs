//! Minimal PE header patcher: increases a PE executable's stack reserve/commit
//! sizes so the Rust compiler can compile huge macro-generated crates (e.g. the
//! `windows` crate) without overflowing its main-thread stack.
//!
//! Usage: patch_stack.exe <input.exe> <output.exe>

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: patch_stack.exe <input.exe> <output.exe>");
        std::process::exit(2);
    }
    let input = &args[1];
    let output = &args[2];
    let mut data = fs::read(input).expect("read input");

    let e_lfanew = read_u32(&data, 0x3C) as usize;
    assert_eq!(&data[e_lfanew..e_lfanew + 4], b"PE\0\0", "bad PE signature");

    let coff = e_lfanew + 4;
    let opt = coff + 20; // COFF header is 20 bytes
    let magic = read_u16(&data, opt);

    // IMAGE_OPTIONAL_HEADER64: SizeOfStackReserve @ 72, SizeOfStackCommit @ 80
    // IMAGE_OPTIONAL_HEADER32: SizeOfStackReserve @ 64, SizeOfStackCommit @ 68
    let (reserve_off, commit_off, wide) = match magic {
        0x20B => (72usize, 80usize, true),
        0x10B => (64usize, 68usize, false),
        m => {
            eprintln!("unknown PE magic 0x{:x}", m);
            std::process::exit(3);
        }
    };

    let reserve = 0x4000_0000u64; // 1 GiB reserve
    let commit = 0x0100_0000u64; // 16 MiB initial commit

    if wide {
        write_u64(&mut data, opt + reserve_off, reserve);
        write_u64(&mut data, opt + commit_off, commit);
    } else {
        write_u32(&mut data, opt + reserve_off, reserve as u32);
        write_u32(&mut data, opt + commit_off, commit as u32);
    }

    fs::write(output, &data).expect("write output");
    println!(
        "patched {} -> {} (magic 0x{:x}, stack reserve 0x{:x})",
        input, output, magic, reserve
    );
}

fn read_u32(d: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([d[off], d[off + 1], d[off + 2], d[off + 3]])
}
fn read_u16(d: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([d[off], d[off + 1]])
}
fn write_u64(d: &mut [u8], off: usize, v: u64) {
    d[off..off + 8].copy_from_slice(&v.to_le_bytes());
}
fn write_u32(d: &mut [u8], off: usize, v: u32) {
    d[off..off + 4].copy_from_slice(&v.to_le_bytes());
}
