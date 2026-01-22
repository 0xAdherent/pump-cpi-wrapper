use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke}
};
use anchor_lang::prelude::instruction::Instruction;
use anchor_spl::associated_token::get_associated_token_address;

// 被调用的 pump 程序 ID
pub const PUMP_PROGRAM_ID: Pubkey = pubkey!("7ybnARN6UmPDpV4T3BTcvkS7Nc6vtaQLHXQHFxnXuUNd");

// 种子常量（从 IDL 解码）
const GLOBAL_CONFIG_SEED: &[u8] = b"global-config";
const BONDING_CURVE_SEED: &[u8] = b"bonding-curve";

// ATA 程序 ID（用于 PDA 派生）
const ATA_PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

// Token Program
const TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

// Metadata Program
//const METADATA_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

declare_id!("DRe39NGdWBmQKdZ6pdurCD3Zvh6JeoA67Qnecfnm9jd9"); // 替换为你自己的 program ID

#[program]
pub mod pump_cpi_wrapper {
    use super::*;

    // ========================
    // CPI: swap
    // ========================
    pub fn swap(
        ctx: Context<CpiSwap>,
        amount: u64,
        direction: u8,
        min_out: u64,
    ) -> Result<()> {
        let (global_config_pda, _bump_gc) =
            Pubkey::find_program_address(&[GLOBAL_CONFIG_SEED], &PUMP_PROGRAM_ID);
        let (bonding_curve_pda, _bump_bc) = Pubkey::find_program_address(
            &[BONDING_CURVE_SEED, ctx.accounts.token_mint.key().as_ref()],
            &PUMP_PROGRAM_ID,
        );

        let curve_ata_pda = get_associated_token_address(&bonding_curve_pda, &ctx.accounts.token_mint.key());

        let user_ata_pda = get_associated_token_address(&ctx.accounts.user.key(), &ctx.accounts.token_mint.key());

        let accounts = vec![
            AccountMeta::new(ctx.accounts.user.key(), true),
            AccountMeta::new_readonly(global_config_pda, false),
            AccountMeta::new(ctx.accounts.fee_recipient.key(), false),
            AccountMeta::new(bonding_curve_pda, false),
            AccountMeta::new_readonly(ctx.accounts.token_mint.key(), false),
            AccountMeta::new(curve_ata_pda, false),
            AccountMeta::new(user_ata_pda, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(ATA_PROGRAM_ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        let mut data = Vec::with_capacity(24);
        data.extend_from_slice(&[248, 198, 158, 145, 225, 117, 135, 200]); // swap discriminator
        amount.serialize(&mut data)?;
        direction.serialize(&mut data)?;
        min_out.serialize(&mut data)?;

        invoke(
            &Instruction {
                program_id: PUMP_PROGRAM_ID,
                accounts,
                data,
            },
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.global_config.to_account_info(), // dummy
                ctx.accounts.fee_recipient.to_account_info(),
                ctx.accounts.bonding_curve.to_account_info(),
                ctx.accounts.token_mint.to_account_info(),
                ctx.accounts.curve_token_account.to_account_info(),
                ctx.accounts.user_token_account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.associated_token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.pump_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}

// ========================
// CPI Contexts
// ========================

#[derive(Accounts)]
pub struct CpiSwap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: dummy
    pub global_config: UncheckedAccount<'info>,
    /// CHECK: fee recipient
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,
    /// CHECK: PDA
    #[account(mut)]
    pub bonding_curve: UncheckedAccount<'info>,
    /// CHECK: token mint
    pub token_mint: UncheckedAccount<'info>,
    /// CHECK: curve ATA
    #[account(mut)]
    pub curve_token_account: UncheckedAccount<'info>,
    /// CHECK: user ATA
    #[account(mut)]
    pub user_token_account: UncheckedAccount<'info>,
    /// CHECK: Token program
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: ATA program
    pub associated_token_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: pump program
    #[account(address = PUMP_PROGRAM_ID)]
    pub pump_program: AccountInfo<'info>,
}

// ========================
// Types from IDL
// ========================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Config {
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub curve_limit: u64,
    pub initial_virtual_token_reserves: u64,
    pub initial_virtual_sol_reserves: u64,
    pub initial_real_token_reserves: u64,
    pub total_token_supply: u64,
    pub buy_fee_percent: f64,
    pub sell_fee_percent: f64,
    pub migration_fee_percent: f64,
}