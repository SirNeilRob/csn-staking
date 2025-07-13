use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface, Transfer};
use anchor_spl::associated_token::Create;
use anchor_spl::associated_token::ID as ASSOCIATED_TOKEN_PROGRAM_ID;

mod mint {
    use super::*;
    pub const ID: Pubkey = Pubkey::new_from_array([
        69, 113, 160, 232, 3, 155, 186, 73, 227, 247, 43, 178, 200, 106, 110, 50,
        176, 186, 35, 70, 199, 171, 136, 47, 217, 157, 50, 147, 104, 30, 79, 171
    ]);
}

declare_id!("6CJrxAoB6Bik4TBZdm8Z3vxTCSwqoVdbZNLArfTkuN2v");

#[program]
pub mod staking {
    use super::*;

    pub fn initialize_stake(ctx: Context<InitializeStake>, amount: u64) -> Result<()> {
        require_keys_eq!(ctx.accounts.user_token_account.mint, ctx.accounts.mint.key(), CustomError::InvalidMint);
        require_keys_eq!(ctx.accounts.user_token_account.owner, ctx.accounts.user.key(), CustomError::InvalidOwner);

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.stake_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        anchor_spl::token_interface::transfer(cpi_ctx, amount)?;

        ctx.accounts.stake_state.staker = ctx.accounts.user.key();
        ctx.accounts.stake_state.amount = amount;
        ctx.accounts.stake_state.start_time = Clock::get()?.unix_timestamp;
        ctx.accounts.stake_state.last_claim_time = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn create_vault(_ctx: Context<CreateVault>) -> Result<()> {
        // Anchor handles the creation and initialization!
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let stake_state = &mut ctx.accounts.stake_state;

        // Optionally, enforce a lockup period here
        // let now = Clock::get()?.unix_timestamp;
        // require!(now >= stake_state.start_time + LOCKUP_PERIOD, CustomError::LockupNotExpired);

        let amount = stake_state.amount;

        // Transfer tokens from vault PDA back to user
        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.stake_vault.to_account_info(), // vault PDA is the authority
        };
        let seeds = [b"vault".as_ref(), &[ctx.bumps.stake_vault]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        anchor_spl::token_interface::transfer(cpi_ctx, amount)?;

        // Optionally, close the stake_state account and reclaim rent
        // (You can add a close instruction or do it here if you want)

        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let stake_state = &mut ctx.accounts.stake_state;
        let now = Clock::get()?.unix_timestamp;
        
        // Calculate time since last claim (in days)
        let days_since_claim = (now - stake_state.last_claim_time) / 86400; // 86400 seconds = 1 day
        
        if days_since_claim == 0 {
            return Err(CustomError::NoRewardsAvailable.into());
        }

        // Mock data for testing (in production, this would come from oracles)
        let energy_output_kwh = 5.0; // 5 kWh/day
        let uptime_percentage = 99.8; // 99.8%
        let maintenance_compliant = true; // maintenance is up to date

        // Calculate factors based on whitepaper formula
        let energy_factor = if energy_output_kwh / 10.0 > 1.0 { 1.0 } else { energy_output_kwh / 10.0 }; // capped at 1.0
        let uptime_factor = uptime_percentage / 100.0;
        let maintenance_factor = if maintenance_compliant { 1.0 } else { 0.5 };

        // Base rate: 0.0001 CSN/day per staked CSN (from whitepaper)
        let base_rate = 0.0001;
        
        // Calculate daily reward
        let daily_reward = base_rate * (stake_state.amount as f64) * (energy_factor + uptime_factor + maintenance_factor);
        
        // Calculate total reward for the period
        let total_reward = daily_reward * days_since_claim as f64;
        let reward_amount = total_reward as u64;

        if reward_amount == 0 {
            return Err(CustomError::NoRewardsAvailable.into());
        }

        // Transfer rewards from vault to user
        let cpi_accounts = Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.stake_vault.to_account_info(),
        };
        let seeds = [b"vault".as_ref(), &[ctx.bumps.stake_vault]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        anchor_spl::token_interface::transfer(cpi_ctx, reward_amount)?;

        // Update last claim time
        stake_state.last_claim_time = now;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeStake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        seeds = [b"stake", user.key().as_ref()],
        bump,
        space = 8 + 32 + 8 + 8 + 8 // staker + amount + start_time + last_claim_time
    )]
    pub stake_state: Account<'info, StakeState>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump,
    )]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,

    #[account()]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: validated manually
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"vault"],
        bump,
        token::mint = mint,
        token::authority = vault,
        token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: Token-2022 program
    pub token_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump,
        close = user
    )]
    pub stake_state: Account<'info, StakeState>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump,
    )]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: validated manually
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump,
    )]
    pub stake_state: Account<'info, StakeState>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump,
    )]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: validated manually
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct StakeState {
    pub staker: Pubkey,
    pub amount: u64,
    pub start_time: i64,
    pub last_claim_time: i64,
}

#[error_code]
pub enum CustomError {
    #[msg("Token account mint does not match expected mint")]
    InvalidMint,
    #[msg("Token account owner does not match signer")]
    InvalidOwner,
    #[msg("No rewards available to claim")]
    NoRewardsAvailable,
}
