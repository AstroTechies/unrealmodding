use num_enum::IntoPrimitive;

#[derive(Debug)]
pub enum EObjectFlags
{
    RF_NoFlags = 0x00000000,
    RF_Public = 0x00000001,
    RF_Standalone = 0x00000002,
    RF_MarkAsNative = 0x00000004,
    RF_Transactional = 0x00000008,
    RF_ClassDefaultObject = 0x00000010,
    RF_ArchetypeObject = 0x00000020,
    RF_Transient = 0x00000040,
    RF_MarkAsRootSet = 0x00000080,
    RF_TagGarbageTemp = 0x00000100,
    RF_NeedInitialization = 0x00000200,
    RF_NeedLoad = 0x00000400,
    RF_KeepForCooker = 0x00000800,
    RF_NeedPostLoad = 0x00001000,
    RF_NeedPostLoadSubobjects = 0x00002000,
    RF_NewerVersionExists = 0x00004000,
    RF_BeginDestroyed = 0x00008000,
    RF_FinishDestroyed = 0x00010000,
    RF_BeingRegenerated = 0x00020000,
    RF_DefaultSubObject = 0x00040000,
    RF_WasLoaded = 0x00080000,
    RF_TextExportTransient = 0x00100000,
    RF_LoadCompleted = 0x00200000,
    RF_InheritableComponentTemplate = 0x00400000,
    RF_DuplicateTransient = 0x00800000,
    RF_StrongRefOnFrame = 0x01000000,
    RF_NonPIEDuplicateTransient = 0x02000000,
    RF_Dynamic = 0x04000000,
    RF_WillBeLoaded = 0x08000000,
    RF_HasExternalPackage = 0x10000000
}

impl Default for EObjectFlags {
    fn default() -> Self {
        Self::RF_NoFlags
    }
}

#[derive(Debug)]
pub enum EPackageFlags
{
    //No flags
    PKG_None = 0x00000000,
    //Newly created package, not saved yet. In editor only.
    PKG_NewlyCreated = 0x00000001,
    //Purely optional for clients.
    PKG_ClientOptional = 0x00000002,
    //Only needed on the server side.
    PKG_ServerSideOnly = 0x00000004,
    //This package is from "compiled in" classes.
    PKG_CompiledIn = 0x00000010,
    //This package was loaded just for the purposes of diffing
    PKG_ForDiffing = 0x00000020,
    //This is editor-only package (for example: editor module script package)
    PKG_EditorOnly = 0x00000040,
    //Developer module
    PKG_Developer = 0x00000080,
    //Loaded only in uncooked builds (i.e. runtime in editor)
    PKG_UncookedOnly = 0x00000100,
    //Package is cooked
    PKG_Cooked = 0x00000200,
    //Package doesn't contain any asset object (although asset tags can be present)
    PKG_ContainsNoAsset = 0x00000400,
    //Uses unversioned property serialization instead of versioned tagged property serialization
    PKG_UnversionedProperties = 0x00002000,
    //Contains map data (UObjects only referenced by a single ULevel) but is stored in a different package
    PKG_ContainsMapData = 0x00004000,
    //package is currently being compiled
    PKG_Compiling = 0x00010000,
    //Set if the package contains a ULevel/ UWorld object
    PKG_ContainsMap = 0x00020000,
    //???
    PKG_RequiresLocalizationGather = 0x00040000,
    //Set if the package was created for the purpose of PIE
    PKG_PlayInEditor = 0x00100000,
    //Package is allowed to contain UClass objects
    PKG_ContainsScript = 0x00200000,
    //Editor should not export asset in this package
    PKG_DisallowExport = 0x00400000,
    //This package should resolve dynamic imports from its export at runtime.
    PKG_DynamicImports = 0x10000000,
    //This package contains elements that are runtime generated, and may not follow standard loading order rules
    PKG_RuntimeGenerated = 0x20000000,
    //This package is reloading in the cooker, try to avoid getting data we will never need. We won't save this package.
    PKG_ReloadingForCooker = 0x40000000,
    //Package has editor-only data filtered out
    PKG_FilterEditorOnly = 0x80000000,
}

impl Default for EPackageFlags {
    fn default() -> Self {
        EPackageFlags::PKG_None
    }
}


#[derive(Debug)]
pub enum EPropertyFlags
{
    CPF_None = 0,

