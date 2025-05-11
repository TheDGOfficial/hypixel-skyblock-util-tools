// Source: https://wiki.hypixel.net/General's_Medallion#Usage
pub(crate) const SECRETS_NEEDED_FOR_MAX_GENERALS_MEDALLION: i32 = 100_000;

// Source: https://hypixel-skyblock.fandom.com/wiki/Magic_Find#Trivia
pub(crate) const MAXIMUM_MAGIC_FIND: i32 = 900;

// Source: https://wiki.hypixel.net/Minos_Inquisitor#Loot
pub(crate) const CHIMERA_DROP_CHANCE: f64 = 1.0;

// Source: In-game RNG Meter base chance.
pub(crate) const JUDGEMENT_CORE_DROP_CHANCE: f64 = 0.0565;

// Source: In-game RNG Meter base chance.
pub(crate) const WARDEN_HEART_DROP_CHANCE: f64 = 0.0138;

// Source: In-game RNG Meter base chance.
pub(crate) const OVERFLUX_CAPACITOR_DROP_CHANCE: f64 = 0.0406;

// Source: In-game RNG Meter base chance.
pub(crate) const NECRONS_HANDLE_DROP_CHANCE: f64 = 0.1094;

// Source: In-game RNG Meter base chance.
pub(crate) const NECRONS_HANDLE_MASTER_MODE_DROP_CHANCE: f64 = 0.1296;

// Source: In-game RNG Meter base chance.
pub(crate) const DARK_CLAYMORE_DROP_CHANCE: f64 = 0.072;

// Shadow Assassin
// Source: https://wiki.hypixel.net/Shadow_Assassin#Stats
pub(crate) const F3_SHADOW_ASSASSIN_DAMAGE: i32 = 3280;
pub(crate) const F4_SHADOW_ASSASSIN_DAMAGE: i32 = F3_SHADOW_ASSASSIN_DAMAGE;
pub(crate) const F5_SHADOW_ASSASSIN_DAMAGE: i32 = 6640; // Has 2 variants, take the higher one
pub(crate) const F6_SHADOW_ASSASSIN_DAMAGE: i32 = 8640;
pub(crate) const F7_SHADOW_ASSASSIN_DAMAGE: i32 = 48000; // Has 2 variants, take the higher one

// Master Shadow Assassin
// Source: https://wiki.hypixel.net/Shadow_Assassin#Master_Mode_Shadow_Assassin
pub(crate) const M3_SHADOW_ASSASSIN_DAMAGE: i32 = 175_000;
pub(crate) const M4_SHADOW_ASSASSIN_DAMAGE: i32 = M3_SHADOW_ASSASSIN_DAMAGE;
pub(crate) const M5_SHADOW_ASSASSIN_DAMAGE: i32 = 270_000; // Has 2 variants, take the higher one
pub(crate) const M6_SHADOW_ASSASSIN_DAMAGE: i32 = 400_000;
pub(crate) const M7_SHADOW_ASSASSIN_DAMAGE: i32 = M6_SHADOW_ASSASSIN_DAMAGE; // Has 2 variants, but they do the same damage unlike F7, and they also do same
// damage as Shadow Assassins on M6. (although they have much more HP)

// Fels
// Source: https://wiki.hypixel.net/Fels#Stats
pub(crate) const F5_FELS_DAMAGE: i32 = 8000;
pub(crate) const F6_FELS_DAMAGE: i32 = 9600;
pub(crate) const F7_FELS_DAMAGE: i32 = 20000;

// Master Fels
// Source: https://wiki.hypixel.net/Fel#Master_Mode_Fels
pub(crate) const M5_FELS_DAMAGE: i32 = 150_000;
pub(crate) const M6_FELS_DAMAGE: i32 = 200_000;
pub(crate) const M7_FELS_DAMAGE: i32 = 240_000;

// Voidgloom Seraph (Tier 1)
pub(crate) const VOIDGLOOM_SERAPH_TIER_1_BASE_DAMAGE: i32 = 1200;
pub(crate) const VOIDGLOOM_SERAPH_TIER_1_AOE_DAMAGE: i32 = 720;

pub(crate) const VOIDGLOOM_SERAPH_TIER_1_TOTAL_DAMAGE: i32 =
    VOIDGLOOM_SERAPH_TIER_1_BASE_DAMAGE + VOIDGLOOM_SERAPH_TIER_1_AOE_DAMAGE;

// Voidgloom Seraph (Tier 2)
pub(crate) const VOIDGLOOM_SERAPH_TIER_2_BASE_DAMAGE: i32 = 5000;
pub(crate) const VOIDGLOOM_SERAPH_TIER_2_AOE_DAMAGE: i32 = 3000;

pub(crate) const VOIDGLOOM_SERAPH_TIER_2_TOTAL_DAMAGE: i32 =
    VOIDGLOOM_SERAPH_TIER_2_BASE_DAMAGE + VOIDGLOOM_SERAPH_TIER_2_AOE_DAMAGE;

// Voidgloom Seraph (Tier 3)
pub(crate) const VOIDGLOOM_SERAPH_TIER_3_BASE_DAMAGE: i32 = 12000;
pub(crate) const VOIDGLOOM_SERAPH_TIER_3_AOE_DAMAGE: i32 = 7200;

pub(crate) const VOIDGLOOM_SERAPH_TIER_3_TOTAL_DAMAGE: i32 =
    VOIDGLOOM_SERAPH_TIER_3_BASE_DAMAGE + VOIDGLOOM_SERAPH_TIER_3_AOE_DAMAGE;

// Voidgloom Seraph (Tier 4)
pub(crate) const VOIDGLOOM_SERAPH_TIER_4_BASE_DAMAGE: i32 = 21000;
pub(crate) const VOIDGLOOM_SERAPH_TIER_4_AOE_DAMAGE: i32 = 12600;

pub(crate) const VOIDGLOOM_SERAPH_TIER_4_TOTAL_DAMAGE: i32 =
    VOIDGLOOM_SERAPH_TIER_4_BASE_DAMAGE + VOIDGLOOM_SERAPH_TIER_4_AOE_DAMAGE;
