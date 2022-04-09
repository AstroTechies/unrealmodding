use std::io::{Cursor, Read, Write};
use std::mem::size_of;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use enum_dispatch::enum_dispatch;
use crate::uasset::Asset;
use crate::uasset::Error;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::exports::property_export::PropertyExport;
use crate::uasset::enums::EBlueprintTextLiteralType;
use crate::uasset::enums::EBlueprintTextLiteralType::LocalizedText;
use crate::uasset::properties::Property;
use crate::uasset::types::{Transform, Vector, Vector4};
use crate::uasset::ue4version::{VER_UE4_ADDED_PACKAGE_OWNER, VER_UE4_CHANGE_SETARRAY_BYTECODE};
use crate::uasset::unreal_types::{FieldPath, FName, PackageIndex};

use super::error::KismetError;

#[derive(PartialEq, Eq, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EExprToken {
    // A local variable.
    EX_LocalVariable = 0x00,
    // An object variable.
    EX_InstanceVariable = 0x01,
    // Default variable for a class context.
    EX_DefaultVariable = 0x02,
    // Return from function.
    EX_Return = 0x04,
    // Goto a local address in code.
    EX_Jump = 0x06,
    // Goto if not expression.
    EX_JumpIfNot = 0x07,
    // Assertion.
    EX_Assert = 0x09,
    // No operation.
    EX_Nothing = 0x0B,
    // Assign an arbitrary size value to a variable.
    EX_Let = 0x0F,
    // Class default object context.
    EX_ClassContext = 0x12,
    // Metaclass cast.
    EX_MetaCast = 0x13,
    // Let boolean variable.
    EX_LetBool = 0x14,
    // end of default value for optional function parameter
    EX_EndParmValue = 0x15,
    // End of function call parameters.
    EX_EndFunctionParms = 0x16,
    // Self object.
    EX_Self = 0x17,
    // Skippable expression.
    EX_Skip = 0x18,
    // Call a function through an object context.
    EX_Context = 0x19,
    // Call a function through an object context (can fail silently if the context is NULL; only generated for functions that don't have output or return values).
    EX_Context_FailSilent = 0x1A,
    // A function call with parameters.
    EX_VirtualFunction = 0x1B,
    // A prebound function call with parameters.
    EX_FinalFunction = 0x1C,
    // Int constant.
    EX_IntConst = 0x1D,
    // Floating point constant.
    EX_FloatConst = 0x1E,
    // String constant.
    EX_StringConst = 0x1F,
    // An object constant.
    EX_ObjectConst = 0x20,
    // A name constant.
    EX_NameConst = 0x21,
    // A rotation constant.
    EX_RotationConst = 0x22,
    // A vector constant.
    EX_VectorConst = 0x23,
    // A byte constant.
    EX_ByteConst = 0x24,
    // Zero.
    EX_IntZero = 0x25,
    // One.
    EX_IntOne = 0x26,
    // Bool True.
    EX_True = 0x27,
    // Bool False.
    EX_False = 0x28,
    // FText constant
    EX_TextConst = 0x29,
    // NoObject.
    EX_NoObject = 0x2A,
    // A transform constant
    EX_TransformConst = 0x2B,
    // Int constant that requires 1 byte.
    EX_IntConstByte = 0x2C,
    // A null interface (similar to EX_NoObject, but for interfaces)
    EX_NoInterface = 0x2D,
    // Safe dynamic class casting.
    EX_DynamicCast = 0x2E,
    // An arbitrary UStruct constant
    EX_StructConst = 0x2F,
    // End of UStruct constant
    EX_EndStructConst = 0x30,
    // Set the value of arbitrary array
    EX_SetArray = 0x31,
    EX_EndArray = 0x32,
    // FProperty constant.
    EX_PropertyConst = 0x33,
    // Unicode string constant.
    EX_UnicodeStringConst = 0x34,
    // 64-bit integer constant.
    EX_Int64Const = 0x35,
    // 64-bit unsigned integer constant.
    EX_UInt64Const = 0x36,
    // A casting operator for primitives which reads the type as the subsequent byte
    EX_PrimitiveCast = 0x38,
    EX_SetSet = 0x39,
    EX_EndSet = 0x3A,
    EX_SetMap = 0x3B,
    EX_EndMap = 0x3C,
    EX_SetConst = 0x3D,
    EX_EndSetConst = 0x3E,
    EX_MapConst = 0x3F,
    EX_EndMapConst = 0x40,
    // Context expression to address a property within a struct
    EX_StructMemberContext = 0x42,
    // Assignment to a multi-cast delegate
    EX_LetMulticastDelegate = 0x43,
    // Assignment to a delegate
    EX_LetDelegate = 0x44,
    // Special instructions to quickly call a virtual function that we know is going to run only locally
    EX_LocalVirtualFunction = 0x45,
    // Special instructions to quickly call a final function that we know is going to run only locally
    EX_LocalFinalFunction = 0x46,
    // local out (pass by reference) function parameter
    EX_LocalOutVariable = 0x48,
    EX_DeprecatedOp4A = 0x4A,
    // const reference to a delegate or normal function object
    EX_InstanceDelegate = 0x4B,
    // push an address on to the execution flow stack for future execution when a EX_PopExecutionFlow is executed. Execution continues on normally and doesn't change to the pushed address.
    EX_PushExecutionFlow = 0x4C,
    // continue execution at the last address previously pushed onto the execution flow stack.
    EX_PopExecutionFlow = 0x4D,
    // Goto a local address in code, specified by an integer value.
    EX_ComputedJump = 0x4E,
    // continue execution at the last address previously pushed onto the execution flow stack, if the condition is not true.
    EX_PopExecutionFlowIfNot = 0x4F,
    // Breakpoint. Only observed in the editor, otherwise it behaves like EX_Nothing.
    EX_Breakpoint = 0x50,
    // Call a function through a native interface variable
    EX_InterfaceContext = 0x51,
    // Converting an object reference to native interface variable
    EX_ObjToInterfaceCast = 0x52,
    // Last byte in script code
    EX_EndOfScript = 0x53,
    // Converting an interface variable reference to native interface variable
    EX_CrossInterfaceCast = 0x54,
    // Converting an interface variable reference to an object
    EX_InterfaceToObjCast = 0x55,
    // Trace point.  Only observed in the editor, otherwise it behaves like EX_Nothing.
    EX_WireTracepoint = 0x5A,
    // A CodeSizeSkipOffset constant
    EX_SkipOffsetConst = 0x5B,
    // Adds a delegate to a multicast delegate's targets
    EX_AddMulticastDelegate = 0x5C,
    // Clears all delegates in a multicast target
    EX_ClearMulticastDelegate = 0x5D,
    // Trace point.  Only observed in the editor, otherwise it behaves like EX_Nothing.
    EX_Tracepoint = 0x5E,
    // assign to any object ref pointer
    EX_LetObj = 0x5F,
    // assign to a weak object pointer
    EX_LetWeakObjPtr = 0x60,
    // bind object and name to delegate
    EX_BindDelegate = 0x61,
    // Remove a delegate from a multicast delegate's targets
    EX_RemoveMulticastDelegate = 0x62,
    // Call multicast delegate
    EX_CallMulticastDelegate = 0x63,
    EX_LetValueOnPersistentFrame = 0x64,
    EX_ArrayConst = 0x65,
    EX_EndArrayConst = 0x66,
    EX_SoftObjectConst = 0x67,
    // static pure function from on local call space
    EX_CallMath = 0x68,
    EX_SwitchValue = 0x69,
    // Instrumentation event
    EX_InstrumentationEvent = 0x6A,
    EX_ArrayGetByRef = 0x6B,
    // Sparse data variable
    EX_ClassSparseDataVariable = 0x6C,
    EX_FieldPathConst = 0x6D,
    EX_Max = 0xff,
}

fn read_kismet_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String, Error> {
    let mut data = Vec::new();
    loop {
        let read = cursor.read_u8()?;
        if read == 0 { break; }
        data.push(read);
    }
    Ok(String::from_utf8(data)?)
}

