//! Kismet bytecode
use std::fmt::Debug;
use std::hash::Hash;
use std::mem::size_of;

use byteorder::LittleEndian;
use enum_dispatch::enum_dispatch;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ordered_float::OrderedFloat;

use crate::error::KismetError;
use crate::object_version::ObjectVersion;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::vector::{Transform, Vector, Vector4};
use crate::types::{FName, PackageIndex};
use crate::Error;

/// Kismet expression token
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EExprToken {
    /// A local variable.
    ExLocalVariable = 0x00,
    /// An object variable.
    ExInstanceVariable = 0x01,
    /// Default variable for a class context.
    ExDefaultVariable = 0x02,
    /// Return from function.
    ExReturn = 0x04,
    /// Goto a local address in code.
    ExJump = 0x06,
    /// Goto if not expression.
    ExJumpIfNot = 0x07,
    /// Assertion.
    ExAssert = 0x09,
    /// No operation.
    ExNothing = 0x0B,
    /// Assign an arbitrary size value to a variable.
    ExLet = 0x0F,
    /// Class default object context.
    ExClassContext = 0x12,
    /// Metaclass cast.
    ExMetaCast = 0x13,
    /// Let boolean variable.
    ExLetBool = 0x14,
    /// end of default value for optional function parameter
    ExEndParmValue = 0x15,
    /// End of function call parameters.
    ExEndFunctionParms = 0x16,
    /// Self object.
    ExSelf = 0x17,
    /// Skippable expression.
    ExSkip = 0x18,
    /// Call a function through an object context.
    ExContext = 0x19,
    /// Call a function through an object context (can fail silently if the context is NULL; only generated for functions that don't have output or return values).
    ExContextFailSilent = 0x1A,
    /// A function call with parameters.
    ExVirtualFunction = 0x1B,
    /// A prebound function call with parameters.
    ExFinalFunction = 0x1C,
    /// Int constant.
    ExIntConst = 0x1D,
    /// Floating point constant.
    ExFloatConst = 0x1E,
    /// String constant.
    ExStringConst = 0x1F,
    /// An object constant.
    ExObjectConst = 0x20,
    /// A name constant.
    ExNameConst = 0x21,
    /// A rotation constant.
    ExRotationConst = 0x22,
    /// A vector constant.
    ExVectorConst = 0x23,
    /// A byte constant.
    ExByteConst = 0x24,
    /// Zero.
    ExIntZero = 0x25,
    /// One.
    ExIntOne = 0x26,
    /// Bool True.
    ExTrue = 0x27,
    /// Bool False.
    ExFalse = 0x28,
    /// FText constant
    ExTextConst = 0x29,
    /// NoObject.
    ExNoObject = 0x2A,
    /// A transform constant
    ExTransformConst = 0x2B,
    /// Int constant that requires 1 byte.
    ExIntConstByte = 0x2C,
    /// A null interface (similar to ExNoObject, but for interfaces)
    ExNoInterface = 0x2D,
    /// Safe dynamic class casting.
    ExDynamicCast = 0x2E,
    /// An arbitrary UStruct constant
    ExStructConst = 0x2F,
    /// End of UStruct constant
    ExEndStructConst = 0x30,
    /// Set the value of arbitrary array
    ExSetArray = 0x31,
    /// End an array
    ExEndArray = 0x32,
    /// FProperty constant.
    ExPropertyConst = 0x33,
    /// Unicode string constant.
    ExUnicodeStringConst = 0x34,
    /// 64-bit integer constant.
    ExInt64Const = 0x35,
    /// 64-bit unsigned integer constant.
    ExUInt64Const = 0x36,
    /// A casting operator for primitives which reads the type as the subsequent byte
    ExPrimitiveCast = 0x38,
    /// Set the value of an arbitrary set
    ExSetSet = 0x39,
    /// End a set
    ExEndSet = 0x3A,
    /// Set the value of an arbitrary map
    ExSetMap = 0x3B,
    /// End a map
    ExEndMap = 0x3C,
    /// Set a value of an arbitrary const
    ExSetConst = 0x3D,
    /// End const
    ExEndSetConst = 0x3E,
    /// Create a constant map
    ExMapConst = 0x3F,
    /// End a constant map
    ExEndMapConst = 0x40,
    /// Context expression to address a property within a struct
    ExStructMemberContext = 0x42,
    /// Assignment to a multi-cast delegate
    ExLetMulticastDelegate = 0x43,
    /// Assignment to a delegate
    ExLetDelegate = 0x44,
    /// Special instructions to quickly call a virtual function that we know is going to run only locally
    ExLocalVirtualFunction = 0x45,
    /// Special instructions to quickly call a final function that we know is going to run only locally
    ExLocalFinalFunction = 0x46,
    /// local out (pass by reference) function parameter
    ExLocalOutVariable = 0x48,
    /// Deprecated operation
    ExDeprecatedOp4A = 0x4A,
    /// const reference to a delegate or normal function object
    ExInstanceDelegate = 0x4B,
    /// push an address on to the execution flow stack for future execution when a ExPopExecutionFlow is executed. Execution continues on normally and doesn't change to the pushed address.
    ExPushExecutionFlow = 0x4C,
    /// continue execution at the last address previously pushed onto the execution flow stack.
    ExPopExecutionFlow = 0x4D,
    /// Goto a local address in code, specified by an integer value.
    ExComputedJump = 0x4E,
    /// continue execution at the last address previously pushed onto the execution flow stack, if the condition is not true.
    ExPopExecutionFlowIfNot = 0x4F,
    /// Breakpoint. Only observed in the editor, otherwise it behaves like ExNothing.
    ExBreakpoint = 0x50,
    /// Call a function through a native interface variable
    ExInterfaceContext = 0x51,
    /// Converting an object reference to native interface variable
    ExObjToInterfaceCast = 0x52,
    /// Last byte in script code
    ExEndOfScript = 0x53,
    /// Converting an interface variable reference to native interface variable
    ExCrossInterfaceCast = 0x54,
    /// Converting an interface variable reference to an object
    ExInterfaceToObjCast = 0x55,
    /// Trace point.  Only observed in the editor, otherwise it behaves like ExNothing.
    ExWireTracepoint = 0x5A,
    /// A CodeSizeSkipOffset constant
    ExSkipOffsetConst = 0x5B,
    /// Adds a delegate to a multicast delegate's targets
    ExAddMulticastDelegate = 0x5C,
    /// Clears all delegates in a multicast target
    ExClearMulticastDelegate = 0x5D,
    /// Trace point.  Only observed in the editor, otherwise it behaves like ExNothing.
    ExTracepoint = 0x5E,
    /// assign to any object ref pointer
    ExLetObj = 0x5F,
    /// assign to a weak object pointer
    ExLetWeakObjPtr = 0x60,
    /// bind object and name to delegate
    ExBindDelegate = 0x61,
    /// Remove a delegate from a multicast delegate's targets
    ExRemoveMulticastDelegate = 0x62,
    /// Call multicast delegate
    ExCallMulticastDelegate = 0x63,
    /// Let value on persistent frame
    ExLetValueOnPersistentFrame = 0x64,
    /// Declare an array const
    ExArrayConst = 0x65,
    /// End an array const
    ExEndArrayConst = 0x66,
    /// Declare a soft object const
    ExSoftObjectConst = 0x67,
    /// static pure function from on local call space
    ExCallMath = 0x68,
    /// Switch value
    ExSwitchValue = 0x69,
    /// Instrumentation event
    ExInstrumentationEvent = 0x6A,
    /// Get array by ref
    ExArrayGetByRef = 0x6B,
    /// Sparse data variable
    ExClassSparseDataVariable = 0x6C,
    /// Decclare a field path const
    ExFieldPathConst = 0x6D,
    /// Max
    ExMax = 0xff,
}

/// Kismet cast token
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ECastToken {
    /// Cast object to interface
    ObjectToInterface = 0x46,
    /// Cast object to bool
    ObjectToBool = 0x47,
    /// Cast interface to bool
    InterfaceToBool = 0x49,
    /// Max
    Max = 0xFF,
}

/// Kismet instrumentation type
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EScriptInstrumentationType {
    /// Class
    Class = 0,
    /// Class scope
    ClassScope,
    /// Instance
    Instance,
    /// Event
    Event,
    /// Inline event
    InlineEvent,
    /// Resume event
    ResumeEvent,
    /// Pure node entry
    PureNodeEntry,
    /// Node debug site
    NodeDebugSite,
    /// Node entry
    NodeEntry,
    /// Node exit
    NodeExit,
    /// Push state
    PushState,
    /// Restore state
    RestoreState,
    /// Reset state
    ResetState,
    /// Suspend state
    SuspendState,
    /// Pop state
    PopState,
    /// Tunnel end of thread
    TunnelEndOfThread,
    /// Stop
    Stop,
}

/// Kismet text literal type
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EBlueprintTextLiteralType {
    /// Text is an empty string. The bytecode contains no strings, and you should use FText::GetEmpty() to initialize the FText instance.
    Empty,
    /// Text is localized. The bytecode will contain three strings - source, key, and namespace - and should be loaded via FInternationalization
    LocalizedText,
    /// Text is culture invariant. The bytecode will contain one string, and you should use FText::AsCultureInvariant to initialize the FText instance.
    InvariantText,
    /// Text is a literal FString. The bytecode will contain one string, and you should use FText::FromString to initialize the FText instance.
    LiteralString,
    /// Text is from a string table. The bytecode will contain an object pointer (not used) and two strings - the table ID, and key - and should be found via FText::FromStringTable
    StringTableEntry,
}

/// Kismet field path
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FieldPath {
    /// Path
    pub path: Vec<FName>,
    /// Path owner
    pub resolved_owner: PackageIndex,
}

impl FieldPath {
    /// Create a new `FieldPath` instance
    pub fn new(path: Vec<FName>, resolved_owner: PackageIndex) -> Self {
        FieldPath {
            path,
            resolved_owner,
        }
    }
}

/// Read a UTF-8 kismet string
fn read_kismet_string<Reader: AssetReader>(asset: &mut Reader) -> Result<String, Error> {
    let mut data = Vec::new();
    loop {
        let read = asset.read_u8()?;
        if read == 0 {
            break;
        }
        data.push(read);
    }
    Ok(String::from_utf8(data)?)
}

