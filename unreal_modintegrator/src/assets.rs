// todo: use bulk data
// todo: make assets universal

macro_rules! cooked_path {
    () => {
        concat!(
            env!("OUT_DIR"),
            "/ModIntegrator/Saved/Cooked/WindowsNoEditor/ModIntegrator/Content/Integrator/"
        )
    };
}

pub(crate) const METADATA_JSON: &'static [u8] = include_bytes!("../assets/metadata.json");

pub(crate) const INTEGRATOR_STATICS_ASSET: &'static [u8] =
    include_bytes!(concat!(cooked_path!(), "IntegratorStatics_BP.uasset"));
#[cfg(bulk_data)]
pub(crate) const INTEGRATOR_STATICS_BULK: &'static [u8] =
    include_bytes!(concat!(cooked_path!(), "IntegratorStatics_BP.uexp"));

pub(crate) const LIST_OF_MODS_ASSET: &'static [u8] =
    include_bytes!(concat!(cooked_path!(), "ListOfMods.uasset"));
#[cfg(bulk_data)]
pub(crate) const LIST_OF_MODS_BULK: &'static [u8] =
    include_bytes!(concat!(cooked_path!(), "ListOfMods.uexp"));

#[cfg(not(bulk_data))]
pub(crate) const COPY_OVER: [(&'static [u8], &'static str); 5] = [
    (
        include_bytes!(concat!(cooked_path!(), "Mod.uasset")),
        "Mod.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModIntegrator.uasset")),
        "ModIntegrator.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModIntegratorComponent.uasset")),
        "ModIntegratorComponent.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModMismatchWidget.uasset")),
        "ModMismatchWidget.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "SyncMode.uasset")),
        "SyncMode.uasset",
    ),
];

#[cfg(bulk_data)]
pub(crate) const COPY_OVER: [(&'static [u8], &'static str); 10] = [
    (
        include_bytes!(concat!(cooked_path!(), "Mod.uasset")),
        "Mod.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "Mod.uexp")),
        "Mod.uexp",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModIntegrator.uasset")),
        "ModIntegrator.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModIntegrator.uexp")),
        "ModIntegrator.uexp",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModIntegratorComponent.uasset")),
        "ModIntegratorComponent.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModIntegratorComponent.uexp")),
        "ModIntegratorComponent.uexp",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModMismatchWidget.uasset")),
        "ModMismatchWidget.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "ModMismatchWidget.uexp")),
        "ModMismatchWidget.uexp",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "SyncMode.uasset")),
        "SyncMode.uasset",
    ),
    (
        include_bytes!(concat!(cooked_path!(), "SyncMode.uexp")),
        "SyncMode.uexp",
    ),
];
