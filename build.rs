use ff::{Field, PrimeField};
use paired::bls12_381::Fr as Scalar;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn scalar_from_u64(i: u64) -> Scalar {
    use ff::{PrimeField, PrimeFieldRepr};
    Scalar::from_repr(paired::bls12_381::FrRepr::from(i)).unwrap()
}

fn generate_mds(t: usize) -> Vec<Vec<Scalar>> {
    let mut matrix: Vec<Vec<Scalar>> = Vec::with_capacity(t);
    let mut xs: Vec<Scalar> = Vec::with_capacity(t);
    let mut ys: Vec<Scalar> = Vec::with_capacity(t);

    // Generate x and y values deterministically for the cauchy matrix
    // where x[i] != y[i] to allow the values to be inverted
    // and there are no duplicates in the x vector or y vector, so that the determinant is always non-zero
    // [a b]
    // [c d]
    // det(M) = (ad - bc) ; if a == b and c == d => det(M) =0
    // For an MDS matrix, every possible mxm submatrix, must have det(M) != 0
    for i in 0..t {
        let x = scalar_from_u64((i) as u64);
        let y = scalar_from_u64((i + t) as u64);
        xs.push(x);
        ys.push(y);
    }

    for i in 0..t {
        let mut row: Vec<Scalar> = Vec::with_capacity(t);
        for j in 0..t {
            // Generate the entry at (i,j)
            let mut entry = xs[i];
            entry.add_assign(&ys[i]);
            entry = entry.inverse().unwrap();
            row.insert(j, entry);
        }
        matrix.push(row);
    }

    matrix
}

fn main() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").expect("No out dir");
    let dest_path = Path::new(&out_dir).join("src").join("constants.rs");
    let mut f = File::create(&dest_path).expect("Could not create file");

    let merkle_arity = env::var("POSEIDON_MERKLE_ARITY")
        .map(|s| s.parse().expect("Failed to parse POSEIDON_MERKLE_ARITY"))
        .unwrap_or(4);

    let merkle_width = env::var("POSEIDON_MERKLE_WIDTH")
        .map(|s| s.parse().expect("Failed to parse POSEIDON_MERKLE_WIDTH"))
        .unwrap_or(64);

    let full_rounds = env::var("POSEIDON_FULL_ROUNDS")
        .map(|s| s.parse().expect("Failed to parse POSEIDON_FULL_ROUNDS"))
        .unwrap_or(8);

    let partial_rounds = env::var("POSEIDON_PARTIAL_ROUNDS")
        .map(|s| s.parse().expect("Failed to parse POSEIDON_PARTIAL_ROUNDS"))
        .unwrap_or(59);

    let width = merkle_arity + 1;
    let merkle_height = merkle_width as f64;
    let merkle_height = merkle_height.log(merkle_arity as f64) as usize;

    write!(
        &mut f,
        r#"// Poseidon constants
pub(crate) const WIDTH: usize = {};
pub(crate) const FULL_ROUNDS: usize = {};
pub(crate) const PARTIAL_ROUNDS: usize = {};

// Merkle constants
/// Arity of the merkle tree
pub const MERKLE_ARITY: usize = {};
/// Width of the merkle tree
pub const MERKLE_WIDTH: usize = {};
pub(crate) const MERKLE_HEIGHT: usize = {};

"#,
        width, full_rounds, partial_rounds, merkle_arity, merkle_width, merkle_height
    )
    .expect("Could not write file");

    let dest_path = Path::new(&out_dir).join("assets").join("mds.bin");
    let mut f = File::create(&dest_path).expect("Could not create file");
    let mds = generate_mds(width)
        .into_iter()
        .flatten()
        .fold(vec![], |mut v, scalars| {
            let raw = scalars.into_repr();
            let raw_u64: &[u64] = raw.as_ref();
            let raw_u8: &[u8] = unsafe { std::mem::transmute(raw_u64) };
            v.extend_from_slice(raw_u8);
            v
        });

    f.write_all(mds.as_slice())
        .expect("Failed to write MDS matrix bin file.");
}
