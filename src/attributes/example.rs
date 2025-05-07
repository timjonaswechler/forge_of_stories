// --- Asset Collection Definition ---
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "species.elf", typed)] // 'typed' wichtig für RonAssetPlugin via Key
    pub elf_species: Handle<SpeciesData>,
    #[asset(key = "species.human", typed)]
    pub human_species: Handle<SpeciesData>,
    #[asset(key = "species.ork", typed)]
    pub ork_species: Handle<SpeciesData>,
}

// --- RON Data Structures ---
#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct SpeciesData {
    pub species_name: String,
    // Verwenden Sie den tatsächlichen Typ aus Ihrem attributes-Modul
    pub attribute_distributions:
        HashMap<attributes::AttributeType, attributes::AttributeDistribution>,
}
