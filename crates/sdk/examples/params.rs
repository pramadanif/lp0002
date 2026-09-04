fn main() {
    use pmsig_core::{tree::MemberTree, Digest32};
    use pmsig_membership_core::verify::npk_of;
    let nsks: [Digest32; 3] = [[0x11;32],[0x22;32],[0x33;32]];
    let npks: Vec<Digest32> = nsks.iter().map(|n| npk_of(n).to_byte_array()).collect();
    let tree = MemberTree::new(&npks).expect("members");
    let verifier: [u32;8] = [3642956930,3671430927,465146623,3273241339,149686206,3437671857,611411258,4284544097];
    let multisig_id: Digest32 = [0xA1;32];
    let ch = pmsig_core::config_hash(&tree.root(), 2, 3, &multisig_id, &verifier);
    let pid: Digest32 = [0xB2;32];
    let seed = pmsig_core::proposal_seed(&ch, &pid);
    println!("MEMBER_ROOT={}", hex::encode(tree.root()));
    println!("CONFIG_HASH={}", hex::encode(ch));
    println!("MULTISIG_ID={}", hex::encode(multisig_id));
    println!("PROPOSAL_ID={}", hex::encode(pid));
    println!("PROPOSAL_SEED={}", hex::encode(seed));
    println!("VERIFIER={}", verifier.iter().map(|x|x.to_string()).collect::<Vec<_>>().join(","));
}
