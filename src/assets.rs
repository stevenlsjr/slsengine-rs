use crate::platform_system::asset_path;
use serde::ser::Serialize;
use serde::Serializer;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub name: Option<String>,
    pub path: String,
    pub kind: AssetKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetList {
    pub assets: Vec<Asset>,
}

#[derive(Debug)]
pub struct IndexedAssetList {
    pub assets: Vec<Asset>,
    pub names: HashMap<String, usize>,
    pub paths: HashMap<String, usize>,
}

impl IndexedAssetList {
    pub fn new(list: AssetList) -> Self {
        let assets = list.assets;
        let mut names = HashMap::new();
        let mut paths = HashMap::new();

        for (i, asset) in assets.iter().enumerate() {
            if let Some(name) = asset.name.as_ref() {
                names.insert(name.clone(), i);
            }

            paths.insert(asset.path.clone(), i);
        }

        return IndexedAssetList {
            assets,
            names,
            paths,
        };
    }
}

impl From<AssetList> for IndexedAssetList {
    fn from(l: AssetList) -> Self {
        IndexedAssetList::new(l)
    }
}

impl From<IndexedAssetList> for AssetList {
    fn from(l: IndexedAssetList) -> Self {
        AssetList { assets: l.assets }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AssetKind {
    Image,
    GltfMesh,
    GlbMesh,
    Shader,
}

#[test]
fn test_serialize_assets() {
    use std::fs::*;
    use toml;
    let asset = Asset {
        name: None,
        path: "shaders/brdf.frag".to_owned(),
        kind: AssetKind::Shader,
    };

    let toml = toml::to_string(&AssetList {
        assets: vec![asset],
    });
    assert!(toml.is_ok());
}

#[test]
fn test_deser_assets() {
    use std::{fs::*, ops::*};
    use toml;
    let list: HashMap<String, Vec<Asset>> = toml::from_str(
        r#"
        [[assets]]
        name="foo",
        path="foo.vs",
        kind="Shader"
        [[assets]]
        path="foo.fs",
        kind="Shader"
    "#,
    )
    .unwrap();
    let list = list.index("assets");
    assert_eq!(list[0].name.as_ref().map(&String::as_str), Some("foo"));
    assert_eq!(&list[0].path, "foo.vs");
    assert_eq!(list[1].name.as_ref(), None);
}

#[test]
fn test_indexing() {
    let list: AssetList = toml::from_str(
        r#"
        [[assets]]
        name="foo"
        path="foo.vs"
        kind="Shader"
        [[assets]]
        path="foo.fs"
        kind="Shader"
    "#,
    )
    .unwrap();
    let list = IndexedAssetList::new(list);
    assert_eq!(list.names.get("foo"), Some(&0));
    assert_eq!(list.paths.get("foo.fs"), Some(&1));
}