fn read_kismet_unicode_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String, Error> {
    let mut data = Vec::new();
    loop {
        let b1 = cursor.read_u8()?;
        let b2 = cursor.read_u8()?;
        if b1 == 0 && b2 == 0 { break; }
        data.push(((b1 as u16) << 8) | b2 as u16)
    }
    Ok(String::from_utf16(&data)?)
}

fn write_kismet_string(string: &String, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
    let begin = cursor.position();
    cursor.write(string.as_bytes())?;
    cursor.write(&[0u8; 1])?;
    Ok((cursor.position() - begin) as usize)
}

macro_rules! declare_expression {
    ($name:ident, $($v:ident: $t:ty),*) => {
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
            pub struct $name { pub token: EExprToken }

            impl KismetExpressionTrait for $name {
                fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
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
                pub fn new(asset: &mut Asset) -> Result<Self, Error> {
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
            pub fn new(asset: &mut Asset) -> Result<Self, Error> {
                Ok($name {
                    token: EExprToken::$name,
                    value: asset.cursor.$read_func()?
                })
            }
        }

        impl KismetExpressionTrait for $name {
            fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
                cursor.$write_func(self.value)?;
                Ok(size_of::<$param>())
            }
        }
    };

    ($name:ident, $param:ty, $read_func:ident, $write_func:ident, $endianness:ident) => {
        declare_expression!($name, value: $param);
        impl $name {
            pub fn new(asset: &mut Asset) -> Result<Self, Error> {
                Ok($name {
                    token: EExprToken::$name,
                    value: asset.cursor.$read_func::<$endianness>()?
                })
            }
        }

        impl KismetExpressionTrait for $name {
            fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
                cursor.$write_func::<$endianness>(self.value)?;
                Ok(size_of::<$param>())
            }
        }
    }
}

pub struct FScriptText {
    text_literal_type: EBlueprintTextLiteralType,
    localized_source: Option<KismetExpression>,
    localized_key: Option<KismetExpression>,
    localized_namespace: Option<KismetExpression>,
    invariant_literal_string: Option<KismetExpression>,
    literal_string: Option<KismetExpression>,
    string_table_asset: Option<PackageIndex>,
    string_table_id: Option<KismetExpression>,
    string_table_key: Option<KismetExpression>
}

impl FScriptText {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let mut cursor = &mut asset.cursor;
        let text_literal_type: EBlueprintTextLiteralType = asset.cursor.read_u8()?.try_into()?;
        let (mut localized_source, mut localized_key, mut localized_namespace,
            mut invariant_literal_string, mut literal_string, mut string_table_asset,
            mut string_table_id, mut string_table_key) =
                        (None, None, None, None, None, None, None, None);

        match text_literal_type {
            EBlueprintTextLiteralType::LocalizedText => {
                localized_source = Some(KismetExpression::new(asset)?);
                localized_key = Some(KismetExpression::new(asset)?);
                localized_namespace = Some(KismetExpression::new(asset)?);
            },
            EBlueprintTextLiteralType::InvariantText => {
                invariant_literal_string = Some(KismetExpression::new(asset)?);
            },
            EBlueprintTextLiteralType::LiteralString => {
                literal_string = Some(KismetExpression::new(asset)?);
            },
            EBlueprintTextLiteralType::StringTableEntry => {
                string_table_asset = Some(PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?));
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
            string_table_key
        })
    }

    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u8>();
        cursor.write_u8(self.text_literal_type.into())?;
        match self.text_literal_type {
            EBlueprintTextLiteralType::Empty => {},
            EBlueprintTextLiteralType::LocalizedText => {
                offset = offset + KismetExpression::write(self.localized_source.as_ref().ok_or(Error::no_data("text_literal_type is LocalizedText but localized_source is None".to_string()))?, asset, cursor)?;
                offset = offset + KismetExpression::write(self.localized_key.as_ref().ok_or(Error::no_data("text_literal_type is LocalizedText but localized_key is None".to_string()))?, asset, cursor)?;
                offset = offset + KismetExpression::write(self.localized_namespace.as_ref().ok_or(Error::no_data("text_literal_type is LocalizedText but localized_namespace is None".to_string()))?, asset, cursor)?;
            },
            EBlueprintTextLiteralType::InvariantText => {
                offset = offset + KismetExpression::write(self.invariant_literal_string.as_ref().ok_or(Error::no_data("text_literal_type is InvariantText but invariant_literal_string is None".to_string()))?, asset, cursor)?;
            },
            EBlueprintTextLiteralType::LiteralString => {
                offset = offset + KismetExpression::write(self.literal_string.as_ref().ok_or(Error::no_data("text_literal_type is LiteralString but literal_string is None".to_string()))?, asset, cursor)?;
            },
            EBlueprintTextLiteralType::StringTableEntry => {
                cursor.write_i32::<LittleEndian>(self.string_table_asset.map(|e| e.index).ok_or(Error::no_data("text_literal_type is StringTableEntry but string_table_asset is None".to_string()))?)?;
                offset = offset + size_of::<u64>();
                offset = offset + KismetExpression::write(self.string_table_id.as_ref().ok_or(Error::no_data("text_literal_type is StringTalbleEntry but string_table_id is None".to_string()))?, asset, cursor)?;
                offset = offset + KismetExpression::write(self.string_table_key.as_ref().ok_or(Error::no_data("text_literal_type is StringTableEntry but string_table_key is None".to_string()))?, asset, cursor)?;
            }
        }
        Ok(offset)
    }
}

#[derive(Default)]
pub struct KismetPropertyPointer {
    pub old: Option<PackageIndex>,
    pub new: Option<FieldPath>
}

impl KismetPropertyPointer {
    pub fn from_old(old: PackageIndex) -> Self {
        KismetPropertyPointer { old: Some(old), new: None }
    }

    pub fn from_new(new: FieldPath) -> Self {
        KismetPropertyPointer { old: None, new: Some(new) }
    }

    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let mut cursor = &mut asset.cursor;
        if asset.engine_version >= VER_UE4_ADDED_PACKAGE_OWNER {
            let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
            let mut names = Vec::with_capacity(num_entries as usize);
            for i in 0..num_entries as usize {
                names.push(asset.read_fname()?);
            }
            let owner = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
            Ok(KismetPropertyPointer::from_new(FieldPath::new(names, owner)))
        } else {
            Ok(KismetPropertyPointer::from_old(PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?)))
        }
    }

    pub fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        if asset.engine_version >= VER_UE4_ADDED_PACKAGE_OWNER {
            let new = self.new.as_ref().ok_or(Error::no_data("engine_version >= UE4_ADDED_PACKAGE_OWNER but new is None".to_string()))?;
            cursor.write_i32::<LittleEndian>(new.path.len() as i32)?;
            for entry in &new.path {
                asset.write_fname(cursor, entry)?;
            }
            cursor.write_i32::<LittleEndian>(new.resolved_owner.index)?;
        } else {
            cursor.write_i32::<LittleEndian>(self.old.map(|e| e.index).ok_or(Error::no_data("engine_version < UE4_ADDED_PAFCKAGE_OWNER but old is None".to_string()))?)?;
        }
        Ok(size_of::<u64>())
    }
}

pub struct KismetSwitchCase {
    case_index_value_term: KismetExpression,
    next_offset: u32,
    case_term: KismetExpression
}

impl KismetSwitchCase {
    pub fn new(case_index_value_term: KismetExpression, next_offset: u32, case_term: KismetExpression) -> Self {
        KismetSwitchCase { case_index_value_term, next_offset, case_term }
    }
}

#[enum_dispatch]
pub trait KismetExpressionTrait {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error>;
}

#[enum_dispatch]
pub trait KismetExpressionDataTrait {
    fn get_token(&self) -> EExprToken;
}

#[enum_dispatch]
pub trait KismetExpressionEnumEqTrait {
    fn enum_eq(&self, token: &EExprToken) -> bool;
}

