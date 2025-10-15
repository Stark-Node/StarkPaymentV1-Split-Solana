use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenAccount, TokenInterface, Mint};
use anchor_spl::token::{self, Transfer};
//use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("BLAqFZF7PKcwPEKwsS2sXdzzjwxDCion3HkQtK54mgQS");

#[program]
pub mod stark_pay_split {
    use super::*;

    pub fn split<'info>(ctx: Context<'_, '_, '_, 'info, Split<'info>>, amounts: Vec<u64>) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let payer_token_account = &ctx.accounts.payer_token_account;
        let token_program = &ctx.accounts.token_program;

        require!(
            ctx.remaining_accounts.len() == amounts.len(),
            ErrorCode::LengthMismatch
        );

        for (i, receiver_account_info) in ctx.remaining_accounts.iter().enumerate() {
            let amount = amounts[i];

            let cpi_accounts = Transfer {
                from: payer_token_account.to_account_info(),
                to: receiver_account_info.clone(),
                authority: payer.to_account_info(),
            };

            let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
            token::transfer(cpi_ctx, amount)?;
        }

        Ok(())
    }

    pub fn split_percentage<'info>(
        ctx: Context<'_, '_, '_, 'info, Split<'info>>,
        amount: u64,
        percentages: Vec<u64>,
    ) -> Result<()> {
        let payer = &ctx.accounts.payer;
        let payer_token_account = &ctx.accounts.payer_token_account;
        let token_program = &ctx.accounts.token_program;

        require!(
            ctx.remaining_accounts.len() == percentages.len(),
            ErrorCode::LengthMismatch
        );

        let mut total_percentage = 0;
        for i in 0..=percentages.len() {
            total_percentage += percentages[i];
        }

        for (i, receiver_account_info) in ctx.remaining_accounts.iter().enumerate() {
            let amount_to_send = (amount * percentages[i]) / total_percentage;

            let cpi_accounts = Transfer {
                from: payer_token_account.to_account_info(),
                to: receiver_account_info.clone(),
                authority: payer.to_account_info(),
            };

            let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
            token::transfer(cpi_ctx, amount_to_send)?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Split<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, constraint = payer_token_account.owner == payer.key(), constraint = payer_token_account.mint == mint.key())]
    pub payer_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The number of receivers and amounts do not match")]
    LengthMismatch,
}