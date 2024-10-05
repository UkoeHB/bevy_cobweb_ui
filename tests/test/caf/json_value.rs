// Need to distinguish between CAF input and expected CAF output (after JSON round trip),
// since multi-line string formatting is lossy when entering JSON/Rust.

// Value round trip: rust type -> json value -> Caf -> json value -> reflect -> rust type
//   - Replace with CAF round trip once CAF parsing is ready. Note that Caf -> CAF -> Caf is potentially mutating
//   if whitespace is inserted during serialization.
// CAF round trip: CAF -> Caf -> json value -> reflect rust type (check against expected) -> json value
// -> Caf (+ recover fill) -> CAF
//   - Need separate sequence for testing #[reflect(default)] fields, since defaulted 'dont show' fields are not
//   known in rust.

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
