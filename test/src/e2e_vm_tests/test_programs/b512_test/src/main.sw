script;

// if test passes, return true

use std::types::B512;
use std::types::build_from_b256;
use std::constants::ETH_COLOR;

// helper to prove contiguity of memory in B512 type's hi & lo fields.
fn are_fields_aligned(big_value: B512) -> bool {
    let next_bits = asm(r1: big_value.hi, r2, r3: 32, r4) {
        move r1 sp;   // set the stack pointer to start of hi val
        add r4 r1 r3; // set r4 to hi + 32 bytes (use addi when implemented)
        // move r4 sp;   // move stack pointer to r4 - don't think I need this...
        mcp r2 r4 r3; // copy the next 32 bytes to r2
        r2: b256      // return what should be lo val
    };
    next_bits == big_value.lo
}

fn main() -> bool {
    let hi_bits: b256 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let lo_bits: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let zero: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    // it allows building from 2 b256's:
    let b: B512 = build_from_b256(hi_bits, lo_bits); // switch to method ~B512::from_b256()
    let t1 = (b.lo == lo_bits) && (b.hi == hi_bits);

    // it allows creation of new empty type:
    let mut a = ~B512::new();
    let t2 = (a.hi == zero) && (a.lo == zero);

    // it allows modification of fields:
    a.hi = hi_bits;
    a.lo = lo_bits;
    let t3 =  (a.hi == hi_bits) && (a.lo == lo_bits);

    // it guarantees memory conitiguity:
    let mut c = ~B512::new();
    c.hi = 11;
    c.lo = 42;
    let t4 = are_fields_aligned(c);

    // all checks must pass:
    t1 && t2 && t3 && t4




}