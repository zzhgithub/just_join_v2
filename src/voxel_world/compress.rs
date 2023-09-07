use bevy::utils::HashMap;
use bit_vec::BitVec;
use huffman_compress::{CodeBuilder, Tree};

use super::voxel::Voxel;

pub fn compress(data: Vec<Voxel>) -> (BitVec, Tree<Voxel>) {
    let mut weights: HashMap<Voxel, i32> = HashMap::new();
    for &voxel in &data {
        let count = weights.entry(voxel).or_insert(0);
        *count += 1;
    }

    let (book, tree) = CodeBuilder::from_iter(weights).finish();
    let mut buffer = BitVec::new();

    for voxel in &data {
        match book.encode(&mut buffer, voxel) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err);
            }
        }
    }
    (buffer, tree)
}

pub fn uncompress(buffer: &BitVec, tree: Tree<Voxel>) -> Vec<Voxel> {
    tree.decoder(buffer, buffer.len()).collect()
}

#[test]
fn test() {
    let data = vec![
        Voxel::EMPTY,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
    ];
    let (buffer, tree) = compress(data.clone());

    let new_data = uncompress(&buffer, tree);
    print!("{}", data.len());
    print!("{}", new_data.len());
    assert_eq!(data, new_data);
}

#[test]
fn test_serialization() {
    let data = vec![
        Voxel::EMPTY,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
        Voxel::FILLED,
    ];
    let (buffer, tree) = compress(data.clone());
    let tree_s = bincode::serialize(&tree).unwrap();
    let tree_ds: Tree<Voxel> = bincode::deserialize(&tree_s).unwrap();

    let a = uncompress(&buffer.clone(), tree);
    let b = uncompress(&buffer.clone(), tree_ds);

    assert_eq!(a, b);
}
