//! Kismet bytecode
use std::mem::size_of;

use byteorder::LittleEndian;
use enum_dispatch::enum_dispatch;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::enums::EBlueprintTextLiteralType;
use crate::error::KismetError;
use crate::object_version::ObjectVersion;
use crate::reader::{asset_reader::AssetReader, asset_writer::AssetWriter};
use crate::types::{Transform, Vector, Vector4};
use crate::unreal_types::{FName, FieldPath, PackageIndex};
use crate::Error;

#[derive(Debug, PartialEq, Eq, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
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
    ExSetSet = 0x39,
    ExEndSet = 0x3A,
    ExSetMap = 0x3B,
    ExEndMap = 0x3C,
    ExSetConst = 0x3D,
    ExEndSetConst = 0x3E,
    ExMapConst = 0x3F,
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
    ExLetValueOnPersistentFrame = 0x64,
    ExArrayConst = 0x65,
    ExEndArrayConst = 0x66,
    ExSoftObjectConst = 0x67,
    /// static pure function from on local call space
    ExCallMath = 0x68,
    ExSwitchValue = 0x69,
    /// Instrumentation event
    ExInstrumentationEvent = 0x6A,
    ExArrayGetByRef = 0x6B,
    /// Sparse data variable
    ExClassSparseDataVariable = 0x6C,
    ExFieldPathConst = 0x6D,
    ExMax = 0xff,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ECastToken {
    ObjectToInterface = 0x46,
    ObjectToBool = 0x47,
    InterfaceToBool = 0x49,
    Max = 0xFF,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EScriptInstrumentationType {
    Class = 0,
    ClassScope,
    Instance,
    Event,
    InlineEvent,
    ResumeEvent,
    PureNodeEntry,
    NodeDebugSite,
    NodeEntry,
    NodeExit,
    PushState,
    RestoreState,
    ResetState,
    SuspendState,
    PopState,
    TunnelEndOfThread,
    Stop,
}

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

fn read_kismet_unicode_string<Reader: AssetReader>(asset: &mut Reader) -> Result<String, Error> {
    let mut data = Vec::new();
    loop {
        let b1 = asset.read_u8()?;
        let b2 = asset.read_u8()?;
        if b1 == 0 && b2 == 0 {
            break;
        }
        data.push(((b1 as u16) << 8) | b2 as u16)
    }
    Ok(String::from_utf16(&data)?)
}

fn write_kismet_string<Writer: AssetWriter>(
    string: &str,
    asset: &mut Writer,
) -> Result<usize, Error> {
    let begin = asset.position();
    asset.write_all(string.as_bytes())?;
    asset.write_all(&[0u8; 1])?;
    Ok((asset.position() - begin) as usize)
}

macro_rules! declare_expression {
    ($name:ident, $($v:ident: $t:ty),*) => {
        #[derive(Clone)]
        pub struct $name {
            pub token: EExprToken,
            $(
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
    ($($name:ident),*) => {
        $(
            #[derive(Clone)]
            pub struct $name { pub token: EExprToken }

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
        declare_expression!($name, value: $param);
        impl $name {
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
        declare_expression!($name, value: $param);
        impl $name {
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

#[derive(Clone)]
pub struct FScriptText {
    text_literal_type: EBlueprintTextLiteralType,
    localized_source: Option<KismetExpression>,
    localized_key: Option<KismetExpression>,
    localized_namespace: Option<KismetExpression>,
    invariant_literal_string: Option<KismetExpression>,
    literal_string: Option<KismetExpression>,
    string_table_asset: Option<PackageIndex>,
    string_table_id: Option<KismetExpression>,
    string_table_key: Option<KismetExpression>,
}

impl FScriptText {
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
#[derive(Default, Clone)]
pub struct KismetPropertyPointer {
    /// Pointer serialized as PackageIndex. Used in versions older than [`KismetPropertyPointer::XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION`]
    pub old: Option<PackageIndex>,
    /// Pointer serialized as an FFieldPath. Used in versions newer than [`KismetPropertyPointer::XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION`]
    pub new: Option<FieldPath>,
}

impl KismetPropertyPointer {
    const XFER_PROP_POINTER_SWITCH_TO_SERIALIZING_AS_FIELD_PATH_VERSION: ObjectVersion =
        ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER;

    pub fn from_old(old: PackageIndex) -> Self {
        KismetPropertyPointer {
            old: Some(old),
            new: None,
        }
    }

    pub fn from_new(new: FieldPath) -> Self {
        KismetPropertyPointer {
            old: None,
            new: Some(new),
        }
    }

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

#[derive(Clone)]
pub struct KismetSwitchCase {
    case_index_value_term: KismetExpression,
    next_offset: u32,
    case_term: KismetExpression,
}

impl KismetSwitchCase {
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
pub trait KismetExpressionTrait {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error>;
}

/// Allows for getting a token from a KismetExpression
#[enum_dispatch]
pub trait KismetExpressionDataTrait {
    fn get_token(&self) -> EExprToken;
}

/// Allows for comparing two KismetExpressions based on their token
#[enum_dispatch]
pub trait KismetExpressionEnumEqTrait {
    fn enum_eq(&self, token: &EExprToken) -> bool;
}

#[derive(PartialEq, Eq)]
#[enum_dispatch(
    KismetExpressionTrait,
    KismetExpressionEnumEqTrait,
    KismetExpressionDataTrait
)]
pub enum KismetExpression {
    ExLocalVariable,
    ExInstanceVariable,
    ExDefaultVariable,
    ExReturn,
    ExJump,
    ExJumpIfNot,
    ExAssert,
    ExNothing,
    ExLet,
    ExClassContext,
    ExMetaCast,
    ExLetBool,
    ExEndParmValue,
    ExEndFunctionParms,
    ExSelf,
    ExSkip,
    ExContext,
    ExContextFailSilent,
    ExVirtualFunction,
    ExFinalFunction,
    ExIntConst,
    ExFloatConst,
    ExStringConst,
    ExObjectConst,
    ExNameConst,
    ExRotationConst,
    ExVectorConst,
    ExByteConst,
    ExIntZero,
    ExIntOne,
    ExTrue,
    ExFalse,
    ExTextConst,
    ExNoObject,
    ExTransformConst,
    ExIntConstByte,
    ExNoInterface,
    ExDynamicCast,
    ExStructConst,
    ExEndStructConst,
    ExSetArray,
    ExEndArray,
    ExPropertyConst,
    ExUnicodeStringConst,
    ExInt64Const,
    ExUInt64Const,
    ExPrimitiveCast,
    ExSetSet,
    ExEndSet,
    ExSetMap,
    ExEndMap,
    ExSetConst,
    ExEndSetConst,
    ExMapConst,
    ExEndMapConst,
    ExStructMemberContext,
    ExLetMulticastDelegate,
    ExLetDelegate,
    ExLocalVirtualFunction,
    ExLocalFinalFunction,
    ExLocalOutVariable,
    ExDeprecatedOp4A,
    ExInstanceDelegate,
    ExPushExecutionFlow,
    ExPopExecutionFlow,
    ExComputedJump,
    ExPopExecutionFlowIfNot,
    ExBreakpoint,
    ExInterfaceContext,
    ExObjToInterfaceCast,
    ExEndOfScript,
    ExCrossInterfaceCast,
    ExInterfaceToObjCast,
    ExWireTracepoint,
    ExSkipOffsetConst,
    ExAddMulticastDelegate,
    ExClearMulticastDelegate,
    ExTracepoint,
    ExLetObj,
    ExLetWeakObjPtr,
    ExBindDelegate,
    ExRemoveMulticastDelegate,
    ExCallMulticastDelegate,
    ExLetValueOnPersistentFrame,
    ExArrayConst,
    ExEndArrayConst,
    ExSoftObjectConst,
    ExCallMath,
    ExSwitchValue,
    ExInstrumentationEvent,
    ExArrayGetByRef,
    ExClassSparseDataVariable,
    ExFieldPathConst,
}

impl Clone for KismetExpression {
    fn clone(&self) -> Self {
        match self {
            Self::ExLocalVariable(arg0) => Self::ExLocalVariable(arg0.clone()),
            Self::ExInstanceVariable(arg0) => Self::ExInstanceVariable(arg0.clone()),
            Self::ExDefaultVariable(arg0) => Self::ExDefaultVariable(arg0.clone()),
            Self::ExReturn(arg0) => Self::ExReturn(arg0.clone()),
            Self::ExJump(arg0) => Self::ExJump(arg0.clone()),
            Self::ExJumpIfNot(arg0) => Self::ExJumpIfNot(arg0.clone()),
            Self::ExAssert(arg0) => Self::ExAssert(arg0.clone()),
            Self::ExNothing(arg0) => Self::ExNothing(arg0.clone()),
            Self::ExLet(arg0) => Self::ExLet(arg0.clone()),
            Self::ExClassContext(arg0) => Self::ExClassContext(arg0.clone()),
            Self::ExMetaCast(arg0) => Self::ExMetaCast(arg0.clone()),
            Self::ExLetBool(arg0) => Self::ExLetBool(arg0.clone()),
            Self::ExEndParmValue(arg0) => Self::ExEndParmValue(arg0.clone()),
            Self::ExEndFunctionParms(arg0) => Self::ExEndFunctionParms(arg0.clone()),
            Self::ExSelf(arg0) => Self::ExSelf(arg0.clone()),
            Self::ExSkip(arg0) => Self::ExSkip(arg0.clone()),
            Self::ExContext(arg0) => Self::ExContext(arg0.clone()),
            Self::ExContextFailSilent(arg0) => Self::ExContextFailSilent(arg0.clone()),
            Self::ExVirtualFunction(arg0) => Self::ExVirtualFunction(arg0.clone()),
            Self::ExFinalFunction(arg0) => Self::ExFinalFunction(arg0.clone()),
            Self::ExIntConst(arg0) => Self::ExIntConst(arg0.clone()),
            Self::ExFloatConst(arg0) => Self::ExFloatConst(arg0.clone()),
            Self::ExStringConst(arg0) => Self::ExStringConst(arg0.clone()),
            Self::ExObjectConst(arg0) => Self::ExObjectConst(arg0.clone()),
            Self::ExNameConst(arg0) => Self::ExNameConst(arg0.clone()),
            Self::ExRotationConst(arg0) => Self::ExRotationConst(arg0.clone()),
            Self::ExVectorConst(arg0) => Self::ExVectorConst(arg0.clone()),
            Self::ExByteConst(arg0) => Self::ExByteConst(arg0.clone()),
            Self::ExIntZero(arg0) => Self::ExIntZero(arg0.clone()),
            Self::ExIntOne(arg0) => Self::ExIntOne(arg0.clone()),
            Self::ExTrue(arg0) => Self::ExTrue(arg0.clone()),
            Self::ExFalse(arg0) => Self::ExFalse(arg0.clone()),
            Self::ExTextConst(arg0) => Self::ExTextConst(arg0.clone()),
            Self::ExNoObject(arg0) => Self::ExNoObject(arg0.clone()),
            Self::ExTransformConst(arg0) => Self::ExTransformConst(arg0.clone()),
            Self::ExIntConstByte(arg0) => Self::ExIntConstByte(arg0.clone()),
            Self::ExNoInterface(arg0) => Self::ExNoInterface(arg0.clone()),
            Self::ExDynamicCast(arg0) => Self::ExDynamicCast(arg0.clone()),
            Self::ExStructConst(arg0) => Self::ExStructConst(arg0.clone()),
            Self::ExEndStructConst(arg0) => Self::ExEndStructConst(arg0.clone()),
            Self::ExSetArray(arg0) => Self::ExSetArray(arg0.clone()),
            Self::ExEndArray(arg0) => Self::ExEndArray(arg0.clone()),
            Self::ExPropertyConst(arg0) => Self::ExPropertyConst(arg0.clone()),
            Self::ExUnicodeStringConst(arg0) => Self::ExUnicodeStringConst(arg0.clone()),
            Self::ExInt64Const(arg0) => Self::ExInt64Const(arg0.clone()),
            Self::ExUInt64Const(arg0) => Self::ExUInt64Const(arg0.clone()),
            Self::ExPrimitiveCast(arg0) => Self::ExPrimitiveCast(arg0.clone()),
            Self::ExSetSet(arg0) => Self::ExSetSet(arg0.clone()),
            Self::ExEndSet(arg0) => Self::ExEndSet(arg0.clone()),
            Self::ExSetMap(arg0) => Self::ExSetMap(arg0.clone()),
            Self::ExEndMap(arg0) => Self::ExEndMap(arg0.clone()),
            Self::ExSetConst(arg0) => Self::ExSetConst(arg0.clone()),
            Self::ExEndSetConst(arg0) => Self::ExEndSetConst(arg0.clone()),
            Self::ExMapConst(arg0) => Self::ExMapConst(arg0.clone()),
            Self::ExEndMapConst(arg0) => Self::ExEndMapConst(arg0.clone()),
            Self::ExStructMemberContext(arg0) => Self::ExStructMemberContext(arg0.clone()),
            Self::ExLetMulticastDelegate(arg0) => Self::ExLetMulticastDelegate(arg0.clone()),
            Self::ExLetDelegate(arg0) => Self::ExLetDelegate(arg0.clone()),
            Self::ExLocalVirtualFunction(arg0) => Self::ExLocalVirtualFunction(arg0.clone()),
            Self::ExLocalFinalFunction(arg0) => Self::ExLocalFinalFunction(arg0.clone()),
            Self::ExLocalOutVariable(arg0) => Self::ExLocalOutVariable(arg0.clone()),
            Self::ExDeprecatedOp4A(arg0) => Self::ExDeprecatedOp4A(arg0.clone()),
            Self::ExInstanceDelegate(arg0) => Self::ExInstanceDelegate(arg0.clone()),
            Self::ExPushExecutionFlow(arg0) => Self::ExPushExecutionFlow(arg0.clone()),
            Self::ExPopExecutionFlow(arg0) => Self::ExPopExecutionFlow(arg0.clone()),
            Self::ExComputedJump(arg0) => Self::ExComputedJump(arg0.clone()),
            Self::ExPopExecutionFlowIfNot(arg0) => Self::ExPopExecutionFlowIfNot(arg0.clone()),
            Self::ExBreakpoint(arg0) => Self::ExBreakpoint(arg0.clone()),
            Self::ExInterfaceContext(arg0) => Self::ExInterfaceContext(arg0.clone()),
            Self::ExObjToInterfaceCast(arg0) => Self::ExObjToInterfaceCast(arg0.clone()),
            Self::ExEndOfScript(arg0) => Self::ExEndOfScript(arg0.clone()),
            Self::ExCrossInterfaceCast(arg0) => Self::ExCrossInterfaceCast(arg0.clone()),
            Self::ExInterfaceToObjCast(arg0) => Self::ExInterfaceToObjCast(arg0.clone()),
            Self::ExWireTracepoint(arg0) => Self::ExWireTracepoint(arg0.clone()),
            Self::ExSkipOffsetConst(arg0) => Self::ExSkipOffsetConst(arg0.clone()),
            Self::ExAddMulticastDelegate(arg0) => Self::ExAddMulticastDelegate(arg0.clone()),
            Self::ExClearMulticastDelegate(arg0) => Self::ExClearMulticastDelegate(arg0.clone()),
            Self::ExTracepoint(arg0) => Self::ExTracepoint(arg0.clone()),
            Self::ExLetObj(arg0) => Self::ExLetObj(arg0.clone()),
            Self::ExLetWeakObjPtr(arg0) => Self::ExLetWeakObjPtr(arg0.clone()),
            Self::ExBindDelegate(arg0) => Self::ExBindDelegate(arg0.clone()),
            Self::ExRemoveMulticastDelegate(arg0) => Self::ExRemoveMulticastDelegate(arg0.clone()),
            Self::ExCallMulticastDelegate(arg0) => Self::ExCallMulticastDelegate(arg0.clone()),
            Self::ExLetValueOnPersistentFrame(arg0) => {
                Self::ExLetValueOnPersistentFrame(arg0.clone())
            }
            Self::ExArrayConst(arg0) => Self::ExArrayConst(arg0.clone()),
            Self::ExEndArrayConst(arg0) => Self::ExEndArrayConst(arg0.clone()),
            Self::ExSoftObjectConst(arg0) => Self::ExSoftObjectConst(arg0.clone()),
            Self::ExCallMath(arg0) => Self::ExCallMath(arg0.clone()),
            Self::ExSwitchValue(arg0) => Self::ExSwitchValue(arg0.clone()),
            Self::ExInstrumentationEvent(arg0) => Self::ExInstrumentationEvent(arg0.clone()),
            Self::ExArrayGetByRef(arg0) => Self::ExArrayGetByRef(arg0.clone()),
            Self::ExClassSparseDataVariable(arg0) => Self::ExClassSparseDataVariable(arg0.clone()),
            Self::ExFieldPathConst(arg0) => Self::ExFieldPathConst(arg0.clone()),
        }
    }
}

impl KismetExpression {
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

    pub fn write<Writer: AssetWriter>(
        expr: &KismetExpression,
        asset: &mut Writer,
    ) -> Result<usize, Error> {
        asset.write_u8(expr.get_token().into())?;
        Ok(expr.write(asset)? + size_of::<u8>())
    }
}

declare_expression!(ExFieldPathConst, value: Box<KismetExpression>);
impl ExFieldPathConst {
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
declare_expression!(ExNameConst, value: FName);
impl ExNameConst {
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
declare_expression!(ExObjectConst, value: PackageIndex);
impl ExObjectConst {
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
declare_expression!(ExSoftObjectConst, value: Box<KismetExpression>);
impl ExSoftObjectConst {
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
declare_expression!(ExTransformConst, value: Transform<f32>);
impl ExTransformConst {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let rotation = Vector4::new(
            asset.read_f32::<LittleEndian>()?,
            asset.read_f32::<LittleEndian>()?,
            asset.read_f32::<LittleEndian>()?,
            asset.read_f32::<LittleEndian>()?,
        );
        let translation = Vector::new(
            asset.read_f32::<LittleEndian>()?,
            asset.read_f32::<LittleEndian>()?,
            asset.read_f32::<LittleEndian>()?,
        );
        let scale = Vector::new(
            asset.read_f32::<LittleEndian>()?,
            asset.read_f32::<LittleEndian>()?,
            asset.read_f32::<LittleEndian>()?,
        );
        Ok(ExTransformConst {
            token: EExprToken::ExTransformConst,
            value: Transform::new(rotation, translation, scale),
        })
    }
}
impl KismetExpressionTrait for ExTransformConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_f32::<LittleEndian>(self.value.rotation.x)?;
        asset.write_f32::<LittleEndian>(self.value.rotation.y)?;
        asset.write_f32::<LittleEndian>(self.value.rotation.z)?;
        asset.write_f32::<LittleEndian>(self.value.rotation.w)?;
        asset.write_f32::<LittleEndian>(self.value.translation.x)?;
        asset.write_f32::<LittleEndian>(self.value.translation.y)?;
        asset.write_f32::<LittleEndian>(self.value.translation.z)?;
        asset.write_f32::<LittleEndian>(self.value.scale.x)?;
        asset.write_f32::<LittleEndian>(self.value.scale.y)?;
        asset.write_f32::<LittleEndian>(self.value.scale.z)?;
        Ok(size_of::<f32>() * 10)
    }
}
declare_expression!(ExVectorConst, value: Vector<f32>);
impl ExVectorConst {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExVectorConst {
            token: EExprToken::ExVectorConst,
            value: Vector::new(
                asset.read_f32::<LittleEndian>()?,
                asset.read_f32::<LittleEndian>()?,
                asset.read_f32::<LittleEndian>()?,
            ),
        })
    }
}
impl KismetExpressionTrait for ExVectorConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        asset.write_f32::<LittleEndian>(self.value.x)?;
        asset.write_f32::<LittleEndian>(self.value.y)?;
        asset.write_f32::<LittleEndian>(self.value.z)?;
        Ok(size_of::<f32>() * 3)
    }
}
declare_expression!(ExTextConst, value: Box<FScriptText>);
impl ExTextConst {
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
    delegate: Box<KismetExpression>,
    delegate_to_add: Box<KismetExpression>
);
impl ExAddMulticastDelegate {
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
    inner_property: KismetPropertyPointer,
    elements: Vec<KismetExpression>
);
impl ExArrayConst {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        let inner_property = KismetPropertyPointer::new(asset)?;
        asset.read_i32::<LittleEndian>()?; // num_entries
        let elements = KismetExpression::read_arr(asset, EExprToken::ExEndArrayConst)?;
        Ok(ExArrayConst {
            token: EExprToken::ExAddMulticastDelegate,
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
    array_variable: Box<KismetExpression>,
    array_index: Box<KismetExpression>
);
impl ExArrayGetByRef {
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
    line_number: u16,
    debug_mode: bool,
    assert_expression: Box<KismetExpression>
);
impl ExAssert {
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
    function_name: FName,
    delegate: Box<KismetExpression>,
    object_term: Box<KismetExpression>
);
impl ExBindDelegate {
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
    stack_node: PackageIndex,
    parameters: Vec<KismetExpression>
);
impl ExCallMath {
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
    stack_node: PackageIndex,
    parameters: Vec<KismetExpression>,
    delegate: Box<KismetExpression>
);
impl ExCallMulticastDelegate {
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
        for parameter in &self.parameters {
            offset += KismetExpression::write(parameter, asset)?;
        }
        offset += KismetExpression::write(&ExEndFunctionParms::default().into(), asset)?;
        Ok(offset)
    }
}
declare_expression!(
    ExClassContext,
    object_expression: Box<KismetExpression>,
    offset: u32,
    r_value_pointer: KismetPropertyPointer,
    context_expression: Box<KismetExpression>
);
impl ExClassContext {
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
declare_expression!(ExClassSparseDataVariable, variable: KismetPropertyPointer);
impl ExClassSparseDataVariable {
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
    delegate_to_clear: Box<KismetExpression>
);
impl ExClearMulticastDelegate {
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
    code_offset_expression: Box<KismetExpression>
);
impl ExComputedJump {
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
    object_expression: Box<KismetExpression>,
    offset: u32,
    r_value_pointer: KismetPropertyPointer,
    context_expression: Box<KismetExpression>
);
impl ExContext {
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
    object_expression: Box<KismetExpression>,
    offset: u32,
    r_value_pointer: KismetPropertyPointer,
    context_expression: Box<KismetExpression>
);
impl ExContextFailSilent {
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
    class_ptr: PackageIndex,
    target: Box<KismetExpression>
);
impl ExCrossInterfaceCast {
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
declare_expression!(ExDefaultVariable, variable: KismetPropertyPointer);
impl ExDefaultVariable {
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
    class_ptr: PackageIndex,
    target_expression: Box<KismetExpression>
);
impl ExDynamicCast {
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
    stack_node: PackageIndex,
    parameters: Vec<KismetExpression>
);
impl ExFinalFunction {
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
declare_expression!(ExInstanceDelegate, function_name: FName);
impl ExInstanceDelegate {
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
declare_expression!(ExInstanceVariable, variable: KismetPropertyPointer);
impl ExInstanceVariable {
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
declare_expression!(ExInterfaceContext, interface_value: Box<KismetExpression>);
impl ExInterfaceContext {
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
    class_ptr: PackageIndex,
    target: Box<KismetExpression>
);
impl ExInterfaceToObjCast {
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
declare_expression!(ExJump, code_offset: u32);
impl ExJump {
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
    code_offset: u32,
    boolean_expression: Box<KismetExpression>
);
impl ExJumpIfNot {
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
    value: KismetPropertyPointer,
    variable: Box<KismetExpression>,
    expression: Box<KismetExpression>
);
impl ExLet {
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
    variable_expression: Box<KismetExpression>,
    assignment_expression: Box<KismetExpression>
);
impl ExLetBool {
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
    variable_expression: Box<KismetExpression>,
    assignment_expression: Box<KismetExpression>
);
impl ExLetDelegate {
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
    variable_expression: Box<KismetExpression>,
    assignment_expression: Box<KismetExpression>
);
impl ExLetMulticastDelegate {
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
    variable_expression: Box<KismetExpression>,
    assignment_expression: Box<KismetExpression>
);
impl ExLetObj {
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
    destination_property: KismetPropertyPointer,
    assignment_expression: Box<KismetExpression>
);
impl ExLetValueOnPersistentFrame {
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
    variable_expression: Box<KismetExpression>,
    assignment_expression: Box<KismetExpression>
);
impl ExLetWeakObjPtr {
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
    stack_node: PackageIndex,
    parameters: Vec<KismetExpression>
);
impl ExLocalFinalFunction {
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
declare_expression!(ExLocalOutVariable, variable: KismetPropertyPointer);
impl ExLocalOutVariable {
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
declare_expression!(ExLocalVariable, variable: KismetPropertyPointer);
impl ExLocalVariable {
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
    virtual_function_name: FName,
    parameters: Vec<KismetExpression>
);
impl ExLocalVirtualFunction {
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
    key_property: KismetPropertyPointer,
    value_property: KismetPropertyPointer,
    elements: Vec<KismetExpression>
);
impl ExMapConst {
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
    class_ptr: PackageIndex,
    target_expression: Box<KismetExpression>
);
impl ExMetaCast {
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
    class_ptr: PackageIndex,
    target: Box<KismetExpression>
);
impl ExObjToInterfaceCast {
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
    boolean_expression: Box<KismetExpression>
);
impl ExPopExecutionFlowIfNot {
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
    conversion_type: ECastToken,
    target: Box<KismetExpression>
);
impl ExPrimitiveCast {
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
declare_expression!(ExPropertyConst, property: KismetPropertyPointer);
impl ExPropertyConst {
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
declare_expression!(ExPushExecutionFlow, pushing_address: u32);
impl ExPushExecutionFlow {
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
    delegate: Box<KismetExpression>,
    delegate_to_add: Box<KismetExpression>
);
impl ExRemoveMulticastDelegate {
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
declare_expression!(ExReturn, return_expression: Box<KismetExpression>);
impl ExReturn {
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
declare_expression!(ExRotationConst, pitch: i32, yaw: i32, roll: i32);
impl ExRotationConst {
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
    assigning_property: Option<Box<KismetExpression>>,
    array_inner_prop: Option<PackageIndex>,
    elements: Vec<KismetExpression>
);
impl ExSetArray {
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
    inner_property: KismetPropertyPointer,
    elements: Vec<KismetExpression>
);
impl ExSetConst {
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
    map_property: Box<KismetExpression>,
    elements: Vec<KismetExpression>
);
impl ExSetMap {
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
    set_property: Box<KismetExpression>,
    elements: Vec<KismetExpression>
);
impl ExSetSet {
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
    code_offset: u32,
    skip_expression: Box<KismetExpression>
);
impl ExSkip {
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
    struct_value: PackageIndex,
    struct_size: i32,
    value: Vec<KismetExpression>
);
impl ExStructConst {
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
    struct_member_expression: KismetPropertyPointer,
    struct_expression: Box<KismetExpression>
);
impl ExStructMemberContext {
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
    end_goto_offset: u32,
    index_term: Box<KismetExpression>,
    default_term: Box<KismetExpression>,
    cases: Vec<KismetSwitchCase>
);
impl ExSwitchValue {
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
    virtual_function_name: FName,
    parameters: Vec<KismetExpression>
);
impl ExVirtualFunction {
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
declare_expression!(ExStringConst, value: String);
impl ExStringConst {
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
declare_expression!(ExUnicodeStringConst, value: String);
impl ExUnicodeStringConst {
    pub fn new<Reader: AssetReader>(asset: &mut Reader) -> Result<Self, Error> {
        Ok(ExUnicodeStringConst {
            token: EExprToken::ExUnicodeStringConst,
            value: read_kismet_unicode_string(asset)?,
        })
    }
}
impl KismetExpressionTrait for ExUnicodeStringConst {
    fn write<Writer: AssetWriter>(&self, asset: &mut Writer) -> Result<usize, Error> {
        write_kismet_string(&self.value, asset)
    }
}

declare_expression!(
    ExInstrumentationEvent,
    event_type: EScriptInstrumentationType,
    event_name: Option<FName>
);
impl ExInstrumentationEvent {
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

implement_expression!(
    ExBreakpoint,
    ExDeprecatedOp4A,
    ExEndArray,
    ExEndArrayConst,
    ExEndFunctionParms,
    ExEndMap,
    ExEndMapConst,
    ExEndOfScript,
    ExEndParmValue,
    ExEndSet,
    ExEndSetConst,
    ExEndStructConst,
    ExFalse,
    ExIntOne,
    ExIntZero,
    ExNoInterface,
    ExNoObject,
    ExNothing,
    ExPopExecutionFlow,
    ExSelf,
    ExTracepoint,
    ExTrue,
    ExWireTracepoint
);

implement_value_expression!(ExByteConst, u8, read_u8, write_u8);
implement_value_expression!(ExInt64Const, i64, read_i64, write_i64, LittleEndian);
implement_value_expression!(ExIntConst, i32, read_i32, write_i32, LittleEndian);
implement_value_expression!(ExIntConstByte, u8, read_u8, write_u8);
implement_value_expression!(ExSkipOffsetConst, u32, read_u32, write_u32, LittleEndian);
implement_value_expression!(ExUInt64Const, u64, read_u64, write_u64, LittleEndian);
implement_value_expression!(ExFloatConst, f32, read_f32, write_f32, LittleEndian);
