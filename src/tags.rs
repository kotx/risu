use rand::{prelude::SmallRng, Rng, SeedableRng};

// TODO: static/no alloc
pub fn random() -> String {
    // TODO: once_cell
    let tags_file = std::fs::read_to_string("./data/tags.txt").unwrap();
    let tags: Vec<&str> = tags_file.lines().collect();

    let i = SmallRng::from_entropy().gen_range(0..tags.len());
    tags[i].to_string()
}
