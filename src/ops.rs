use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};
use tar::Archive;
use toml::Value;
use walkdir::WalkDir;
use flate2::read::GzDecoder;
use tempfile;

const DB_PATH: &str = "/var/lib/retropkg/packages.json";

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let rel_path = path.strip_prefix(src)?;
        let dst_path = dst.join(rel_path);
        
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dst_path)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn install_package(package_path: &Path) -> Result<()> {
    // Open and extract package
    let tar_gz = File::open(package_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    
    let temp_dir = tempfile::tempdir()?;
    archive.unpack(&temp_dir)?;
    
    // Parse manifest
    let manifest_path = temp_dir.path().join("manifest.toml");
    let mut manifest_file = File::open(&manifest_path)?;
    let mut manifest_content = String::new();
    manifest_file.read_to_string(&mut manifest_content)?;
    
    let manifest: Value = toml::from_str(&manifest_content)?;
    let pkg = manifest["package"].as_table().context("Missing [package]")?;
    
    let name = pkg["name"].as_str().context("Missing name")?.to_string();
    let version = pkg["version"].as_str().context("Missing version")?.to_string();
    
    // Copy files
    let data_dir = temp_dir.path().join("data");
    copy_dir_all(&data_dir, "/")?;
    
    // Add to database
    let db_path = Path::new(DB_PATH);
    let db_dir = db_path.parent().unwrap();
    if !db_dir.exists() {
        fs::create_dir_all(db_dir)?;
    }
    
    let mut packages: serde_json::Value = if db_path.exists() {
        let db_content = fs::read_to_string(db_path)?;
        serde_json::from_str(&db_content)?
    } else {
        serde_json::json!({ "packages": {} })
    };
    
    packages["packages"][&name] = serde_json::json!({
        "version": version,
        "manifest": manifest_content,
    });
    
    fs::write(db_path, serde_json::to_string_pretty(&packages)?)?;
    
    println!("✅ Installed {} v{}", name, version);
    Ok(())
}

pub fn remove_package(package_name: &str) -> Result<()> {
    let db_path = Path::new(DB_PATH);
    if !db_path.exists() {
        anyhow::bail!("Database not found");
    }
    
    let db_content = fs::read_to_string(db_path)?;
    let mut packages: serde_json::Value = serde_json::from_str(&db_content)?;
    
    if packages["packages"][package_name].is_null() {
        anyhow::bail!("Package not found: {}", package_name);
    }
    
    // Note: Actual file removal would require tracking installed files
    // For now, we just remove from database
    
    packages["packages"][package_name] = serde_json::Value::Null;
    packages["packages"] = packages["packages"].as_object_mut().unwrap().clone()
        .into_iter()
        .filter(|(_, v)| !v.is_null())
        .collect();
    
    fs::write(db_path, serde_json::to_string_pretty(&packages)?)?;
    
    println!("✅ Removed {}", package_name);
    Ok(())
}

pub fn list_packages() -> Result<()> {
    let db_path = Path::new(DB_PATH);
    if !db_path.exists() {
        println!("No packages installed");
        return Ok(());
    }
    
    let db_content = fs::read_to_string(db_path)?;
    let packages: serde_json::Value = serde_json::from_str(&db_content)?;
    
    println!("Installed packages:");
    for (name, info) in packages["packages"].as_object().unwrap() {
        println!("- {} v{}", name, info["version"].as_str().unwrap());
    }
    Ok(())
}
