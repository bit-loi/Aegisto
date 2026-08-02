use anyhow::Result;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, IntelFormatter};

use crate::types::InstructionInfo;

/// Disassemble `code_bytes` starting at virtual address `base_va`.
///
/// Returns at most `max_instructions` entries.
pub fn disassemble(
    code_bytes: &[u8],
    base_va: u64,
    max_instructions: usize,
) -> Result<Vec<InstructionInfo>> {
    let bitness = if base_va > 0xFFFF_FFFF { 64 } else { 32 };

    let mut decoder = Decoder::with_ip(bitness, code_bytes, base_va, DecoderOptions::NONE);

    let mut formatter = IntelFormatter::new();
    formatter.options_mut().set_uppercase_mnemonics(false);
    formatter
        .options_mut()
        .set_space_after_operand_separator(true);
    formatter.options_mut().set_rip_relative_addresses(true);

    let mut results = Vec::new();
    let mut instruction = Instruction::default();
    let mut mnemonic_buf = String::new();
    let mut operands_buf = String::new();
    let mut count = 0;

    while decoder.can_decode() && count < max_instructions {
        decoder.decode_out(&mut instruction);

        if instruction.is_invalid() {
            continue;
        }

        mnemonic_buf.clear();
        formatter.format_mnemonic(&instruction, &mut mnemonic_buf);

        operands_buf.clear();
        let op_count = instruction.op_count();
        if op_count > 0 {
            let _ = formatter.format_operand(&instruction, &mut operands_buf, 0);
            for i in 1..op_count {
                operands_buf.push_str(", ");
                let _ = formatter.format_operand(&instruction, &mut operands_buf, i);
            }
        }

        results.push(InstructionInfo {
            address: instruction.ip(),
            mnemonic: mnemonic_buf.clone(),
            operands: operands_buf.clone(),
        });
        count += 1;
    }

    Ok(results)
}