#[derive(PartialEq, Eq)]
#[enum_dispatch(KismetExpressionTrait, KismetExpressionEnumEqTrait, KismetExpressionDataTrait)]
pub enum KismetExpression {
    EX_LocalVariable,
    EX_InstanceVariable,
    EX_DefaultVariable,
    EX_Return,
    EX_Jump,
    EX_JumpIfNot,
    EX_Assert,
    EX_Nothing,
    EX_Let,
    EX_ClassContext,
    EX_MetaCast,
    EX_LetBool,
    EX_EndParmValue,
    EX_EndFunctionParms,
    EX_Self,
    EX_Skip,
    EX_Context,
    EX_Context_FailSilent,
    EX_VirtualFunction,
    EX_FinalFunction,
    EX_IntConst,
    EX_FloatConst,
    EX_StringConst,
    EX_ObjectConst,
    EX_NameConst,
    EX_RotationConst,
    EX_VectorConst,
    EX_ByteConst,
    EX_IntZero,
    EX_IntOne,
    EX_True,
    EX_False,
    EX_TextConst,
    EX_NoObject,
    EX_TransformConst,
    EX_IntConstByte,
    EX_NoInterface,
    EX_DynamicCast,
    EX_StructConst,
    EX_EndStructConst,
    EX_SetArray,
    EX_EndArray,
    EX_PropertyConst,
    EX_UnicodeStringConst,
    EX_Int64Const,
    EX_UInt64Const,
    EX_PrimitiveCast,
    EX_SetSet,
    EX_EndSet,
    EX_SetMap,
    EX_EndMap,
    EX_SetConst,
    EX_EndSetConst,
    EX_MapConst,
    EX_EndMapConst,
    EX_StructMemberContext,
    EX_LetMulticastDelegate,
    EX_LetDelegate,
    EX_LocalVirtualFunction,
    EX_LocalFinalFunction,
    EX_LocalOutVariable,
    EX_DeprecatedOp4A,
    EX_InstanceDelegate,
    EX_PushExecutionFlow,
    EX_PopExecutionFlow,
    EX_ComputedJump,
    EX_PopExecutionFlowIfNot,
    EX_Breakpoint,
    EX_InterfaceContext,
    EX_ObjToInterfaceCast,
    EX_EndOfScript,
    EX_CrossInterfaceCast,
    EX_InterfaceToObjCast,
    EX_WireTracepoint,
    EX_SkipOffsetConst,
    EX_AddMulticastDelegate,
    EX_ClearMulticastDelegate,
    EX_Tracepoint,
    EX_LetObj,
    EX_LetWeakObjPtr,
    EX_BindDelegate,
    EX_RemoveMulticastDelegate,
    EX_CallMulticastDelegate,
    EX_LetValueOnPersistentFrame,
    EX_ArrayConst,
    EX_EndArrayConst,
    EX_SoftObjectConst,
    EX_CallMath,
    EX_SwitchValue,
    EX_InstrumentationEvent,
    EX_ArrayGetByRef,
    EX_ClassSparseDataVariable,
    EX_FieldPathConst
}

