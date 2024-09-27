struct UnitStruct;

struct PlainStruct
{
    boolean: bool
}

enum EnumStruct
{
    A,
    B(UnitStruct),
    C{
        boolean: bool,
        s_plain: PlainStruct
    }
}

struct AggregateStruct
{
    uint: uint64,
    float: f32,
    boolean: bool,
    string: String,
    vec: Vec<PlainStruct>,
    map: HashMap<u32, u32>,
    s_struct: UnitStruct,
    s_enum: EnumStruct,
    s_plain: PlainStruct,
}

struct WrapArray(Vec<UnitStruct>);

struct TupleStruct(UnitStruct, PlainStruct, bool);

struct SingleGeneric<A>
{
    #[reflect(ignore)]
    _p: PhantomData<A>
}

struct SingleGenericTuple<A>(A);

// Test these with whitespace on the CAF value, which should be properly ignored when converting to JSON.
// MultiGeneric<uint32, bool, SingleGeneric<f32>>
// MultiGeneric<uint32, bool, (SingleGeneric<f32>, UnitStruct)>
struct MultiGeneric<A, B, C>
{
    #[reflect(ignore)]
    _p: PhantomData<(A, B, C)>
}

enum EnumGeneric<A>
{
    A{
        uint: uint64,

        #[reflect(ignore)]
        _p: PhantomData<A>
    },
    B{
        s_enum: EnumStruct,

        #[reflect(ignore)]
        _p: PhantomData<A>
    }
}
