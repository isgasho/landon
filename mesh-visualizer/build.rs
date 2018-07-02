extern crate blender_armature;
extern crate blender_mesh;
extern crate serde_json;

use std::fs;
use std::fs::DirBuilder;
use std::path::PathBuf;
use std::process::Command;

// TODO: Make a directory for all of our temp build stuff (py scripts) so that we can delete it
// all easily when we're done by deleting the dir
fn main() {
    let mut blender_files = vec![];

    let tests_dir = PathBuf::from("../blender-mesh/tests");

    for entry in tests_dir.read_dir().expect("blender-mesh tests dir") {
        let blender_file = entry.unwrap().path().display().to_string();

        if blender_file.ends_with(".blend") {
            blender_files.push(blender_file)
        }
    }

    rm_and_create_dir("/tmp/blender-export");

    let install_mesh2json = include_str!("../install-addon.py");
    let install_mesh2json_path = "/tmp/install-mesh-exporter.py";
    fs::write(install_mesh2json_path, install_mesh2json).unwrap();

    let addon = include_str!("../blender-mesh-to-json.py");
    let temp_addon = "/tmp/blender-mesh-to-json.py";
    fs::write(temp_addon, addon).unwrap();

    let addon = include_str!("../blender-armature/src/blender-armature-to-json.py");
    let temp_addon = "/tmp/blender-export/blender-armature-to-json.py";
    fs::write(temp_addon, addon).unwrap();

    let mut blender_process = Command::new("blender");
    let mut blender_process = blender_process
        .arg("--background")
        .args(&["--python", install_mesh2json_path])
        // TODO: An API in our root crate for writing the script to a tmp file and giving you
        // a link to it
        .args(&["--python", "./blender-armature/install-armature-to-json.py"]);

    for blender_file in blender_files {
        println!("cargo:rerun-if-changed=../tests/{}", blender_file);

        let open_script = &open_blend_file(&blender_file);

        blender_process
            .args(&["--python-expr", open_script])
            .args(&["--python-expr", &export_blender_data()]);
    }

    let blender_output = blender_process
        .output()
        .expect("Failed to execute Blender process");

    let blender_stdout = String::from_utf8(blender_output.stdout).unwrap();
    fs::write("/tmp/foobar", blender_stdout.clone());
    fs::write(
        "/tmp/error",
        String::from_utf8(blender_output.stderr).unwrap(),
    );

    let meshes = blender_mesh::parse_meshes_from_blender_stdout(&blender_stdout).unwrap();
    let armatures = blender_armature::parse_armatures_from_blender_stdout(&blender_stdout).unwrap();

    rm_and_create_dir("./dist");

    for (_filename, meshes) in meshes.iter() {
        for (mesh_name, mesh) in meshes.iter() {
            let mesh_json = serde_json::to_string(mesh).unwrap();

            let mesh_json_filename = &format!("./dist/{}.json", mesh_name);
            fs::write(mesh_json_filename, mesh_json).unwrap();
        }
    }

    for (_filename, armatures) in armatures.iter() {
        for (armature_name, armature) in armatures.iter() {
            let armature_json = serde_json::to_string(armature).unwrap();

            let armature_json_filename = &format!("./dist/{}.json", armature_name);
            fs::write(armature_json_filename, armature_json).unwrap();
        }
    }
}

fn rm_and_create_dir(dirname: &str) {
    DirBuilder::new().recursive(true).create(dirname).unwrap();
    fs::remove_dir_all(dirname).unwrap();
    DirBuilder::new().recursive(true).create(dirname).unwrap();
}

fn open_blend_file<'a>(file: &str) -> String {
    format!(
        r#"
import bpy
bpy.ops.wm.open_mainfile(filepath="{}")"#,
        file
    )
}

fn export_blender_data() -> String {
    r#"
import bpy

bpy.context.scene.objects.active = None

for obj in bpy.context.scene.objects:
    bpy.context.scene.objects.active = obj

    if obj.type == 'MESH':
      bpy.ops.import_export.mesh2json()
    if obj.type == 'ARMATURE':
      bpy.ops.import_export.armature2json()
    "#.to_string()
}