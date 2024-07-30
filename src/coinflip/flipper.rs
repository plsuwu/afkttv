use rand::Rng;

pub fn coinflip() {
    let mut rng = rand::thread_rng();

    println!("{:?}", rng.gen::<bool>());
}