/// Read a UTF-16 kismet string
fn read_kismet_unicode_string<Reader: AssetReader>(asset: &mut Reader) -> Result<String, Error> {
    let mut data = Vec::new();
    loop {
        let b1 = asset.read_u8()?;
        let b2 = asset.read_u8()?;
        if b1 == 0 && b2 == 0 {
            break;
        }
        data.push(((b2 as u16) << 8) | b1 as u16)
    }
    Ok(String::from_utf16(&data)?)
}

/// Write a UTF-8 kismet string
fn write_kismet_string<Writer: AssetWriter>(
    string: &str,
    asset: &mut Writer,
) -> Result<usize, Error> {
    let begin = asset.position();
    asset.write_all(string.as_bytes())?;
    asset.write_all(&[0u8; 1])?;
    Ok((asset.position() - begin) as usize)
}

/// Write a UTF-16 kismet string
fn write_kismet_unicode_string<Writer: AssetWriter>(
    string: &str,
    asset: &mut Writer,
) -> Result<usize, Error> {
    let begin = asset.position();

    let utf16 = string.encode_utf16().collect::<Vec<_>>();
    // this is safe because we know that string is utf16 and therefore can easily be aligned to u8
    // this is also faster than alternatives without unsafe block
    let (_, aligned, _) = unsafe { utf16.align_to::<u8>() };

    asset.write_all(aligned)?;
    asset.write_all(&[0u8; 2])?;

    Ok((asset.position() - begin) as usize)
}

macro_rules! declare_expression {
    (
        $name:ident,
        $(
            $(#[$inner:ident $($args:tt)*])*
            $v:ident: $t:ty
        ),*
    ) => {
        /// $name
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            /// Kismet token
            pub token: EExprToken,
            $(
                $(#[$inner $($args)*])*
                pub $v: $t,
            )*
        }

        impl KismetExpressionEnumEqTrait for $name {
            fn enum_eq(&self, token: &EExprToken) -> bool { self.token == *token }
        }

        impl KismetExpressionDataTrait for $name {
            fn get_token(&self) -> EExprToken { self.token }
        }
    }
}

macro_rules! implement_expression {
    (
        $(
            $(#[$inner:ident $($args:tt)*])*
            $name:ident
        ),*
    ) => {
        $(
            $(#[$inner $($args)*])*
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct $name {
                /// Kismet token
                pub token: EExprToken
            }

            impl KismetExpressionTrait for $name {
                fn write<Writer: AssetWriter>(&self, _asset: &mut Writer) -> Result<usize, Error> {
                    Ok(0)
                }
            }

            impl KismetExpressionEnumEqTrait for $name {
                fn enum_eq(&self, token: &EExprToken) -> bool { self.token == *token }
            }

            impl KismetExpressionDataTrait for $name {
                fn get_token(&self) -> EExprToken { self.token }
            }

            impl $name {
                /// Read `$name` from an asset
                pub fn new<Reader: AssetReader>(_asset: &mut Reader) -> Result<Self, Error> {
                    Ok($name {
                        token: EExprToken::$name
                    })
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    $name { token: EExprToken::$name }
                }
            }
        )*
    }
}

macro_rules! implement_value_expression {
    ($name:ident, $param:ty, $read_func:ident, $write_func:ident) => {
        declare_expression!(
            $name,
            /// Value
            value: $param
        );
        impl $name {
            /// Read `$name` from an asset
            pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($name {
                    token: EExprToken::$name,
                    value: asset.$read_func()?,
                })
            }
        }

        impl KismetExpressionTrait for $name {
            fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
                asset.$write_func(self.value)?;
                Ok(size_of::<$param>())
            }
        }
    };

    ($name:ident, $param:ty, $read_func:ident, $write_func:ident, $endianness:ident) => {
        declare_expression!(
            $name,
            /// Value
            value: $param
        );
        impl $name {
            /// Read `$name` from an asset
            pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
                Ok($name {
                    token: EExprToken::$name,
                    value: asset.$read_func::<$endianness>()?,
                })
            }
        }

        impl KismetExpressionTrait for $name {
            fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
                asset.$write_func::<$endianness>(self.value)?;
                Ok(size_of::<$param>())
            }
        }
    };
}

/// Kismet script text
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FScriptText {
    /// Literal type
    text_literal_type: EBlueprintTextLiteralType,
    /// Localized source
    localized_source: Option<KismetExpression>,
    /// Localized key
    localized_key: Option<KismetExpression>,
    /// Localized namespace
    localized_namespace: Option<KismetExpression>,
    /// Invariant literal string
    invariant_literal_string: Option<KismetExpression>,
    /// Literal string
    literal_string: Option<KismetExpression>,
    /// String table asset this text is localized from
    string_table_asset: Option<PackageIndex>,
    /// String table id in the string table asset
    string_table_id: Option<KismetExpression>,
    /// String table key in the string table asset
    string_table_key: Option<KismetExpression>,
}

impl FScriptText {
    /// Read a `FScriptText` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let text_literal_type: EBlueprintTextLiteralType = asset.read_u8()?.try_into()?;
        let (
            mut localized_source,
            mut localized_key,
            mut localized_namespace,
            mut invariant_literal_string,
            mut literal_string,
            mut string_table_asset,
            mut string_table_id,
            mut string_table_key,
        ) = (None, None, None, None, None, None, None, None);

        match text_literal_type {
            EBlueprintTextLiteralType::LocalizedText => {
                localized_source = Some(KismetExpression::new(asset)?);
                localized_key = Some(KismetExpression::new(asset)?);
                localized_namespace = Some(KismetExpression::new(asset)?);
            }
            EBlueprintTextLiteralType::InvariantText => {
                invariant_literal_string = Some(KismetExpression::new(asset)?);
            }
            EBlueprintTextLiteralType::LiteralString => {
                literal_string = Some(KismetExpression::new(asset)?);
            }
            EBlueprintTextLiteralType::StringTableEntry => {
                string_table_asset = Some(PackageIndex::new(asset.read_i32::<LittleEndian>()?));
                string_table_id = Some(KismetExpression::new(asset)?);
                string_table_key = Some(KismetExpression::new(asset)?);
            }
            _ => {}
        };

        Ok(FScriptText {
            text_literal_type,
            localized_source,
            localized_key,
            localized_namespace,
            invariant_literal_string,
            literal_string,
            string_table_asset,
            string_table_id,
            string_table_key,
        })
    }

    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u8>();
        asset.write_u8(self.text_literal_type.into())?;
        match self.text_literal_type {
            EBlueprintTextLiteralType::Empty => {}
            EBlueprintTextLiteralType::LocalizedText => {
                offset += KismetExpression::write(
                    self.localized_source.as_ref().ok_or_else(|| {
                        Error::no_data(
                            "text_literal_type is LocalizedText but localized_source is None"
                                .to_string(),
                        )
                    })?,
                    asset,
                )?;
                offset += KismetExpression::write(
                    self.localized_key.as_ref().ok_or_else(|| {
                        Error::no_data(
                            "text_literal_type is LocalizedText but localized_key is None"
                                .to_string(),
                        )
                    })?,
                    asset,
                )?;
                offset += KismetExpression::write(
                    self.localized_namespace.as_ref().ok_or_else(|| {
                        Error::no_data(
                            "text_literal_type is LocalizedText but localized_namespace is None"
                                .to_string(),
                        )
                    })?,
                    asset,
                )?;
            }
            EBlueprintTextLiteralType::InvariantText => {
                offset += KismetExpression::write(
                    self.invariant_literal_string.as_ref().ok_or_else(|| {
                        Error::no_data(
                        "text_literal_type is InvariantText but invariant_literal_string is None"
                            .to_string(),
                    )
                    })?,
                    asset,
                )?;
            }
            EBlueprintTextLiteralType::LiteralString => {
                offset += KismetExpression::write(
                    self.literal_string.as_ref().ok_or_else(|| {
                        Error::no_data(
                            "text_literal_type is LiteralString but literal_string is None"
                                .to_string(),
                        )
                    })?,
                    asset,
                )?;
            }
            EBlueprintTextLiteralType::StringTableEntry => {
                asset.write_i32::<LittleEndian>(
                    self.string_table_asset.map(|e| e.index).ok_or_else(|| {
                        Error::no_data(
                            "text_literal_type is StringTableEntry but string_table_asset is None"
                                .to_string(),
                        )
                    })?,
                )?;
                offset += size_of::<u64>();
                offset += KismetExpression::write(
                    self.string_table_id.as_ref().ok_or_else(|| {
                        Error::no_data(
                            "text_literal_type is StringTalbleEntry but string_table_id is None"
                                .to_string(),
                        )
                    })?,
                    asset,
                )?;
                offset += KismetExpression::write(
                    self.string_table_key.as_ref().ok_or_else(|| {
                        Error::no_data(
                            "text_literal_type is StringTableEntry but string_table_key is None"
                                .to_string(),
                        )
                    })?,
                    asset,
                )?;
            }
        }
        Ok(offset)
    }
}

/// Represents a Kismet bytecode pointer to an FProperty or FField.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct KismetPropertyPointer {
    /// Pointer serialized as PackageIndex. Used in versions older than [`KismetPropertyPointer::XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION`]
    pub old: Option<PackageIndex>,
    /// Pointer serialized as an FFieldPath. Used in versions newer than [`KismetPropertyPointer::XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION`]
    pub new: Option<FieldPath>,
}

impl KismetPropertyPointer {
    const XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION: ObjectVersion =
        ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER;

    /// Create an old `KismetPropertyPointer` for an object version smaller than `KismetPropertyPointer::XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION`
    pub fn from_old(old: PackageIndex) -> Self {
        KismetPropertyPointer {
            old: Some(old),
            new: None,
        }
    }

    /// Create a new `KismetPropertyPointer` for an object version >=`KismetPropertyPointer::XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION`
    pub fn from_new(new: FieldPath) -> Self {
        KismetPropertyPointer {
            old: None,
            new: Some(new),
        }
    }

