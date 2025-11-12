use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "tests/data/geavg.t12z.pgrb2a.0p50.f000";

    println!("Reading: {}", file_path);
    let data = fs::read(file_path)?;
    println!("File size: {} MB", data.len() / 1024 / 1024);

    // Try to scan messages
    println!("\n=== Scanning messages (native backend) ===");
    let messages = gribberish::message::read_messages(&data);
    let mut count = 0;
    let mut template_ids = std::collections::HashMap::new();

    let mut unique_pert_numbers = std::collections::HashSet::new();

    for (i, msg) in messages.enumerate() {
        match msg.product_template_id() {
            Ok(template_id) => {
                *template_ids.entry(template_id).or_insert(0) += 1;

                let var = msg.variable_abbrev().unwrap_or("unknown".to_string());
                let is_ens = msg.is_ensemble().unwrap_or(false);
                let pert = msg.perturbation_number().ok().flatten();
                let ens_size = msg.ensemble_size().ok().flatten();
                let key = msg.key().unwrap_or("no key".to_string());

                if let Some(p) = pert {
                    unique_pert_numbers.insert(p);
                }

                if i < 5 || (pert.is_some() && pert != Some(0)) {
                    println!("\nMessage {}: template={}, var={}, ens={}, pert={:?}, size={:?}",
                             i, template_id, var, is_ens, pert, ens_size);
                    println!("  Key: {}", key);
                }
            }
            Err(e) => println!("\nMessage {}: Error getting template: {}", i, e),
        }
        count += 1;

        if count >= 500 {
            println!("\n... stopping after 500 messages for brevity");
            break;
        }
    }

    println!("\n=== Ensemble Members Found ===");
    let mut pert_vec: Vec<_> = unique_pert_numbers.into_iter().collect();
    pert_vec.sort();
    println!("Unique perturbation numbers: {:?}", pert_vec);

    println!("\n=== Summary ===");
    println!("Total messages scanned: {}", count);
    println!("\nTemplate IDs found:");
    for (template_id, count) in template_ids.iter() {
        println!("  Template {}: {} messages", template_id, count);
    }

    Ok(())
}
