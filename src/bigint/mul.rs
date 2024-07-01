use super::*;
use crate::treepp::{pushable, script, Script};

#[allow(non_snake_case)]
impl<const N_BITS: usize, const LIMB_SIZE: usize> BigIntImpl<N_BITS, LIMB_SIZE> {
    /// Multiplies the top two big integers on the stack
    /// represented as little-endian 32-bit limbs
    /// using w-width decomposition.
    pub fn mul_width_w<const WIDTH: usize>() -> Script {
        let decomposition_size = (Self::N_BITS + WIDTH - 1) / WIDTH;

        script! {
            // Convert to w-width form.
            { Self::convert_to_be_w_width_form_toaltstack::<WIDTH>() }

            // Precomputing {0*z, 1*z, ..., ((1<<WIDTH)-1)*z}
            { Self::precompute_w_mul::<WIDTH>() }

            // We initialize the result
            { Self::OP_0() }

            for _ in 0..decomposition_size {
                // Double the result WIDTH times
                for _ in 0..WIDTH {
                    { Self::OP_2MUL(0) }
                }

                // Picking di from the stack
                OP_FROMALTSTACK

                // Add the precomputed value to the result.
                // Since currently stack looks like:
                // {0*z, 1*z, ..., ((1<<WIDTH)-1)*z, r, di} with
                // r being the result, we need to copy
                // (1<<WIDTH - di)th element to the top of the stack.
                { 1<<WIDTH }
                OP_SWAP
                OP_SUB
                { Self::OP_PICKSTACK() }
                { Self::OP_ADD(0, 1) }
            }

            // Clearing the precomputed values from the stack.
            { Self::OP_TOALTSTACK() }
            for _ in 0..1<<WIDTH {
                { Self::OP_DROP() }
            }
            { Self::OP_FROMALTSTACK() }
        }
    }

    /// Multiplies the top two big integers on the stack
    /// represented as little-endian 32-bit limbs
    /// using w-width decomposition.
    pub fn widening_mul_width_w<const WIDTH: usize>() -> Script {
        let decomposition_size = (Self::N_BITS + WIDTH - 1) / WIDTH;

        script! {
            // Convert to w-width form.
            { Self::convert_to_be_w_width_form_toaltstack::<WIDTH>() }

            // Precomputing {0*z, 1*z, ..., ((1<<WIDTH)-1)*z}
            { Self::precompute_w_mul::<WIDTH>() }

            // We initialize the result
            { Self::OP_0() }

            for _ in 0..decomposition_size {
                // Double the result WIDTH times
                for _ in 0..WIDTH {
                    { Self::OP_2MUL(0) }
                }

                // Picking di from the stack
                OP_FROMALTSTACK

                // Add the precomputed value to the result.
                // Since currently stack looks like:
                // {0*z, 1*z, ..., ((1<<WIDTH)-1)*z, r, di} with
                // r being the result, we need to copy
                // (1<<WIDTH - di)th element to the top of the stack.
                { 1<<WIDTH }
                OP_SWAP
                OP_SUB
                { Self::OP_PICKSTACK() }
                { Self::OP_ADD(0, 1) }
            }

            // Clearing the precomputed values from the stack.
            { Self::OP_TOALTSTACK() }
            for _ in 0..1<<WIDTH {
                { Self::OP_DROP() }
            }
            { Self::OP_FROMALTSTACK() }
        }
    }

    /// Precomputes values `{0*z, 1*z, 2*z, ..., 2^(WIDTH)-1}` needed for
    /// multiplication, assuming that `z` is the top stack element. However,
    /// this is done lazily, costing `1` doubling and `2^(WIDTH-3)` additions, which
    /// can be done more optimally using more doublings => less additions.
    pub fn lazy_precompute_w_mul<const WIDTH: usize>() -> Script {
        assert!(WIDTH >= 2, "width should be at least 2");

        script! {
            { Self::precompute_012_mul() } // Precomputing {0, z, 2*z}
            for i in 0..(1<<WIDTH)-3 {
                // Given {0, z, 2z, ..., (i+2)z} we add (i+3)z to the end
                { Self::OP_PICK(0) }   // {0, z, ..., (i+2)z, (i+2)z}
                { Self::OP_PICK(i+2) } // {0, z, ..., (i+2)z, (i+2)z, z}
                { Self::OP_ADD(0, 1) } // {0, z, ..., (i+2)z, (i+3)z}
            }
        }
    }