    /// Read a `KismetPropertyPointer` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        if asset.get_object_version()
            >= KismetPropertyPointer::XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION
        {
            let num_entries = asset.read_i32::<LittleEndian>()?;
            let mut names = Vec::with_capacity(num_entries as usize);
            for _i in 0..num_entries as usize {
                names.push(asset.read_fname()?);
            }
            let owner = PackageIndex::new(asset.read_i32::<LittleEndian>()?);
            Ok(KismetPropertyPointer::from_new(FieldPath::new(
                names, owner,
            )))
        } else {
            Ok(KismetPropertyPointer::from_old(PackageIndex::new(
                asset.read_i32::<LittleEndian>()?,
            )))
        }
    }

    /// Write a `KismetPropertyPointer` to an asset
    pub fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        if asset.get_object_version()
            >= KismetPropertyPointer::XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION
        {
            let new = self.new.as_ref().ok_or_else(|| {
                Error::no_data(
                    "engine_version >= UE4_ADDED_PACKAGE_OWNER but new is None".to_string(),
                )
            })?;
            asset.write_i32::<LittleEndian>(new.path.len() as i32)?;
            for entry in &new.path {
                asset.write_fname(entry)?;
            }
            asset.write_i32::<LittleEndian>(new.resolved_owner.index)?;
        } else {
            asset.write_i32::<LittleEndian>(self.old.map(|e| e.index).ok_or_else(|| {
                Error::no_data(
                    "engine_version < UE4_ADDED_PAFCKAGE_OWNER but old is None".to_string(),
                )
            })?)?;
        }
        Ok(size_of::<u64>())
    }
}

/// Kismet switch case
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KismetSwitchCase {
    /// Case value index
    pub case_index_value_term: KismetExpression,
    /// Next offset
    pub next_offset: u32,
    /// Case value
    pub case_term: KismetExpression,
}

impl KismetSwitchCase {
    /// Create a new `KismetSwitchCase` instance
    pub fn new(
        case_index_value_term: KismetExpression,
        next_offset: u32,
        case_term: KismetExpression,
    ) -> Self {
        KismetSwitchCase {
            case_index_value_term,
            next_offset,
            case_term,
        }
    }
}

/// This must be implemented for all KismetExpressions
#[enum_dispatch]
pub trait KismetExpressionTrait: Debug + Clone + PartialEq + Eq + Hash {
    /// Write a `KismetExpression` to an asset
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error>;
}

/// Allows for getting a token from a KismetExpression
#[enum_dispatch]
pub trait KismetExpressionDataTrait {
    /// Get a `KismetExpression`'s kismet token
    fn get_token(&self) -> EExprToken;
}

/// Allows for comparing two KismetExpressions based on their token
#[enum_dispatch]
pub trait KismetExpressionEnumEqTrait {
    /// Compare two `KismetExpression`s based on their token
    fn enum_eq(&self, token: &EExprToken) -> bool;
}

/// Kismet expression
#[enum_dispatch(
    KismetExpressionTrait,
    KismetExpressionEnumEqTrait,
    KismetExpressionDataTrait
)]
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum KismetExpression {
    /// A local variable.
    ExLocalVariable,
    /// An object variable.
    ExInstanceVariable,
    /// Default variable for a class context.
    ExDefaultVariable,
    /// Return from function.
    ExReturn,
    /// Goto a local address in code.
    ExJump,
    /// Goto if not expression.
    ExJumpIfNot,
    /// Assertion.
    ExAssert,
    /// No operation.
    ExNothing,
    /// Assign an arbitrary size value to a variable.
    ExLet,
    /// Class default object context.
    ExClassContext,
    /// Metaclass cast.
    ExMetaCast,
    /// Let boolean variable.
    ExLetBool,
    /// end of default value for optional function parameter
    ExEndParmValue,
    /// End of function call parameters.
    ExEndFunctionParms,
    /// Self object.
    ExSelf,
    /// Skippable expression.
    ExSkip,
    /// Call a function through an object context.
    ExContext,
    /// Call a function through an object context (can fail silently if the context is NULL; only generated for functions that don't have output or return values).
    ExContextFailSilent,
    /// A function call with parameters.
    ExVirtualFunction,
    /// A prebound function call with parameters.
    ExFinalFunction,
    /// Int constant.
    ExIntConst,
    /// Floating point constant.
    ExFloatConst,
    /// String constant.
    ExStringConst,
    /// An object constant.
    ExObjectConst,
    /// A name constant.
    ExNameConst,
    /// A rotation constant.
    ExRotationConst,
    /// A vector constant.
    ExVectorConst,
    /// A byte constant.
    ExByteConst,
    /// Zero.
    ExIntZero,
    /// One.
    ExIntOne,
    /// Bool True.
    ExTrue,
    /// Bool False.
    ExFalse,
    /// FText constant
    ExTextConst,
    /// NoObject.
    ExNoObject,
    /// A transform constant
    ExTransformConst,
    /// Int constant that requires 1 byte.
    ExIntConstByte,
    /// A null interface (similar to ExNoObject, but for interfaces)
    ExNoInterface,
    /// Safe dynamic class casting.
    ExDynamicCast,
    /// An arbitrary UStruct constant
    ExStructConst,
    /// End of UStruct constant
    ExEndStructConst,
    /// Set the value of arbitrary array
    ExSetArray,
    /// End an array
    ExEndArray,
    /// FProperty constant.
    ExPropertyConst,
    /// Unicode string constant.
    ExUnicodeStringConst,
    /// 64-bit integer constant.
    ExInt64Const,
    /// 64-bit unsigned integer constant.
    ExUInt64Const,
    /// A casting operator for primitives which reads the type as the subsequent byte
    ExPrimitiveCast,
    /// Set the value of an arbitrary set
    ExSetSet,
    /// End a set
    ExEndSet,
    /// Set the value of an arbitrary map
    ExSetMap,
    /// End a map
    ExEndMap,
    /// Set a value of an arbitrary const
    ExSetConst,
    /// End const
    ExEndSetConst,
    /// Create a constant map
    ExMapConst,
    /// End a constant map
    ExEndMapConst,
    /// Context expression to address a property within a struct
    ExStructMemberContext,
    /// Assignment to a multi-cast delegate
    ExLetMulticastDelegate,
    /// Assignment to a delegate
    ExLetDelegate,
    /// Special instructions to quickly call a virtual function that we know is going to run only locally
    ExLocalVirtualFunction,
    /// Special instructions to quickly call a final function that we know is going to run only locally
    ExLocalFinalFunction,
    /// local out (pass by reference) function parameter
    ExLocalOutVariable,
    /// Deprecated operation
    ExDeprecatedOp4A,
    /// const reference to a delegate or normal function object
    ExInstanceDelegate,
    /// push an address on to the execution flow stack for future execution when a ExPopExecutionFlow is executed. Execution continues on normally and doesn't change to the pushed address.
    ExPushExecutionFlow,
    /// continue execution at the last address previously pushed onto the execution flow stack.
    ExPopExecutionFlow,
    /// Goto a local address in code, specified by an integer value.
    ExComputedJump,
    /// continue execution at the last address previously pushed onto the execution flow stack, if the condition is not true.
    ExPopExecutionFlowIfNot,
    /// Breakpoint. Only observed in the editor, otherwise it behaves like ExNothing.
    ExBreakpoint,
    /// Call a function through a native interface variable
    ExInterfaceContext,
    /// Converting an object reference to native interface variable
    ExObjToInterfaceCast,
    /// Last byte in script code
    ExEndOfScript,
    /// Converting an interface variable reference to native interface variable
    ExCrossInterfaceCast,
    /// Converting an interface variable reference to an object
    ExInterfaceToObjCast,
    /// Trace point.  Only observed in the editor, otherwise it behaves like ExNothing.
    ExWireTracepoint,
    /// A CodeSizeSkipOffset constant
    ExSkipOffsetConst,
    /// Adds a delegate to a multicast delegate's targets
    ExAddMulticastDelegate,
    /// Clears all delegates in a multicast target
    ExClearMulticastDelegate,
    /// Trace point.  Only observed in the editor, otherwise it behaves like ExNothing.
    ExTracepoint,
    /// assign to any object ref pointer
    ExLetObj,
    /// assign to a weak object pointer
    ExLetWeakObjPtr,
    /// bind object and name to delegate
    ExBindDelegate,
    /// Remove a delegate from a multicast delegate's targets
    ExRemoveMulticastDelegate,
    /// Call multicast delegate
    ExCallMulticastDelegate,
    /// Let value on persistent frame
    ExLetValueOnPersistentFrame,
    /// Declare an array const
    ExArrayConst,
    /// End an array const
    ExEndArrayConst,
    /// Declare a soft object const
    ExSoftObjectConst,
    /// static pure function from on local call space
    ExCallMath,
    /// Switch value
    ExSwitchValue,
    /// Instrumentation event
    ExInstrumentationEvent,
    /// Get array by ref
    ExArrayGetByRef,
    /// Sparse data variable
    ExClassSparseDataVariable,
    /// Decclare a field path const
    ExFieldPathConst,
}

impl Eq for KismetExpression {}

