//! Managers the loading and storage of our assets.
//! Namely, meshes and armatures that came from Blender and textures png's.

use crate::state_wrapper::{Msg, StateWrapper};
use bincode;
use blender_armature::BlenderArmature;
use blender_mesh::{BlenderMesh, CreateSingleIndexConfig, SingleIndexedVertexAttributes};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::__rt::core::ops::Deref;

/// TODO: Just temporary as we refactor things - need to delete and re-write the mesh visualizer
/// at this point
pub struct MeshAndAttributes {
    pub mesh: BlenderMesh,
    pub attributes: SingleIndexedVertexAttributes,
}

impl Deref for MeshAndAttributes {
    type Target = BlenderMesh;

    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

type Meshes = Rc<RefCell<HashMap<String, MeshAndAttributes>>>;
type Armatures = Rc<RefCell<HashMap<String, BlenderArmature>>>;

#[wasm_bindgen]
extern "C" {
    /// Bridge the gap for things that are currently unsupported or difficult to accomplish with
    /// wasm-bindgen
    pub type WasmHelpers;

    #[wasm_bindgen(static_method_of = WasmHelpers)]
    pub fn fetch_u8_array(url: &str, callback: &js_sys::Function);
}

pub struct Assets {
    /// All of our Blender models that we have downloaded and can render
    meshes: Meshes,
    /// All of our Blender armatures that we have downloaded and can render
    armatures: Armatures,
}

impl Assets {
    pub fn new() -> Assets {
        Assets {
            meshes: Rc::new(RefCell::new(HashMap::new())),
            armatures: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn load_meshes(&mut self, state_wrap: Rc<RefCell<StateWrapper>>) {
        let _request_url = "/dist/meshes.bytes";

        let meshes_clone = Rc::clone(&self.meshes);

        let deserialize_meshes = move |mesh_bytes: Box<[u8]>| {
            let meshes: HashMap<String, BlenderMesh> = bincode::deserialize(&mesh_bytes).unwrap();

            for (mesh_name, mut mesh) in meshes {
                info!("{}", mesh_name);
                let attributes = mesh.combine_vertex_indices(&CreateSingleIndexConfig {
                    calculate_face_tangents: false,
                    bone_influences_per_vertex: None,
                });
                mesh.triangulate();
                mesh.y_up();

                meshes_clone.borrow_mut().insert(
                    mesh_name.to_string(),
                    MeshAndAttributes { mesh, attributes },
                );
            }

            // Refresh material params now that we've loaded our mesh
            let current_model = state_wrap.borrow().current_model.clone();
            state_wrap
                .borrow_mut()
                .msg(Msg::SetCurrentMesh(current_model));
        };

        let closure = Closure::wrap(Box::new(deserialize_meshes) as Box<dyn FnMut(_)>);

        let callback = closure.as_ref().unchecked_ref();
        WasmHelpers::fetch_u8_array("/dist/meshes.bytes", callback);

        closure.forget();
    }

    // TODO: Temporarily commented out while I refactor
    pub fn load_armature(&mut self, _armature_name: &str) {
        //        let armatures_clone = Rc::clone(&self.armatures);
        //
        //        let deserialize_armatures = move |armatures_json: &[u8]| {
        //            let armatures: HashMap<String, BlenderArmature> =
        //                bincode::deserialize(&armatures_json).unwrap();
        //
        //            for (armature_name, mut armature) in armatures {
        //                armature.apply_inverse_bind_poses();
        //                armature.transpose_actions();
        //                armature.actions_to_dual_quats();
        //
        //                armatures_clone
        //                    .borrow_mut()
        //                    .insert(armature_name.to_string(), armature);
        //            }
        //        };
        //
        //        let on_armatures_downloaded = Closure::new(deserialize_armatures);
        //
        //        let _model_path = &format!("dist/{}.json", armature_name);
        //        download_string("/dist/armatures.json".to_string(), &on_armatures_downloaded);
        //
        //        on_armatures_downloaded.forget();
    }

    pub fn meshes(&self) -> Meshes {
        Rc::clone(&self.meshes)
    }

    pub fn armatures(&self) -> Armatures {
        Rc::clone(&self.armatures)
    }
}