    /// Precomputes values `{0*z, 1*z, 2*z}` (corresponding to `WIDTH=2`) needed
    /// for multiplication, assuming that `z` is the top stack element.
    pub fn precompute_012_mul() -> Script {
        script! {
            { Self::OP_0() }    // {z, 0}
            { Self::OP_SWAP() }   // {0, z}
            { Self::OP_PICK(0) }   // {0, z, z}
            { Self::OP_2MUL(0) } // {0, z, 2*z}
        }
    }

    /// Precomputes values `{0*z, 1*z, 2*z, 3*z}` (corresponding to `WIDTH=2`) needed
    /// for multiplication, assuming that `z` is the top stack element.
    pub fn precompute_2_mul() -> Script {
        script! {
            { Self::precompute_012_mul() }
            { Self::OP_PICK(1) }   // {0, z, 2*z, z}
            { Self::OP_PICK(1) }   // {0, z, 2*z, z, 2*z}
            { Self::OP_ADD(0, 1) } // {0, z, 2*z, 3*z}
        }
    }

    /// Precomputes values `{0*z, 1*z, ..., 7*z}` (corresponding to `WIDTH=3`) needed
    /// for multiplication, assuming that `z` is the top stack element.
    pub fn precompute_3_mul() -> Script {
        script! {
            { Self::precompute_2_mul() }
            { Self::OP_PICK(1) }    // {0, z, 2*z, 3*z, 2*z}
            { Self::OP_2MUL(0) }  // {0, z, 2*z, 3*z, 4*z}
            { Self::OP_PICK(3) }    // {0, z, 2*z, 3*z, 4*z, z}
            { Self::OP_PICK(1) }    // {0, z, 2*z, 3*z, 4*z, z, 4*z}
            { Self::OP_ADD(0, 1) }  // {0, z, 2*z, 3*z, 4*z, 5*z}
            { Self::OP_PICK(2) }    // {0, z, 2*z, 3*z, 4*z, 5*z, 3*z}
            { Self::OP_2MUL(0) }  // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z}
            { Self::OP_PICK(5) }    // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, z}
            { Self::OP_PICK(1) }    // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, z, 6*z}
            { Self::OP_ADD(0, 1) }  // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z}
        }
    }

    /// Precomputes values `{0*z, 1*z, ..., 7*z, ..., 14*z, 15*z}` (corresponding to `WIDTH=4`) needed
    /// for multiplication, assuming that `z` is the top stack element.
    pub fn precompute_4_mul() -> Script {
        script! {
            { Self::precompute_3_mul() } // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z}
            { Self::OP_PICK(3) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 4*z}
            { Self::OP_2MUL(0) }          // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z}
            { Self::OP_PICK(7) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, z}
            { Self::OP_PICK(1) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, z, 8*z}
            { Self::OP_ADD(1, 0) }          // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z}
            { Self::OP_PICK(4) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 5*z}
            { Self::OP_2MUL(0) }          // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z}
            { Self::OP_PICK(9) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, z}
            { Self::OP_PICK(1) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, z, 10*z}
            { Self::OP_ADD(1, 0) }          // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z}
            { Self::OP_PICK(5) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 6*z}
            { Self::OP_2MUL(0) }          // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z}
            { Self::OP_PICK(11) }           // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z, z}
            { Self::OP_PICK(1) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z, z, 12*z}
            { Self::OP_ADD(1, 0) }          // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z, 13*z}
            { Self::OP_PICK(6) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z, 13*z, 7*z}
            { Self::OP_2MUL(0) }          // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z, 13*z, 14*z}
            { Self::OP_PICK(13) }           // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z, 13*z, 14*z, z}
            { Self::OP_PICK(1) }            // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z, 13*z, 14*z, z, 14*z}
            { Self::OP_ADD(1, 0) }          // {0, z, 2*z, 3*z, 4*z, 5*z, 6*z, 7*z, 8*z, 9*z, 10*z, 11*z, 12*z, 13*z, 14*z, 15*z}
        }
    }

