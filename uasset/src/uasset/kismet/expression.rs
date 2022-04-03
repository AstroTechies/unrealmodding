use std::io::{Cursor, Error};
use byteorder::{LittleEndian, ReadBytesExt};
use enum_dispatch::enum_dispatch;
use crate::uasset::Asset;
use crate::uasset::cursor_ext::CursorExt;
use crate::uasset::types::{Transform, Vector, Vector4};
use crate::uasset::unreal_types::{FName, PackageIndex};

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
    EX_Max = 0x100,
}

macro_rules! implement_expression {
    ($($name:ident),*) => {
        $(
            pub struct $name {}
            impl $name { pub fn new() -> Self { $name {} }}
        )*
    }
}

macro_rules! implement_value_expression {
    ($name:ident, $param:ident, $read_func:ident) => {
        pub struct $name {
            value: $param
        }

        impl $name {
            pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
                Ok($name {
                    value: cursor.$read_func()?
                })
            }
        }
    };

    ($name:ident, $param:ident, $read_func:ident, $endianness:ident) => {
        pub struct $name {
            value: $param
        }

        impl $name {
            pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
                Ok($name {
                    value: cursor.$read_func::<$endianness>()?
                })
            }
        }
    }
}

#[enum_dispatch]
pub trait KismetExpressionTrait {
}

#[enum_dispatch(KismetExpressionTrait)]
pub enum KismetExpression {

}

impl KismetExpression {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        unimplemented!();
    }
}

pub struct EX_FieldPathConst { value: Box<KismetExpression> }
impl EX_FieldPathConst {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_FieldPathConst {
            value: Box::new(KismetExpression::new(cursor, asset)?)
        })
    }
}

pub struct EX_NameConst { value: FName }
impl EX_NameConst {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_NameConst {
            value: asset.read_fname()?
        })
    }
}

pub struct EX_ObjectConst { value: PackageIndex }
impl EX_ObjectConst {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_ObjectConst {
            value: PackageIndex::new(cursor.read_i32::<LittleEndian>()?)
        })
    }
}

pub struct EX_SoftObjectConst { value: Box<KismetExpression> }
impl EX_SoftObjectConst {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_SoftObjectConst {
            value: Box::new(KismetExpression::new(cursor, asset)?)
        })
    }
}

pub struct EX_TransformConst { value: Transform<f32> }
impl EX_TransformConst {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        let rotation = Vector4::new(cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?);
        let translation = Vector::new(cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?);
        let scale = Vector::new(cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?);
        Ok(EX_TransformConst {
            value: Transform::new(rotation, translation, scale)
        })
    }
}

pub struct EX_VectorConst { value: Vector<f32> }
impl EX_VectorConst {
    pub fn new(cursor: &mut Cursor<Vec<u8>>, asset: &mut Asset) -> Result<Self, Error> {
        Ok(EX_VectorConst {
            value: Vector::new(cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?, cursor.read_f32::<LittleEndian>()?)
        })
    }
}

implement_expression!(EX_Breakpoint, EX_DeprecatedOp4A, EX_EndArray, EX_EndArrayConst, EX_EndFunctionParms,
    EX_EndMap, EX_EndMapConst, EX_EndOfScript, EX_EndParmValue, EX_EndSet, EX_EndSetConst,
    EX_EndStructConst, EX_False, EX_InstrumentationEvent, EX_IntOne, EX_IntZero,
    EX_NoInterface, EX_NoObject, EX_Nothing, EX_PopExecutionFlow, EX_Self, EX_Tracepoint, EX_True, EX_WireTracepoint);

implement_value_expression!(EX_ByteConst, u8, read_u8);
implement_value_expression!(EX_Int64Const, i64, read_i64, LittleEndian);
implement_value_expression!(EX_IntConst, i32, read_i32, LittleEndian);
implement_value_expression!(EX_IntConstByte, u8, read_u8);
implement_value_expression!(EX_SkipOffsetConst, u32, read_u32, LittleEndian);
implement_value_expression!(EX_StringConst, String, read_string);
implement_value_expression!(EX_UInt64Const, u64, read_u64, LittleEndian);
implement_value_expression!(EX_UnicodeStringConst, String, read_string);

