use include_flate::flate;

// todo: use bulk data 
// todo: make assets universal
flate!(pub(crate) static INTEGRATOR_API_ASSET: [u8] from "assets/IntegratorAPI.uasset");
flate!(pub(crate) static INTEGRATOR_STATICS_ASSET: [u8] from "assets/IntegratorStatics.uasset");
flate!(pub(crate) static INTEGRATOR_STATICS_BP_ASSET: [u8] from "assets/IntegratorStatics_BP.uasset");
flate!(pub(crate) static LIST_OF_MODS_ASSET: [u8] from "assets/ListOfMods.uasset");
flate!(pub(crate) static METADATA_JSON: [u8] from "assets/metadata.json");
flate!(pub(crate) static MOD_ASSET: [u8] from "assets/Mod.uasset");
flate!(pub(crate) static MOD_MISMATCH_WIDGET_ASSET: [u8] from "assets/ModMismatchWidget.uasset");
flate!(pub(crate) static SERVER_MOD_COMPONENT_ASSET: [u8] from "assets/ServerModComponent.uasset");
flate!(pub(crate) static SYNC_MODE_ASSET: [u8] from "assets/SyncMode.uasset");