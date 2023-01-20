use sha2::{digest::FixedOutput, Digest, Sha256};

pub fn check_solution(solution: u64, difficulty: u64, challenge: u64) -> bool {
    let hash = hash(solution, challenge);
    hash < difficulty
}

fn hash(solution: u64, challenge: u64) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(solution.to_be_bytes());
    hasher.update(challenge.to_be_bytes());
    u64::from_be_bytes(hasher.finalize_fixed().as_slice()[0..8].try_into().unwrap())
}
