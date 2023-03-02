use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use luminance_front::tess::{Tess, TessError, Interleaved};
use luminance_front::context::GraphicsContext;
use luminance_front::Backend;
use luminance_front::tess::Mode;
use wavefront_obj::obj;

use crate::vertex::{Vertex, VertexIndex, VertexPosition3D, VertexNormal};

#[derive(Debug)]
pub struct Obj {
    vertices: Vec<Vertex>,
    indices: Vec<VertexIndex>,
}

impl Obj {
    pub fn to_tess<C>(self, ctxt: &mut C) -> Result<Tess<Vertex, VertexIndex, (), Interleaved>, TessError>
        where C: GraphicsContext<Backend = Backend>,
    {
        ctxt.new_tess()
            .set_mode(Mode::Triangle)
            .set_vertices(self.vertices)
            .set_indices(self.indices)
            .build()
    }

    pub fn load<P>(path: P) -> Result<Self, String>
        where P: AsRef<Path>,
    {
        let file_content = {
          let mut file = File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
          let mut content = String::new();
          file.read_to_string(&mut content).unwrap();
          content
        };
        let obj_set = obj::parse(file_content).map_err(|e| format!("cannot parse: {:?}", e))?;
        let objects = obj_set.objects;

        //verify!(objects.len() == 1).ok_or("expecting a single object".to_owned())?;
        if objects.len() != 1 {
            Err("expecting a single object".to_owned())?;
        }

        let object = objects.into_iter().next().unwrap();

        //verify!(object.geometry.len() == 1).ok_or("expecting a single geometry".to_owned())?;
        if object.geometry.len() != 1 {
            Err("expecting a single geometry".to_owned())?;
        }

        let geometry = object.geometry.into_iter().next().unwrap();

        println!("loading {}", object.name);
        println!("{} vertices", object.vertices.len());
        println!("{} shapes", geometry.shapes.len());

        // build up vertices; for this to work, we remove duplicated vertices by putting them in a
        // map associating the vertex with its ID
        let mut vertex_cache: HashMap<obj::VTNIndex, VertexIndex> = HashMap::new();
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<VertexIndex> = Vec::new();

        for shape in geometry.shapes {
          if let obj::Primitive::Triangle(a, b, c) = shape.primitive {
            for key in &[a, b, c] {
              if let Some(vertex_index) = vertex_cache.get(key) {
                indices.push(*vertex_index);
              } else {
                let p = object.vertices[key.0];
                let n = object.normals[key.2.ok_or("missing normals for a vertex")?];
                let position = VertexPosition3D::new([p.x as f32, p.y as f32, p.z as f32]);
                let normal = VertexNormal::new([n.x as f32, n.y as f32, n.z as f32]);
                let vertex = Vertex { position, normal };
                let vertex_index = vertices.len() as VertexIndex;

                vertex_cache.insert(*key, vertex_index);
                vertices.push(vertex);
                indices.push(vertex_index);
              }
            }
          } else {
            return Err("unsupported non-triangle shape".to_owned());
          }
        }

        Ok(Obj { vertices, indices })
    }
}