    //Property is user-settable in the editor.
    CPF_Edit = 0x0000000000000001,
    //This is a constant function parameter
    CPF_ConstParm = 0x0000000000000002,
    //This property can be read by blueprint code
    CPF_BlueprintVisible = 0x0000000000000004,
    //Object can be exported with actor.
    CPF_ExportObject = 0x0000000000000008,
    //This property cannot be modified by blueprint code
    CPF_BlueprintReadOnly = 0x0000000000000010,
    //Property is relevant to network replication.
    CPF_Net = 0x0000000000000020,
    //Indicates that elements of an array can be modified, but its size cannot be changed.
    CPF_EditFixedSize = 0x0000000000000040,
    //Function/When call parameter.
    CPF_Parm = 0x0000000000000080,
    //Value is copied out after function call.
    CPF_OutParm = 0x0000000000000100,
    //memset is fine for construction
    CPF_ZeroConstructor = 0x0000000000000200,
    //Return value.
    CPF_ReturnParm = 0x0000000000000400,
    //Disable editing of this property on an archetype/sub-blueprint
    CPF_DisableEditOnTemplate = 0x0000000000000800,
    //Property is transient: shouldn't be saved or loaded, except for Blueprint CDOs.
    CPF_Transient = 0x0000000000002000,
    //Property should be loaded/saved as permanent profile.
    CPF_Config = 0x0000000000004000,
    //Disable editing on an instance of this class
    CPF_DisableEditOnInstance = 0x0000000000010000,
    //Property is uneditable in the editor.
    CPF_EditConst = 0x0000000000020000,
    //Load config from base class, not subclass.
    CPF_GlobalConfig = 0x0000000000040000,
    //Property is a component references.
    CPF_InstancedReference = 0x0000000000080000,
    //Property should always be reset to the default value during any type of duplication (copy/paste, binary duplication, etc.)
    CPF_DuplicateTransient = 0x0000000000200000,
    //Property should be serialized for save games, this is only checked for game-specific archives with ArIsSaveGame
    CPF_SaveGame = 0x0000000001000000,
    //Hide clear (and browse) button.
    CPF_NoClear = 0x0000000002000000,
    //Value is passed by reference; CPF_OutParam and CPF_Param should also be set.
    CPF_ReferenceParm = 0x0000000008000000,
    //MC Delegates only.  Property should be exposed for assigning in blueprint code
    CPF_BlueprintAssignable = 0x0000000010000000,
    //Property is deprecated.  Read it from an archive, but don't save it.
    CPF_Deprecated = 0x0000000020000000,
    //If this is set, then the property can be memcopied instead of CopyCompleteValue / CopySingleValue
    CPF_IsPlainOldData = 0x0000000040000000,
    //Not replicated. For non replicated properties in replicated structs
    CPF_RepSkip = 0x0000000080000000,
    //Notify actors when a property is replicated
    CPF_RepNotify = 0x0000000100000000,
    //interpolatable property for use with matinee
    CPF_Interp = 0x0000000200000000,
    //Property isn't transacted
    CPF_NonTransactional = 0x0000000400000000,
    //Property should only be loaded in the editor
    CPF_EditorOnly = 0x0000000800000000,
    //No destructor
    CPF_NoDestructor = 0x0000001000000000,
    //Only used for weak pointers, means the export type is autoweak
    CPF_AutoWeak = 0x0000004000000000,
    //Property contains component references.
    CPF_ContainsInstancedReference = 0x0000008000000000,
    //asset instances will add properties with this flag to the asset registry automatically
    CPF_AssetRegistrySearchable = 0x0000010000000000,
    //The property is visible by default in the editor details view
    CPF_SimpleDisplay = 0x0000020000000000,
    //The property is advanced and not visible by default in the editor details view
    CPF_AdvancedDisplay = 0x0000040000000000,
    //property is protected from the perspective of script
    CPF_Protected = 0x0000080000000000,
    //MC Delegates only.  Property should be exposed for calling in blueprint code
    CPF_BlueprintCallable = 0x0000100000000000,
    //MC Delegates only.  This delegate accepts (only in blueprint) only events with BlueprintAuthorityOnly.
    CPF_BlueprintAuthorityOnly = 0x0000200000000000,
    //Property shouldn't be exported to text format (e.g. copy/paste)
    CPF_TextExportTransient = 0x0000400000000000,
    //Property should only be copied in PIE
    CPF_NonPIEDuplicateTransient = 0x0000800000000000,
    //Property is exposed on spawn
    CPF_ExposeOnSpawn = 0x0001000000000000,
    //A object referenced by the property is duplicated like a component. (Each actor should have an own instance.)
    CPF_PersistentInstance = 0x0002000000000000,
    //Property was parsed as a wrapper class like TSubclassOf T, FScriptInterface etc., rather than a USomething*
    CPF_UObjectWrapper = 0x0004000000000000,
    //This property can generate a meaningful hash value.
    CPF_HasGetValueTypeHash = 0x0008000000000000,
    //Public native access specifier
    CPF_NativeAccessSpecifierPublic = 0x0010000000000000,
    //Protected native access specifier
    CPF_NativeAccessSpecifierProtected = 0x0020000000000000,
    //Private native access specifier
    CPF_NativeAccessSpecifierPrivate = 0x0040000000000000,
    //Property shouldn't be serialized, can still be exported to text
    CPF_SkipSerialization = 0x0080000000000000,
}

