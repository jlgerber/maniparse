//! Manifest parsing structures
use anyhow::anyhow;
use anyhow::Error as AnyError;
use itertools::iproduct;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;


type ManifestBuildMatrix = HashMap<String,Vec<Version>>;

type RequirementMap = HashMap<String, Version>;
use mustache::MapBuilder;
use mustache;

#[derive(Debug, PartialEq, PartialOrd, Deserialize)]
#[serde(untagged)]
pub enum Version {
    String(String),
    Float(f32),
    Int(u16)
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        match self {
            Self::String(ref s) =>  write!(f, "{}", s),
            Self::Float(fv) => write!(f, "{}",fv),
            Self::Int(i) => write!(f,"{}", i ),
        }
       
    }
}

#[derive(Debug,PartialEq,Deserialize)]
pub struct RecipeInner {
    requires: Option<RequirementMap>,
    #[serde(rename = "loadRequires")]
    load_requires: Option<RequirementMap>,
    steps: Vec<String>,
}

type RecipeMap = HashMap<String, RecipeInner>;

#[derive(Debug,PartialEq,Deserialize)]

pub struct MatrixFlavour{
    name: String,
    matrix: ManifestBuildMatrix,

}

#[derive(Debug,PartialEq,Deserialize)]
pub struct BuildFlavour {
    name: String,
    recipes: RecipeMap,
}
#[derive(Debug,PartialEq,Deserialize)]
pub struct Tools {
    tools: Vec<String>
}

#[derive(Debug,PartialEq,Deserialize)]
#[serde(untagged)]
pub enum Flavours {
    Simple{
        name: String,
        recipes: RecipeMap,
    },
    Matrix{
        name: String,
        matrix: ManifestBuildMatrix,
        requires: Option<RequirementMap>,
        #[serde(rename = "loadRequires")]
        load_requires: Option<RequirementMap>
    
    }
}

#[derive(Debug,PartialEq,Deserialize)]
pub struct Manifest {
    name: String,
    version: String,
    #[serde(rename = "loadRequires")]
    load_requires: Option<RequirementMap>,
    requires: Option<RequirementMap>,
    recipes: Option<RecipeMap>,
    flavours: Option<Vec<Flavours>>,
    exports: Option<Tools>
}

impl Manifest {
    pub fn from_path<I>(path: I) -> Result<Manifest, AnyError> where I: Into<PathBuf> {
        let manifest_path = path.into();
        let contents = std::fs::read_to_string(manifest_path)?;
        let manifest: Manifest = serde_yaml::from_str(&contents)?;
        Ok(manifest)
    }

    pub fn from_str<I>(contents: I) -> Result<Manifest, AnyError> where I: AsRef<str> {
        let contents = contents.as_ref();
        let manifest : Manifest = serde_yaml::from_str(contents)?;
        Ok(manifest)
    }

    pub fn flavors(&self) -> Result<Vec<String>, AnyError> {
        let mut flavors = Vec::new();
        if self.requires.is_some() || self.recipes.is_some() {
            flavors.push("^".to_string());
        }
        if let Some(ref flavs) = self.flavours {
            for fl in flavs {
                    match fl {
                    Flavours::Simple{name,..} => flavors.push(name.to_string()),
                    Flavours::Matrix{name,matrix,..} => {
                        let mut par = Vec::new();
                        let mut keys =Vec::new();
                        for (k, v) in matrix.iter(){
                            keys.push(k.as_str());
                            par.push( v.iter().map(|c| c).collect::<Vec<_>>() );
                        }
                        let mut expand = match keys.len() {
                            1 =>Self::one(name.as_str(), &keys, &par[0]),
                            2 => Self::two(name.as_str(), &keys, &par[0], &par[1]),
                            3 => Self::three(name.as_str(), &keys, &par[0], &par[1], &par[2]),
                            4 => Self::four(name.as_str(), &keys, &par[0], &par[1], &par[2], &par[4]),
                            _ => Err(anyhow!("Cannot expand template with more than four arguments"))
                        }?;
                        
                        //let mut rval = Self::two(name.as_str(), &keys, &par[0], &par[1])?;
                        flavors.append(&mut expand);
                    }
                
                }
            }

        }
        Ok(flavors)
    } 
    // Iterate over single key
    fn one(template: &str, keys: &Vec<&str>, one: &Vec<&Version>) -> Result<Vec<String>, AnyError> {
        let  mut results = Vec::new();
        for  i in one {
            let map = MapBuilder::new()
            .insert_str(keys[0], i.to_string().as_str())
            .build();
            let rtemplate = mustache::compile_str(template.replace("row.","").as_str())?;
            let r = rtemplate.render_data_to_string( &map)?;
            results.push(r);
        }
        Ok(results)
    }
    // iterate over two keys
    fn two(template: &str, keys: &Vec<&str>, one: &Vec<&Version>, two: &Vec<&Version>) -> Result<Vec<String>, AnyError> {
        let  mut results = Vec::new();
        for ( i,j) in iproduct!(one,two) {
            let map = MapBuilder::new()
            .insert_str(keys[0], i.to_string().as_str())
            .insert_str(keys[1], j.to_string().as_str())
            .build();
            let rtemplate = mustache::compile_str(template.replace("row.","").as_str())?;
            let r = rtemplate.render_data_to_string( &map)?;
            results.push(r);
        }
        Ok(results)
    }

    fn three(template: &str, keys: &Vec<&str>, one: &Vec<&Version>, two: &Vec<&Version>, three: &Vec<&Version>) -> Result<Vec<String>,AnyError> {
        let  mut results = Vec::new();
        for ( i,j,k) in iproduct!(one,two,three) {
            let map = MapBuilder::new()
            .insert_str(keys[0], i.to_string().as_str())
            .insert_str(keys[1], j.to_string().as_str())
            .insert_str(keys[2], k.to_string().as_str())
            .build();
            let rtemplate = mustache::compile_str(template.replace("row.","").as_str())?;
            let r = rtemplate.render_data_to_string( &map)?;
            results.push(r);
        }
        Ok(results)
    }

    fn four(template: &str, keys: &Vec<&str>, one: &Vec<&Version>, two: &Vec<&Version>, three: &Vec<&Version>, four: &Vec<&Version>) -> Result<Vec<String>,AnyError> {
        let  mut results = Vec::new();
        for ( i,j,k,l) in iproduct!(one,two,three, four) {
            let map = MapBuilder::new()
            .insert_str(keys[0], i.to_string().as_str())
            .insert_str(keys[1], j.to_string().as_str())
            .insert_str(keys[2], k.to_string().as_str())
            .insert_str(keys[3], l.to_string().as_str())
            .build();
            let rtemplate = mustache::compile_str(template.replace("row.","").as_str())?;
            let r = rtemplate.render_data_to_string( &map)?;
            results.push(r);
        }
        Ok(results)
    }
}