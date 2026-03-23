use crate::{TriIndexes, Vertex, RESOURCES};

pub fn load_obj(name: &str) -> (Vec<Vertex>, Vec<TriIndexes>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<TriIndexes> = Vec::new();

    let file = RESOURCES
        .get_file(name)
        .expect(format!("Failed to read OBJ file \"{}\"", name).as_str());
    let obj_data = file
        .contents_utf8()
        .expect("Unable to read shader file")
        .to_string();
    for line in obj_data.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "v" => {
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                vertices.push([x, y, z]);
            }
            "f" => {
                let first = parse_obj_index(parts[1]);
                let mut previous = parse_obj_index(parts[2]);

                for part in &parts[3..] {
                    let current = parse_obj_index(part);
                    indices.push([first, previous, current]);
                    previous = current;
                }
            }
            _ => {}
        }
    }

    (vertices, indices)
}

fn parse_obj_index(part: &str) -> u32 {
    part.split('/').next().unwrap().parse::<u32>().unwrap() - 1
}

#[cfg(test)]
mod tests {
    use super::load_obj;

    fn assert_indices_in_bounds(name: &str) {
        let (vertices, indices) = load_obj(name);
        let max_index = vertices.len() as u32 - 1;

        assert!(!vertices.is_empty(), "{name} should load vertices");
        assert!(!indices.is_empty(), "{name} should load indices");

        for triangle in indices {
            for index in triangle {
                assert!(
                    index <= max_index,
                    "{name} contains out-of-bounds index {index}, max is {max_index}"
                );
            }
        }
    }

    #[test]
    fn sphere_obj_indices_are_zero_based_and_in_bounds() {
        assert_indices_in_bounds("sphere.obj");
    }

    #[test]
    fn arrow_obj_indices_are_zero_based_and_in_bounds() {
        assert_indices_in_bounds("arrow.obj");
    }
}