impl KismetExpression {
    /// Read a `KismetExpression` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let token: EExprToken = asset.read_u8()?.try_into()?;
        let expr: Result<Self, Error> = match token {
            EExprToken::ExLocalVariable => Ok(ExLocalVariable::new(asset)?.into()),
            EExprToken::ExInstanceVariable => Ok(ExInstanceVariable::new(asset)?.into()),
            EExprToken::ExDefaultVariable => Ok(ExDefaultVariable::new(asset)?.into()),
            EExprToken::ExReturn => Ok(ExReturn::new(asset)?.into()),
            EExprToken::ExJump => Ok(ExJump::new(asset)?.into()),
            EExprToken::ExJumpIfNot => Ok(ExJumpIfNot::new(asset)?.into()),
            EExprToken::ExAssert => Ok(ExAssert::new(asset)?.into()),
            EExprToken::ExNothing => Ok(ExNothing::new(asset)?.into()),
            EExprToken::ExLet => Ok(ExLet::new(asset)?.into()),
            EExprToken::ExClassContext => Ok(ExClassContext::new(asset)?.into()),
            EExprToken::ExMetaCast => Ok(ExMetaCast::new(asset)?.into()),
            EExprToken::ExLetBool => Ok(ExLetBool::new(asset)?.into()),
            EExprToken::ExEndParmValue => Ok(ExEndParmValue::new(asset)?.into()),
            EExprToken::ExEndFunctionParms => Ok(ExEndFunctionParms::new(asset)?.into()),
            EExprToken::ExSelf => Ok(ExSelf::new(asset)?.into()),
            EExprToken::ExSkip => Ok(ExSkip::new(asset)?.into()),
            EExprToken::ExContext => Ok(ExContext::new(asset)?.into()),
            EExprToken::ExContextFailSilent => Ok(ExContextFailSilent::new(asset)?.into()),
            EExprToken::ExVirtualFunction => Ok(ExVirtualFunction::new(asset)?.into()),
            EExprToken::ExFinalFunction => Ok(ExFinalFunction::new(asset)?.into()),
            EExprToken::ExIntConst => Ok(ExIntConst::new(asset)?.into()),
            EExprToken::ExFloatConst => Ok(ExFloatConst::new(asset)?.into()),
            EExprToken::ExStringConst => Ok(ExStringConst::new(asset)?.into()),
            EExprToken::ExObjectConst => Ok(ExObjectConst::new(asset)?.into()),
            EExprToken::ExNameConst => Ok(ExNameConst::new(asset)?.into()),
            EExprToken::ExRotationConst => Ok(ExRotationConst::new(asset)?.into()),
            EExprToken::ExVectorConst => Ok(ExVectorConst::new(asset)?.into()),
            EExprToken::ExByteConst => Ok(ExByteConst::new(asset)?.into()),
            EExprToken::ExIntZero => Ok(ExIntZero::new(asset)?.into()),
            EExprToken::ExIntOne => Ok(ExIntOne::new(asset)?.into()),
            EExprToken::ExTrue => Ok(ExTrue::new(asset)?.into()),
            EExprToken::ExFalse => Ok(ExFalse::new(asset)?.into()),
            EExprToken::ExTextConst => Ok(ExTextConst::new(asset)?.into()),
            EExprToken::ExNoObject => Ok(ExNoObject::new(asset)?.into()),
            EExprToken::ExTransformConst => Ok(ExTransformConst::new(asset)?.into()),
            EExprToken::ExIntConstByte => Ok(ExIntConstByte::new(asset)?.into()),
            EExprToken::ExNoInterface => Ok(ExNoInterface::new(asset)?.into()),
            EExprToken::ExDynamicCast => Ok(ExDynamicCast::new(asset)?.into()),
            EExprToken::ExStructConst => Ok(ExStructConst::new(asset)?.into()),
            EExprToken::ExEndStructConst => Ok(ExEndStructConst::new(asset)?.into()),
            EExprToken::ExSetArray => Ok(ExSetArray::new(asset)?.into()),
            EExprToken::ExEndArray => Ok(ExEndArray::new(asset)?.into()),
            EExprToken::ExPropertyConst => Ok(ExPropertyConst::new(asset)?.into()),
            EExprToken::ExUnicodeStringConst => Ok(ExUnicodeStringConst::new(asset)?.into()),
            EExprToken::ExInt64Const => Ok(ExInt64Const::new(asset)?.into()),
            EExprToken::ExUInt64Const => Ok(ExUInt64Const::new(asset)?.into()),
            EExprToken::ExPrimitiveCast => Ok(ExPrimitiveCast::new(asset)?.into()),
            EExprToken::ExSetSet => Ok(ExSetSet::new(asset)?.into()),
            EExprToken::ExEndSet => Ok(ExEndSet::new(asset)?.into()),
            EExprToken::ExSetMap => Ok(ExSetMap::new(asset)?.into()),
            EExprToken::ExEndMap => Ok(ExEndMap::new(asset)?.into()),
            EExprToken::ExSetConst => Ok(ExSetConst::new(asset)?.into()),
            EExprToken::ExEndSetConst => Ok(ExEndSetConst::new(asset)?.into()),
            EExprToken::ExMapConst => Ok(ExMapConst::new(asset)?.into()),
            EExprToken::ExEndMapConst => Ok(ExEndMapConst::new(asset)?.into()),
            EExprToken::ExStructMemberContext => Ok(ExStructMemberContext::new(asset)?.into()),
            EExprToken::ExLetMulticastDelegate => Ok(ExLetMulticastDelegate::new(asset)?.into()),
            EExprToken::ExLetDelegate => Ok(ExLetDelegate::new(asset)?.into()),
            EExprToken::ExLocalVirtualFunction => Ok(ExLocalVirtualFunction::new(asset)?.into()),
            EExprToken::ExLocalFinalFunction => Ok(ExLocalFinalFunction::new(asset)?.into()),
            EExprToken::ExLocalOutVariable => Ok(ExLocalOutVariable::new(asset)?.into()),
            EExprToken::ExDeprecatedOp4A => Ok(ExDeprecatedOp4A::new(asset)?.into()),
            EExprToken::ExInstanceDelegate => Ok(ExInstanceDelegate::new(asset)?.into()),
            EExprToken::ExPushExecutionFlow => Ok(ExPushExecutionFlow::new(asset)?.into()),
            EExprToken::ExPopExecutionFlow => Ok(ExPopExecutionFlow::new(asset)?.into()),
            EExprToken::ExComputedJump => Ok(ExComputedJump::new(asset)?.into()),
            EExprToken::ExPopExecutionFlowIfNot => Ok(ExPopExecutionFlowIfNot::new(asset)?.into()),
            EExprToken::ExBreakpoint => Ok(ExBreakpoint::new(asset)?.into()),
            EExprToken::ExInterfaceContext => Ok(ExInterfaceContext::new(asset)?.into()),
            EExprToken::ExObjToInterfaceCast => Ok(ExObjToInterfaceCast::new(asset)?.into()),
            EExprToken::ExEndOfScript => Ok(ExEndOfScript::new(asset)?.into()),
            EExprToken::ExCrossInterfaceCast => Ok(ExCrossInterfaceCast::new(asset)?.into()),
            EExprToken::ExInterfaceToObjCast => Ok(ExInterfaceToObjCast::new(asset)?.into()),
            EExprToken::ExWireTracepoint => Ok(ExWireTracepoint::new(asset)?.into()),
            EExprToken::ExSkipOffsetConst => Ok(ExSkipOffsetConst::new(asset)?.into()),
            EExprToken::ExAddMulticastDelegate => Ok(ExAddMulticastDelegate::new(asset)?.into()),
            EExprToken::ExClearMulticastDelegate => {
                Ok(ExClearMulticastDelegate::new(asset)?.into())
            }
            EExprToken::ExTracepoint => Ok(ExTracepoint::new(asset)?.into()),
            EExprToken::ExLetObj => Ok(ExLetObj::new(asset)?.into()),
            EExprToken::ExLetWeakObjPtr => Ok(ExLetWeakObjPtr::new(asset)?.into()),
            EExprToken::ExBindDelegate => Ok(ExBindDelegate::new(asset)?.into()),
            EExprToken::ExRemoveMulticastDelegate => {
                Ok(ExRemoveMulticastDelegate::new(asset)?.into())
            }
            EExprToken::ExCallMulticastDelegate => Ok(ExCallMulticastDelegate::new(asset)?.into()),
            EExprToken::ExLetValueOnPersistentFrame => {
                Ok(ExLetValueOnPersistentFrame::new(asset)?.into())
            }
            EExprToken::ExArrayConst => Ok(ExArrayConst::new(asset)?.into()),
            EExprToken::ExEndArrayConst => Ok(ExEndArrayConst::new(asset)?.into()),
            EExprToken::ExSoftObjectConst => Ok(ExSoftObjectConst::new(asset)?.into()),
            EExprToken::ExCallMath => Ok(ExCallMath::new(asset)?.into()),
            EExprToken::ExSwitchValue => Ok(ExSwitchValue::new(asset)?.into()),
            EExprToken::ExInstrumentationEvent => Ok(ExInstrumentationEvent::new(asset)?.into()),
            EExprToken::ExArrayGetByRef => Ok(ExArrayGetByRef::new(asset)?.into()),
            EExprToken::ExClassSparseDataVariable => {
                Ok(ExClassSparseDataVariable::new(asset)?.into())
            }
            EExprToken::ExFieldPathConst => Ok(ExFieldPathConst::new(asset)?.into()),
            _ => Err(KismetError::expression(format!(
                "Unknown kismet expression {}",
                token as i32
            ))
            .into()),
        };
        expr
    }

    /// Read an array of `KismetExpression`s stopping at end_token
    pub fn read_arr<Reader: AssetReader>(
        asset: &mut Reader,
        end_token: EExprToken,
    ) -> Result<Vec<Self>, Error> {
        let mut data = Vec::new();
        let mut current_expr: Option<KismetExpression> = None;
        while current_expr.is_none() || !current_expr.as_ref().unwrap().enum_eq(&end_token) {
            if let Some(expr) = current_expr {
                data.push(expr);
            }
            current_expr = KismetExpression::new(asset).ok();
        }
        Ok(data)
    }

    /// Write a `KismetExpression`
    pub fn write<Writer: AssetWriter>(
        expr: &KismetExpression,
        asset: &mut Writer,
    ) -> Result<usize, Error> {
        asset.write_u8(expr.get_token().into())?;
        Ok(expr.write(asset)? + size_of::<u8>())
    }
}

