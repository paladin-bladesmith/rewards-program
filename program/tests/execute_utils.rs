use {
    solana_program_test::ProgramTestContext,
    solana_sdk::{
        instruction::Instruction,
        signature::{Keypair, Signer},
        transaction::{Transaction, TransactionError},
    },
};

fn get_transaction(
    context: &mut ProgramTestContext,
    instruction: Instruction,
    signer: Option<&Keypair>,
) -> Transaction {
    let signers = match signer {
        Some(signer) => vec![&context.payer, signer],
        None => vec![&context.payer],
    };
    Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &signers,
        context.last_blockhash,
    )
}

pub async fn execute_with_payer(
    context: &mut ProgramTestContext,
    instruction: Instruction,
    signer: Option<&Keypair>,
) {
    let transaction = get_transaction(context, instruction, signer);

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

pub async fn execute_with_payer_err(
    context: &mut ProgramTestContext,
    instruction: Instruction,
    signer: Option<&Keypair>,
) -> TransactionError {
    let transaction = get_transaction(context, instruction, signer);

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap()
}
