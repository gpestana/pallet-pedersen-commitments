use curve25519_dalek::constants;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use rand_core::OsRng;
use sha2::Sha512;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Commitment {
    g: RistrettoPoint,
    h: RistrettoPoint,
    payload: RistrettoPoint,
    secret: Scalar,
    message: Scalar,
}

impl Commitment {
    pub fn new(message: String, secret: String) -> Self {
        let mut rng = OsRng;

        let g = constants::RISTRETTO_BASEPOINT_POINT;
        let h = RistrettoPoint::random(&mut rng);
        let (m, r) = (
            Scalar::hash_from_bytes::<Sha512>(message.as_bytes()),
            Scalar::hash_from_bytes::<Sha512>(secret.as_bytes()),
        );
        let payload = m * g + r * h;

        Commitment {
            g,
            h,
            payload,
            secret: r,
            message: m,
        }
    }
}

#[cfg(test)]
mod tests {
    use frame_support::{assert_err, assert_ok};

    use super::*;
    use crate::mock::{
        ExtBuilder, MaxLenCommitMessage, PedersenCommitments, Runtime, RuntimeOrigin, System,
    };

    #[test]
    fn check_pallet_settings() {
        ExtBuilder::default().build_and_execute(|| {
            assert_eq!(System::block_number(), 0);

            let configs_max_len_msg = <MaxLenCommitMessage>::get();
            assert_eq!(configs_max_len_msg, 256, "max lex: {}", configs_max_len_msg);

            assert!(PedersenCommitments::commitments(0).is_none());
        })
    }

    #[test]
    fn commit_reveal_ok() {
        ExtBuilder::default().build_and_execute(|| {
            let caller = 0;

            let (message, secret) = ("commited_message".to_string(), "secret".to_string());
            let commitment = Commitment::new(message.clone(), secret.clone());

            let commit_call = PedersenCommitments::commit(
                RuntimeOrigin::signed(caller),
                *commitment.payload.compress().as_bytes(),
                *commitment.g.compress().as_bytes(),
                *commitment.h.compress().as_bytes(),
            );

            assert_ok!(commit_call);

            assert!(PedersenCommitments::commitments(caller).is_some());

            let reveal_call = PedersenCommitments::reveal_and_verify(
                RuntimeOrigin::signed(caller),
                message.into_bytes(),
                secret.into_bytes(),
            );

            assert_ok!(reveal_call);
            assert_eq!(
                PedersenCommitments::commitments(caller)
                    .unwrap()
                    .revealed_at
                    .unwrap(),
                System::block_number()
            );
        })
    }

    #[test]
    fn commit_reveal_nok() {
        ExtBuilder::default().build_and_execute(|| {
            let caller = 0;

            let (message, secret) = ("commited_message".to_string(), "secret".to_string());
            let commitment = Commitment::new(message.clone(), secret.clone());

            let commit_call = PedersenCommitments::commit(
                RuntimeOrigin::signed(caller),
                *commitment.payload.compress().as_bytes(),
                *commitment.g.compress().as_bytes(),
                *commitment.h.compress().as_bytes(),
            );

            assert_ok!(commit_call);

            // wrong message
            let reveal_call = PedersenCommitments::reveal_and_verify(
                RuntimeOrigin::signed(caller),
                "different message".as_bytes().to_vec(),
                secret.into_bytes(),
            );

            assert_err!(reveal_call, crate::Error::<Runtime>::UnableToReveal);

            // wrong secret
            let reveal_call = PedersenCommitments::reveal_and_verify(
                RuntimeOrigin::signed(caller),
                message.into_bytes(),
                "different secret".as_bytes().to_vec(),
            );

            assert_err!(reveal_call, crate::Error::<Runtime>::UnableToReveal);
        })
    }
}