impl Default for EPropertyFlags {
    fn default() -> Self {
        Self::CPF_None
    }
}

#[derive(Debug)]
pub enum EClassFlags
{
    // No Flags
    CLASS_None = 0x00000000,
    // Class is abstract and can't be instantiated directly.
    CLASS_Abstract = 0x00000001,
    // Save object configuration only to Default INIs, never to local INIs. Must be combined with CLASS_Config
    CLASS_DefaultConfig = 0x00000002,
    // Load object configuration at construction time.
    CLASS_Config = 0x00000004,
    // This object type can't be saved; null it out at save time.
    CLASS_Transient = 0x00000008,
    // Successfully parsed.
    CLASS_Parsed = 0x00000010,
    // ???
    CLASS_MatchedSerializers = 0x00000020,
    // Indicates that the config settings for this class will be saved to Project/User*.ini (similar to CLASS_GlobalUserConfig)
    CLASS_ProjectUserConfig = 0x00000040,
    // Class is a native class - native interfaces will have CLASS_Native set, but not RF_MarkAsNative
    CLASS_Native = 0x00000080,
    // Don't export to C++ header.
    CLASS_NoExport = 0x00000100,
    // Do not allow users to create in the editor.
    CLASS_NotPlaceable = 0x00000200,
    // Handle object configuration on a per-object basis, rather than per-class.
    CLASS_PerObjectConfig = 0x00000400,
    // Whether SetUpRuntimeReplicationData still needs to be called for this class
    CLASS_ReplicationDataIsSetUp = 0x00000800u,
    // Class can be constructed from editinline New button.
    CLASS_EditInlineNew = 0x00001000,
    // Display properties in the editor without using categories.
    CLASS_CollapseCategories = 0x00002000,
    // Class is an interface
    CLASS_Interface = 0x00004000,
    // Do not export a constructor for this class, assuming it is in the cpptext
    CLASS_CustomConstructor = 0x00008000,
    // All properties and functions in this class are const and should be exported as const
    CLASS_Const = 0x00010000,
    // Class flag indicating the class is having its layout changed, and therefore is not ready for a CDO to be created
    CLASS_LayoutChanging = 0x00020000,
    // Indicates that the class was created from blueprint source material
    CLASS_CompiledFromBlueprint = 0x00040000,
    // Indicates that only the bare minimum bits of this class should be DLL exported/imported
    CLASS_MinimalAPI = 0x00080000,
    // Indicates this class must be DLL exported/imported (along with all of it's members)
    CLASS_RequiredAPI = 0x00100000,
    // Indicates that references to this class default to instanced. Used to be subclasses of UComponent, but now can be any UObject
    CLASS_DefaultToInstanced = 0x00200000,
    // Indicates that the parent token stream has been merged with ours.
    CLASS_TokenStreamAssembled = 0x00400000,
    // Class has component properties.
    CLASS_HasInstancedReference = 0x00800000,
    // Don't show this class in the editor class browser or edit inline new menus.
    CLASS_Hidden = 0x01000000,
    // Don't save objects of this class when serializing
    CLASS_Deprecated = 0x02000000,
    // Class not shown in editor drop down for class selection
    CLASS_HideDropDown = 0x04000000,
    // Class settings are saved to AppData/..../Blah.ini (as opposed to CLASS_DefaultConfig)
    CLASS_GlobalUserConfig = 0x08000000,
    // Class was declared directly in C++ and has no boilerplate generated by UnrealHeaderTool
    CLASS_Intrinsic = 0x10000000,
    // Class has already been constructed (maybe in a previous DLL version before hot-reload).
    CLASS_Constructed = 0x20000000,
    // Indicates that object configuration will not check against ini base/defaults when serialized
    CLASS_ConfigDoNotCheckDefaults = 0x40000000,
    // Class has been consigned to oblivion as part of a blueprint recompile, and a newer version currently exists.
    CLASS_NewerVersionExists = 0x80000000,
}

impl Default for EClassFlags {
    fn default() -> Self {
        Self::CLASS_None
    }
}

#[derive(Debug, IntoPrimitive)]
#[repr(i8)]
pub enum TextHistoryType {
    None = -1,
    Base = 0,
    NamedFormat,
    OrderedFormat,
    ArgumentFormat,
    AsNumber,
    AsPercent,
    AsCurrency,
    AsDate,
    AsTime,
    AsDateTime,
    Transform,
    StringTableEntry,
    TextGenerator
}

impl Default for TextHistoryType {
    fn default() -> Self {
        Self::None
    }
}