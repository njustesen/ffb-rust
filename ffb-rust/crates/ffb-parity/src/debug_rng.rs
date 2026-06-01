// Temporary debug helper - delete after use
#[cfg(test)]
mod tests {
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};

    #[test]
    fn print_agent_decisions_seed1() {
        let seed: u64 = 1;
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed ^ 0xDEAD_BEEF_CAFE_0001);
        let v1 = rng.next_u64();
        let v2 = rng.next_u64();
        let v3 = rng.next_u64();
        let v4 = rng.next_u64();
        println!("call1={:#018x} heads={}", v1, v1 % 2 == 0);
        println!("call2={:#018x} receive={}", v2, v2 % 2 == 0);
        println!("call3={:#018x} x_raw={}", v3, v3 % 13);
        println!("call4={:#018x} y_raw={}", v4, v4 % 13);
    }
}
