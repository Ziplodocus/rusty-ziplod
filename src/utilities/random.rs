use rand::Rng;

pub fn random_range(min: i32, max: i32) -> i32 {
    let mut rnd = rand::thread_rng();
    rnd.gen_range(min..max)
}
