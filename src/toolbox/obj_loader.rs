use crate::{TriIndexes, Vertex, RESOURCES};

pub fn load_obj(name: &str) -> (Vec<Vertex>, Vec<TriIndexes>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<TriIndexes> = Vec::new();

    let file =RESOURCES.get_file(name).expect(format!("Failed to read OBJ file \"{}\"", name).as_str());
    let obj_data = file.contents_utf8().expect("Unable to read shader file").to_string();
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
                let v1: usize = parts[1].split('/').next().unwrap().parse().unwrap();
                let v2: usize = parts[2].split('/').next().unwrap().parse().unwrap();
                let v3: usize = parts[3].split('/').next().unwrap().parse().unwrap();
                indices.push([v1 as u32, v2 as u32, v3 as u32])
            }
            _ => {}
        }
    }

    (vertices, indices)
}