declare_expression!(
    ExFieldPathConst,
    /// Value
    value: Box<KismetExpression>
);
impl ExFieldPathConst {
    /// Read a `ExFieldPathConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExFieldPathConst {
            token: EExprToken::ExFieldPathConst,
            value: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExFieldPathConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        KismetExpression::write(self.value.as_ref(), asset)
    }
}
declare_expression!(
    ExNameConst,
    /// Value
    value: FName
);
impl ExNameConst {
    /// Read a `ExNameConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExNameConst {
            token: EExprToken::ExNameConst,
            value: asset.read_fname()?,
        })
    }
}
impl KismetExpressionTrait for ExNameConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_fname(&self.value)?;
        Ok(12)
    }
}
declare_expression!(
    ExObjectConst,
    /// Value
    value: PackageIndex
);
impl ExObjectConst {
    /// Read a `ExObjectConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExObjectConst {
            token: EExprToken::ExObjectConst,
            value: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
        })
    }
}
impl KismetExpressionTrait for ExObjectConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_i32::<LittleEndian>(self.value.index)?;
        Ok(size_of::<u64>())
    }
}
declare_expression!(
    ExSoftObjectConst,
    /// Value
    value: Box<KismetExpression>
);
impl ExSoftObjectConst {
    /// Read a `ExSoftObjectConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExSoftObjectConst {
            token: EExprToken::ExSoftObjectConst,
            value: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExSoftObjectConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        KismetExpression::write(self.value.as_ref(), asset)
    }
}
declare_expression!(
    ExTransformConst,
    /// Value
    value: Transform<OrderedFloat<f32>>
);
impl ExTransformConst {
    /// Read a `ExTransformConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let rotation = Vector4::new(
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
        );
        let translation = Vector::new(
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
        );
        let scale = Vector::new(
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
            OrderedFloat(asset.read_f32::<LittleEndian>()?),
        );
        Ok(ExTransformConst {
            token: EExprToken::ExTransformConst,
            value: Transform::new(rotation, translation, scale),
        })
    }
}
impl KismetExpressionTrait for ExTransformConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_f32::<LittleEndian>(self.value.rotation.x.0)?;
        asset.write_f32::<LittleEndian>(self.value.rotation.y.0)?;
        asset.write_f32::<LittleEndian>(self.value.rotation.z.0)?;
        asset.write_f32::<LittleEndian>(self.value.rotation.w.0)?;
        asset.write_f32::<LittleEndian>(self.value.translation.x.0)?;
        asset.write_f32::<LittleEndian>(self.value.translation.y.0)?;
        asset.write_f32::<LittleEndian>(self.value.translation.z.0)?;
        asset.write_f32::<LittleEndian>(self.value.scale.x.0)?;
        asset.write_f32::<LittleEndian>(self.value.scale.y.0)?;
        asset.write_f32::<LittleEndian>(self.value.scale.z.0)?;
        Ok(size_of::<f32>() * 10)
    }
}
declare_expression!(
    ExVectorConst,
    /// Value
    value: Vector<OrderedFloat<f32>>
);
impl ExVectorConst {
    /// Read a `ExVectorConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExVectorConst {
            token: EExprToken::ExVectorConst,
            value: Vector::new(
                OrderedFloat(asset.read_f32::<LittleEndian>()?),
                OrderedFloat(asset.read_f32::<LittleEndian>()?),
                OrderedFloat(asset.read_f32::<LittleEndian>()?),
            ),
        })
    }
}
impl KismetExpressionTrait for ExVectorConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_f32::<LittleEndian>(self.value.x.0)?;
        asset.write_f32::<LittleEndian>(self.value.y.0)?;
        asset.write_f32::<LittleEndian>(self.value.z.0)?;
        Ok(size_of::<f32>() * 3)
    }
}
declare_expression!(
    ExTextConst,
    /// Value
    value: Box<FScriptText>
);
impl ExTextConst {
    /// Read a `ExTextConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExTextConst {
            token: EExprToken::ExTextConst,
            value: Box::new(FScriptText::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExTextConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        self.value.write(asset)
    }
}
declare_expression!(
    ExAddMulticastDelegate,
    /// Delegate
    delegate: Box<KismetExpression>,
    /// Delegate to add
    delegate_to_add: Box<KismetExpression>
);
impl ExAddMulticastDelegate {
    /// Read a `ExAddMulticastDelegate` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExAddMulticastDelegate {
            token: EExprToken::ExAddMulticastDelegate,
            delegate: Box::new(KismetExpression::new(asset)?),
            delegate_to_add: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExAddMulticastDelegate {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = KismetExpression::write(self.delegate.as_ref(), asset)?
            + KismetExpression::write(self.delegate_to_add.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExArrayConst,
    /// Inner property
    inner_property: KismetPropertyPointer,
    /// Array elements
    elements: Vec<KismetExpression>
);
impl ExArrayConst {
    /// Read a `ExArrayConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let inner_property = KismetPropertyPointer::new(asset)?;
        asset.read_i32::<LittleEndian>()?; // num_entries
        let elements = KismetExpression::read_arr(asset, EExprToken::ExEndArrayConst)?;
        Ok(ExArrayConst {
            token: EExprToken::ExArrayConst,
            inner_property,
            elements,
        })
    }
}
impl KismetExpressionTrait for ExArrayConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<i32>();
        offset += self.inner_property.write(asset)?;
        asset.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset += KismetExpression::write(element, asset)?;
        }
        offset += KismetExpression::write(&ExEndArrayConst::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExArrayGetByRef,
    /// Array variable
    array_variable: Box<KismetExpression>,
    /// Array element index
    array_index: Box<KismetExpression>
);
impl ExArrayGetByRef {
    /// Read a `ExArrayGetByRef` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExArrayGetByRef {
            token: EExprToken::ExArrayGetByRef,
            array_variable: Box::new(KismetExpression::new(asset)?),
            array_index: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExArrayGetByRef {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = KismetExpression::write(self.array_variable.as_ref(), asset)?
            + KismetExpression::write(self.array_index.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExAssert,
    /// Line number
    line_number: u16,
    /// Is debug
    debug_mode: bool,
    /// Expression to assert on
    assert_expression: Box<KismetExpression>
);
impl ExAssert {
    /// Read a `ExAssert` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExAssert {
            token: EExprToken::ExAssert,
            line_number: asset.read_u16::<LittleEndian>()?,
            debug_mode: asset.read_bool()?,
            assert_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExAssert {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u16::<LittleEndian>(self.line_number)?;
        asset.write_bool(self.debug_mode)?;
        let offset = size_of::<u32>()
            + size_of::<bool>()
            + KismetExpression::write(self.assert_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExBindDelegate,
    /// Function name
    function_name: FName,
    /// Delegate to bind
    delegate: Box<KismetExpression>,
    /// Object term
    object_term: Box<KismetExpression>
);
impl ExBindDelegate {
    /// Read a `ExBindDelegate` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExBindDelegate {
            token: EExprToken::ExBindDelegate,
            function_name: asset.read_fname()?,
            delegate: Box::new(KismetExpression::new(asset)?),
            object_term: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExBindDelegate {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_fname(&self.function_name)?;
        let offset = 12 /* FScriptName's iCode offset */ +
            KismetExpression::write(self.delegate.as_ref(), asset)? +
            KismetExpression::write(self.object_term.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExCallMath,
    /// Stack node
    stack_node: PackageIndex,
    /// Parameters
    parameters: Vec<KismetExpression>
);
impl ExCallMath {
    /// Read a `ExCallMath` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExCallMath {
            token: EExprToken::ExCallMath,
            stack_node: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            parameters: KismetExpression::read_arr(asset, EExprToken::ExEndFunctionParms)?,
        })
    }
}
impl KismetExpressionTrait for ExCallMath {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.stack_node.index)?;
        for parameter in &self.parameters {
            offset += KismetExpression::write(parameter, asset)?;
        }
        offset += KismetExpression::write(&ExEndFunctionParms::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExCallMulticastDelegate,
    /// Stack node
    stack_node: PackageIndex,
    /// Parameters
    parameters: Vec<KismetExpression>,
    /// Delegate to call
    delegate: Box<KismetExpression>
);
impl ExCallMulticastDelegate {
    /// Read a `ExCallMulticastDelegate` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let stack_node = PackageIndex::new(asset.read_i32::<LittleEndian>()?);
        let delegate = KismetExpression::new(asset)?;
        let parameters = KismetExpression::read_arr(asset, EExprToken::ExEndFunctionParms)?;
        Ok(ExCallMulticastDelegate {
            token: EExprToken::ExCallMulticastDelegate,
            stack_node,
            parameters,
            delegate: Box::new(delegate),
        })
    }
}
impl KismetExpressionTrait for ExCallMulticastDelegate {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.stack_node.index)?;
        offset += KismetExpression::write(&self.delegate, asset)?;
        for parameter in &self.parameters {
            offset += KismetExpression::write(parameter, asset)?;
        }
        offset += KismetExpression::write(&ExEndFunctionParms::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExClassContext,
    /// Object expression
    object_expression: Box<KismetExpression>,
    /// Offset
    offset: u32,
    /// r-value pointer
    r_value_pointer: KismetPropertyPointer,
    /// Context expression
    context_expression: Box<KismetExpression>
);
impl ExClassContext {
    /// Read a `ExClassContext` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExClassContext {
            token: EExprToken::ExClassContext,
            object_expression: Box::new(KismetExpression::new(asset)?),
            offset: asset.read_u32::<LittleEndian>()?,
            r_value_pointer: KismetPropertyPointer::new(asset)?,
            context_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExClassContext {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        offset += KismetExpression::write(self.object_expression.as_ref(), asset)?;
        asset.write_u32::<LittleEndian>(self.offset)?;
        offset += self.r_value_pointer.write(asset)?;
        offset += KismetExpression::write(self.context_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExClassSparseDataVariable,
    /// Variable
    variable: KismetPropertyPointer
);
impl ExClassSparseDataVariable {
    /// Read a `ExClassSparseDataVariable` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExClassSparseDataVariable {
            token: EExprToken::ExClassSparseDataVariable,
            variable: KismetPropertyPointer::new(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExClassSparseDataVariable {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        self.variable.write(asset)
    }
}
declare_expression!(
    ExClearMulticastDelegate,
    /// Delegate to clear
    delegate_to_clear: Box<KismetExpression>
);
impl ExClearMulticastDelegate {
    /// Read a `ExClearMulticastDelegate` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExClearMulticastDelegate {
            token: EExprToken::ExClearMulticastDelegate,
            delegate_to_clear: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExClearMulticastDelegate {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        KismetExpression::write(self.delegate_to_clear.as_ref(), asset)
    }
}
declare_expression!(
    ExComputedJump,
    /// Code offset expression
    code_offset_expression: Box<KismetExpression>
);
impl ExComputedJump {
    /// Read a `ExComputedJump` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExComputedJump {
            token: EExprToken::ExComputedJump,
            code_offset_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExComputedJump {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        KismetExpression::write(self.code_offset_expression.as_ref(), asset)
    }
}
declare_expression!(
    ExContext,
    /// Object expression
    object_expression: Box<KismetExpression>,
    /// Offset
    offset: u32,
    /// r-value pointer
    r_value_pointer: KismetPropertyPointer,
    /// Context expression
    context_expression: Box<KismetExpression>
);
impl ExContext {
    /// Read a `ExContext` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExContext {
            token: EExprToken::ExContext,
            object_expression: Box::new(KismetExpression::new(asset)?),
            offset: asset.read_u32::<LittleEndian>()?,
            r_value_pointer: KismetPropertyPointer::new(asset)?,
            context_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExContext {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        offset += KismetExpression::write(self.object_expression.as_ref(), asset)?;
        asset.write_u32::<LittleEndian>(self.offset)?;
        offset += self.r_value_pointer.write(asset)?;
        offset += KismetExpression::write(self.context_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExContextFailSilent,
    /// Object expression
    object_expression: Box<KismetExpression>,
    /// Offset
    offset: u32,
    /// r-value pointer
    r_value_pointer: KismetPropertyPointer,
    /// Context expression
    context_expression: Box<KismetExpression>
);
impl ExContextFailSilent {
    /// Read a `ExContextFailSilent` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExContextFailSilent {
            token: EExprToken::ExContextFailSilent,
            object_expression: Box::new(KismetExpression::new(asset)?),
            offset: asset.read_u32::<LittleEndian>()?,
            r_value_pointer: KismetPropertyPointer::new(asset)?,
            context_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExContextFailSilent {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        offset += KismetExpression::write(self.object_expression.as_ref(), asset)?;
        asset.write_u32::<LittleEndian>(self.offset)?;
        offset += self.r_value_pointer.write(asset)?;
        offset += KismetExpression::write(self.context_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExCrossInterfaceCast,
    /// Class pointer
    class_ptr: PackageIndex,
    /// Cast target
    target: Box<KismetExpression>
);
impl ExCrossInterfaceCast {
    /// Read a `ExCrossInterfaceCast` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExCrossInterfaceCast {
            token: EExprToken::ExCrossInterfaceCast,
            class_ptr: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            target: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExCrossInterfaceCast {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset += KismetExpression::write(self.target.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExDefaultVariable,
    /// Variable
    variable: KismetPropertyPointer
);
impl ExDefaultVariable {
    /// Read a `ExDefaultVariable` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExDefaultVariable {
            token: EExprToken::ExDefaultVariable,
            variable: KismetPropertyPointer::new(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExDefaultVariable {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        self.variable.write(asset)
    }
}
declare_expression!(
    ExDynamicCast,
    /// Class pointer
    class_ptr: PackageIndex,
    /// Cast target
    target_expression: Box<KismetExpression>
);
impl ExDynamicCast {
    /// Read a `ExDynamicCast` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExDynamicCast {
            token: EExprToken::ExDynamicCast,
            class_ptr: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            target_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExDynamicCast {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset += KismetExpression::write(self.target_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExFinalFunction,
    /// Stack node
    stack_node: PackageIndex,
    /// Parameters
    parameters: Vec<KismetExpression>
);
impl ExFinalFunction {
    /// Read a `ExFinalFunction` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExFinalFunction {
            token: EExprToken::ExFinalFunction,
            stack_node: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            parameters: KismetExpression::read_arr(asset, EExprToken::ExEndFunctionParms)?,
        })
    }
}
impl KismetExpressionTrait for ExFinalFunction {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.stack_node.index)?;
        for parameter in &self.parameters {
            offset += KismetExpression::write(parameter, asset)?;
        }
        offset += KismetExpression::write(&ExEndFunctionParms::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExInstanceDelegate,
    /// Function name
    function_name: FName
);
impl ExInstanceDelegate {
    /// Read a `ExInstanceDelegate` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExInstanceDelegate {
            token: EExprToken::ExInstanceDelegate,
            function_name: asset.read_fname()?,
        })
    }
}
impl KismetExpressionTrait for ExInstanceDelegate {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_fname(&self.function_name)?;
        Ok(12) // FScriptName's iCode offset
    }
}
declare_expression!(
    ExInstanceVariable,
    /// Variable
    variable: KismetPropertyPointer
);
impl ExInstanceVariable {
    /// Read a `ExInstanceVariable` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExInstanceVariable {
            token: EExprToken::ExInstanceVariable,
            variable: KismetPropertyPointer::new(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExInstanceVariable {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        self.variable.write(asset)
    }
}
declare_expression!(
    ExInterfaceContext,
    /// Interface value
    interface_value: Box<KismetExpression>
);
impl ExInterfaceContext {
    /// Read a `ExInterfaceContext` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExInterfaceContext {
            token: EExprToken::ExInterfaceContext,
            interface_value: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExInterfaceContext {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        KismetExpression::write(self.interface_value.as_ref(), asset)
    }
}
declare_expression!(
    ExInterfaceToObjCast,
    /// Class pointer
    class_ptr: PackageIndex,
    /// Cast target
    target: Box<KismetExpression>
);
impl ExInterfaceToObjCast {
    /// Read a `ExInterfaceToObjCast` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExInterfaceToObjCast {
            token: EExprToken::ExInterfaceToObjCast,
            class_ptr: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            target: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExInterfaceToObjCast {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset += KismetExpression::write(self.target.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExJump,
    /// Code offset
    code_offset: u32
);
impl ExJump {
    /// Read a `ExJump` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExJump {
            token: EExprToken::ExJump,
            code_offset: asset.read_u32::<LittleEndian>()?,
        })
    }
}
impl KismetExpressionTrait for ExJump {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u32::<LittleEndian>(self.code_offset)?;
        Ok(size_of::<u32>())
    }
}
declare_expression!(
    ExJumpIfNot,
    /// Code offset
    code_offset: u32,
    /// Expression to check
    boolean_expression: Box<KismetExpression>
);
impl ExJumpIfNot {
    /// Read a `ExJumpIfNot` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExJumpIfNot {
            token: EExprToken::ExJumpIfNot,
            code_offset: asset.read_u32::<LittleEndian>()?,
            boolean_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExJumpIfNot {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        asset.write_u32::<LittleEndian>(self.code_offset)?;
        offset += KismetExpression::write(self.boolean_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLet,
    /// Value
    value: KismetPropertyPointer,
    /// Variable
    variable: Box<KismetExpression>,
    /// Expression
    expression: Box<KismetExpression>
);
impl ExLet {
    /// Read a `ExLet` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLet {
            token: EExprToken::ExLet,
            value: KismetPropertyPointer::new(asset)?,
            variable: Box::new(KismetExpression::new(asset)?),
            expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExLet {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = self.value.write(asset)?;
        offset += KismetExpression::write(self.variable.as_ref(), asset)?;
        offset += KismetExpression::write(self.expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLetBool,
    /// Variable expression
    variable_expression: Box<KismetExpression>,
    /// Assignment
    assignment_expression: Box<KismetExpression>
);
impl ExLetBool {
    /// Read a `ExLetBool` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLetBool {
            token: EExprToken::ExLetBool,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExLetBool {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = KismetExpression::write(self.variable_expression.as_ref(), asset)?
            + KismetExpression::write(self.assignment_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLetDelegate,
    /// Variable expression
    variable_expression: Box<KismetExpression>,
    /// Assignment expression
    assignment_expression: Box<KismetExpression>
);
impl ExLetDelegate {
    /// Read a `ExLetDelegate` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLetDelegate {
            token: EExprToken::ExLetDelegate,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExLetDelegate {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = KismetExpression::write(self.variable_expression.as_ref(), asset)?
            + KismetExpression::write(self.assignment_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLetMulticastDelegate,
    /// Variable expression
    variable_expression: Box<KismetExpression>,
    /// Assignment expression
    assignment_expression: Box<KismetExpression>
);
impl ExLetMulticastDelegate {
    /// Read a `ExLetMulticastDelegate` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLetMulticastDelegate {
            token: EExprToken::ExLetMulticastDelegate,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExLetMulticastDelegate {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = KismetExpression::write(self.variable_expression.as_ref(), asset)?
            + KismetExpression::write(self.assignment_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLetObj,
    /// Variable expression
    variable_expression: Box<KismetExpression>,
    /// Assignment expression
    assignment_expression: Box<KismetExpression>
);
impl ExLetObj {
    /// Read a `ExLetObj` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLetObj {
            token: EExprToken::ExLetObj,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExLetObj {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = KismetExpression::write(self.variable_expression.as_ref(), asset)?
            + KismetExpression::write(self.assignment_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLetValueOnPersistentFrame,
    /// Destination property
    destination_property: KismetPropertyPointer,
    /// Assignment expression
    assignment_expression: Box<KismetExpression>
);
impl ExLetValueOnPersistentFrame {
    /// Read a `ExLetValueOnPersistentFrame` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLetValueOnPersistentFrame {
            token: EExprToken::ExLetValueOnPersistentFrame,
            destination_property: KismetPropertyPointer::new(asset)?,
            assignment_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExLetValueOnPersistentFrame {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = self.destination_property.write(asset)?
            + KismetExpression::write(self.assignment_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLetWeakObjPtr,
    /// Variable expression
    variable_expression: Box<KismetExpression>,
    /// Assignment expression
    assignment_expression: Box<KismetExpression>
);
impl ExLetWeakObjPtr {
    /// Read a `ExLetWeakObjPtr` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLetWeakObjPtr {
            token: EExprToken::ExLetWeakObjPtr,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExLetWeakObjPtr {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = KismetExpression::write(self.variable_expression.as_ref(), asset)?
            + KismetExpression::write(self.assignment_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLocalFinalFunction,
    /// Stack node
    stack_node: PackageIndex,
    /// Function parameters
    parameters: Vec<KismetExpression>
);
impl ExLocalFinalFunction {
    /// Read a `ExLocalFinalFunction` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLocalFinalFunction {
            token: EExprToken::ExLocalFinalFunction,
            stack_node: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            parameters: KismetExpression::read_arr(asset, EExprToken::ExEndFunctionParms)?,
        })
    }
}
impl KismetExpressionTrait for ExLocalFinalFunction {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.stack_node.index)?;
        for parameter in &self.parameters {
            offset += KismetExpression::write(parameter, asset)?;
        }
        offset += KismetExpression::write(&ExEndFunctionParms::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExLocalOutVariable,
    /// Variable
    variable: KismetPropertyPointer
);
impl ExLocalOutVariable {
    /// Read a `ExLocalOutVariable` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLocalOutVariable {
            token: EExprToken::ExLocalOutVariable,
            variable: KismetPropertyPointer::new(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExLocalOutVariable {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        self.variable.write(asset)
    }
}
declare_expression!(
    ExLocalVariable,
    /// Variable
    variable: KismetPropertyPointer
);
impl ExLocalVariable {
    /// Read a `ExLocalVariable` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLocalVariable {
            token: EExprToken::ExLocalVariable,
            variable: KismetPropertyPointer::new(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExLocalVariable {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        self.variable.write(asset)
    }
}
declare_expression!(
    ExLocalVirtualFunction,
    /// Virtual function name
    virtual_function_name: FName,
    /// Function parameters
    parameters: Vec<KismetExpression>
);
impl ExLocalVirtualFunction {
    /// Read a `ExLocalVirtualFunction` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExLocalVirtualFunction {
            token: EExprToken::ExLocalVirtualFunction,
            virtual_function_name: asset.read_fname()?,
            parameters: KismetExpression::read_arr(asset, EExprToken::ExEndFunctionParms)?,
        })
    }
}
impl KismetExpressionTrait for ExLocalVirtualFunction {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = 12; // FScriptName's iCode offset
        asset.write_fname(&self.virtual_function_name)?;
        for parameter in &self.parameters {
            offset += KismetExpression::write(parameter, asset)?;
        }
        offset += KismetExpression::write(&ExEndFunctionParms::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExMapConst,
    /// Key property
    key_property: KismetPropertyPointer,
    /// Value property
    value_property: KismetPropertyPointer,
    /// Elements
    elements: Vec<KismetExpression>
);
impl ExMapConst {
    /// Read a `ExMapConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let key_property = KismetPropertyPointer::new(asset)?;
        let value_property = KismetPropertyPointer::new(asset)?;
        let _num_entries = asset.read_i32::<LittleEndian>()?;
        let elements = KismetExpression::read_arr(asset, EExprToken::ExEndMapConst)?;
        Ok(ExMapConst {
            token: EExprToken::ExMapConst,
            key_property,
            value_property,
            elements,
        })
    }
}
impl KismetExpressionTrait for ExMapConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<i32>();
        offset += self.key_property.write(asset)?;
        offset += self.value_property.write(asset)?;
        asset.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset += KismetExpression::write(element, asset)?;
        }
        offset += KismetExpression::write(&ExEndMapConst::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExMetaCast,
    /// Class pointer
    class_ptr: PackageIndex,
    /// Target expression
    target_expression: Box<KismetExpression>
);
impl ExMetaCast {
    /// Read a `ExMetaCast` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExMetaCast {
            token: EExprToken::ExMetaCast,
            class_ptr: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            target_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExMetaCast {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset += KismetExpression::write(self.target_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExObjToInterfaceCast,
    /// Class pointer
    class_ptr: PackageIndex,
    /// Target expression
    target: Box<KismetExpression>
);
impl ExObjToInterfaceCast {
    /// Read a `ExObjToInterfaceCast` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExObjToInterfaceCast {
            token: EExprToken::ExObjToInterfaceCast,
            class_ptr: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            target: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExObjToInterfaceCast {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        asset.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset += KismetExpression::write(self.target.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExPopExecutionFlowIfNot,
    /// Boolean expression to check
    boolean_expression: Box<KismetExpression>
);
impl ExPopExecutionFlowIfNot {
    /// Read a `ExPopExecutionFlowIfNot` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExPopExecutionFlowIfNot {
            token: EExprToken::ExPopExecutionFlowIfNot,
            boolean_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExPopExecutionFlowIfNot {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        KismetExpression::write(self.boolean_expression.as_ref(), asset)
    }
}
declare_expression!(
    ExPrimitiveCast,
    /// Conversion type
    conversion_type: ECastToken,
    /// Cast target
    target: Box<KismetExpression>
);
impl ExPrimitiveCast {
    /// Read a `ExPrimitiveCast` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExPrimitiveCast {
            token: EExprToken::ExPrimitiveCast,
            conversion_type: asset.read_u8()?.try_into()?,
            target: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExPrimitiveCast {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u8>();
        asset.write_u8(self.conversion_type.into())?;
        offset += KismetExpression::write(self.target.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExPropertyConst,
    /// Property
    property: KismetPropertyPointer
);
impl ExPropertyConst {
    /// Read a `ExPropertyConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExPropertyConst {
            token: EExprToken::ExPropertyConst,
            property: KismetPropertyPointer::new(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExPropertyConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        self.property.write(asset)
    }
}
declare_expression!(
    ExPushExecutionFlow,
    /// Pushing address
    pushing_address: u32
);
impl ExPushExecutionFlow {
    /// Read a `ExPushExecutionFlow` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExPushExecutionFlow {
            token: EExprToken::ExPushExecutionFlow,
            pushing_address: asset.read_u32::<LittleEndian>()?,
        })
    }
}
impl KismetExpressionTrait for ExPushExecutionFlow {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u32::<LittleEndian>(self.pushing_address)?;
        Ok(size_of::<u32>())
    }
}
declare_expression!(
    ExRemoveMulticastDelegate,
    /// Delegate
    delegate: Box<KismetExpression>,
    /// Delegate to add
    delegate_to_add: Box<KismetExpression>
);
impl ExRemoveMulticastDelegate {
    /// Read a `ExRemoveMulticastDelegate` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExRemoveMulticastDelegate {
            token: EExprToken::ExRemoveMulticastDelegate,
            delegate: Box::new(KismetExpression::new(asset)?),
            delegate_to_add: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExRemoveMulticastDelegate {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let offset = KismetExpression::write(self.delegate.as_ref(), asset)?
            + KismetExpression::write(self.delegate_to_add.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExReturn,
    /// Return expression
    return_expression: Box<KismetExpression>
);
impl ExReturn {
    /// Read a `ExReturn` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExReturn {
            token: EExprToken::ExReturn,
            return_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExReturn {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        KismetExpression::write(self.return_expression.as_ref(), asset)
    }
}
declare_expression!(
    ExRotationConst,
    /// Pitch
    pitch: i32,
    /// Yaw
    yaw: i32,
    /// Roll
    roll: i32
);
impl ExRotationConst {
    /// Read a `ExRotationConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExRotationConst {
            token: EExprToken::ExRotationConst,
            pitch: asset.read_i32::<LittleEndian>()?,
            yaw: asset.read_i32::<LittleEndian>()?,
            roll: asset.read_i32::<LittleEndian>()?,
        })
    }
}
impl KismetExpressionTrait for ExRotationConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_i32::<LittleEndian>(self.pitch)?;
        asset.write_i32::<LittleEndian>(self.yaw)?;
        asset.write_i32::<LittleEndian>(self.roll)?;
        Ok(size_of::<i32>() * 3)
    }
}
declare_expression!(
    ExSetArray,
    /// Assigning property
    assigning_property: Option<Box<KismetExpression>>,
    /// Array inner prop
    array_inner_prop: Option<PackageIndex>,
    /// Elements
    elements: Vec<KismetExpression>
);
impl ExSetArray {
    /// Read a `ExSetArray` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let (assigning_property, array_inner_prop) =
            match asset.get_object_version() >= ObjectVersion::VER_UE4_CHANGE_SETARRAY_BYTECODE {
                true => (Some(Box::new(KismetExpression::new(asset)?)), None),
                false => (
                    None,
                    Some(PackageIndex::new(asset.read_i32::<LittleEndian>()?)),
                ),
            };
        Ok(ExSetArray {
            token: EExprToken::ExSetArray,
            assigning_property,
            array_inner_prop,
            elements: KismetExpression::read_arr(asset, EExprToken::ExEndArray)?,
        })
    }
}
impl KismetExpressionTrait for ExSetArray {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = 0;
        if asset.get_object_version() >= ObjectVersion::VER_UE4_CHANGE_SETARRAY_BYTECODE {
            offset += KismetExpression::write(
                self.assigning_property.as_ref().ok_or_else(|| {
                    Error::no_data(
                    "engine_version >= UE4_CHANGE_SETARRAY_BYTECODE but assigning_property is None"
                        .to_string(),
                )
                })?,
                asset,
            )?;
        } else {
            asset.write_i32::<LittleEndian>(self.array_inner_prop.map(|e| e.index).ok_or_else(
                || {
                    Error::no_data(
                    "engine_version < UE4_CHANGE_SETARRAY_BYTECODE but array_inner_prop is None"
                        .to_string(),
                )
                },
            )?)?;
            offset += size_of::<u64>();
        }

        for element in &self.elements {
            offset += KismetExpression::write(element, asset)?;
        }
        offset += KismetExpression::write(&ExEndArray::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExSetConst,
    /// Inner property
    inner_property: KismetPropertyPointer,
    /// Set elements
    elements: Vec<KismetExpression>
);
impl ExSetConst {
    /// Read a `ExSetConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let inner_property = KismetPropertyPointer::new(asset)?;
        let _num_entries = asset.read_i32::<LittleEndian>()?;
        let elements = KismetExpression::read_arr(asset, EExprToken::ExEndSetConst)?;
        Ok(ExSetConst {
            token: EExprToken::ExSetConst,
            inner_property,
            elements,
        })
    }
}
impl KismetExpressionTrait for ExSetConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<i32>();
        offset += self.inner_property.write(asset)?;
        asset.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset += KismetExpression::write(element, asset)?;
        }
        offset += KismetExpression::write(&ExEndSetConst::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExSetMap,
    /// Map property
    map_property: Box<KismetExpression>,
    /// Map elements
    elements: Vec<KismetExpression>
);
impl ExSetMap {
    /// Read a `ExSetMap` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let map_property = Box::new(KismetExpression::new(asset)?);
        let _num_entries = asset.read_i32::<LittleEndian>()?;
        let elements = KismetExpression::read_arr(asset, EExprToken::ExEndMap)?;
        Ok(ExSetMap {
            token: EExprToken::ExSetMap,
            map_property,
            elements,
        })
    }
}
impl KismetExpressionTrait for ExSetMap {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<i32>();
        offset += KismetExpression::write(self.map_property.as_ref(), asset)?;
        asset.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset += KismetExpression::write(element, asset)?;
        }
        offset += KismetExpression::write(&ExEndMap::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExSetSet,
    /// Set property
    set_property: Box<KismetExpression>,
    /// Elements to set
    elements: Vec<KismetExpression>
);
impl ExSetSet {
    /// Read a `ExSetSet` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let set_property = Box::new(KismetExpression::new(asset)?);
        let _num_entries = asset.read_i32::<LittleEndian>()?;
        let elements = KismetExpression::read_arr(asset, EExprToken::ExEndSet)?;
        Ok(ExSetSet {
            token: EExprToken::ExSetSet,
            set_property,
            elements,
        })
    }
}
impl KismetExpressionTrait for ExSetSet {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<i32>();
        offset += KismetExpression::write(self.set_property.as_ref(), asset)?;
        asset.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset += KismetExpression::write(element, asset)?;
        }
        offset += KismetExpression::write(&ExEndSet::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExSkip,
    /// Code offset
    code_offset: u32,
    /// Expression to skip
    skip_expression: Box<KismetExpression>
);
impl ExSkip {
    /// Read a `ExSkip` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExSkip {
            token: EExprToken::ExSkip,
            code_offset: asset.read_u32::<LittleEndian>()?,
            skip_expression: Box::new(KismetExpression::new(asset)?),
        })
    }
}
impl KismetExpressionTrait for ExSkip {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        asset.write_u32::<LittleEndian>(self.code_offset)?;
        offset += KismetExpression::write(self.skip_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExStructConst,
    /// Struct value
    struct_value: PackageIndex,
    /// Struct size
    struct_size: i32,
    /// Value
    value: Vec<KismetExpression>
);
impl ExStructConst {
    /// Read a `ExStructConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExStructConst {
            token: EExprToken::ExStructConst,
            struct_value: PackageIndex::new(asset.read_i32::<LittleEndian>()?),
            struct_size: asset.read_i32::<LittleEndian>()?,
            value: KismetExpression::read_arr(asset, EExprToken::ExEndStructConst)?,
        })
    }
}
impl KismetExpressionTrait for ExStructConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u64>() + size_of::<i32>();
        asset.write_i32::<LittleEndian>(self.struct_value.index)?;
        asset.write_i32::<LittleEndian>(self.struct_size)?;
        for entry in &self.value {
            offset += KismetExpression::write(entry, asset)?;
        }
        offset += KismetExpression::write(&ExEndStructConst::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExStructMemberContext,
    /// Struct member expression
    struct_member_expression: KismetPropertyPointer,
    /// Struct expression
    struct_expression: Box<KismetExpression>
);
impl ExStructMemberContext {
    /// Read a `ExStructMemberContext` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let struct_member_expression = KismetPropertyPointer::new(asset)?;
        let struct_expression = KismetExpression::new(asset)?;
        Ok(ExStructMemberContext {
            token: EExprToken::ExStructMemberContext,
            struct_member_expression,
            struct_expression: Box::new(struct_expression),
        })
    }
}
impl KismetExpressionTrait for ExStructMemberContext {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = 0;
        offset += self.struct_member_expression.write(asset)?;
        offset += KismetExpression::write(self.struct_expression.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExSwitchValue,
    /// End goto offset
    end_goto_offset: u32,
    /// Index term
    index_term: Box<KismetExpression>,
    /// Default term
    default_term: Box<KismetExpression>,
    /// Cases
    cases: Vec<KismetSwitchCase>
);
impl ExSwitchValue {
    /// Read a `ExSwitchValue` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let num_cases = asset.read_u16::<LittleEndian>()?;
        let end_goto_offset = asset.read_u32::<LittleEndian>()?;
        let index_term = Box::new(KismetExpression::new(asset)?);

        let mut cases = Vec::with_capacity(num_cases as usize);
        for _i in 0..num_cases as usize {
            let term_a = KismetExpression::new(asset)?;
            let term_b = asset.read_u32::<LittleEndian>()?;
            let term_c = KismetExpression::new(asset)?;
            cases.push(KismetSwitchCase::new(term_a, term_b, term_c));
        }
        let default_term = Box::new(KismetExpression::new(asset)?);
        Ok(ExSwitchValue {
            token: EExprToken::ExSwitchValue,
            end_goto_offset,
            index_term,
            default_term,
            cases,
        })
    }
}
impl KismetExpressionTrait for ExSwitchValue {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = size_of::<u16>() + size_of::<u32>();
        asset.write_u16::<LittleEndian>(self.cases.len() as u16)?;
        asset.write_u32::<LittleEndian>(self.end_goto_offset)?;
        offset += KismetExpression::write(self.index_term.as_ref(), asset)?;
        for case in &self.cases {
            offset += KismetExpression::write(&case.case_index_value_term, asset)?;
            offset += size_of::<u32>();
            asset.write_u32::<LittleEndian>(case.next_offset)?;
            offset += KismetExpression::write(&case.case_term, asset)?;
        }
        offset += KismetExpression::write(self.default_term.as_ref(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExVirtualFunction,
    /// Virtual function name
    virtual_function_name: FName,
    /// Function parameters
    parameters: Vec<KismetExpression>
);
impl ExVirtualFunction {
    /// Read a `ExVirtualFunction` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExVirtualFunction {
            token: EExprToken::ExVirtualFunction,
            virtual_function_name: asset.read_fname()?,
            parameters: KismetExpression::read_arr(asset, EExprToken::ExEndFunctionParms)?,
        })
    }
}
impl KismetExpressionTrait for ExVirtualFunction {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        let mut offset = 12; // FScriptName's iCode offset
        asset.write_fname(&self.virtual_function_name)?;
        for parameter in &self.parameters {
            offset += KismetExpression::write(parameter, asset)?;
        }
        offset += KismetExpression::write(&ExEndFunctionParms::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExStringConst,
    /// Value
    value: String
);
impl ExStringConst {
    /// Read a `ExStringConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExStringConst {
            token: EExprToken::ExStringConst,
            value: read_kismet_string(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExStringConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        write_kismet_string(&self.value, asset)
    }
}
declare_expression!(
    ExUnicodeStringConst,
    /// Value
    value: String
);
impl ExUnicodeStringConst {
    /// Read a `ExUnicodeStringConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExUnicodeStringConst {
            token: EExprToken::ExUnicodeStringConst,
            value: read_kismet_unicode_string(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExUnicodeStringConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        write_kismet_unicode_string(&self.value, asset)
    }
}

declare_expression!(
    ExInstrumentationEvent,
    /// Event type
    event_type: EScriptInstrumentationType,
    /// Event name
    event_name: Option<FName>
);
impl ExInstrumentationEvent {
    /// Read a `ExInstrumentationEvent` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let event_type: EScriptInstrumentationType =
            EScriptInstrumentationType::try_from(asset.read_u8()?)?;

        let mut event_name = None;
        if event_type == EScriptInstrumentationType::InlineEvent {
            event_name = Some(asset.read_fname()?);
        }

        Ok(ExInstrumentationEvent {
            token: EExprToken::ExInstrumentationEvent,
            event_type,
            event_name,
        })
    }
}
impl KismetExpressionTrait for ExInstrumentationEvent {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_u8(self.event_type as u8)?;

        if self.event_type == EScriptInstrumentationType::InlineEvent {
            asset.write_fname(self.event_name.as_ref().ok_or_else(|| {
                Error::no_data("event_type is InlineEvent but event_name is None".to_string())
            })?)?;
            return Ok(1 + 2 * size_of::<i32>());
        }

        Ok(1)
    }
}

declare_expression!(
    ExFloatConst,
    /// Value
    value: OrderedFloat<f32>
);
impl ExFloatConst {
    /// Read a `ExFloatConst` from an asset
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExFloatConst {
            token: EExprToken::ExFloatConst,
            value: OrderedFloat(asset.read_f32::<LittleEndian>()?),
        })
    }
}
impl KismetExpressionTrait for ExFloatConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_f32::<LittleEndian>(self.value.0)?;
        Ok(size_of::<f32>())
    }
}

