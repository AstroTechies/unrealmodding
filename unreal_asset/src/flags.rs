//! Various UAsset flags

use bitflags::bitflags;

bitflags! {
    /// Object instance flags
    pub struct EObjectFlags : u32
    {
        /// No flags
        const RF_NO_FLAGS = 0x00000000;
        /// Public
        const RF_PUBLIC = 0x00000001;
        /// Standalone
        const RF_STANDALONE = 0x00000002;
        /// Mark as native
        const RF_MARK_AS_NATIVE = 0x00000004;
        /// Transactional
        const RF_TRANSACTIONAL = 0x00000008;
        /// Class Default Object
        const RF_CLASS_DEFAULT_OBJECT = 0x00000010;
        /// Archetype Object
        const RF_ARCHETYPE_OBJECT = 0x00000020;
        /// Transient
        const RF_TRANSIENT = 0x00000040;
        /// Mark as root set
        const RF_MARK_AS_ROOT_SET = 0x00000080;
        /// Tag Garbage Temp
        const RF_TAG_GARBAGE_TEMP = 0x00000100;
        /// Needs initialization
        const RF_NEED_INITIALIZATION = 0x00000200;
        /// Needs load
        const RF_NEED_LOAD = 0x00000400;
        /// Keep object for cooker
        const RF_KEEP_FOR_COOKER = 0x00000800;
        /// Needed post load
        const RF_NEED_POST_LOAD = 0x00001000;
        /// Needed post load subobjects
        const RF_NEED_POST_LOAD_SUBOBJECTS = 0x00002000;
        /// A newer version of the object exists
        const RF_NEWER_VERSION_EXISTS = 0x00004000;
        /// Set when the object is starting to get destroyed
        const RF_BEGIN_DESTROYED = 0x00008000;
        /// Set when the object is finished being destroyed
        const RF_FINISH_DESTROYED = 0x00010000;
        /// Object is being regenerated
        const RF_BEING_REGENERATED = 0x00020000;
        /// Object is the default sub object
        const RF_DEFAULT_SUB_OBJECT = 0x00040000;
        /// Object was loaded
        const RF_WAS_LOADED = 0x00080000;
        /// Text export transient
        const RF_TEXT_EXPORT_TRANSIENT = 0x00100000;
        /// Load of this object was completed
        const RF_LOAD_COMPLETED = 0x00200000;
        /// Object is an inheritable component template
        const RF_INHERITABLE_COMPONENT_TEMPLATE = 0x00400000;
        /// Duplicate transient
        const RF_DUPLICATE_TRANSIENT = 0x00800000;
        /// Strong ref on frame
        const RF_STRONG_REF_ON_FRAME = 0x01000000;
        /// Non pie duplicate transient
        const RF_NON_P_I_E_DUPLICATE_TRANSIENT = 0x02000000;
        /// Object is dynamic
        const RF_DYNAMIC = 0x04000000;
        /// Object will be loaded
        const RF_WILL_BE_LOADED = 0x08000000;
        /// Object has an external package
        const RF_HAS_EXTERNAL_PACKAGE = 0x1000000;
    }

    /// Package flags
    pub struct EPackageFlags : u32
    {
        /// No flags
        const PKG_NONE = 0x00000000;
        /// Newly created package, not saved yet. In editor only.
        const PKG_NEWLY_CREATED = 0x00000001;
        /// Purely optional for clients.
        const PKG_CLIENT_OPTIONAL = 0x00000002;
        /// Only needed on the server side.
        const PKG_SERVER_SIDE_ONLY = 0x00000004;
        /// This package is from "compiled in" classes.
        const PKG_COMPILED_IN = 0x00000010;
        /// This package was loaded just for the purposes of diffing
        const PKG_FOR_DIFFING = 0x00000020;
        /// This is editor-only package (for example: editor module script package)
        const PKG_EDITOR_ONLY = 0x00000040;
        /// Developer module
        const PKG_DEVELOPER = 0x00000080;
        /// Loaded only in uncooked builds (i.e. runtime in editor)
        const PKG_UNCOOKED_ONLY = 0x00000100;
        /// Package is cooked
        const PKG_COOKED = 0x00000200;
        /// Package doesn't contain any asset object (although asset tags can be present)
        const PKG_CONTAINS_NO_ASSET = 0x00000400;
        /// Uses unversioned property serialization instead of versioned tagged property serialization
        const PKG_UNVERSIONED_PROPERTIES = 0x00002000;
        /// Contains map data (UObjects only referenced by a single ULevel) but is stored in a different package
        const PKG_CONTAINS_MAP_DATA = 0x00004000;
        /// package is currently being compiled
        const PKG_COMPILING = 0x00010000;
        /// Set if the package contains a ULevel/ UWorld object
        const PKG_CONTAINS_MAP = 0x00020000;
        /// ???
        const PKG_REQUIRES_LOCALIZATION_GATHER = 0x00040000;
        /// Set if the package was created for the purpose of PIE
        const PKG_PLAY_IN_EDITOR = 0x00100000;
        /// Package is allowed to contain UClass objects
        const PKG_CONTAINS_SCRIPT = 0x00200000;
        /// Editor should not export asset in this package
        const PKG_DISALLOW_EXPORT = 0x00400000;
        /// This package should resolve dynamic imports from its export at runtime.
        const PKG_DYNAMIC_IMPORTS = 0x10000000;
        /// This package contains elements that are runtime generated, and may not follow standard loading order rules
        const PKG_RUNTIME_GENERATED = 0x20000000;
        /// This package is reloading in the cooker, try to avoid getting data we will never need. We won't save this package.
        const PKG_RELOADING_FOR_COOKER = 0x40000000;
        /// Package has editor-only data filtered out
        const PKG_FILTER_EDITOR_ONLY = 0x80000000;
    }

    /// Property flags
    pub struct EPropertyFlags : u64
    {
        /// None
        const CPF_NONE = 0;

        /// Property is user-settable in the editor.
        const CPF_EDIT = 0x0000000000000001;
        /// This is a constant function parameter
        const CPF_CONST_PARM = 0x0000000000000002;
        /// This property can be read by blueprint code
        const CPF_BLUEPRINT_VISIBLE = 0x0000000000000004;
        /// Object can be exported with actor.
        const CPF_EXPORT_OBJECT = 0x0000000000000008;
        /// This property cannot be modified by blueprint code
        const CPF_BLUEPRINT_READ_ONLY = 0x0000000000000010;
        /// Property is relevant to network replication.
        const CPF_NET = 0x0000000000000020;
        /// Indicates that elements of an array can be modified, but its size cannot be changed.
        const CPF_EDIT_FIXED_SIZE = 0x0000000000000040;
        /// Function/When call parameter.
        const CPF_PARM = 0x0000000000000080;
        /// Value is copied out after function call.
        const CPF_OUT_PARM = 0x0000000000000100;
        /// memset is fine for construction
        const CPF_ZERO_CONSTRUCTOR = 0x0000000000000200;
        /// Return value.
        const CPF_RETURN_PARM = 0x0000000000000400;
        /// Disable editing of this property on an archetype/sub-blueprint
        const CPF_DISABLE_EDIT_ON_TEMPLATE = 0x0000000000000800;
        /// Property is transient: shouldn't be saved or loaded, except for Blueprint CDOs.
        const CPF_TRANSIENT = 0x0000000000002000;
        /// Property should be loaded/saved as permanent profile.
        const CPF_CONFIG = 0x0000000000004000;
        /// Disable editing on an instance of this class
        const CPF_DISABLE_EDIT_ON_INSTANCE = 0x0000000000010000;
        /// Property is uneditable in the editor.
        const CPF_EDIT_CONST = 0x0000000000020000;
        /// Load config from base class, not subclass.
        const CPF_GLOBAL_CONFIG = 0x0000000000040000;
        /// Property is a component references.
        const CPF_INSTANCED_REFERENCE = 0x0000000000080000;
        /// Property should always be reset to the default value during any type of duplication (copy/paste, binary duplication, etc.)
        const CPF_DUPLICATE_TRANSIENT = 0x0000000000200000;
        /// Property should be serialized for save games, this is only checked for game-specific archives with ArIsSaveGame
        const CPF_SAVE_GAME = 0x0000000001000000;
        /// Hide clear (and browse) button.
        const CPF_NO_CLEAR = 0x0000000002000000;
        /// Value is passed by reference; CPF_OutParam and CPF_Param should also be set.
        const CPF_REFERENCE_PARM = 0x0000000008000000;
        /// MC Delegates only.  Property should be exposed for assigning in blueprint code
        const CPF_BLUEPRINT_ASSIGNABLE = 0x0000000010000000;
        /// Property is deprecated.  Read it from an archive, but don't save it.
        const CPF_DEPRECATED = 0x0000000020000000;
        /// If this is set, then the property can be memcopied instead of CopyCompleteValue / CopySingleValue
        const CPF_IS_PLAIN_OLD_DATA = 0x0000000040000000;
        /// Not replicated. For non replicated properties in replicated structs
        const CPF_REP_SKIP = 0x0000000080000000;
        /// Notify actors when a property is replicated
        const CPF_REP_NOTIFY = 0x0000000100000000;
        /// interpolatable property for use with matinee
        const CPF_INTERP = 0x0000000200000000;
        /// Property isn't transacted
        const CPF_NON_TRANSACTIONAL = 0x0000000400000000;
        /// Property should only be loaded in the editor
        const CPF_EDITOR_ONLY = 0x0000000800000000;
        /// No destructor
        const CPF_NO_DESTRUCTOR = 0x0000001000000000;
        /// Only used for weak pointers, means the export type is autoweak
        const CPF_AUTO_WEAK = 0x0000004000000000;
        /// Property contains component references.
        const CPF_CONTAINS_INSTANCED_REFERENCE = 0x0000008000000000;
        /// asset instances will add properties with this flag to the asset registry automatically
        const CPF_ASSET_REGISTRY_SEARCHABLE = 0x0000010000000000;
        /// The property is visible by default in the editor details view
        const CPF_SIMPLE_DISPLAY = 0x0000020000000000;
        /// The property is advanced and not visible by default in the editor details view
        const CPF_ADVANCED_DISPLAY = 0x0000040000000000;
        /// property is protected from the perspective of script
        const CPF_PROTECTED = 0x0000080000000000;
        /// MC Delegates only.  Property should be exposed for calling in blueprint code
        const CPF_BLUEPRINT_CALLABLE = 0x0000100000000000;
        /// MC Delegates only.  This delegate accepts (only in blueprint) only events with BlueprintAuthorityOnly.
        const CPF_BLUEPRINT_AUTHORITY_ONLY = 0x0000200000000000;
        /// Property shouldn't be exported to text format (e.g. copy/paste)
        const CPF_TEXT_EXPORT_TRANSIENT = 0x0000400000000000;
        /// Property should only be copied in PIE
        const CPF_NON_P_I_E_DUPLICATE_TRANSIENT = 0x0000800000000000;
        /// Property is exposed on spawn
        const CPF_EXPOSE_ON_SPAWN = 0x0001000000000000;
        /// A object referenced by the property is duplicated like a component. (Each actor should have an own instance.)
        const CPF_PERSISTENT_INSTANCE = 0x0002000000000000;
        /// Property was parsed as a wrapper class like TSubclassOf T, FScriptInterface etc., rather than a USomething*
        const CPF_U_OBJECT_WRAPPER = 0x0004000000000000;
        /// This property can generate a meaningful hash value.
        const CPF_HAS_GET_VALUE_TYPE_HASH = 0x0008000000000000;
        /// Public native access specifier
        const CPF_NATIVE_ACCESS_SPECIFIER_PUBLIC = 0x0010000000000000;
        /// Protected native access specifier
        const CPF_NATIVE_ACCESS_SPECIFIER_PROTECTED = 0x0020000000000000;
        /// Private native access specifier
        const CPF_NATIVE_ACCESS_SPECIFIER_PRIVATE = 0x0040000000000000;
        /// Property shouldn't be serialized, can still be exported to text
        const CPF_SKIP_SERIALIZATION = 0x0080000000000000;
    }

    /// Class flags
    pub struct EClassFlags : u32
    {
        /// No Flags
        const CLASS_NONE = 0x00000000;
        /// Class is abstract and can't be instantiated directly.
        const CLASS_ABSTRACT = 0x00000001;
        /// Save object configuration only to Default INIs, never to local INIs. Must be combined with CLASS_Config
        const CLASS_DEFAULT_CONFIG = 0x00000002;
        /// Load object configuration at construction time.
        const CLASS_CONFIG = 0x00000004;
        /// This object type can't be saved; null it out at save time.
        const CLASS_TRANSIENT = 0x00000008;
        /// Successfully parsed.
        const CLASS_PARSED = 0x00000010;
        /// ???
        const CLASS_MATCHED_SERIALIZERS = 0x00000020;
        /// Indicates that the config settings for this class will be saved to Project/User*.ini (similar to CLASS_GlobalUserConfig)
        const CLASS_PROJECT_USER_CONFIG = 0x00000040;
        /// Class is a native class - native interfaces will have CLASS_Native set, but not RF_MarkAsNative
        const CLASS_NATIVE = 0x00000080;
        /// Don't export to C++ header.
        const CLASS_NO_EXPORT = 0x00000100;
        /// Do not allow users to create in the editor.
        const CLASS_NOT_PLACEABLE = 0x00000200;
        /// Handle object configuration on a per-object basis, rather than per-class.
        const CLASS_PER_OBJECT_CONFIG = 0x00000400;
        /// Whether SetUpRuntimeReplicationData still needs to be called for this class
        const CLASS_REPLICATION_DATA_IS_SET_UP = 0x00000800;
        /// Class can be constructed from editinline New button.
        const CLASS_EDIT_INLINE_NEW = 0x00001000;
        /// Display properties in the editor without using categories.
        const CLASS_COLLAPSE_CATEGORIES = 0x00002000;
        /// Class is an interface
        const CLASS_INTERFACE = 0x00004000;
        /// Do not export a constructor for this class, assuming it is in the cpptext
        const CLASS_CUSTOM_CONSTRUCTOR = 0x00008000;
        /// All properties and functions in this class are const and should be exported as const
        const CLASS_CONST = 0x00010000;
        /// Class flag indicating the class is having its layout changed, and therefore is not ready for a CDO to be created
        const CLASS_LAYOUT_CHANGING = 0x00020000;
        /// Indicates that the class was created from blueprint source material
        const CLASS_COMPILED_FROM_BLUEPRINT = 0x00040000;
        /// Indicates that only the bare minimum bits of this class should be DLL exported/imported
        const CLASS_MINIMAL_A_P_I = 0x00080000;
        /// Indicates this class must be DLL exported/imported (along with all of it's members)
        const CLASS_REQUIRED_A_P_I = 0x00100000;
        /// Indicates that references to this class default to instanced. Used to be subclasses of UComponent, but now can be any UObject
        const CLASS_DEFAULT_TO_INSTANCED = 0x00200000;
        /// Indicates that the parent token stream has been merged with ours.
        const CLASS_TOKEN_STREAM_ASSEMBLED = 0x00400000;
        /// Class has component properties.
        const CLASS_HAS_INSTANCED_REFERENCE = 0x00800000;
        /// Don't show this class in the editor class browser or edit inline new menus.
        const CLASS_HIDDEN = 0x01000000;
        /// Don't save objects of this class when serializing
        const CLASS_DEPRECATED = 0x02000000;
        /// Class not shown in editor drop down for class selection
        const CLASS_HIDE_DROP_DOWN = 0x04000000;
        /// Class settings are saved to AppData/..../Blah.ini (as opposed to CLASS_DefaultConfig)
        const CLASS_GLOBAL_USER_CONFIG = 0x08000000;
        /// Class was declared directly in C++ and has no boilerplate generated by UnrealHeaderTool
        const CLASS_INTRINSIC = 0x10000000;
        /// Class has already been constructed (maybe in a previous DLL version before hot-reload).
        const CLASS_CONSTRUCTED = 0x20000000;
        /// Indicates that object configuration will not check against ini base/defaults when serialized
        const CLASS_CONFIG_DO_NOT_CHECK_DEFAULTS = 0x40000000;
        /// Class has been consigned to oblivion as part of a blueprint recompile, and a newer version currently exists.
        const CLASS_NEWER_VERSION_EXISTS = 0x80000000;
    }

    /// Function flags
    pub struct EFunctionFlags : u32 {
        /// None
        const FUNC_NONE = 0x00000000;
        /// Final function
        const FUNC_FINAL = 0x00000001;
        /// Required api
        const FUNC_REQUIRED_API = 0x00000002;
        /// Function can only get executed if the executor has authority
        const FUNC_BLUEPRINT_AUTHORITY_ONLY = 0x00000004;
        /// Cosmetic
        const FUNC_BLUEPRINT_COSMETIC = 0x00000008;
        /// Networked function
        const FUNC_NET = 0x00000040;
        /// Networked reliable function
        const FUNC_NET_RELIABLE = 0x00000080;
        /// Net request function
        const FUNC_NET_REQUEST = 0x00000100;
        /// Exec
        const FUNC_EXEC = 0x00000200;
        /// Native function
        const FUNC_NATIVE = 0x00000400;
        /// Event function
        const FUNC_EVENT = 0x00000800;
        /// Net response
        const FUNC_NETRESPONSE = 0x00001000;
        /// Static function
        const FUNC_STATIC = 0x00002000;
        /// Net multicast
        const FUNC_NETMULTICAST = 0x00004000;
        /// UberGraph function
        const FUNC_UBERGRAPHFUNCTION = 0x00008000;
        /// MulticastDelegate function
        const FUNC_MULTICASTDELEGATE = 0x00010000;
        /// Public function
        const FUNC_PUBLIC = 0x00020000;
        /// Private function
        const FUNC_PRIVATE = 0x00040000;
        /// Protected function
        const FUNC_PROTECTED = 0x00080000;
        /// Delegate function
        const FUNC_DELEGATE = 0x00100000;
        /// Netserver
        const FUNC_NETSERVER = 0x00200000;
        /// Set if the function has out parameters
        const FUNC_HASOUTPARMS = 0x00400000;
        /// Set if the function has default parameters
        const FUNC_HASDEFAULTS = 0x00800000;
        /// Net client
        const FUNC_NETCLIENT = 0x01000000;
        /// DllImport function
        const FUNC_DLLIMPORT = 0x02000000;
        /// Function is blueprint callable UFUNCTION(BlueprintCallable)
        const FUNC_BLUEPRINTCALLABLE = 0x04000000;
        /// UFUNCTION(BlueprintEvent)
        const FUNC_BLUEPRINTEVENT = 0x08000000;
        /// UFUNCTION(BlueprintPure)
        const FUNC_BLUEPRINTPURE = 0x10000000;
        /// Function is editor only
        const FUNC_EDITORONLY = 0x20000000;
        /// Const function
        const FUNC_CONST = 0x40000000;
        /// Net validate
        const FUNC_NETVALIDATE = 0x80000000;
        /// All flags
        const FUNC_ALLFLAGS = 0xFFFFFFFF;
    }

    /// Asset registry dependency propety
    pub struct EDependencyProperty : u32 {
        /// None
        const NONE = 0;

        /// Package Dependencies
        const PACKAGE_MASK = 0x7;
        /// The target asset must be loaded before the source asset can finish loading. The lack of this property is known as a Soft dependency, and indicates only that the source asset expects the target asset to be loadable on demand.
        const HARD = 0x1;
        /// The target asset is needed in the game as well as the editor. The lack of this property is known as an EditorOnly dependency.
        const GAME = 0x2;
        /// Fields on the target asset are used in the transformation of the source asset during cooking in addition to being required in the game or editor. The lack of this property indicates that the target asset is required in game or editor, but is not required during cooking.
        const BUILD = 0x4;

        /// SearchableName Dependencies, None yet
        const SEARCHABLE_NAME_MASK = 0x0;

        /// ManageDependencies
        const MANAGE_MASK = 0x8;
        /// The target asset was specified explicitly as a managee by the source asset. Lack of this property is known as an indirect dependency; the target asset is reachable by following the transitive closure of Direct Manage Dependencies and Package dependencies from the source asset.
        const DIRECT = 0x8;

        /// PACKAGE_MASK | SEARCHABLE_NAME_MASK | MANAGE_MASK
        #[allow(clippy::identity_op)]
        const ALL_MASK = 0x7 | 0x0 | 0x8;
    }
}

impl Default for EObjectFlags {
    fn default() -> Self {
        Self {
            bits: EObjectFlags::RF_NO_FLAGS.bits(),
        }
    }
}
