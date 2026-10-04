//! Local chain process. `init` writes an empty chain. `run` mines, and
//! will pull a longer chain from `--peer` or serve this one on `--listen`.
//! This is not a public network.

use std::env;
use std::fs;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cypher_dilithium::Keypair;
use cypher_node::runtime::{self, Node};
use cypher_node::LEAVES;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let mut dir = None;
    let mut bits = 0u32;
    let mut listen = None;
    let mut peer = None;
    let mut blocks = 0u64;
    let mut serve = 0u32;
    while let Some(arg) = args.next() {
        let value = args.next();
        match arg.as_str() {
            "--dir" => dir = value,
            "--bits" => bits = value.and_then(|text| text.parse().ok()).unwrap_or(0),
            "--listen" => listen = value,
            "--peer" => peer = value,
            "--blocks" => blocks = value.and_then(|text| text.parse().ok()).unwrap_or(0),
            "--serve" => serve = value.and_then(|text| text.parse().ok()).unwrap_or(0),
            other => {
                eprintln!("unknown argument {other}");
                return usage();
            }
        }
    }
    let Some(dir) = dir.map(PathBuf::from) else {
        return usage();
    };
    let result = match command.as_str() {
        "init" => init(&dir, bits),
        "tip" => tip(&dir),
        "run" => run(&dir, listen.as_deref(), peer.as_deref(), blocks, serve),
        _ => return usage(),
    };
    if let Err(err) = result {
        eprintln!("{err}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  cypher-node init --dir DIR [--bits N]\n  cypher-node tip --dir DIR\n  cypher-node run --dir DIR [--peer ADDR] [--listen ADDR] [--blocks N] [--serve N]"
    );
    ExitCode::from(2)
}

fn init(dir: &Path, bits: u32) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    let infra = Keypair::dilithium5();
    let miner = Keypair::dilithium2();
    fs::write(dir.join("infra.pk"), &infra.public).map_err(|err| err.to_string())?;
    fs::write(dir.join("infra.sk"), &infra.secret).map_err(|err| err.to_string())?;
    fs::write(dir.join("miner.pk"), &miner.public).map_err(|err| err.to_string())?;
    fs::write(dir.join("miner.sk"), &miner.secret).map_err(|err| err.to_string())?;
    fs::write(dir.join("field.sk"), 7u64.to_le_bytes()).map_err(|err| err.to_string())?;
    let node =
        Node::open(&infra, &miner, 7, [[0; 4]; LEAVES], bits).map_err(|err| err.to_string())?;
    fs::write(dir.join("chain.bin"), node.to_bytes()).map_err(|err| err.to_string())?;
    println!("initialized {}", dir.display());
    Ok(())
}

fn load(dir: &Path) -> Result<Node, String> {
    let infra = Keypair {
        public: fs::read(dir.join("infra.pk")).map_err(|err| err.to_string())?,
        secret: fs::read(dir.join("infra.sk")).map_err(|err| err.to_string())?,
    };
    let miner = Keypair {
        public: fs::read(dir.join("miner.pk")).map_err(|err| err.to_string())?,
        secret: fs::read(dir.join("miner.sk")).map_err(|err| err.to_string())?,
    };
    let field_bytes = fs::read(dir.join("field.sk")).map_err(|err| err.to_string())?;
    let field_sk = u64::from_le_bytes(field_bytes.as_slice().try_into().map_err(|_| "field.sk")?);
    let bytes = fs::read(dir.join("chain.bin")).map_err(|err| err.to_string())?;
    let node = Node::from_bytes(&bytes, &miner, field_sk).map_err(|err| err.to_string())?;
    if node.chain.infra_pk() != infra.public.as_slice() {
        return Err("infra key does not match the chain".into());
    }
    Ok(node)
}

fn tip(dir: &Path) -> Result<(), String> {
    let node = load(dir)?;
    println!(
        "height {} trees {} tip {}",
        node.chain.height(),
        node.chain.note_trees(),
        hex(&node.chain.tip())
    );
    Ok(())
}

fn run(
    dir: &Path,
    listen: Option<&str>,
    peer: Option<&str>,
    blocks: u64,
    serve: u32,
) -> Result<(), String> {
    let mut node = load(dir)?;
    if let Some(peer) = peer {
        let mut stream = TcpStream::connect(peer).map_err(|err| err.to_string())?;
        runtime::exchange(&mut stream, &mut node.chain)?;
        println!(
            "height {} tip {}",
            node.chain.height(),
            hex(&node.chain.tip())
        );
    }
    let start = node.chain.height();
    for n in 0..blocks {
        let timestamp = 1_700_000_000 + (start + n) * 120;
        node.mine_next(timestamp).map_err(|err| err.to_string())?;
        println!("mined {} {}", node.chain.height(), hex(&node.chain.tip()));
    }
    if blocks > 0 || peer.is_some() {
        fs::write(dir.join("chain.bin"), node.to_bytes()).map_err(|err| err.to_string())?;
    }
    if let Some(addr) = listen {
        let listener = runtime::listen(addr).map_err(|err| err.to_string())?;
        println!(
            "listening {}",
            listener.local_addr().map_err(|err| err.to_string())?
        );
        for _ in 0..serve {
            let (mut stream, _) = listener.accept().map_err(|err| err.to_string())?;
            runtime::serve_one(&mut stream, &node.chain)?;
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
