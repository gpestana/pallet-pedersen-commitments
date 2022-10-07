use std::fmt::Display;

use curve25519_dalek::constants;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use rand_core::OsRng;
use sha2::Sha512;

use hex;

#[derive(Debug)]
struct Commitment {
    g: RistrettoPoint,
    h: RistrettoPoint,
    payload: RistrettoPoint,
    secret: Scalar,
    message: Scalar,
}

impl Display for Commitment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let g_encoded = bincode::serialize(&self.g).unwrap();
        let h_encoded = bincode::serialize(&self.h).unwrap();
        let payload_encoded = bincode::serialize(&self.payload).unwrap();
        let secret_encoded = bincode::serialize(&self.secret).unwrap();

        let g = hex::encode(self.g.compress().as_bytes());

        write!(
            f,
            "{}",
            format!(
                "g: {:?}\nh: {:?}\npayload: {:?}\nsecret: {:?}\nmessage:{:?}",
                g, h_encoded, payload_encoded, secret_encoded, secret_encoded,
            )
        )
    }
}

fn main() {
    let (message, secret) = parse_inputs();

    println!(
        "Generating commitment payload message: {} and secret: {}..",
        message, secret
    );

    let mut rng = OsRng;

    let g = constants::RISTRETTO_BASEPOINT_POINT;
    let h = RistrettoPoint::random(&mut rng);

    let (scalar_m, scalar_r) = {
        (
            Scalar::hash_from_bytes::<Sha512>(message.as_bytes()),
            Scalar::hash_from_bytes::<Sha512>(secret.as_bytes()),
        )
    };

    let payload = scalar_m * g + scalar_r * h;

    let commitment = Commitment {
        g,
        h,
        payload,
        message: scalar_m,
        secret: scalar_r,
    };

    println!("{}", commitment);
}

fn parse_inputs() -> (String, String) {
    let message = match std::env::var("COMMIT_MESSAGE") {
        Ok(m) => m,
        Err(_) => panic!("Env variable `COMMIT_MESSSAGE` must be defined"),
    };

    let secret = match std::env::var("SECRET") {
        Ok(s) => s,
        Err(_) => panic!("Env variable `SECRET` must be defined"),
    };

    (message, secret)
}