impl KismetExpression {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let token: EExprToken = asset.cursor.read_u8()?.try_into()?;
        let expr: Result<Self, Error> = match token {
            EExprToken::EX_LocalVariable => Ok(EX_LocalVariable::new(asset)?.into()),
            EExprToken::EX_InstanceVariable => Ok(EX_InstanceVariable::new(asset)?.into()),
            EExprToken::EX_DefaultVariable => Ok(EX_DefaultVariable::new(asset)?.into()),
            EExprToken::EX_Return => Ok(EX_Return::new(asset)?.into()),
            EExprToken::EX_Jump => Ok(EX_Jump::new(asset)?.into()),
            EExprToken::EX_JumpIfNot => Ok(EX_JumpIfNot::new(asset)?.into()),
            EExprToken::EX_Assert => Ok(EX_Assert::new(asset)?.into()),
            EExprToken::EX_Nothing => Ok(EX_Nothing::new(asset)?.into()),
            EExprToken::EX_Let => Ok(EX_Let::new(asset)?.into()),
            EExprToken::EX_ClassContext => Ok(EX_ClassContext::new(asset)?.into()),
            EExprToken::EX_MetaCast => Ok(EX_MetaCast::new(asset)?.into()),
            EExprToken::EX_LetBool => Ok(EX_LetBool::new(asset)?.into()),
            EExprToken::EX_EndParmValue => Ok(EX_EndParmValue::new(asset)?.into()),
            EExprToken::EX_EndFunctionParms => Ok(EX_EndFunctionParms::new(asset)?.into()),
            EExprToken::EX_Self => Ok(EX_Self::new(asset)?.into()),
            EExprToken::EX_Skip => Ok(EX_Skip::new(asset)?.into()),
            EExprToken::EX_Context => Ok(EX_Context::new(asset)?.into()),
            EExprToken::EX_Context_FailSilent => Ok(EX_Context_FailSilent::new(asset)?.into()),
            EExprToken::EX_VirtualFunction => Ok(EX_VirtualFunction::new(asset)?.into()),
            EExprToken::EX_FinalFunction => Ok(EX_FinalFunction::new(asset)?.into()),
            EExprToken::EX_IntConst => Ok(EX_IntConst::new(asset)?.into()),
            EExprToken::EX_FloatConst => Ok(EX_FloatConst::new(asset)?.into()),
            EExprToken::EX_StringConst => Ok(EX_StringConst::new(asset)?.into()),
            EExprToken::EX_ObjectConst => Ok(EX_ObjectConst::new(asset)?.into()),
            EExprToken::EX_NameConst => Ok(EX_NameConst::new(asset)?.into()),
            EExprToken::EX_RotationConst => Ok(EX_RotationConst::new(asset)?.into()),
            EExprToken::EX_VectorConst => Ok(EX_VectorConst::new(asset)?.into()),
            EExprToken::EX_ByteConst => Ok(EX_ByteConst::new(asset)?.into()),
            EExprToken::EX_IntZero => Ok(EX_IntZero::new(asset)?.into()),
            EExprToken::EX_IntOne => Ok(EX_IntOne::new(asset)?.into()),
            EExprToken::EX_True => Ok(EX_True::new(asset)?.into()),
            EExprToken::EX_False => Ok(EX_False::new(asset)?.into()),
            EExprToken::EX_TextConst => Ok(EX_TextConst::new(asset)?.into()),
            EExprToken::EX_NoObject => Ok(EX_NoObject::new(asset)?.into()),
            EExprToken::EX_TransformConst => Ok(EX_TransformConst::new(asset)?.into()),
            EExprToken::EX_IntConstByte => Ok(EX_IntConstByte::new(asset)?.into()),
            EExprToken::EX_NoInterface => Ok(EX_NoInterface::new(asset)?.into()),
            EExprToken::EX_DynamicCast => Ok(EX_DynamicCast::new(asset)?.into()),
            EExprToken::EX_StructConst => Ok(EX_StructConst::new(asset)?.into()),
            EExprToken::EX_EndStructConst => Ok(EX_EndStructConst::new(asset)?.into()),
            EExprToken::EX_SetArray => Ok(EX_SetArray::new(asset)?.into()),
            EExprToken::EX_EndArray => Ok(EX_EndArray::new(asset)?.into()),
            EExprToken::EX_PropertyConst => Ok(EX_PropertyConst::new(asset)?.into()),
            EExprToken::EX_UnicodeStringConst => Ok(EX_UnicodeStringConst::new(asset)?.into()),
            EExprToken::EX_Int64Const => Ok(EX_Int64Const::new(asset)?.into()),
            EExprToken::EX_UInt64Const => Ok(EX_UInt64Const::new(asset)?.into()),
            EExprToken::EX_PrimitiveCast => Ok(EX_PrimitiveCast::new(asset)?.into()),
            EExprToken::EX_SetSet => Ok(EX_SetSet::new(asset)?.into()),
            EExprToken::EX_EndSet => Ok(EX_EndSet::new(asset)?.into()),
            EExprToken::EX_SetMap => Ok(EX_SetMap::new(asset)?.into()),
            EExprToken::EX_EndMap => Ok(EX_EndMap::new(asset)?.into()),
            EExprToken::EX_SetConst => Ok(EX_SetConst::new(asset)?.into()),
            EExprToken::EX_EndSetConst => Ok(EX_EndSetConst::new(asset)?.into()),
            EExprToken::EX_MapConst => Ok(EX_MapConst::new(asset)?.into()),
            EExprToken::EX_EndMapConst => Ok(EX_EndMapConst::new(asset)?.into()),
            EExprToken::EX_StructMemberContext => Ok(EX_StructMemberContext::new(asset)?.into()),
            EExprToken::EX_LetMulticastDelegate => Ok(EX_LetMulticastDelegate::new(asset)?.into()),
            EExprToken::EX_LetDelegate => Ok(EX_LetDelegate::new(asset)?.into()),
            EExprToken::EX_LocalVirtualFunction => Ok(EX_LocalVirtualFunction::new(asset)?.into()),
            EExprToken::EX_LocalFinalFunction => Ok(EX_LocalFinalFunction::new(asset)?.into()),
            EExprToken::EX_LocalOutVariable => Ok(EX_LocalOutVariable::new(asset)?.into()),
            EExprToken::EX_DeprecatedOp4A => Ok(EX_DeprecatedOp4A::new(asset)?.into()),
            EExprToken::EX_InstanceDelegate => Ok(EX_InstanceDelegate::new(asset)?.into()),
            EExprToken::EX_PushExecutionFlow => Ok(EX_PushExecutionFlow::new(asset)?.into()),
            EExprToken::EX_PopExecutionFlow => Ok(EX_PopExecutionFlow::new(asset)?.into()),
            EExprToken::EX_ComputedJump => Ok(EX_ComputedJump::new(asset)?.into()),
            EExprToken::EX_PopExecutionFlowIfNot => Ok(EX_PopExecutionFlowIfNot::new(asset)?.into()),
            EExprToken::EX_Breakpoint => Ok(EX_Breakpoint::new(asset)?.into()),
            EExprToken::EX_InterfaceContext => Ok(EX_InterfaceContext::new(asset)?.into()),
            EExprToken::EX_ObjToInterfaceCast => Ok(EX_ObjToInterfaceCast::new(asset)?.into()),
            EExprToken::EX_EndOfScript => Ok(EX_EndOfScript::new(asset)?.into()),
            EExprToken::EX_CrossInterfaceCast => Ok(EX_CrossInterfaceCast::new(asset)?.into()),
            EExprToken::EX_InterfaceToObjCast => Ok(EX_InterfaceToObjCast::new(asset)?.into()),
            EExprToken::EX_WireTracepoint => Ok(EX_WireTracepoint::new(asset)?.into()),
            EExprToken::EX_SkipOffsetConst => Ok(EX_SkipOffsetConst::new(asset)?.into()),
            EExprToken::EX_AddMulticastDelegate => Ok(EX_AddMulticastDelegate::new(asset)?.into()),
            EExprToken::EX_ClearMulticastDelegate => Ok(EX_ClearMulticastDelegate::new(asset)?.into()),
            EExprToken::EX_Tracepoint => Ok(EX_Tracepoint::new(asset)?.into()),
            EExprToken::EX_LetObj => Ok(EX_LetObj::new(asset)?.into()),
            EExprToken::EX_LetWeakObjPtr => Ok(EX_LetWeakObjPtr::new(asset)?.into()),
            EExprToken::EX_BindDelegate => Ok(EX_BindDelegate::new(asset)?.into()),
            EExprToken::EX_RemoveMulticastDelegate => Ok(EX_RemoveMulticastDelegate::new(asset)?.into()),
            EExprToken::EX_CallMulticastDelegate => Ok(EX_CallMulticastDelegate::new(asset)?.into()),
            EExprToken::EX_LetValueOnPersistentFrame => Ok(EX_LetValueOnPersistentFrame::new(asset)?.into()),
            EExprToken::EX_ArrayConst => Ok(EX_ArrayConst::new(asset)?.into()),
            EExprToken::EX_EndArrayConst => Ok(EX_EndArrayConst::new(asset)?.into()),
            EExprToken::EX_SoftObjectConst => Ok(EX_SoftObjectConst::new(asset)?.into()),
            EExprToken::EX_CallMath => Ok(EX_CallMath::new(asset)?.into()),
            EExprToken::EX_SwitchValue => Ok(EX_SwitchValue::new(asset)?.into()),
            EExprToken::EX_InstrumentationEvent => Ok(EX_InstrumentationEvent::new(asset)?.into()),
            EExprToken::EX_ArrayGetByRef => Ok(EX_ArrayGetByRef::new(asset)?.into()),
            EExprToken::EX_ClassSparseDataVariable => Ok(EX_ClassSparseDataVariable::new(asset)?.into()),
            EExprToken::EX_FieldPathConst => Ok(EX_FieldPathConst::new(asset)?.into()),
            _ => Err(KismetError::expression(format!("Unknown kismet expression {}", token as i32)).into())
        };
        expr
    }

    pub fn read_arr(asset: &mut Asset, end_token: EExprToken) -> Result<Vec<Self>, Error> {
        let mut data = Vec::new();
        let mut current_expr: Option<KismetExpression> = None;
        while current_expr.is_none() || current_expr.as_ref().unwrap().enum_eq(&end_token) {
            if let Some(expr) = current_expr {
                data.push(expr);
            }
            current_expr = KismetExpression::new(asset).ok();
        }
        Ok(data)
    }

    pub fn write(expr: &KismetExpression, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        cursor.write_u8(expr.get_token().into())?;
        Ok(expr.write(asset, cursor)? + size_of::<u8>())
    }
}

