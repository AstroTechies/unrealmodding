use bitflags::bitflags;

bitflags! {
    pub struct EObjectFlags : u32
    {
        const RF_NoFlags = 0x00000000;
        const RF_Public = 0x00000001;
        const RF_Standalone = 0x00000002;
        const RF_MarkAsNative = 0x00000004;
        const RF_Transactional = 0x00000008;
        const RF_ClassDefaultObject = 0x00000010;
        const RF_ArchetypeObject = 0x00000020;
        const RF_Transient = 0x00000040;
        const RF_MarkAsRootSet = 0x00000080;
        const RF_TagGarbageTemp = 0x00000100;
        const RF_NeedInitialization = 0x00000200;
        const RF_NeedLoad = 0x00000400;
        const RF_KeepForCooker = 0x00000800;
        const RF_NeedPostLoad = 0x00001000;
        const RF_NeedPostLoadSubobjects = 0x00002000;
        const RF_NewerVersionExists = 0x00004000;
        const RF_BeginDestroyed = 0x00008000;
        const RF_FinishDestroyed = 0x00010000;
        const RF_BeingRegenerated = 0x00020000;
        const RF_DefaultSubObject = 0x00040000;
        const RF_WasLoaded = 0x00080000;
        const RF_TextExportTransient = 0x00100000;
        const RF_LoadCompleted = 0x00200000;
        const RF_InheritableComponentTemplate = 0x00400000;
        const RF_DuplicateTransient = 0x00800000;
        const RF_StrongRefOnFrame = 0x01000000;
        const RF_NonPIEDuplicateTransient = 0x02000000;
        const RF_Dynamic = 0x04000000;
        const RF_WillBeLoaded = 0x08000000;
        const RF_HasExternalPackage = 0x1000000;
    }

    pub struct EPackageFlags : u32
    {
        //No flags
        const PKG_None = 0x00000000;
        //Newly created package, not saved yet. In editor only.
        const PKG_NewlyCreated = 0x00000001;
        //Purely optional for clients.
        const PKG_ClientOptional = 0x00000002;
        //Only needed on the server side.
        const PKG_ServerSideOnly = 0x00000004;
        //This package is from "compiled in" classes.
        const PKG_CompiledIn = 0x00000010;
        //This package was loaded just for the purposes of diffing
        const PKG_ForDiffing = 0x00000020;
        //This is editor-only package (for example: editor module script package)
        const PKG_EditorOnly = 0x00000040;
        //Developer module
        const PKG_Developer = 0x00000080;
        //Loaded only in uncooked builds (i.e. runtime in editor)
        const PKG_UncookedOnly = 0x00000100;
        //Package is cooked
        const PKG_Cooked = 0x00000200;
        //Package doesn't contain any asset object (although asset tags can be present)
        const PKG_ContainsNoAsset = 0x00000400;
        //Uses unversioned property serialization instead of versioned tagged property serialization
        const PKG_UnversionedProperties = 0x00002000;
        //Contains map data (UObjects only referenced by a single ULevel) but is stored in a different package
        const PKG_ContainsMapData = 0x00004000;
        //package is currently being compiled
        const PKG_Compiling = 0x00010000;
        //Set if the package contains a ULevel/ UWorld object
        const PKG_ContainsMap = 0x00020000;
        //???
        const PKG_RequiresLocalizationGather = 0x00040000;
        //Set if the package was created for the purpose of PIE
        const PKG_PlayInEditor = 0x00100000;
        //Package is allowed to contain UClass objects
        const PKG_ContainsScript = 0x00200000;
        //Editor should not export asset in this package
        const PKG_DisallowExport = 0x00400000;
        //This package should resolve dynamic imports from its export at runtime.
        const PKG_DynamicImports = 0x10000000;
        //This package contains elements that are runtime generated, and may not follow standard loading order rules
        const PKG_RuntimeGenerated = 0x20000000;
        //This package is reloading in the cooker, try to avoid getting data we will never need. We won't save this package.
        const PKG_ReloadingForCooker = 0x40000000;
        //Package has editor-only data filtered out
        const PKG_FilterEditorOnly = 0x80000000;
    }

    pub struct EPropertyFlags : u64
    {
        const CPF_None = 0;

        //Property is user-settable in the editor.
        const CPF_Edit = 0x0000000000000001;
        //This is a constant function parameter
        const CPF_ConstParm = 0x0000000000000002;
        //This property can be read by blueprint code
        const CPF_BlueprintVisible = 0x0000000000000004;
        //Object can be exported with actor.
        const CPF_ExportObject = 0x0000000000000008;
        //This property cannot be modified by blueprint code
        const CPF_BlueprintReadOnly = 0x0000000000000010;
        //Property is relevant to network replication.
        const CPF_Net = 0x0000000000000020;
        //Indicates that elements of an array can be modified, but its size cannot be changed.
        const CPF_EditFixedSize = 0x0000000000000040;
        //Function/When call parameter.
        const CPF_Parm = 0x0000000000000080;
        //Value is copied out after function call.
        const CPF_OutParm = 0x0000000000000100;
        //memset is fine for construction
        const CPF_ZeroConstructor = 0x0000000000000200;
        //Return value.
        const CPF_ReturnParm = 0x0000000000000400;
        //Disable editing of this property on an archetype/sub-blueprint
        const CPF_DisableEditOnTemplate = 0x0000000000000800;
        //Property is transient: shouldn't be saved or loaded, except for Blueprint CDOs.
        const CPF_Transient = 0x0000000000002000;
        //Property should be loaded/saved as permanent profile.
        const CPF_Config = 0x0000000000004000;
        //Disable editing on an instance of this class
        const CPF_DisableEditOnInstance = 0x0000000000010000;
        //Property is uneditable in the editor.
        const CPF_EditConst = 0x0000000000020000;
        //Load config from base class, not subclass.
        const CPF_GlobalConfig = 0x0000000000040000;
        //Property is a component references.
        const CPF_InstancedReference = 0x0000000000080000;
        //Property should always be reset to the default value during any type of duplication (copy/paste, binary duplication, etc.)
        const CPF_DuplicateTransient = 0x0000000000200000;
        //Property should be serialized for save games, this is only checked for game-specific archives with ArIsSaveGame
        const CPF_SaveGame = 0x0000000001000000;
        //Hide clear (and browse) button.
        const CPF_NoClear = 0x0000000002000000;
        //Value is passed by reference; CPF_OutParam and CPF_Param should also be set.
        const CPF_ReferenceParm = 0x0000000008000000;
        //MC Delegates only.  Property should be exposed for assigning in blueprint code
        const CPF_BlueprintAssignable = 0x0000000010000000;
        //Property is deprecated.  Read it from an archive, but don't save it.
        const CPF_Deprecated = 0x0000000020000000;
        //If this is set, then the property can be memcopied instead of CopyCompleteValue / CopySingleValue
        const CPF_IsPlainOldData = 0x0000000040000000;
        //Not replicated. For non replicated properties in replicated structs
        const CPF_RepSkip = 0x0000000080000000;
        //Notify actors when a property is replicated
        const CPF_RepNotify = 0x0000000100000000;
        //interpolatable property for use with matinee
        const CPF_Interp = 0x0000000200000000;
        //Property isn't transacted
        const CPF_NonTransactional = 0x0000000400000000;
        //Property should only be loaded in the editor
        const CPF_EditorOnly = 0x0000000800000000;
        //No destructor
        const CPF_NoDestructor = 0x0000001000000000;
        //Only used for weak pointers, means the export type is autoweak
        const CPF_AutoWeak = 0x0000004000000000;
        //Property contains component references.
        const CPF_ContainsInstancedReference = 0x0000008000000000;
        //asset instances will add properties with this flag to the asset registry automatically
        const CPF_AssetRegistrySearchable = 0x0000010000000000;
        //The property is visible by default in the editor details view
        const CPF_SimpleDisplay = 0x0000020000000000;
        //The property is advanced and not visible by default in the editor details view
        const CPF_AdvancedDisplay = 0x0000040000000000;
        //property is protected from the perspective of script
        const CPF_Protected = 0x0000080000000000;
        //MC Delegates only.  Property should be exposed for calling in blueprint code
        const CPF_BlueprintCallable = 0x0000100000000000;
        //MC Delegates only.  This delegate accepts (only in blueprint) only events with BlueprintAuthorityOnly.
        const CPF_BlueprintAuthorityOnly = 0x0000200000000000;
        //Property shouldn't be exported to text format (e.g. copy/paste)
        const CPF_TextExportTransient = 0x0000400000000000;
        //Property should only be copied in PIE
        const CPF_NonPIEDuplicateTransient = 0x0000800000000000;
        //Property is exposed on spawn
        const CPF_ExposeOnSpawn = 0x0001000000000000;
        //A object referenced by the property is duplicated like a component. (Each actor should have an own instance.)
        const CPF_PersistentInstance = 0x0002000000000000;
        //Property was parsed as a wrapper class like TSubclassOf T, FScriptInterface etc., rather than a USomething*
        const CPF_UObjectWrapper = 0x0004000000000000;
        //This property can generate a meaningful hash value.
        const CPF_HasGetValueTypeHash = 0x0008000000000000;
        //Public native access specifier
        const CPF_NativeAccessSpecifierPublic = 0x0010000000000000;
        //Protected native access specifier
        const CPF_NativeAccessSpecifierProtected = 0x0020000000000000;
        //Private native access specifier
        const CPF_NativeAccessSpecifierPrivate = 0x0040000000000000;
        //Property shouldn't be serialized, can still be exported to text
        const CPF_SkipSerialization = 0x0080000000000000;
    }

    pub struct EClassFlags : u32
    {
        // No Flags
        const CLASS_None = 0x00000000;
        // Class is abstract and can't be instantiated directly.
        const CLASS_Abstract = 0x00000001;
        // Save object configuration only to Default INIs, never to local INIs. Must be combined with CLASS_Config
        const CLASS_DefaultConfig = 0x00000002;
        // Load object configuration at construction time.
        const CLASS_Config = 0x00000004;
        // This object type can't be saved; null it out at save time.
        const CLASS_Transient = 0x00000008;
        // Successfully parsed.
        const CLASS_Parsed = 0x00000010;
        // ???
        const CLASS_MatchedSerializers = 0x00000020;
        // Indicates that the config settings for this class will be saved to Project/User*.ini (similar to CLASS_GlobalUserConfig)
        const CLASS_ProjectUserConfig = 0x00000040;
        // Class is a native class - native interfaces will have CLASS_Native set, but not RF_MarkAsNative
        const CLASS_Native = 0x00000080;
        // Don't export to C++ header.
        const CLASS_NoExport = 0x00000100;
        // Do not allow users to create in the editor.
        const CLASS_NotPlaceable = 0x00000200;
        // Handle object configuration on a per-object basis, rather than per-class.
        const CLASS_PerObjectConfig = 0x00000400;
        // Whether SetUpRuntimeReplicationData still needs to be called for this class
        const CLASS_ReplicationDataIsSetUp = 0x00000800;
        // Class can be constructed from editinline New button.
        const CLASS_EditInlineNew = 0x00001000;
        // Display properties in the editor without using categories.
        const CLASS_CollapseCategories = 0x00002000;
        // Class is an interface
        const CLASS_Interface = 0x00004000;
        // Do not export a constructor for this class, assuming it is in the cpptext
        const CLASS_CustomConstructor = 0x00008000;
        // All properties and functions in this class are const and should be exported as const
        const CLASS_Const = 0x00010000;
        // Class flag indicating the class is having its layout changed, and therefore is not ready for a CDO to be created
        const CLASS_LayoutChanging = 0x00020000;
        // Indicates that the class was created from blueprint source material
        const CLASS_CompiledFromBlueprint = 0x00040000;
        // Indicates that only the bare minimum bits of this class should be DLL exported/imported
        const CLASS_MinimalAPI = 0x00080000;
        // Indicates this class must be DLL exported/imported (along with all of it's members)
        const CLASS_RequiredAPI = 0x00100000;
        // Indicates that references to this class default to instanced. Used to be subclasses of UComponent, but now can be any UObject
        const CLASS_DefaultToInstanced = 0x00200000;
        // Indicates that the parent token stream has been merged with ours.
        const CLASS_TokenStreamAssembled = 0x00400000;
        // Class has component properties.
        const CLASS_HasInstancedReference = 0x00800000;
        // Don't show this class in the editor class browser or edit inline new menus.
        const CLASS_Hidden = 0x01000000;
        // Don't save objects of this class when serializing
        const CLASS_Deprecated = 0x02000000;
        // Class not shown in editor drop down for class selection
        const CLASS_HideDropDown = 0x04000000;
        // Class settings are saved to AppData/..../Blah.ini (as opposed to CLASS_DefaultConfig)
        const CLASS_GlobalUserConfig = 0x08000000;
        // Class was declared directly in C++ and has no boilerplate generated by UnrealHeaderTool
        const CLASS_Intrinsic = 0x10000000;
        // Class has already been constructed (maybe in a previous DLL version before hot-reload).
        const CLASS_Constructed = 0x20000000;
        // Indicates that object configuration will not check against ini base/defaults when serialized
        const CLASS_ConfigDoNotCheckDefaults = 0x40000000;
        // Class has been consigned to oblivion as part of a blueprint recompile, and a newer version currently exists.
        const CLASS_NewerVersionExists = 0x80000000;
    }
}