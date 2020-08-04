use maniparse::Manifest;
use std::env;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: maniparse <path>");
        std::process::exit(1);
    }
    let results = Manifest::from_path(args[1].as_str())?;

    println!("Name: {}", results.name());
    println!("Version: {}", results.version());
    println!("Exports: {:?}", results.export_keys());
    println!("Flavors:");

    let flavs = results.flavors()?;
    flavs.iter().for_each(|v| println!("\t{}", v));
    if let Some(ref mut keys) = results.export_keys() {
        println!("Exports:");
        for key in keys {
            println!("\t{}", key);
            results.exports_for(key).unwrap().iter().for_each(|t| println!("\t\t{}",t));
        }
    }
 

    Ok(())

}