declare_expression!(EX_FieldPathConst, value: Box<KismetExpression>);
impl EX_FieldPathConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_FieldPathConst {
            token: EExprToken::EX_FieldPathConst,
            value: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_FieldPathConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        KismetExpression::write(self.value.as_ref(), asset, cursor)
    }
}
declare_expression!(EX_NameConst, value: FName);
impl EX_NameConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_NameConst {
            token: EExprToken::EX_NameConst,
            value: asset.read_fname()?
        })
    }
}
impl KismetExpressionTrait for EX_NameConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        asset.write_fname(cursor, &self.value)?;
        Ok(12)
    }
}
declare_expression!(EX_ObjectConst, value: PackageIndex);
impl EX_ObjectConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_ObjectConst {
            token: EExprToken::EX_ObjectConst,
            value: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?)
        })
    }
}
impl KismetExpressionTrait for EX_ObjectConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        cursor.write_i32::<LittleEndian>(self.value.index)?;
        Ok(size_of::<u64>())
    }
}
declare_expression!(EX_SoftObjectConst, value: Box<KismetExpression>);
impl EX_SoftObjectConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_SoftObjectConst {
            token: EExprToken::EX_SoftObjectConst,
            value: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_SoftObjectConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        KismetExpression::write(self.value.as_ref(), asset, cursor)
    }
}
declare_expression!(EX_TransformConst, value: Transform<f32>);
impl EX_TransformConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let rotation = Vector4::new(asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?);
        let translation = Vector::new(asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?);
        let scale = Vector::new(asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?);
        Ok(EX_TransformConst {
            token: EExprToken::EX_TransformConst,
            value: Transform::new(rotation, translation, scale)
        })
    }
}
impl KismetExpressionTrait for EX_TransformConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        cursor.write_f32::<LittleEndian>(self.value.rotation.x)?;
        cursor.write_f32::<LittleEndian>(self.value.rotation.y)?;
        cursor.write_f32::<LittleEndian>(self.value.rotation.z)?;
        cursor.write_f32::<LittleEndian>(self.value.rotation.w)?;
        cursor.write_f32::<LittleEndian>(self.value.translation.x)?;
        cursor.write_f32::<LittleEndian>(self.value.translation.y)?;
        cursor.write_f32::<LittleEndian>(self.value.translation.z)?;
        cursor.write_f32::<LittleEndian>(self.value.scale.x)?;
        cursor.write_f32::<LittleEndian>(self.value.scale.y)?;
        cursor.write_f32::<LittleEndian>(self.value.scale.z)?;
        Ok(size_of::<f32>() * 10)
    }
}
declare_expression!(EX_VectorConst, value: Vector<f32>);
impl EX_VectorConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_VectorConst {
            token: EExprToken::EX_VectorConst,
            value: Vector::new(asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?, asset.cursor.read_f32::<LittleEndian>()?)
        })
    }
}
impl KismetExpressionTrait for EX_VectorConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        cursor.write_f32::<LittleEndian>(self.value.x)?;
        cursor.write_f32::<LittleEndian>(self.value.y)?;
        cursor.write_f32::<LittleEndian>(self.value.z)?;
        Ok(size_of::<f32>() * 3)
    }
}
declare_expression!(EX_TextConst, value: Box<FScriptText>);
impl EX_TextConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_TextConst {
            token: EExprToken::EX_TextConst,
            value: Box::new(FScriptText::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_TextConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        self.value.write(asset, cursor)
    }
}
declare_expression!(EX_AddMulticastDelegate, delegate: Box<KismetExpression>, delegate_to_add: Box<KismetExpression>);
impl EX_AddMulticastDelegate {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_AddMulticastDelegate {
            token: EExprToken::EX_AddMulticastDelegate,
            delegate: Box::new(KismetExpression::new(asset)?),
            delegate_to_add: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_AddMulticastDelegate {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            KismetExpression::write(self.delegate.as_ref(), asset, cursor)? +
            KismetExpression::write(self.delegate_to_add.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_ArrayConst, inner_property: PackageIndex, elements: Vec<KismetExpression>);
impl EX_ArrayConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let inner_property = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
        asset.cursor.read_i32::<LittleEndian>()?; // num_entries
        let elements = KismetExpression::read_arr(asset, EExprToken::EX_EndArrayConst)?;
        Ok(EX_ArrayConst {
            token: EExprToken::EX_AddMulticastDelegate,
            inner_property,
            elements
        })
    }
}
impl KismetExpressionTrait for EX_ArrayConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>() + size_of::<i32>();
        cursor.write_i32::<LittleEndian>(self.inner_property.index)?;
        cursor.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset = offset + KismetExpression::write(element, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndArrayConst::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_ArrayGetByRef, array_variable: Box<KismetExpression>, array_index: Box<KismetExpression>);
impl EX_ArrayGetByRef {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_ArrayGetByRef {
            token: EExprToken::EX_ArrayGetByRef,
            array_variable: Box::new(KismetExpression::new(asset)?),
            array_index: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_ArrayGetByRef {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            KismetExpression::write(self.array_variable.as_ref(), asset, cursor)? +
            KismetExpression::write(self.array_index.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_Assert, line_number: u16, debug_mode: bool, assert_expression: Box<KismetExpression>);
impl EX_Assert {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_Assert {
            token: EExprToken::EX_Assert,
            line_number: asset.cursor.read_u16::<LittleEndian>()?,
            debug_mode: asset.cursor.read_bool()?,
            assert_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_Assert {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        cursor.write_u16::<LittleEndian>(self.line_number)?;
        cursor.write_bool(self.debug_mode)?;
        let offset = size_of::<u32>() + size_of::<bool>() +
            KismetExpression::write(self.assert_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_BindDelegate, function_name: FName, delegate: Box<KismetExpression>, object_term: Box<KismetExpression>);
impl EX_BindDelegate {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_BindDelegate {
            token: EExprToken::EX_BindDelegate,
            function_name: asset.read_fname()?,
            delegate: Box::new(KismetExpression::new(asset)?),
            object_term: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_BindDelegate {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset = 12 /* FScriptName's iCode offset */ +
            KismetExpression::write(self.delegate.as_ref(), asset, cursor)? +
            KismetExpression::write(self.object_term.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_CallMath, stack_node: PackageIndex, parameters: Vec<KismetExpression>);
impl EX_CallMath {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_CallMath {
            token: EExprToken::EX_CallMath,
            stack_node: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            parameters: KismetExpression::read_arr(asset, EExprToken::EX_EndFunctionParms)?
        })
    }
}
impl KismetExpressionTrait for EX_CallMath {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.stack_node.index)?;
        for parameter in &self.parameters {
            offset = offset + KismetExpression::write(parameter, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndFunctionParms::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_CallMulticastDelegate, stack_node: PackageIndex, parameters: Vec<KismetExpression>);
impl EX_CallMulticastDelegate {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_CallMulticastDelegate {
            token: EExprToken::EX_CallMulticastDelegate,
            stack_node: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            parameters: KismetExpression::read_arr(asset, EExprToken::EX_EndFunctionParms)?
        })
    }
}
impl KismetExpressionTrait for EX_CallMulticastDelegate {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.stack_node.index)?;
        for parameter in &self.parameters {
            offset = offset + KismetExpression::write(parameter, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndFunctionParms::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_ClassContext, object_expression: Box<KismetExpression>, offset: u32, r_value_pointer: KismetPropertyPointer, context_expression: Box<KismetExpression>);
impl EX_ClassContext {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_ClassContext {
            token: EExprToken::EX_ClassContext,
            object_expression: Box::new(KismetExpression::new(asset)?),
            offset: asset.cursor.read_u32::<LittleEndian>()?,
            r_value_pointer: KismetPropertyPointer::new(asset)?,
            context_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_ClassContext {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        offset = offset + KismetExpression::write(self.object_expression.as_ref(), asset, cursor)?;
        cursor.write_u32::<LittleEndian>(self.offset)?;
        offset = offset + self.r_value_pointer.write(asset, cursor)?;
        offset = offset + KismetExpression::write(self.context_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_ClassSparseDataVariable, variable: KismetPropertyPointer);
impl EX_ClassSparseDataVariable {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_ClassSparseDataVariable {
            token: EExprToken::EX_ClassSparseDataVariable,
            variable: KismetPropertyPointer::new(asset)?
        })
    }
}
impl KismetExpressionTrait for EX_ClassSparseDataVariable {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        self.variable.write(asset, cursor)
    }
}
declare_expression!(EX_ClearMulticastDelegate, delegate_to_clear: Box<KismetExpression>);
impl EX_ClearMulticastDelegate {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_ClearMulticastDelegate {
            token: EExprToken::EX_ClearMulticastDelegate,
            delegate_to_clear: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_ClearMulticastDelegate {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        KismetExpression::write(self.delegate_to_clear.as_ref(), asset, cursor)
    }
}
declare_expression!(EX_ComputedJump, code_offset_expression: Box<KismetExpression>);
impl EX_ComputedJump {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_ComputedJump {
            token: EExprToken::EX_ComputedJump,
            code_offset_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_ComputedJump {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        KismetExpression::write(self.code_offset_expression.as_ref(), asset, cursor)
    }
}
declare_expression!(EX_Context, object_expression: Box<KismetExpression>, offset: u32, r_value_pointer: KismetPropertyPointer, context_expression: Box<KismetExpression>);
impl EX_Context {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_Context {
            token: EExprToken::EX_Context,
            object_expression: Box::new(KismetExpression::new(asset)?),
            offset: asset.cursor.read_u32::<LittleEndian>()?,
            r_value_pointer: KismetPropertyPointer::new(asset)?,
            context_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_Context {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        offset = offset + KismetExpression::write(self.object_expression.as_ref(), asset, cursor)?;
        cursor.write_u32::<LittleEndian>(self.offset)?;
        offset = offset + self.r_value_pointer.write(asset, cursor)?;
        offset = offset + KismetExpression::write(self.context_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_Context_FailSilent, object_expression: Box<KismetExpression>, offset: u32, r_value_pointer: KismetPropertyPointer, context_expression: Box<KismetExpression>);
impl EX_Context_FailSilent {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_Context_FailSilent {
            token: EExprToken::EX_Context_FailSilent,
            object_expression: Box::new(KismetExpression::new(asset)?),
            offset: asset.cursor.read_u32::<LittleEndian>()?,
            r_value_pointer: KismetPropertyPointer::new(asset)?,
            context_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_Context_FailSilent {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        offset = offset + KismetExpression::write(self.object_expression.as_ref(), asset, cursor)?;
        cursor.write_u32::<LittleEndian>(self.offset)?;
        offset = offset + self.r_value_pointer.write(asset, cursor)?;
        offset = offset + KismetExpression::write(self.context_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_CrossInterfaceCast, class_ptr: PackageIndex, target: Box<KismetExpression>);
impl EX_CrossInterfaceCast {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_CrossInterfaceCast {
            token: EExprToken::EX_CrossInterfaceCast,
            class_ptr: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            target: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_CrossInterfaceCast {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset = offset + KismetExpression::write(self.target.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_DefaultVariable, variable: KismetPropertyPointer);
impl EX_DefaultVariable {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_DefaultVariable {
            token: EExprToken::EX_DefaultVariable,
            variable: KismetPropertyPointer::new(asset)?
        })
    }
}
impl KismetExpressionTrait for EX_DefaultVariable {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        self.variable.write(asset, cursor)
    }
}
declare_expression!(EX_DynamicCast, class_ptr: PackageIndex, target_expression: Box<KismetExpression>);
impl EX_DynamicCast {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_DynamicCast {
            token: EExprToken::EX_DynamicCast,
            class_ptr: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            target_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_DynamicCast {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset = offset + KismetExpression::write(self.target_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_FinalFunction, stack_node: PackageIndex, parameters: Vec<KismetExpression>);
impl EX_FinalFunction {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_FinalFunction {
            token: EExprToken::EX_FinalFunction,
            stack_node: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            parameters: KismetExpression::read_arr(asset, EExprToken::EX_EndFunctionParms)?
        })
    }
}
impl KismetExpressionTrait for EX_FinalFunction {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.stack_node.index)?;
        for parameter in &self.parameters {
            offset = offset + KismetExpression::write(parameter, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndFunctionParms::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_InstanceDelegate, function_name: FName);
impl EX_InstanceDelegate {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_InstanceDelegate {
            token: EExprToken::EX_InstanceDelegate,
            function_name: asset.read_fname()?
        })
    }
}
impl KismetExpressionTrait for EX_InstanceDelegate {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        asset.write_fname(cursor, &self.function_name)?;
        Ok(12) // FScriptName's iCode offset
    }
}
declare_expression!(EX_InstanceVariable, variable: KismetPropertyPointer);
impl EX_InstanceVariable {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_InstanceVariable {
            token: EExprToken::EX_InstanceVariable,
            variable: KismetPropertyPointer::new(asset)?
        })
    }
}
impl KismetExpressionTrait for EX_InstanceVariable {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        self.variable.write(asset, cursor)
    }
}
declare_expression!(EX_InterfaceContext, interface_value: Box<KismetExpression>);
impl EX_InterfaceContext {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_InterfaceContext {
            token: EExprToken::EX_InterfaceContext,
            interface_value: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_InterfaceContext {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        KismetExpression::write(self.interface_value.as_ref(), asset, cursor)
    }
}
declare_expression!(EX_InterfaceToObjCast, class_ptr: PackageIndex, target: Box<KismetExpression>);
impl EX_InterfaceToObjCast {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_InterfaceToObjCast {
            token: EExprToken::EX_InterfaceToObjCast,
            class_ptr: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            target: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_InterfaceToObjCast {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset = offset + KismetExpression::write(self.target.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_Jump, code_offset: u32);
impl EX_Jump {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_Jump {
            token: EExprToken::EX_Jump,
            code_offset: asset.cursor.read_u32::<LittleEndian>()?
        })
    }
}
impl KismetExpressionTrait for EX_Jump {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        cursor.write_u32::<LittleEndian>(self.code_offset)?;
        Ok(size_of::<u32>())
    }
}
declare_expression!(EX_JumpIfNot, code_offset: u32, boolean_expression: Box<KismetExpression>);
impl EX_JumpIfNot {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_JumpIfNot {
            token: EExprToken::EX_JumpIfNot,
            code_offset: asset.cursor.read_u32::<LittleEndian>()?,
            boolean_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_JumpIfNot {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        cursor.write_u32::<LittleEndian>(self.code_offset)?;
        offset = offset + KismetExpression::write(self.boolean_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_Let, value: KismetPropertyPointer);
impl EX_Let {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_Let {
            token: EExprToken::EX_Let,
            value: KismetPropertyPointer::new(asset)?
        })
    }
}
impl KismetExpressionTrait for EX_Let {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        self.value.write(asset, cursor)
    }
}
declare_expression!(EX_LetBool, variable_expression: Box<KismetExpression>, assignment_expression: Box<KismetExpression>);
impl EX_LetBool {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LetBool {
            token: EExprToken::EX_LetBool,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_LetBool {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            KismetExpression::write(self.variable_expression.as_ref(), asset, cursor)? +
            KismetExpression::write(self.assignment_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_LetDelegate, variable_expression: Box<KismetExpression>, assignment_expression: Box<KismetExpression>);
impl EX_LetDelegate {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LetDelegate {
            token: EExprToken::EX_LetDelegate,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_LetDelegate {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            KismetExpression::write(self.variable_expression.as_ref(), asset, cursor)? +
            KismetExpression::write(self.assignment_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_LetMulticastDelegate, variable_expression: Box<KismetExpression>, assignment_expression: Box<KismetExpression>);
impl EX_LetMulticastDelegate {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LetMulticastDelegate {
            token: EExprToken::EX_LetMulticastDelegate,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_LetMulticastDelegate {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            KismetExpression::write(self.variable_expression.as_ref(), asset, cursor)? +
            KismetExpression::write(self.assignment_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_LetObj, variable_expression: Box<KismetExpression>, assignment_expression: Box<KismetExpression>);
impl EX_LetObj {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LetObj {
            token: EExprToken::EX_LetObj,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_LetObj {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            KismetExpression::write(self.variable_expression.as_ref(), asset, cursor)? +
            KismetExpression::write(self.assignment_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_LetValueOnPersistentFrame, destination_property: KismetPropertyPointer, assignment_expression: Box<KismetExpression>);
impl EX_LetValueOnPersistentFrame {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LetValueOnPersistentFrame {
            token: EExprToken::EX_LetValueOnPersistentFrame,
            destination_property: KismetPropertyPointer::new(asset)?,
            assignment_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_LetValueOnPersistentFrame {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            self.destination_property.write(asset, cursor)? +
            KismetExpression::write(self.assignment_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_LetWeakObjPtr, variable_expression: Box<KismetExpression>, assignment_expression: Box<KismetExpression>);
impl EX_LetWeakObjPtr {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LetWeakObjPtr {
            token: EExprToken::EX_LetWeakObjPtr,
            variable_expression: Box::new(KismetExpression::new(asset)?),
            assignment_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_LetWeakObjPtr {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            KismetExpression::write(self.variable_expression.as_ref(), asset, cursor)? +
            KismetExpression::write(self.assignment_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_LocalFinalFunction, stack_node: PackageIndex, parameters: Vec<KismetExpression>);
impl EX_LocalFinalFunction {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LocalFinalFunction {
            token: EExprToken::EX_LocalFinalFunction,
            stack_node: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            parameters: KismetExpression::read_arr(asset, EExprToken::EX_EndFunctionParms)?
        })
    }
}
impl KismetExpressionTrait for EX_LocalFinalFunction {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.stack_node.index)?;
        for parameter in &self.parameters {
            offset = offset + KismetExpression::write(parameter, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndFunctionParms::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_LocalOutVariable, variable: KismetPropertyPointer);
impl EX_LocalOutVariable {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LocalOutVariable {
            token: EExprToken::EX_LocalOutVariable,
            variable: KismetPropertyPointer::new(asset)?
        })
    }
}
impl KismetExpressionTrait for EX_LocalOutVariable {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        self.variable.write(asset, cursor)
    }
}
declare_expression!(EX_LocalVariable, variable: KismetPropertyPointer);
impl EX_LocalVariable {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LocalVariable {
            token: EExprToken::EX_LocalVariable,
            variable: KismetPropertyPointer::new(asset)?
        })
    }
}
impl KismetExpressionTrait for EX_LocalVariable {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        self.variable.write(asset, cursor)
    }
}
declare_expression!(EX_LocalVirtualFunction, virtual_function_name: FName, parameters: Vec<KismetExpression>);
impl EX_LocalVirtualFunction {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_LocalVirtualFunction {
            token: EExprToken::EX_LocalVirtualFunction,
            virtual_function_name: asset.read_fname()?,
            parameters: KismetExpression::read_arr(asset, EExprToken::EX_EndFunctionParms)?
        })
    }
}
impl KismetExpressionTrait for EX_LocalVirtualFunction {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = 12; // FScriptName's iCode offset
        asset.write_fname(cursor, &self.virtual_function_name)?;
        for parameter in &self.parameters {
            offset = offset + KismetExpression::write(parameter, asset, cursor)?;
        }
        offset += KismetExpression::write(&EX_EndFunctionParms::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_MapConst, key_property: PackageIndex, value_property: PackageIndex, elements: Vec<KismetExpression>);
impl EX_MapConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let key_property = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
        let value_property = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let elements = KismetExpression::read_arr(asset, EExprToken::EX_EndMapConst)?;
        Ok(EX_MapConst {
            token: EExprToken::EX_MapConst,
            key_property,
            value_property,
            elements
        })
    }
}
impl KismetExpressionTrait for EX_MapConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>() * 2 + size_of::<i32>();
        cursor.write_i32::<LittleEndian>(self.key_property.index)?;
        cursor.write_i32::<LittleEndian>(self.value_property.index)?;
        cursor.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset = offset + KismetExpression::write(element, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndMapConst::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_MetaCast, class_ptr: PackageIndex, target_expression: Box<KismetExpression>);
impl EX_MetaCast {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_MetaCast {
            token: EExprToken::EX_MetaCast,
            class_ptr: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            target_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_MetaCast {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset = offset + KismetExpression::write(self.target_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_ObjToInterfaceCast, class_ptr: PackageIndex, target: Box<KismetExpression>);
impl EX_ObjToInterfaceCast {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_ObjToInterfaceCast {
            token: EExprToken::EX_ObjToInterfaceCast,
            class_ptr: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            target: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_ObjToInterfaceCast {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.class_ptr.index)?;
        offset = offset + KismetExpression::write(self.target.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_PopExecutionFlowIfNot, boolean_expression: Box<KismetExpression>);
impl EX_PopExecutionFlowIfNot {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_PopExecutionFlowIfNot {
            token: EExprToken::EX_PopExecutionFlowIfNot,
            boolean_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_PopExecutionFlowIfNot {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        KismetExpression::write(self.boolean_expression.as_ref(), asset, cursor)
    }
}
declare_expression!(EX_PrimitiveCast, conversion_type: EExprToken, target: Box<KismetExpression>);
impl EX_PrimitiveCast {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_PrimitiveCast {
            token: EExprToken::EX_PrimitiveCast,
            conversion_type: asset.cursor.read_u8()?.try_into()?,
            target: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_PrimitiveCast {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u8>();
        cursor.write_u8(self.conversion_type.into())?;
        offset = offset + KismetExpression::write(self.target.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_PropertyConst, property: KismetPropertyPointer);
impl EX_PropertyConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_PropertyConst {
            token: EExprToken::EX_PropertyConst,
            property: KismetPropertyPointer::new(asset)?
        })
    }
}
impl KismetExpressionTrait for EX_PropertyConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        self.property.write(asset, cursor)
    }
}
declare_expression!(EX_PushExecutionFlow, pushing_address: u32);
impl EX_PushExecutionFlow {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_PushExecutionFlow {
            token: EExprToken::EX_PushExecutionFlow,
            pushing_address: asset.cursor.read_u32::<LittleEndian>()?
        })
    }
}
impl KismetExpressionTrait for EX_PushExecutionFlow {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        cursor.write_u32::<LittleEndian>(self.pushing_address)?;
        Ok(size_of::<u32>())
    }
}
declare_expression!(EX_RemoveMulticastDelegate, delegate: Box<KismetExpression>, delegate_to_add: Box<KismetExpression>);
impl EX_RemoveMulticastDelegate {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_RemoveMulticastDelegate {
            token: EExprToken::EX_RemoveMulticastDelegate,
            delegate: Box::new(KismetExpression::new(asset)?),
            delegate_to_add: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_RemoveMulticastDelegate {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let offset =
            KismetExpression::write(self.delegate.as_ref(), asset, cursor)? +
            KismetExpression::write(self.delegate_to_add.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_Return, return_expression: Box<KismetExpression>);
impl EX_Return {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_Return {
            token: EExprToken::EX_Return,
            return_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_Return {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        KismetExpression::write(self.return_expression.as_ref(), asset, cursor)
    }
}
declare_expression!(EX_RotationConst, pitch: i32, yaw: i32, roll: i32);
impl EX_RotationConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_RotationConst {
            token: EExprToken::EX_RotationConst,
            pitch: asset.cursor.read_i32::<LittleEndian>()?,
            yaw: asset.cursor.read_i32::<LittleEndian>()?,
            roll: asset.cursor.read_i32::<LittleEndian>()?
        })
    }
}
impl KismetExpressionTrait for EX_RotationConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        cursor.write_i32::<LittleEndian>(self.pitch)?;
        cursor.write_i32::<LittleEndian>(self.yaw)?;
        cursor.write_i32::<LittleEndian>(self.roll)?;
        Ok(size_of::<i32>())
    }
}
declare_expression!(EX_SetArray, assigning_property: Option<Box<KismetExpression>>, array_inner_prop: Option<PackageIndex>, elements: Vec<KismetExpression>);
impl EX_SetArray {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let (assigning_property, array_inner_prop) = match asset.engine_version >= VER_UE4_CHANGE_SETARRAY_BYTECODE {
            true => (Some(Box::new(KismetExpression::new(asset)?)), None),
            false => (None, Some(PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?)))
        };
        Ok(EX_SetArray {
            token: EExprToken::EX_SetArray,
            assigning_property,
            array_inner_prop,
            elements: KismetExpression::read_arr(asset, EExprToken::EX_EndArray)?
        })
    }
}
impl KismetExpressionTrait for EX_SetArray {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = 0;
        if asset.engine_version >= VER_UE4_CHANGE_SETARRAY_BYTECODE {
            offset = offset + KismetExpression::write(self.assigning_property.as_ref().ok_or(Error::no_data("engine_version >= UE4_CHANGE_SETARRAY_BYTECODE but assigning_property is None".to_string()))?, asset, cursor)?;
        } else {
            cursor.write_i32::<LittleEndian>(self.array_inner_prop.map(|e| e.index).ok_or(Error::no_data("engine_version < UE4_CHANGE_SETARRAY_BYTECODE but array_inner_prop is None".to_string()))?)?;
            offset = offset + size_of::<u64>();
        }

        for element in &self.elements {
            offset = offset + KismetExpression::write(element, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndArray::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_SetConst, inner_property: PackageIndex, elements: Vec<KismetExpression>);
impl EX_SetConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let inner_property = PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?);
        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let elements = KismetExpression::read_arr(asset, EExprToken::EX_EndSetConst)?;
        Ok(EX_SetConst {
            token: EExprToken::EX_SetConst,
            inner_property,
            elements
        })
    }
}
impl KismetExpressionTrait for EX_SetConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>() + size_of::<i32>();
        cursor.write_i32::<LittleEndian>(self.inner_property.index)?;
        cursor.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset = offset + KismetExpression::write(element, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndSetConst::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_SetMap, map_property: Box<KismetExpression>, elements: Vec<KismetExpression>);
impl EX_SetMap {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let map_property = Box::new(KismetExpression::new(asset)?);
        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let elements = KismetExpression::read_arr(asset, EExprToken::EX_EndMap)?;
        Ok(EX_SetMap {
            token: EExprToken::EX_SetMap,
            map_property,
            elements
        })
    }
}
impl KismetExpressionTrait for EX_SetMap {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<i32>();
        offset = offset + KismetExpression::write(self.map_property.as_ref(), asset, cursor)?;
        cursor.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset = offset + KismetExpression::write(element, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndMap::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_SetSet, set_property: Box<KismetExpression>, elements: Vec<KismetExpression>);
impl EX_SetSet {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let set_property = Box::new(KismetExpression::new(asset)?);
        let num_entries = asset.cursor.read_i32::<LittleEndian>()?;
        let elements = KismetExpression::read_arr(asset, EExprToken::EX_EndSet)?;
        Ok(EX_SetSet {
            token: EExprToken::EX_SetSet,
            set_property,
            elements
        })
    }
}
impl KismetExpressionTrait for EX_SetSet {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<i32>();
        offset = offset + KismetExpression::write(self.set_property.as_ref(), asset, cursor)?;
        cursor.write_i32::<LittleEndian>(self.elements.len() as i32)?;
        for element in &self.elements {
            offset = offset + KismetExpression::write(element, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndSet::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_Skip, code_offset: u32, skip_expression: Box<KismetExpression>);
impl EX_Skip {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_Skip {
            token: EExprToken::EX_Skip,
            code_offset: asset.cursor.read_u32::<LittleEndian>()?,
            skip_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_Skip {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u32>();
        cursor.write_u32::<LittleEndian>(self.code_offset)?;
        offset = offset + KismetExpression::write(self.skip_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_StructConst, struct_value: PackageIndex, struct_size: i32, value: Vec<KismetExpression>);
impl EX_StructConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_StructConst {
            token: EExprToken::EX_StructConst,
            struct_value: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            struct_size: asset.cursor.read_i32::<LittleEndian>()?,
            value: KismetExpression::read_arr(asset, EExprToken::EX_EndStructConst)?
        })
    }
}
impl KismetExpressionTrait for EX_StructConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>() + size_of::<i32>();
        cursor.write_i32::<LittleEndian>(self.struct_value.index)?;
        cursor.write_i32::<LittleEndian>(self.struct_size)?;
        for entry in &self.value {
            offset = offset + KismetExpression::write(entry, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndStructConst::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_StructMemberContext, struct_member_expression: PackageIndex, struct_expression: Box<KismetExpression>);
impl EX_StructMemberContext {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_StructMemberContext {
            token: EExprToken::EX_StructMemberContext,
            struct_member_expression: PackageIndex::new(asset.cursor.read_i32::<LittleEndian>()?),
            struct_expression: Box::new(KismetExpression::new(asset)?)
        })
    }
}
impl KismetExpressionTrait for EX_StructMemberContext {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u64>();
        cursor.write_i32::<LittleEndian>(self.struct_member_expression.index)?;
        offset = offset + KismetExpression::write(self.struct_expression.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_SwitchValue, end_goto_offset: u32, index_term: Box<KismetExpression>, default_term: Box<KismetExpression>, cases: Vec<KismetSwitchCase>);
impl EX_SwitchValue {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        let num_cases = asset.cursor.read_u16::<LittleEndian>()?;
        let end_goto_offset = asset.cursor.read_u32::<LittleEndian>()?;
        let index_term = Box::new(KismetExpression::new(asset)?);

        let mut cases = Vec::with_capacity(num_cases as usize);
        for i in 0..num_cases as usize {
            let term_a = KismetExpression::new(asset)?;
            let term_b = asset.cursor.read_u32::<LittleEndian>()?;
            let term_c = KismetExpression::new(asset)?;
            cases.push(KismetSwitchCase::new(term_a, term_b, term_c));
        }
        let default_term = Box::new(KismetExpression::new(asset)?);
        Ok(EX_SwitchValue {
            token: EExprToken::EX_SwitchValue,
            end_goto_offset,
            index_term,
            default_term,
            cases
        })
    }
}
impl KismetExpressionTrait for EX_SwitchValue {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = size_of::<u16>() + size_of::<u32>();
        cursor.write_u16::<LittleEndian>(self.cases.len() as u16);
        cursor.write_u32::<LittleEndian>(self.end_goto_offset)?;
        offset = offset + KismetExpression::write(self.index_term.as_ref(), asset, cursor)?;
        for case in &self.cases {
            offset = offset + KismetExpression::write(&case.case_index_value_term, asset, cursor)?;
            offset = offset + size_of::<u32>();
            cursor.write_u32::<LittleEndian>(case.next_offset)?;
            offset = offset + KismetExpression::write(&case.case_term, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(self.default_term.as_ref(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_VirtualFunction, virtual_function_name: FName, parameters: Vec<KismetExpression>);
impl EX_VirtualFunction {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_VirtualFunction {
            token: EExprToken::EX_VirtualFunction,
            virtual_function_name: asset.read_fname()?,
            parameters: KismetExpression::read_arr(asset, EExprToken::EX_EndFunctionParms)?
        })
    }
}
impl KismetExpressionTrait for EX_VirtualFunction {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        let mut offset = 12; // FScriptName's iCode offset
        asset.write_fname(cursor, &self.virtual_function_name)?;
        for parameter in &self.parameters {
            offset = offset + KismetExpression::write(parameter, asset, cursor)?;
        }
        offset = offset + KismetExpression::write(&EX_EndFunctionParms::default().into(), asset, cursor)?;
        Ok(offset)
    }
}
declare_expression!(EX_StringConst, value: String);
impl EX_StringConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_StringConst {
            token: EExprToken::EX_StringConst,
            value: read_kismet_string(&mut asset.cursor)?
        })
    }
}
impl KismetExpressionTrait for EX_StringConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        write_kismet_string(&self.value, cursor)
    }
}
declare_expression!(EX_UnicodeStringConst, value: String);
impl EX_UnicodeStringConst {
    pub fn new(asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_UnicodeStringConst {
            token: EExprToken::EX_UnicodeStringConst,
            value: read_kismet_unicode_string(&mut asset.cursor)?
        })
    }
}
impl KismetExpressionTrait for EX_UnicodeStringConst {
    fn write(&self, asset: &Asset, cursor: &mut Cursor<Vec<u8>>) -> Result<usize, Error> {
        write_kismet_string(&self.value, cursor)
    }
}

implement_expression!(EX_Breakpoint, EX_DeprecatedOp4A, EX_EndArray, EX_EndArrayConst, EX_EndFunctionParms,
    EX_EndMap, EX_EndMapConst, EX_EndOfScript, EX_EndParmValue, EX_EndSet, EX_EndSetConst,
    EX_EndStructConst, EX_False, EX_InstrumentationEvent, EX_IntOne, EX_IntZero,
    EX_NoInterface, EX_NoObject, EX_Nothing, EX_PopExecutionFlow, EX_Self, EX_Tracepoint, EX_True, EX_WireTracepoint);

implement_value_expression!(EX_ByteConst, u8, read_u8, write_u8);
implement_value_expression!(EX_Int64Const, i64, read_i64, write_i64, LittleEndian);
implement_value_expression!(EX_IntConst, i32, read_i32, write_i32, LittleEndian);
implement_value_expression!(EX_IntConstByte, u8, read_u8, write_u8);
implement_value_expression!(EX_SkipOffsetConst, u32, read_u32, write_u32, LittleEndian);
implement_value_expression!(EX_UInt64Const, u64, read_u64, write_u64, LittleEndian);
implement_value_expression!(EX_FloatConst, f32, read_f32, write_f32, LittleEndian);