    pub fn precompute_w_mul<const WIDTH: usize>() -> Script {
        assert!(WIDTH >= 2, "width should be at least 2");

        match WIDTH {
            2 => Self::precompute_2_mul(),
            3 => Self::precompute_3_mul(),
            4 => Self::precompute_4_mul(),
            _ => Self::lazy_precompute_w_mul::<WIDTH>(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::bigint::{BigInt, Comparable, U254, U64};
    use crate::{print_script_size, treepp::*};
    use core::ops::{Mul, Rem, Shl};
    use num_bigint::{BigUint, RandomBits, ToBigUint};
    use num_traits::One;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    /// Tests the multiplication of two 254-bit numbers and two 64-bit numbers.
    #[test]
    fn test_mul() {
        print_script_size("254-bit mul", U254::OP_MUL());

        let mut prng = ChaCha20Rng::seed_from_u64(0);
        for _ in 0..3 {
            let a: BigUint = prng.sample(RandomBits::new(254));
            let b: BigUint = prng.sample(RandomBits::new(254));
            let c: BigUint = a.clone().mul(b.clone()).rem(BigUint::one().shl(254));

            let script = script! {
                { U254::OP_PUSHU32LESLICE(&a.to_u32_digits()) }
                { U254::OP_PUSHU32LESLICE(&b.to_u32_digits()) }
                { U254::OP_MUL() }
                { U254::OP_PUSHU32LESLICE(&c.to_u32_digits()) }
                { U254::OP_EQUALVERIFY(1, 0) }
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }

        for _ in 0..3 {
            let a: BigUint = prng.sample(RandomBits::new(64));
            let b: BigUint = prng.sample(RandomBits::new(64));
            let c: BigUint = (a.clone().mul(b.clone())).rem(BigUint::one().shl(64));

            let script = script! {
                { U64::OP_PUSHU32LESLICE(&a.to_u32_digits()) }
                { U64::OP_PUSHU32LESLICE(&b.to_u32_digits()) }
                { U64::OP_MUL() }
                { U64::OP_PUSHU32LESLICE(&c.to_u32_digits()) }
                { U64::OP_EQUALVERIFY(1, 0) }
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_mul_width_3_precompute() {
        print_script_size("254-bit 3-width precompute", U254::precompute_3_mul());

        let mut prng = ChaCha20Rng::seed_from_u64(0);
        let a: BigUint = prng.sample(RandomBits::new(254));

        let expected_precomputed_values = {
            let mut precomputed_values: Vec<BigUint> = vec![];
            for i in 0..1 << 3 {
                precomputed_values.push(i.to_biguint().unwrap() * a.clone());
            }

            precomputed_values
        };

        assert_eq!(
            expected_precomputed_values.len(),
            1 << 3,
            "precomputed values are incorrect"
        );
        assert_eq!(
            *expected_precomputed_values.first().unwrap(),
            0.to_biguint().unwrap(),
            "precomputed values are incorrect"
        );
        assert_eq!(
            *expected_precomputed_values.last().unwrap(),
            a.clone() * 7u32,
            "precomputed values are incorrect"
        );

        let script = script! {
            { U254::OP_PUSHU32LESLICE(&a.to_u32_digits()) }
            { U254::precompute_3_mul() }
            for expected_value in expected_precomputed_values.iter().rev() {
                { U254::OP_PUSHU32LESLICE(&expected_value.to_u32_digits()) }
                { U254::OP_EQUALVERIFY(1, 0) }
            }
            OP_TRUE
        };

        let exec_result = execute_script(script);
        println!("{:?}", exec_result.final_stack.len());
        assert!(exec_result.success, "3-width precompute test failed");
    }

    #[test]
    fn test_lazy_mul_width_w_precompute() {
        const WIDTH: usize = 4;
        print_script_size(
            format!("254-bit {:?}-width lazy precompute", WIDTH).as_str(),
            U254::lazy_precompute_w_mul::<WIDTH>(),
        );

        let mut prng = ChaCha20Rng::seed_from_u64(0);
        let a: BigUint = prng.sample(RandomBits::new(254));

        let expected_precomputed_values = {
            let mut precomputed_values: Vec<BigUint> = vec![];
            for i in 0..1 << WIDTH {
                precomputed_values.push(i.to_biguint().unwrap() * a.clone());
            }

            precomputed_values
        };

        assert_eq!(
            expected_precomputed_values.len(),
            1 << WIDTH,
            "precomputed values are incorrect"
        );
        assert_eq!(
            *expected_precomputed_values.first().unwrap(),
            0.to_biguint().unwrap(),
            "precomputed values are incorrect"
        );
        assert_eq!(
            *expected_precomputed_values.last().unwrap(),
            a.clone() * ((1 << WIDTH) - 1).to_biguint().unwrap(),
            "precomputed values are incorrect"
        );

        let script = script! {
            { U254::OP_PUSHU32LESLICE(&a.to_u32_digits()) }
            { U254::lazy_precompute_w_mul::<WIDTH>() }
            for expected_value in expected_precomputed_values.iter().rev() {
                { U254::OP_PUSHU32LESLICE(&expected_value.to_u32_digits()) }
                { U254::OP_EQUALVERIFY(1, 0) }
            }
            OP_TRUE
        };

        let exec_result = execute_script(script);
        assert!(exec_result.success, "lazy precompute test failed");
    }

    /// Tests the multiplication of two 254-bit numbers using w width approach.
    #[test]
    fn test_mul_w_width_254bit() {
        const TESTS_NUM: usize = 10;
        const WIDTH: usize = 4;

        print_script_size("254-bit w-width mul", U254::mul_width_w::<WIDTH>());

        let mut prng = ChaCha20Rng::seed_from_u64(0);
        for _ in 0..TESTS_NUM {
            let a: BigUint = prng.sample(RandomBits::new(254));
            let b: BigUint = prng.sample(RandomBits::new(254));
            let c: BigUint = a.clone().mul(b.clone()).rem(BigUint::one().shl(254));

            let script = script! {
                { U254::OP_PUSHU32LESLICE(&a.to_u32_digits()) }
                { U254::OP_PUSHU32LESLICE(&b.to_u32_digits()) }
                { U254::mul_width_w::<WIDTH>() }
                { U254::OP_PUSHU32LESLICE(&c.to_u32_digits()) }
                { U254::OP_EQUALVERIFY(1, 0) }
                OP_TRUE
            };

            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    // Tests the multiplication of two 254-bit numbers using w width approach.
    // #[test]
    // fn test_mul_w_width_508bit() {
    //     const TESTS_NUM: usize = 10;
    //     const WIDTH: usize = 4;

    //     print_script_size("508-bit w-width mul", U508::mul_width_w::<WIDTH>());

    //     let mut prng = ChaCha20Rng::seed_from_u64(0);
    //     for _ in 0..TESTS_NUM {
    //         let a: BigUint = prng.sample(RandomBits::new(254*2));
    //         let b: BigUint = prng.sample(RandomBits::new(254*2));
    //         let c: BigUint = a.clone().mul(b.clone()).rem(BigUint::one().shl(254*2));

    //         let script = script! {
    //             { U508::push_u32_le(&a.to_u32_digits()) }
    //             { U508::push_u32_le(&b.to_u32_digits()) }
    //             { U508::mul_width_w::<WIDTH>() }
    //             { U508::push_u32_le(&c.to_u32_digits()) }
    //             { U508::eq_verify(1, 0) }
    //             OP_TRUE
    //         };

    //         let exec_result = execute_script(script);
    //         assert!(exec_result.success);
    //     }
    // }
}
