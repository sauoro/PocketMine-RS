// src/main.rs
mod utils;
mod color;
mod math;
mod nbt;

// Import necessary NBT components
use nbt::{
    BigEndianNbtSerializer, // Choose your serializer (BE is common for Java Edition)
    CompoundTag,            // Main container tag
    ListTag,                // List container tag
    TagType,                // Enum for tag types
    TreeRoot,               // Wrapper for the root tag
    Result as NbtResult,    // NBT specific Result type
    tag::*,                 // Import all concrete tag types (ByteTag, IntTag, etc.)
};
use std::error::Error; // For generic error handling

fn run_nbt_example() -> NbtResult<()> { // Use NbtResult for NBT operations
    println!("--- NBT Example Start ---");

    // 1. Create an NBT structure
    let mut root_compound = CompoundTag::new();

    root_compound.set_string("Name".to_string(), "Example Player".to_string())?;
    root_compound.set_short("Health".to_string(), 20)?;
    root_compound.set_float("AbsorptionAmount".to_string(), 5.5)?;
    root_compound.set_byte("Invulnerable".to_string(), 0)?; // 0 for false

    // Create a list tag for inventory items
    let mut inventory_list = ListTag::new(TagType::Compound); // List holds CompoundTags

    // Item 1: Stone
    let mut item1 = CompoundTag::new();
    item1.set_string("id".to_string(), "minecraft:stone".to_string())?;
    item1.set_byte("Count".to_string(), 64)?;
    item1.set_short("Damage".to_string(), 0)?;
    inventory_list.push(Box::new(item1))?; // Add item to list

    // Item 2: Diamond Sword
    let mut item2 = CompoundTag::new();
    item2.set_string("id".to_string(), "minecraft:diamond_sword".to_string())?;
    item2.set_byte("Count".to_string(), 1)?;
    item2.set_short("Damage".to_string(), 15)?; // Example damage value
    // Add enchantments (List of Compounds)
    let mut enchantments = ListTag::new(TagType::Compound);
    let mut ench1 = CompoundTag::new();
    ench1.set_short("id".to_string(), 16)?; // Sharpness
    ench1.set_short("lvl".to_string(), 5)?;
    enchantments.push(Box::new(ench1))?;
    item2.set_list("ench".to_string(), enchantments)?; // Add enchantments list to item2
    inventory_list.push(Box::new(item2))?; // Add item to list

    // Add the inventory list to the root compound
    root_compound.set_list("Inventory".to_string(), inventory_list)?;

    // Create the TreeRoot (required for serialization with a name)
    let tree = TreeRoot::new("playerData".to_string(), Box::new(root_compound))?;

    println!("Original NBT Structure:");
    println!("{}", tree); // Uses the Display impl (fmt_pretty)

    // 2. Serialize the TreeRoot to bytes (Big Endian)
    let serialized_bytes = BigEndianNbtSerializer::write_to_bytes(&tree)?;

    println!("\nSerialized Bytes (Hex):");
    // Print bytes in a readable hex format (optional)
    for byte in &serialized_bytes {
        print!("{:02X} ", byte);
    }
    println!("\nSerialized Bytes Length: {} bytes", serialized_bytes.len());


    // 3. Deserialize the bytes back into a TreeRoot
    println!("\nDeserializing...");
    // Use max_depth 0 for no limit, or a reasonable number like 512 for safety
    let deserialized_tree = BigEndianNbtSerializer::read_from_buffer(&serialized_bytes, 512)?;

    println!("\nDeserialized NBT Structure:");
    println!("{}", deserialized_tree);

    // 4. Access data from the deserialized structure
    println!("\nAccessing Deserialized Data:");

    // Get the root compound tag (mustGetCompoundTag returns Result)
    let deserialized_root = deserialized_tree.must_get_compound_tag()?;

    // Access primitive values (using default None will error if tag doesn't exist)
    let player_name = deserialized_root.get_string("Name", None)?;
    let health = deserialized_root.get_short("Health", None)?;
    println!("Player Name: {}", player_name);
    println!("Health: {}", health);

    // Access a ListTag
    if let Some(inventory) = deserialized_root.get_list_tag("Inventory")? {
        println!("Inventory contains {} items:", inventory.len());
        for (index, item_tag_dyn) in inventory.iter().enumerate() {
            // Downcast the dyn Tag to CompoundTag
            if let Some(item_compound) = item_tag_dyn.as_any().downcast_ref::<CompoundTag>() {
                let item_id = item_compound.get_string("id", None).unwrap_or_else(|_| "Unknown".to_string());
                let count = item_compound.get_byte("Count", None).unwrap_or(0);
                println!(" - Item {}: ID={}, Count={}", index, item_id, count);

                // Access nested list (enchantments)
                if item_id == "minecraft:diamond_sword" {
                    if let Some(ench_list) = item_compound.get_list_tag("ench")? {
                        println!("   Enchantments:");
                        for ench_tag_dyn in ench_list.iter() {
                            if let Some(ench_compound) = ench_tag_dyn.as_any().downcast_ref::<CompoundTag>() {
                                let ench_id = ench_compound.get_short("id", None).unwrap_or(-1);
                                let ench_lvl = ench_compound.get_short("lvl", None).unwrap_or(-1);
                                println!("     - ID: {}, Level: {}", ench_id, ench_lvl);
                            }
                        }
                    }
                }
            } else {
                println!(" - Item {}: Error - Expected CompoundTag", index);
            }
        }
    } else {
        println!("Inventory tag not found or not a ListTag.");
    }

    println!("\n--- NBT Example End ---");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> { // Use generic Error for main
    if let Err(e) = run_nbt_example() {
        eprintln!("NBT Example failed: {}", e);
        // Optionally return the specific error or exit
        // return Err(Box::new(e));
    }
    Ok(())
}