implement_expression!(
    /// Breakpoint. Only observed in the editor, otherwise it behaves like ExNothing.
    ExBreakpoint,
    /// Deprecated operation
    ExDeprecatedOp4A,
    /// End an array
    ExEndArray,
    /// End an array const
    ExEndArrayConst,
    /// End of function call parameters.
    ExEndFunctionParms,
    /// End a map
    ExEndMap,
    /// End a constant map
    ExEndMapConst,
    /// Last byte in script code
    ExEndOfScript,
    /// end of default value for optional function parameter
    ExEndParmValue,
    /// End a set
    ExEndSet,
    /// End a set const
    ExEndSetConst,
    /// End of UStruct constant
    ExEndStructConst,
    /// Bool False.
    ExFalse,
    /// One.
    ExIntOne,
    /// Zero.
    ExIntZero,
    /// A null interface (similar to ExNoObject, but for interfaces)
    ExNoInterface,
    /// NoObject.
    ExNoObject,
    /// No operation.
    ExNothing,
    /// continue execution at the last address previously pushed onto the execution flow stack.
    ExPopExecutionFlow,
    /// Self object.
    ExSelf,
    /// Trace point.  Only observed in the editor, otherwise it behaves like ExNothing.
    ExTracepoint,
    /// Bool True.
    ExTrue,
    /// Trace point.  Only observed in the editor, otherwise it behaves like ExNothing.
    ExWireTracepoint
);

implement_value_expression!(ExByteConst, u8, read_u8, write_u8);
implement_value_expression!(ExInt64Const, i64, read_i64, write_i64, LittleEndian);
implement_value_expression!(ExIntConst, i32, read_i32, write_i32, LittleEndian);
implement_value_expression!(ExIntConstByte, u8, read_u8, write_u8);
implement_value_expression!(ExSkipOffsetConst, u32, read_u32, write_u32, LittleEndian);
implement_value_expression!(ExUInt64Const, u64, read_u64, write_u64, LittleEndian);
