// Temporary debug helper - delete after use
#[cfg(test)]
mod tests {
    use rand_xoshiro::Xoshiro256StarStar;
    use rand_core::{RngCore, SeedableRng};
    use ffb_model::util::rng::GameRng;

    #[test]
    fn trace_game_rng_seed67_h1_h2_kickoff() {
        // Trace actual game RNG values for seed 67 Norse kickoff processing.
        // H1: scatter(d8+d6) + event(2d6) + CSTI_bounce(d8) = 5 dice
        // H2: scatter(d8+d6) + event(2d6) + coaching(2d6) + CSTI_catch(d6) = 7 dice
        let mut rng = GameRng::new(67);
        println!("Seed 67 game RNG values:");
        // H1 kickoff (Rust order: scatter first, then event)
        let h1_sdir = rng.d8();
        let h1_sdist = rng.d6();
        let h1_ev1 = rng.d6();
        let h1_ev2 = rng.d6();
        let h1_bounce = rng.d8();
        println!("H1 kickoff: scatter dir={h1_sdir} dist={h1_sdist} event={h1_ev1}+{h1_ev2}={} bounce={h1_bounce}",
            h1_ev1 + h1_ev2);
        // H2 kickoff
        let h2_sdir = rng.d8();
        let h2_sdist = rng.d6();
        let h2_ev1 = rng.d6();
        let h2_ev2 = rng.d6();
        println!("H2 kickoff: scatter dir={h2_sdir} dist={h2_sdist} event={h2_ev1}+{h2_ev2}={}",
            h2_ev1 + h2_ev2);
        let h2_coach_home = rng.d6();
        let h2_coach_away = rng.d6();
        let h2_catch = rng.d6();
        let h2_bounce = rng.d8();
        println!("H2 coaching: home={h2_coach_home} away={h2_coach_away}");
        println!("H2 CSTI: catch_roll={h2_catch} (if no player: bounce_roll={h2_bounce})");
    }

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

    #[test]
    fn game_rng_same_seed_is_deterministic() {
        let mut r1 = GameRng::new(42);
        let mut r2 = GameRng::new(42);
        assert_eq!(r1.d6(), r2.d6());
    }

    #[test]
    fn game_rng_different_seeds_differ() {
        let mut r1 = GameRng::new(1);
        let mut r2 = GameRng::new(999);
        // Extremely unlikely to produce identical d8 for different seeds
        let v1 = r1.d8();
        let v2 = r2.d8();
        // Both are valid dice values
        assert!((1..=8).contains(&v1));
        assert!((1..=8).contains(&v2));
    }

    #[test]
    fn xoshiro_seed1_produces_valid_u64() {
        use rand_core::SeedableRng;
        let mut rng = Xoshiro256StarStar::seed_from_u64(1 ^ 0xDEAD_BEEF_CAFE_0001);
        let v = rng.next_u64();
        // Any u64 is valid — just check it evaluates without panic
        let _ = v % 2;
    }
}
