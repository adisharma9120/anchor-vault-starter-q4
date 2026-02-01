use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, CreateAccount, Transfer};

declare_id!("7D9c2HFgZwyZxjQYujKZ4QZmzXihrBKqVVvzDC8jeNPw");

#[program]
pub mod anchor_vault_q4_25 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Save bumps
        ctx.accounts.vault_state.vault_bump = ctx.bumps.vault;
        ctx.accounts.vault_state.state_bump = ctx.bumps.vault_state;

        // Rent-exempt balance for vault
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);

        // ---- IMPORTANT: lifetime-safe bindings ----
        let vault_state_key = ctx.accounts.vault_state.key();
        let vault_bump = ctx.bumps.vault;

        let vault_seeds: &[&[u8]] = &[
            b"vault",
            vault_state_key.as_ref(),
            &[vault_bump],
        ];

        let signer_seeds: &[&[&[u8]]] = &[vault_seeds];
        // -------------------------------------------

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            CreateAccount {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
            signer_seeds,
        );

        system_program::create_account(
            cpi_ctx,
            lamports,
            0, // no data, SOL-only vault
            &system_program::ID,
        )?;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );

        system_program::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = 8 + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,

    /// CHECK: Vault PDA (created manually via CPI)
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Vault PDA
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: UncheckedAccount<'info>,

    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
