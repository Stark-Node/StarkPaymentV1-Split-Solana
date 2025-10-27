use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface, Mint};
use anchor_spl::token::{self, Transfer};

declare_id!("G6XPNrj2EZiH5ivDfMbzLuhuYg5XzoBqjYFECAQuxPs6");

#[program]
pub mod stark_pay_split {
    use super::*;

    pub fn split<'info>(
        ctx: Context<'_, '_, '_, 'info, Split<'info>>,
        amounts: Vec<u64>,
    ) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let maybe_mint = ctx.accounts.mint.as_ref();

        require!(
            ctx.remaining_accounts.len() == amounts.len(),
            ErrorCode::LengthMismatch
        );

        for (i, receiver_account_info) in ctx.remaining_accounts.iter().enumerate() {
            let amount = amounts[i];

            // Case 1: SPL token transfer
            if let Some(mint) = maybe_mint {
                let payer_token_account = ctx
                    .accounts
                    .payer_token_account
                    .as_ref()
                    .ok_or(ErrorCode::MissingTokenAccount)?;

                let token_program = ctx
                    .accounts
                    .token_program
                    .as_ref()
                    .ok_or(ErrorCode::MissingTokenProgram)?;

                require_keys_eq!(
                    payer_token_account.mint,
                    mint.key(),
                    ErrorCode::InvalidMint
                );

                let cpi_accounts = Transfer {
                    from: payer_token_account.to_account_info(),
                    to: receiver_account_info.clone(),
                    authority: payer.to_account_info(),
                };

                let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
                token::transfer(cpi_ctx, amount)?;
            }
            // Case 2: Native SOL transfer
            else {
                let ix = anchor_lang::solana_program::system_instruction::transfer(
                    &payer.key(),
                    &receiver_account_info.key(),
                    amount,
                );

                anchor_lang::solana_program::program::invoke(
                    &ix,
                    &[
                        payer.to_account_info(),
                        receiver_account_info.clone(),
                    ],
                )?;
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Split<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Optional: required for SPL transfers only
    #[account(mut)]
    pub payer_token_account: Option<InterfaceAccount<'info, TokenAccount>>,

    /// Optional: SPL mint, skip for SOL
    pub mint: Option<InterfaceAccount<'info, Mint>>,

    /// Optional: Token program
    pub token_program: Option<Interface<'info, TokenInterface>>,

    /// Always present for SOL transfers
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The number of receivers and amounts do not match")]
    LengthMismatch,
    #[msg("Missing token account for SPL transfer")]
    MissingTokenAccount,
    #[msg("Missing token program for SPL transfer")]
    MissingTokenProgram,
    #[msg("Invalid mint for payer token account")]
    InvalidMint,
}