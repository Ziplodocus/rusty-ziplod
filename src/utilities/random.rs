use rand::{Rng, distributions::uniform::{SampleUniform}};

pub fn random_range<T>(min: T, max: T) -> T where
    T: SampleUniform + PartialOrd {
    let mut rnd = rand::thread_rng();
    rnd.gen_range(min..max)
}
